//! WebSocket Protocol V2 with State Sync
//!
//! Contract: ARCH-005 (Transport Layer Abstraction)
//! Invariants:
//! - I-WS-V2-001: State Consistency - Client state matches robot after sync
//! - I-WS-V2-002: Message Order - Messages processed in order sent
//!
//! Features:
//! - Full state snapshot on connect
//! - Message batching (100ms window)
//! - Auto-reconnect support
//! - Version field for protocol evolution

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Protocol version
pub const PROTOCOL_VERSION: u8 = 2;

/// Message batching window (100ms per requirements)
pub const BATCH_WINDOW_MS: u64 = 100;

/// Auto-reconnect timeout (30s per requirements)
pub const RECONNECT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Error)]
pub enum WebSocketV2Error {
    #[error("Invalid protocol version: expected {expected}, got {actual}")]
    InvalidVersion { expected: u8, actual: u8 },

    #[error("Connection timeout after {0}s")]
    ConnectionTimeout(u64),

    #[error("Message serialization error: {0}")]
    SerializationError(String),

    #[error("State sync failed: {0}")]
    StateSyncError(String),

    #[error("Invalid message type: {0}")]
    InvalidMessageType(String),
}

/// WebSocket V2 message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageType {
    /// Full state snapshot (sent on connect)
    State,
    /// Command from client to robot
    Command,
    /// Event from robot to client
    Event,
    /// Batch of multiple messages
    Batch,
    /// Ping/pong for keep-alive
    Ping,
    Pong,
}

/// Core WebSocket V2 message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// Protocol version (always 2)
    pub version: u8,

    /// Message type
    #[serde(rename = "type")]
    pub msg_type: String,

    /// Message payload
    pub payload: serde_json::Value,

    /// Message timestamp (milliseconds since epoch)
    pub timestamp: u64,

    /// Optional sequence number for ordering (I-WS-V2-002)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u64>,
}

impl WebSocketMessage {
    /// Create a new message with the current timestamp
    pub fn new(msg_type: String, payload: serde_json::Value) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            msg_type,
            payload,
            timestamp: current_timestamp_ms(),
            sequence: None,
        }
    }

    /// Create a message with an explicit sequence number
    pub fn with_sequence(mut self, seq: u64) -> Self {
        self.sequence = Some(seq);
        self
    }

    /// Validate protocol version
    pub fn validate_version(&self) -> Result<(), WebSocketV2Error> {
        if self.version != PROTOCOL_VERSION {
            return Err(WebSocketV2Error::InvalidVersion {
                expected: PROTOCOL_VERSION,
                actual: self.version,
            });
        }
        Ok(())
    }
}

/// Full state snapshot sent on connect (I-WS-V2-001)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Personality configuration
    pub personality: PersonalityState,

    /// Neural system state
    pub neural_state: NeuralStateData,

    /// Inventory state (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory: Option<InventoryState>,

    /// Current game state (if in game)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_state: Option<GameState>,

    /// Robot capabilities and features
    pub capabilities: RobotCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityState {
    pub tension_baseline: f32,
    pub coherence_baseline: f32,
    pub energy_baseline: f32,
    pub startle_sensitivity: f32,
    pub recovery_speed: f32,
    pub curiosity_drive: f32,
    pub movement_expressiveness: f32,
    pub sound_expressiveness: f32,
    pub light_expressiveness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralStateData {
    pub mode: String, // "Calm", "Active", "Spike", "Protect"
    pub tension: f32,
    pub coherence: f32,
    pub energy: f32,
    pub curiosity: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gyro: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryState {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub yellow: u32,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub game_type: String, // "tictactoe", etc.
    pub state: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotCapabilities {
    pub has_drawing: bool,
    pub has_sorter: bool,
    pub has_games: bool,
    pub has_learning_lab: bool,
    pub firmware_version: String,
}

/// Message batcher - collects messages within a time window
pub struct MessageBatcher {
    queue: Arc<Mutex<VecDeque<WebSocketMessage>>>,
    last_flush: Arc<Mutex<Instant>>,
    batch_window: Duration,
}

impl MessageBatcher {
    /// Create a new message batcher
    pub fn new(batch_window_ms: u64) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            last_flush: Arc::new(Mutex::new(Instant::now())),
            batch_window: Duration::from_millis(batch_window_ms),
        }
    }

    /// Add a message to the batch queue
    pub fn enqueue(&self, message: WebSocketMessage) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(message);
    }

    /// Check if it's time to flush the batch
    pub fn should_flush(&self) -> bool {
        let last_flush = self.last_flush.lock().unwrap();
        last_flush.elapsed() >= self.batch_window
    }

    /// Flush and return all queued messages
    pub fn flush(&self) -> Vec<WebSocketMessage> {
        let mut queue = self.queue.lock().unwrap();
        let mut last_flush = self.last_flush.lock().unwrap();

        let messages: Vec<_> = queue.drain(..).collect();
        *last_flush = Instant::now();

        messages
    }

    /// Get current queue length
    pub fn queue_length(&self) -> usize {
        self.queue.lock().unwrap().len()
    }
}

/// WebSocket V2 connection manager
pub struct ConnectionManager {
    /// Current connection sequence number (I-WS-V2-002)
    sequence_counter: Arc<Mutex<u64>>,

    /// Message batcher
    batcher: MessageBatcher,

    /// Connection state
    is_connected: Arc<Mutex<bool>>,

    /// Last ping timestamp
    last_ping: Arc<Mutex<Option<Instant>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            sequence_counter: Arc::new(Mutex::new(0)),
            batcher: MessageBatcher::new(BATCH_WINDOW_MS),
            is_connected: Arc::new(Mutex::new(false)),
            last_ping: Arc::new(Mutex::new(None)),
        }
    }

    /// Get next sequence number
    pub fn next_sequence(&self) -> u64 {
        let mut counter = self.sequence_counter.lock().unwrap();
        let seq = *counter;
        *counter += 1;
        seq
    }

    /// Mark connection as established
    pub fn connect(&self) {
        let mut connected = self.is_connected.lock().unwrap();
        *connected = true;
    }

    /// Mark connection as closed
    pub fn disconnect(&self) {
        let mut connected = self.is_connected.lock().unwrap();
        *connected = false;
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        *self.is_connected.lock().unwrap()
    }

    /// Send a message (adds to batch)
    pub fn send_message(&self, msg_type: String, payload: serde_json::Value) {
        let message = WebSocketMessage::new(msg_type, payload)
            .with_sequence(self.next_sequence());

        self.batcher.enqueue(message);
    }

    /// Get messages ready to send (respects batching window)
    pub fn get_pending_messages(&self) -> Vec<WebSocketMessage> {
        if self.batcher.should_flush() {
            self.batcher.flush()
        } else {
            Vec::new()
        }
    }

    /// Record ping time
    pub fn record_ping(&self) {
        let mut last_ping = self.last_ping.lock().unwrap();
        *last_ping = Some(Instant::now());
    }

    /// Check if connection is stale (no ping in 30s)
    pub fn is_stale(&self) -> bool {
        let last_ping = self.last_ping.lock().unwrap();
        match *last_ping {
            Some(time) => time.elapsed() > Duration::from_secs(RECONNECT_TIMEOUT_SECS),
            None => false,
        }
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp in milliseconds
fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Create a state snapshot message
pub fn create_state_snapshot(snapshot: StateSnapshot) -> Result<WebSocketMessage, WebSocketV2Error> {
    let payload = serde_json::to_value(snapshot)
        .map_err(|e| WebSocketV2Error::SerializationError(e.to_string()))?;

    Ok(WebSocketMessage::new("state".to_string(), payload))
}

/// Create an event message
pub fn create_event(event_type: &str, data: serde_json::Value) -> WebSocketMessage {
    let payload = serde_json::json!({
        "event": event_type,
        "data": data,
    });

    WebSocketMessage::new("event".to_string(), payload)
}

/// Create a ping message
pub fn create_ping() -> WebSocketMessage {
    WebSocketMessage::new("ping".to_string(), serde_json::json!({}))
}

/// Create a pong message
pub fn create_pong() -> WebSocketMessage {
    WebSocketMessage::new("pong".to_string(), serde_json::json!({}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, 2);
    }

    #[test]
    fn test_message_creation() {
        let msg = WebSocketMessage::new(
            "test".to_string(),
            serde_json::json!({"key": "value"}),
        );

        assert_eq!(msg.version, 2);
        assert_eq!(msg.msg_type, "test");
        assert!(msg.timestamp > 0);
        assert!(msg.sequence.is_none());
    }

    #[test]
    fn test_message_with_sequence() {
        let msg = WebSocketMessage::new("test".to_string(), serde_json::json!({}))
            .with_sequence(42);

        assert_eq!(msg.sequence, Some(42));
    }

    #[test]
    fn test_version_validation() {
        let msg = WebSocketMessage::new("test".to_string(), serde_json::json!({}));
        assert!(msg.validate_version().is_ok());

        let mut bad_msg = msg.clone();
        bad_msg.version = 1;
        assert!(bad_msg.validate_version().is_err());
    }

    #[test]
    fn test_message_batcher() {
        let batcher = MessageBatcher::new(100);

        assert_eq!(batcher.queue_length(), 0);

        let msg1 = WebSocketMessage::new("test1".to_string(), serde_json::json!({}));
        let msg2 = WebSocketMessage::new("test2".to_string(), serde_json::json!({}));

        batcher.enqueue(msg1);
        batcher.enqueue(msg2);

        assert_eq!(batcher.queue_length(), 2);

        // Immediately flushing shouldn't return messages (window not elapsed)
        std::thread::sleep(std::time::Duration::from_millis(10));
        if !batcher.should_flush() {
            assert_eq!(batcher.queue_length(), 2);
        }

        // After batch window, should flush
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(batcher.should_flush());

        let messages = batcher.flush();
        assert_eq!(messages.len(), 2);
        assert_eq!(batcher.queue_length(), 0);
    }

    #[test]
    fn test_connection_manager() {
        let manager = ConnectionManager::new();

        assert!(!manager.is_connected());

        manager.connect();
        assert!(manager.is_connected());

        let seq1 = manager.next_sequence();
        let seq2 = manager.next_sequence();
        assert_eq!(seq2, seq1 + 1);

        manager.disconnect();
        assert!(!manager.is_connected());
    }

    #[test]
    fn test_connection_staleness() {
        let manager = ConnectionManager::new();

        // No ping recorded yet
        assert!(!manager.is_stale());

        manager.record_ping();
        assert!(!manager.is_stale());

        // Can't easily test 30s timeout in unit test
        // Would require mocking time or making timeout configurable
    }

    #[test]
    fn test_state_snapshot_serialization() {
        let snapshot = StateSnapshot {
            personality: PersonalityState {
                tension_baseline: 0.5,
                coherence_baseline: 0.5,
                energy_baseline: 0.5,
                startle_sensitivity: 0.5,
                recovery_speed: 0.5,
                curiosity_drive: 0.5,
                movement_expressiveness: 0.5,
                sound_expressiveness: 0.5,
                light_expressiveness: 0.5,
            },
            neural_state: NeuralStateData {
                mode: "Calm".to_string(),
                tension: 0.3,
                coherence: 0.7,
                energy: 0.5,
                curiosity: 0.6,
                distance: Some(25.0),
                gyro: None,
                sound: None,
                light: None,
            },
            inventory: None,
            game_state: None,
            capabilities: RobotCapabilities {
                has_drawing: true,
                has_sorter: true,
                has_games: true,
                has_learning_lab: false,
                firmware_version: "1.0.0".to_string(),
            },
        };

        let message = create_state_snapshot(snapshot).unwrap();
        assert_eq!(message.msg_type, "state");
        assert_eq!(message.version, 2);
    }

    #[test]
    fn test_ping_pong_messages() {
        let ping = create_ping();
        assert_eq!(ping.msg_type, "ping");
        assert_eq!(ping.version, 2);

        let pong = create_pong();
        assert_eq!(pong.msg_type, "pong");
        assert_eq!(pong.version, 2);
    }

    #[test]
    fn test_event_message() {
        let event = create_event("test_event", serde_json::json!({"data": "value"}));
        assert_eq!(event.msg_type, "event");
        assert_eq!(event.version, 2);
    }

    #[test]
    fn test_message_ordering_invariant() {
        // I-WS-V2-002: Messages must be processed in order
        let manager = ConnectionManager::new();

        let seq1 = manager.next_sequence();
        let seq2 = manager.next_sequence();
        let seq3 = manager.next_sequence();

        // Sequence numbers must be strictly increasing
        assert!(seq1 < seq2);
        assert!(seq2 < seq3);
    }
}
