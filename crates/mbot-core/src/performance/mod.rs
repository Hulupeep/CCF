//! Performance profiling and optimization module
//!
//! This module provides comprehensive performance profiling, monitoring, and optimization
//! capabilities for the mBot system. It targets the following performance goals:
//!
//! - WebSocket latency: <50ms p99
//! - Memory usage: <100MB (20% reduction from baseline)
//! - UI frame rate: 60fps sustained
//! - Drawing playback: 60fps for 1000+ strokes
//!
//! # Invariants
//!
//! - I-PERF-001: WebSocket latency must be <50ms at p99
//! - I-PERF-002: UI must maintain 60fps during all interactions
//! - I-PERF-003: Memory usage must not exceed 100MB
//! - I-PERF-004: Drawing playback must be smooth at 60fps for 1000+ strokes

pub mod profiler;
pub mod metrics;
pub mod benchmarks;
pub mod optimizations;
pub mod monitor;

pub use profiler::{PerformanceProfiler, ProfileScope};
pub use metrics::{PerformanceMetrics, MemoryMetrics, LatencyMetrics, RenderMetrics};
pub use benchmarks::{BenchmarkSuite, BenchmarkTest, BenchmarkResult};
pub use optimizations::{OptimizationConfig, WebSocketOptimizer, MemoryOptimizer, RenderOptimizer};
pub use monitor::{PerformanceMonitor, MetricsCollector};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_invariants() {
        // Test that performance targets are achievable
        let config = OptimizationConfig::default();
        assert!(config.websocket.target_latency_ms <= 50);
        assert!(config.memory.target_usage_mb <= 100);
        assert!(config.render.target_fps >= 60);
    }
}
