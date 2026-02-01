//! Performance metrics collection and reporting
//!
//! Provides data structures and utilities for tracking performance metrics
//! across WebSocket, memory, rendering, and network operations.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Comprehensive performance metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: u64,
    pub websocket: WebSocketMetrics,
    pub ui: UIMetrics,
    pub memory: MemoryMetrics,
    pub network: NetworkMetrics,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            websocket: WebSocketMetrics::default(),
            ui: UIMetrics::default(),
            memory: MemoryMetrics::default(),
            network: NetworkMetrics::default(),
        }
    }

    /// Check if metrics meet performance invariants
    pub fn meets_invariants(&self) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();

        // I-PERF-001: WebSocket latency must be <50ms at p99
        if self.websocket.latency_p99_ms > 50.0 {
            violations.push(format!(
                "I-PERF-001 violation: WebSocket p99 latency {}ms exceeds 50ms target",
                self.websocket.latency_p99_ms
            ));
        }

        // I-PERF-002: UI must maintain 60fps
        if self.ui.frame_rate < 60.0 {
            violations.push(format!(
                "I-PERF-002 violation: UI frame rate {}fps below 60fps target",
                self.ui.frame_rate
            ));
        }

        // I-PERF-003: Memory usage must not exceed 100MB
        if self.memory.heap_used_mb > 100.0 {
            violations.push(format!(
                "I-PERF-003 violation: Memory usage {}MB exceeds 100MB target",
                self.memory.heap_used_mb
            ));
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    /// Calculate improvement percentage from baseline
    pub fn improvement_from_baseline(&self, baseline: &PerformanceMetrics) -> ImprovementReport {
        ImprovementReport {
            latency_improvement: ((baseline.websocket.latency_p99_ms - self.websocket.latency_p99_ms)
                / baseline.websocket.latency_p99_ms
                * 100.0),
            memory_reduction: ((baseline.memory.heap_used_mb - self.memory.heap_used_mb)
                / baseline.memory.heap_used_mb
                * 100.0),
            fps_improvement: ((self.ui.frame_rate - baseline.ui.frame_rate) / baseline.ui.frame_rate
                * 100.0),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMetrics {
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub message_rate_per_sec: f64,
    pub throughput_bytes_per_sec: f64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub batching_enabled: bool,
    pub avg_batch_size: f64,
}

impl Default for WebSocketMetrics {
    fn default() -> Self {
        Self {
            latency_p50_ms: 0.0,
            latency_p95_ms: 0.0,
            latency_p99_ms: 0.0,
            message_rate_per_sec: 0.0,
            throughput_bytes_per_sec: 0.0,
            messages_sent: 0,
            messages_received: 0,
            batching_enabled: false,
            avg_batch_size: 1.0,
        }
    }
}

/// UI rendering performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIMetrics {
    pub frame_rate: f64,
    pub avg_render_time_ms: f64,
    pub slow_renders: u32, // Count of renders >16ms
    pub dropped_frames: u32,
    pub total_frames: u64,
}

impl Default for UIMetrics {
    fn default() -> Self {
        Self {
            frame_rate: 60.0,
            avg_render_time_ms: 0.0,
            slow_renders: 0,
            dropped_frames: 0,
            total_frames: 0,
        }
    }
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub heap_used_mb: f64,
    pub heap_total_mb: f64,
    pub external_mb: f64,
    pub gc_count: u32,
    pub gc_pause_ms: f64,
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            heap_used_mb: 0.0,
            heap_total_mb: 0.0,
            external_mb: 0.0,
            gc_count: 0,
            gc_pause_ms: 0.0,
        }
    }
}

/// Network and loading metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub page_load_time_ms: f64,
    pub bundle_size_bytes: u64,
    pub cache_hit_rate: f64, // 0.0 to 1.0
    pub requests_total: u32,
    pub requests_cached: u32,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            page_load_time_ms: 0.0,
            bundle_size_bytes: 0,
            cache_hit_rate: 0.0,
            requests_total: 0,
            requests_cached: 0,
        }
    }
}

/// Latency tracking for individual operations
#[derive(Debug, Clone)]
pub struct LatencyMetrics {
    samples: Vec<Duration>,
    max_samples: usize,
}

impl LatencyMetrics {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn record(&mut self, duration: Duration) {
        if self.samples.len() >= self.max_samples {
            self.samples.remove(0);
        }
        self.samples.push(duration);
    }

    pub fn percentile(&self, p: f64) -> Duration {
        if self.samples.is_empty() {
            return Duration::from_secs(0);
        }

        let mut sorted = self.samples.clone();
        sorted.sort();

        let index = ((p / 100.0) * (sorted.len() - 1) as f64) as usize;
        sorted[index]
    }

    pub fn p50(&self) -> Duration {
        self.percentile(50.0)
    }

    pub fn p95(&self) -> Duration {
        self.percentile(95.0)
    }

    pub fn p99(&self) -> Duration {
        self.percentile(99.0)
    }

    pub fn avg(&self) -> Duration {
        if self.samples.is_empty() {
            return Duration::from_secs(0);
        }

        let sum: Duration = self.samples.iter().sum();
        sum / self.samples.len() as u32
    }
}

impl Default for LatencyMetrics {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Rendering performance tracking
#[derive(Debug, Clone)]
pub struct RenderMetrics {
    frame_times: Vec<Duration>,
    frame_start: Option<Instant>,
    max_samples: usize,
}

impl RenderMetrics {
    pub fn new(max_samples: usize) -> Self {
        Self {
            frame_times: Vec::with_capacity(max_samples),
            frame_start: None,
            max_samples,
        }
    }

    pub fn start_frame(&mut self) {
        self.frame_start = Some(Instant::now());
    }

    pub fn end_frame(&mut self) {
        if let Some(start) = self.frame_start {
            let duration = start.elapsed();
            if self.frame_times.len() >= self.max_samples {
                self.frame_times.remove(0);
            }
            self.frame_times.push(duration);
            self.frame_start = None;
        }
    }

    pub fn fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let avg_frame_time = self.avg_frame_time();
        if avg_frame_time.as_secs_f64() == 0.0 {
            return 0.0;
        }

        1.0 / avg_frame_time.as_secs_f64()
    }

    pub fn avg_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::from_secs(0);
        }

        let sum: Duration = self.frame_times.iter().sum();
        sum / self.frame_times.len() as u32
    }

    pub fn slow_frame_count(&self, threshold: Duration) -> usize {
        self.frame_times.iter().filter(|&&t| t > threshold).count()
    }

    /// I-PERF-002: Check if maintaining 60fps
    pub fn maintains_60fps(&self) -> bool {
        self.fps() >= 60.0
    }

    /// I-PERF-004: Check if smooth for 1000+ strokes
    pub fn smooth_for_strokes(&self, stroke_count: usize) -> bool {
        if stroke_count < 1000 {
            return true;
        }

        self.maintains_60fps() && self.slow_frame_count(Duration::from_millis(16)) < 10
    }
}

impl Default for RenderMetrics {
    fn default() -> Self {
        Self::new(120) // Track last 2 seconds at 60fps
    }
}

/// Performance improvement report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementReport {
    pub latency_improvement: f64,   // Percentage
    pub memory_reduction: f64,      // Percentage
    pub fps_improvement: f64,       // Percentage
}

impl ImprovementReport {
    /// Check if meets 20% improvement target
    pub fn meets_target(&self) -> bool {
        self.latency_improvement >= 20.0
            && self.memory_reduction >= 20.0
            && self.fps_improvement >= 20.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_percentiles() {
        let mut metrics = LatencyMetrics::new(100);

        // Record samples from 10ms to 100ms
        for i in 1..=100 {
            metrics.record(Duration::from_millis(i));
        }

        assert_eq!(metrics.p50().as_millis(), 50);
        assert!(metrics.p95().as_millis() >= 95);
        assert!(metrics.p99().as_millis() >= 99);
    }

    #[test]
    fn test_render_fps_calculation() {
        let mut metrics = RenderMetrics::new(60);

        // Simulate 60fps (16.67ms per frame)
        for _ in 0..60 {
            metrics.start_frame();
            std::thread::sleep(Duration::from_millis(16));
            metrics.end_frame();
        }

        let fps = metrics.fps();
        assert!(fps >= 55.0 && fps <= 65.0, "FPS should be around 60, got {}", fps);
    }

    #[test]
    fn test_invariant_checking() {
        let mut metrics = PerformanceMetrics::new();

        // Set metrics that meet invariants
        metrics.websocket.latency_p99_ms = 45.0;
        metrics.ui.frame_rate = 60.0;
        metrics.memory.heap_used_mb = 95.0;

        assert!(metrics.meets_invariants().is_ok());

        // Violate I-PERF-001
        metrics.websocket.latency_p99_ms = 55.0;
        let result = metrics.meets_invariants();
        assert!(result.is_err());
        assert!(result.unwrap_err()[0].contains("I-PERF-001"));
    }

    #[test]
    fn test_improvement_calculation() {
        let baseline = PerformanceMetrics {
            timestamp: 0,
            websocket: WebSocketMetrics {
                latency_p99_ms: 80.0,
                ..Default::default()
            },
            memory: MemoryMetrics {
                heap_used_mb: 120.0,
                ..Default::default()
            },
            ui: UIMetrics {
                frame_rate: 45.0,
                ..Default::default()
            },
            network: NetworkMetrics::default(),
        };

        let optimized = PerformanceMetrics {
            timestamp: 0,
            websocket: WebSocketMetrics {
                latency_p99_ms: 40.0, // 50% improvement
                ..Default::default()
            },
            memory: MemoryMetrics {
                heap_used_mb: 90.0, // 25% reduction
                ..Default::default()
            },
            ui: UIMetrics {
                frame_rate: 60.0, // 33% improvement
                ..Default::default()
            },
            network: NetworkMetrics::default(),
        };

        let report = optimized.improvement_from_baseline(&baseline);
        assert!(report.latency_improvement >= 49.0 && report.latency_improvement <= 51.0);
        assert!(report.memory_reduction >= 24.0 && report.memory_reduction <= 26.0);
        assert!(report.meets_target());
    }
}
