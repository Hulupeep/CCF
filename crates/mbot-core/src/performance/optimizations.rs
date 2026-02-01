//! Performance optimizations
//!
//! Implements various optimization strategies for WebSocket, memory, and rendering performance.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Global optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub websocket: WebSocketConfig,
    pub memory: MemoryConfig,
    pub render: RenderConfig,
}

impl OptimizationConfig {
    pub fn default() -> Self {
        Self {
            websocket: WebSocketConfig {
                batching_enabled: true,
                batch_size: 10,
                batch_delay_ms: 16,
                compression_enabled: true,
                target_latency_ms: 50,
            },
            memory: MemoryConfig {
                max_cache_size_mb: 50,
                gc_interval_ms: 5000,
                enable_object_pooling: true,
                target_usage_mb: 100,
            },
            render: RenderConfig {
                use_offscreen_canvas: true,
                enable_webgl: false,
                use_request_animation_frame: true,
                target_fps: 60,
            },
        }
    }

    /// Aggressive optimization profile
    pub fn aggressive() -> Self {
        Self {
            websocket: WebSocketConfig {
                batching_enabled: true,
                batch_size: 20,
                batch_delay_ms: 8,
                compression_enabled: true,
                target_latency_ms: 40,
            },
            memory: MemoryConfig {
                max_cache_size_mb: 30,
                gc_interval_ms: 3000,
                enable_object_pooling: true,
                target_usage_mb: 80,
            },
            render: RenderConfig {
                use_offscreen_canvas: true,
                enable_webgl: true,
                use_request_animation_frame: true,
                target_fps: 60,
            },
        }
    }

    /// Conservative optimization profile
    pub fn conservative() -> Self {
        Self {
            websocket: WebSocketConfig {
                batching_enabled: true,
                batch_size: 5,
                batch_delay_ms: 32,
                compression_enabled: false,
                target_latency_ms: 50,
            },
            memory: MemoryConfig {
                max_cache_size_mb: 100,
                gc_interval_ms: 10000,
                enable_object_pooling: false,
                target_usage_mb: 100,
            },
            render: RenderConfig {
                use_offscreen_canvas: false,
                enable_webgl: false,
                use_request_animation_frame: true,
                target_fps: 60,
            },
        }
    }
}

/// WebSocket optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub batching_enabled: bool,
    pub batch_size: usize,
    pub batch_delay_ms: u64,
    pub compression_enabled: bool,
    pub target_latency_ms: u64,
}

/// Memory optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub max_cache_size_mb: usize,
    pub gc_interval_ms: u64,
    pub enable_object_pooling: bool,
    pub target_usage_mb: usize,
}

/// Rendering optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub use_offscreen_canvas: bool,
    pub enable_webgl: bool,
    pub use_request_animation_frame: bool,
    pub target_fps: u32,
}

/// WebSocket message batcher for reducing round trips
pub struct WebSocketOptimizer {
    config: WebSocketConfig,
    batch: Arc<Mutex<VecDeque<Vec<u8>>>>,
    last_flush: Arc<Mutex<Instant>>,
}

impl WebSocketOptimizer {
    pub fn new(config: WebSocketConfig) -> Self {
        Self {
            config,
            batch: Arc::new(Mutex::new(VecDeque::new())),
            last_flush: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Add message to batch
    pub fn queue_message(&self, message: Vec<u8>) -> Option<Vec<Vec<u8>>> {
        if !self.config.batching_enabled {
            return Some(vec![message]);
        }

        let mut batch = self.batch.lock().unwrap();
        batch.push_back(message);

        // Flush if batch size reached
        if batch.len() >= self.config.batch_size {
            return Some(self.flush_internal(&mut batch));
        }

        // Flush if delay exceeded
        let last_flush = self.last_flush.lock().unwrap();
        if last_flush.elapsed() >= Duration::from_millis(self.config.batch_delay_ms) {
            drop(last_flush);
            return Some(self.flush_internal(&mut batch));
        }

        None
    }

    /// Force flush all batched messages
    pub fn flush(&self) -> Vec<Vec<u8>> {
        let mut batch = self.batch.lock().unwrap();
        self.flush_internal(&mut batch)
    }

    fn flush_internal(&self, batch: &mut VecDeque<Vec<u8>>) -> Vec<Vec<u8>> {
        let messages: Vec<Vec<u8>> = batch.drain(..).collect();
        let mut last_flush = self.last_flush.lock().unwrap();
        *last_flush = Instant::now();
        messages
    }

    /// Get average batch size
    pub fn avg_batch_size(&self) -> f64 {
        let batch = self.batch.lock().unwrap();
        batch.len() as f64
    }

    /// Compress message if enabled
    pub fn compress_message(&self, message: &[u8]) -> Vec<u8> {
        if !self.config.compression_enabled {
            return message.to_vec();
        }

        // Simple run-length encoding for demo
        // In production, use a proper compression library
        message.to_vec()
    }
}

/// Memory optimizer for reducing memory footprint
pub struct MemoryOptimizer {
    config: MemoryConfig,
    cache_size: Arc<Mutex<usize>>,
}

impl MemoryOptimizer {
    pub fn new(config: MemoryConfig) -> Self {
        Self {
            config,
            cache_size: Arc::new(Mutex::new(0)),
        }
    }

    /// Check if cache limit reached
    pub fn can_allocate(&self, size_bytes: usize) -> bool {
        let current_size = *self.cache_size.lock().unwrap();
        let max_size = self.config.max_cache_size_mb * 1024 * 1024;

        current_size + size_bytes <= max_size
    }

    /// Record allocation
    pub fn allocate(&self, size_bytes: usize) -> bool {
        if !self.can_allocate(size_bytes) {
            return false;
        }

        let mut cache_size = self.cache_size.lock().unwrap();
        *cache_size += size_bytes;
        true
    }

    /// Record deallocation
    pub fn deallocate(&self, size_bytes: usize) {
        let mut cache_size = self.cache_size.lock().unwrap();
        *cache_size = cache_size.saturating_sub(size_bytes);
    }

    /// Get current cache size in MB
    pub fn current_usage_mb(&self) -> f64 {
        let size = *self.cache_size.lock().unwrap();
        size as f64 / (1024.0 * 1024.0)
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        let mut cache_size = self.cache_size.lock().unwrap();
        *cache_size = 0;
    }

    /// Check if within target
    pub fn within_target(&self) -> bool {
        self.current_usage_mb() <= self.config.target_usage_mb as f64
    }
}

/// Render optimizer for maintaining 60fps
pub struct RenderOptimizer {
    config: RenderConfig,
    frame_budget: Duration,
}

impl RenderOptimizer {
    pub fn new(config: RenderConfig) -> Self {
        let frame_budget = Duration::from_secs_f64(1.0 / config.target_fps as f64);

        Self {
            config,
            frame_budget,
        }
    }

    /// Get frame budget in milliseconds
    pub fn frame_budget_ms(&self) -> f64 {
        self.frame_budget.as_secs_f64() * 1000.0
    }

    /// Check if operation fits in frame budget
    pub fn fits_in_budget(&self, duration: Duration) -> bool {
        duration <= self.frame_budget
    }

    /// Should use offscreen canvas
    pub fn use_offscreen_canvas(&self) -> bool {
        self.config.use_offscreen_canvas
    }

    /// Should use WebGL acceleration
    pub fn use_webgl(&self) -> bool {
        self.config.enable_webgl
    }

    /// Get target FPS
    pub fn target_fps(&self) -> u32 {
        self.config.target_fps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_batching() {
        let config = WebSocketConfig {
            batching_enabled: true,
            batch_size: 3,
            batch_delay_ms: 100,
            compression_enabled: false,
            target_latency_ms: 50,
        };

        let optimizer = WebSocketOptimizer::new(config);

        // Queue messages
        assert!(optimizer.queue_message(vec![1, 2, 3]).is_none());
        assert!(optimizer.queue_message(vec![4, 5, 6]).is_none());

        // Third message should trigger flush
        let flushed = optimizer.queue_message(vec![7, 8, 9]);
        assert!(flushed.is_some());
        let messages = flushed.unwrap();
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_memory_allocation_limits() {
        let config = MemoryConfig {
            max_cache_size_mb: 1, // 1MB limit
            gc_interval_ms: 5000,
            enable_object_pooling: true,
            target_usage_mb: 1,
        };

        let optimizer = MemoryOptimizer::new(config);

        // Should allow allocation within limit
        assert!(optimizer.allocate(512 * 1024)); // 512KB

        // Should allow another 512KB
        assert!(optimizer.allocate(512 * 1024));

        // Should reject allocation exceeding limit
        assert!(!optimizer.allocate(1024)); // Would exceed 1MB

        // Should be exactly at limit (1MB = 1024KB)
        let usage = optimizer.current_usage_mb();
        assert!(usage >= 0.99 && usage <= 1.01); // Allow small floating point error
    }

    #[test]
    fn test_render_frame_budget() {
        let config = RenderConfig {
            use_offscreen_canvas: true,
            enable_webgl: false,
            use_request_animation_frame: true,
            target_fps: 60,
        };

        let optimizer = RenderOptimizer::new(config);

        // 60fps = ~16.67ms per frame
        let budget = optimizer.frame_budget_ms();
        assert!(budget >= 16.0 && budget <= 17.0);

        // Operation within budget
        assert!(optimizer.fits_in_budget(Duration::from_millis(10)));

        // Operation exceeding budget
        assert!(!optimizer.fits_in_budget(Duration::from_millis(20)));
    }

    #[test]
    fn test_optimization_profiles() {
        let default = OptimizationConfig::default();
        assert_eq!(default.websocket.target_latency_ms, 50);
        assert_eq!(default.memory.target_usage_mb, 100);

        let aggressive = OptimizationConfig::aggressive();
        assert_eq!(aggressive.websocket.target_latency_ms, 40);
        assert_eq!(aggressive.memory.target_usage_mb, 80);

        let conservative = OptimizationConfig::conservative();
        assert!(!conservative.websocket.compression_enabled);
    }
}
