//! Upward messages from core (reflexive) to companion (deliberative).
//!
//! These messages carry stimulus reports, state snapshots, and novelty
//! signals upward for the companion's learning and adaptation layers.
//!
//! Invariants:
//! - I-STRT-001: no_std compatible, fixed-size arrays, no heap allocation
//! - I-CHAN-001: Ring buffer never exceeds UPWARD_CHANNEL_SIZE entries
//! - I-CHAN-002: Overflow drops oldest messages (never blocks)

use super::stimulus_log::StimulusLogEntry;
use super::startle::StartleResult;

/// Maximum number of buffered upward messages.
pub const UPWARD_CHANNEL_SIZE: usize = 16;

/// Messages the core sends to the companion for learning and adaptation.
#[derive(Clone, Debug)]
pub enum UpwardMessage {
    /// Stimulus event and processing result (for suppression learning).
    StimulusReport {
        entry: StimulusLogEntry,
        result: StartleResult,
    },
    /// Periodic state snapshot (for episode recording).
    StateSnapshot {
        tick: u64,
        context_hash: u32,
        tension: f32,
        instant_coherence: f32,
        effective_coherence: f32,
        energy: f32,
    },
    /// Novel context detected (previously unobserved context hash).
    NovelContext(u32),
}

/// Fixed-size ring buffer for upward messages from core to companion.
///
/// no_std compatible. When full, `push` overwrites the oldest message.
pub struct UpwardChannel {
    buffer: [Option<UpwardMessage>; UPWARD_CHANNEL_SIZE],
    write_idx: usize,
    read_idx: usize,
    count: usize,
}

impl UpwardChannel {
    /// Create an empty channel.
    pub fn new() -> Self {
        // Initialize with array of None using const pattern (no Copy on UpwardMessage)
        Self {
            buffer: Default::default(),
            write_idx: 0,
            read_idx: 0,
            count: 0,
        }
    }

    /// Push a message onto the channel.
    ///
    /// If the buffer is full, the oldest unread message is dropped (I-CHAN-002).
    pub fn push(&mut self, msg: UpwardMessage) {
        if self.count == UPWARD_CHANNEL_SIZE {
            // Overflow: advance read_idx to drop oldest
            self.read_idx = (self.read_idx + 1) % UPWARD_CHANNEL_SIZE;
            self.count -= 1;
        }
        self.buffer[self.write_idx] = Some(msg);
        self.write_idx = (self.write_idx + 1) % UPWARD_CHANNEL_SIZE;
        self.count += 1;
    }

    /// Drain all buffered messages in FIFO order.
    ///
    /// Returns an iterator that consumes messages from oldest to newest.
    /// After draining, the channel is empty.
    pub fn drain(&mut self) -> Drain<'_> {
        Drain { channel: self }
    }

    /// Whether the channel has no buffered messages.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Number of buffered messages.
    pub fn len(&self) -> usize {
        self.count
    }
}

impl Default for UpwardChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Draining iterator over buffered upward messages.
pub struct Drain<'a> {
    channel: &'a mut UpwardChannel,
}

impl<'a> Iterator for Drain<'a> {
    type Item = UpwardMessage;

    fn next(&mut self) -> Option<Self::Item> {
        if self.channel.count == 0 {
            return None;
        }
        let idx = self.channel.read_idx;
        let msg = self.channel.buffer[idx].take();
        self.channel.read_idx = (self.channel.read_idx + 1) % UPWARD_CHANNEL_SIZE;
        self.channel.count -= 1;
        msg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nervous_system::stimulus::{StimulusEvent, StimulusKind};
    use crate::nervous_system::stimulus_log::{PostStimulusOutcome, StimulusLogEntry};
    use crate::nervous_system::startle::StartleResult;

    fn make_stimulus_report(tick: u64) -> UpwardMessage {
        UpwardMessage::StimulusReport {
            entry: StimulusLogEntry {
                stimulus: StimulusEvent {
                    kind: StimulusKind::LoudnessSpike,
                    magnitude: 0.5,
                    tick,
                },
                context_hash: 42,
                suppression_applied: 1.0,
                tension_delta: 0.3,
                post_stimulus_outcome: PostStimulusOutcome::Pending,
            },
            result: StartleResult {
                tension_delta: 0.3,
                suppressed: false,
                suppression_factor: 1.0,
                stimulus: StimulusEvent {
                    kind: StimulusKind::LoudnessSpike,
                    magnitude: 0.5,
                    tick,
                },
            },
        }
    }

    #[test]
    fn push_and_drain_single() {
        let mut ch = UpwardChannel::new();
        assert!(ch.is_empty());
        assert_eq!(ch.len(), 0);

        ch.push(make_stimulus_report(1));
        assert!(!ch.is_empty());
        assert_eq!(ch.len(), 1);

        let msgs: Vec<_> = ch.drain().collect();
        assert_eq!(msgs.len(), 1);
        assert!(ch.is_empty());
        assert_eq!(ch.len(), 0);
    }

    #[test]
    fn push_and_drain_fifo_order() {
        let mut ch = UpwardChannel::new();
        ch.push(UpwardMessage::NovelContext(10));
        ch.push(UpwardMessage::NovelContext(20));
        ch.push(UpwardMessage::NovelContext(30));

        let msgs: Vec<_> = ch.drain().collect();
        assert_eq!(msgs.len(), 3);

        // Verify FIFO order
        match &msgs[0] {
            UpwardMessage::NovelContext(h) => assert_eq!(*h, 10),
            _ => panic!("expected NovelContext"),
        }
        match &msgs[1] {
            UpwardMessage::NovelContext(h) => assert_eq!(*h, 20),
            _ => panic!("expected NovelContext"),
        }
        match &msgs[2] {
            UpwardMessage::NovelContext(h) => assert_eq!(*h, 30),
            _ => panic!("expected NovelContext"),
        }
    }

    #[test]
    fn overflow_drops_oldest() {
        let mut ch = UpwardChannel::new();

        // Fill to capacity
        for i in 0..UPWARD_CHANNEL_SIZE {
            ch.push(UpwardMessage::NovelContext(i as u32));
        }
        assert_eq!(ch.len(), UPWARD_CHANNEL_SIZE);

        // Push one more -- oldest (context_hash=0) should be dropped
        ch.push(UpwardMessage::NovelContext(999));
        assert_eq!(ch.len(), UPWARD_CHANNEL_SIZE);

        let msgs: Vec<_> = ch.drain().collect();
        assert_eq!(msgs.len(), UPWARD_CHANNEL_SIZE);

        // First message should now be context_hash=1 (0 was dropped)
        match &msgs[0] {
            UpwardMessage::NovelContext(h) => assert_eq!(*h, 1),
            _ => panic!("expected NovelContext(1)"),
        }
        // Last should be the overflow message
        match msgs.last().unwrap() {
            UpwardMessage::NovelContext(h) => assert_eq!(*h, 999),
            _ => panic!("expected NovelContext(999)"),
        }
    }

    #[test]
    fn empty_drain_returns_nothing() {
        let mut ch = UpwardChannel::new();
        let msgs: Vec<_> = ch.drain().collect();
        assert!(msgs.is_empty());
    }

    #[test]
    fn state_snapshot_variant() {
        let mut ch = UpwardChannel::new();
        ch.push(UpwardMessage::StateSnapshot {
            tick: 100,
            context_hash: 7,
            tension: 0.5,
            instant_coherence: 0.8,
            effective_coherence: 0.75,
            energy: 0.6,
        });

        let msgs: Vec<_> = ch.drain().collect();
        assert_eq!(msgs.len(), 1);
        match &msgs[0] {
            UpwardMessage::StateSnapshot { tick, tension, .. } => {
                assert_eq!(*tick, 100);
                assert!((tension - 0.5).abs() < f32::EPSILON);
            }
            _ => panic!("expected StateSnapshot"),
        }
    }

    #[test]
    fn interleaved_push_drain() {
        let mut ch = UpwardChannel::new();

        ch.push(UpwardMessage::NovelContext(1));
        ch.push(UpwardMessage::NovelContext(2));

        // Drain first batch
        let batch1: Vec<_> = ch.drain().collect();
        assert_eq!(batch1.len(), 2);
        assert!(ch.is_empty());

        // Push more
        ch.push(UpwardMessage::NovelContext(3));
        assert_eq!(ch.len(), 1);

        let batch2: Vec<_> = ch.drain().collect();
        assert_eq!(batch2.len(), 1);
        match &batch2[0] {
            UpwardMessage::NovelContext(h) => assert_eq!(*h, 3),
            _ => panic!("expected NovelContext(3)"),
        }
    }
}
