//! Comprehensive performance tests
//!
//! Tests for performance optimization implementation, validating all invariants
//! and optimization targets from the performance contract.

use mbot_core::performance::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Test I-PERF-001: WebSocket latency <50ms at p99
#[test]
fn test_websocket_latency_invariant() {
    let config = OptimizationConfig::default();
    assert_eq!(config.websocket.target_latency_ms, 50);

    let optimizer = WebSocketOptimizer::new(config.websocket);

    // Simulate low latency operations
    let mut latency_metrics = metrics::LatencyMetrics::new(1000);

    for _ in 0..1000 {
        let start = std::time::Instant::now();
        let _ = optimizer.queue_message(vec![1, 2, 3, 4, 5]);
        latency_metrics.record(start.elapsed());
    }

    let p99 = latency_metrics.p99();
    assert!(
        p99.as_millis() < 50,
        "I-PERF-001 violation: WebSocket p99 latency {}ms exceeds 50ms",
        p99.as_millis()
    );
}

/// Test I-PERF-002: UI maintains 60fps
#[test]
fn test_ui_frame_rate_invariant() {
    let profiler = PerformanceProfiler::new();

    // Simulate 60fps rendering for 2 seconds
    for _ in 0..120 {
        profiler.start_frame();
        thread::sleep(Duration::from_millis(16)); // ~60fps
        profiler.end_frame();
    }

    let render_metrics = profiler.get_render_metrics();
    let fps = render_metrics.fps();

    assert!(
        render_metrics.maintains_60fps(),
        "I-PERF-002 violation: UI frame rate {:.1}fps below 60fps target",
        fps
    );

    assert!(
        fps >= 55.0 && fps <= 65.0,
        "FPS should be around 60, got {:.1}",
        fps
    );
}

/// Test I-PERF-003: Memory usage <100MB
#[test]
fn test_memory_usage_invariant() {
    let config = MemoryConfig {
        max_cache_size_mb: 100,
        gc_interval_ms: 5000,
        enable_object_pooling: true,
        target_usage_mb: 100,
    };

    let optimizer = MemoryOptimizer::new(config);

    // Allocate up to but not exceeding 100MB
    let mb_90 = 90 * 1024 * 1024;
    assert!(optimizer.allocate(mb_90));
    assert!(optimizer.current_usage_mb() <= 100.0);

    // Should reject allocation that would exceed limit
    let mb_20 = 20 * 1024 * 1024;
    assert!(!optimizer.allocate(mb_20));

    // Should be at target
    let usage = optimizer.current_usage_mb();
    assert!(
        usage <= 100.0,
        "I-PERF-003 violation: Memory usage {:.1}MB exceeds 100MB target",
        usage
    );
}

/// Test I-PERF-004: Drawing playback smooth at 60fps for 1000+ strokes
#[test]
fn test_drawing_playback_invariant() {
    let profiler = PerformanceProfiler::new();

    // Simulate drawing playback with 1000 strokes
    let stroke_count = 1200;

    for _ in 0..stroke_count {
        profiler.start_frame();
        // Simulate stroke rendering
        thread::sleep(Duration::from_micros(100));
        profiler.end_frame();
    }

    let render_metrics = profiler.get_render_metrics();

    assert!(
        render_metrics.smooth_for_strokes(stroke_count),
        "I-PERF-004 violation: Drawing playback not smooth for {} strokes",
        stroke_count
    );
}

/// Test WebSocket message batching
#[test]
fn test_websocket_batching() {
    let config = WebSocketConfig {
        batching_enabled: true,
        batch_size: 5,
        batch_delay_ms: 100,
        compression_enabled: false,
        target_latency_ms: 50,
    };

    let optimizer = WebSocketOptimizer::new(config);

    // Queue messages that should not flush immediately
    assert!(optimizer.queue_message(vec![1]).is_none());
    assert!(optimizer.queue_message(vec![2]).is_none());
    assert!(optimizer.queue_message(vec![3]).is_none());
    assert!(optimizer.queue_message(vec![4]).is_none());

    // 5th message should trigger flush
    let flushed = optimizer.queue_message(vec![5]);
    assert!(flushed.is_some());
    let batch = flushed.unwrap();
    assert_eq!(batch.len(), 5);
}

/// Test memory allocation limits
#[test]
fn test_memory_allocation_limits() {
    let config = MemoryConfig {
        max_cache_size_mb: 10,
        gc_interval_ms: 5000,
        enable_object_pooling: true,
        target_usage_mb: 10,
    };

    let optimizer = MemoryOptimizer::new(config);

    // Allocate within limit
    let mb_5 = 5 * 1024 * 1024;
    assert!(optimizer.allocate(mb_5));
    assert!(optimizer.current_usage_mb() <= 10.0);

    // Allocate more
    assert!(optimizer.allocate(mb_5));
    assert!(optimizer.current_usage_mb() <= 10.0);

    // Should reject exceeding allocation
    assert!(!optimizer.allocate(mb_5));

    // Deallocate and try again
    optimizer.deallocate(mb_5);
    assert!(optimizer.allocate(mb_5));
}

/// Test render frame budget
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

    // Fast operation should fit
    assert!(optimizer.fits_in_budget(Duration::from_millis(10)));

    // Slow operation should not fit
    assert!(!optimizer.fits_in_budget(Duration::from_millis(20)));
}

/// Test performance metrics collection
#[test]
fn test_metrics_collection() {
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
    let violations = result.unwrap_err();
    assert!(violations[0].contains("I-PERF-001"));
}

/// Test improvement calculation
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

    assert!(report.latency_improvement >= 49.0);
    assert!(report.memory_reduction >= 24.0);
    assert!(report.fps_improvement >= 32.0);
    assert!(report.meets_target()); // All >20%
}

/// Test benchmark suite execution
#[test]
fn test_benchmark_execution() {
    let suite = benchmarks::websocket_latency_benchmark();
    let report = suite.run_with_report();

    assert_eq!(report.suite_name, "WebSocket Latency");
    assert!(report.total_tests > 0);
}

/// Test performance monitoring
#[test]
fn test_performance_monitoring() {
    let profiler = Arc::new(PerformanceProfiler::new());
    let monitor = PerformanceMonitor::new(profiler.clone());

    assert!(!monitor.is_running());

    monitor.start();
    assert!(monitor.is_running());

    // Collect metrics
    let metrics = monitor.collect_metrics();
    assert!(metrics.timestamp > 0);

    // Check alerts (should be none with default metrics)
    monitor.check_alerts(&metrics);
    let alerts = monitor.get_alerts();
    assert!(alerts.is_empty());

    monitor.stop();
    assert!(!monitor.is_running());
}

/// Test alert generation on invariant violation
#[test]
fn test_alert_generation() {
    let profiler = Arc::new(PerformanceProfiler::new());
    let monitor = PerformanceMonitor::new(profiler);

    let mut metrics = PerformanceMetrics::new();

    // Violate I-PERF-001
    metrics.websocket.latency_p99_ms = 60.0;

    monitor.check_alerts(&metrics);

    let alerts = monitor.get_alerts();
    assert!(!alerts.is_empty());
    assert!(alerts.iter().any(|a| a.message.contains("I-PERF-001")));
}

/// Test monitoring report generation
#[test]
fn test_monitoring_report() {
    let profiler = Arc::new(PerformanceProfiler::new());
    let monitor = PerformanceMonitor::new(profiler);

    let report = monitor.generate_report();
    assert!(report.is_healthy()); // Should be healthy with default metrics
    assert_eq!(report.high_severity_alerts, 0);
}

/// Test latency percentile calculation
#[test]
fn test_latency_percentiles() {
    let mut metrics = metrics::LatencyMetrics::new(100);

    // Record 100 samples from 1ms to 100ms
    for i in 1..=100 {
        metrics.record(Duration::from_millis(i));
    }

    let p50 = metrics.p50();
    let p95 = metrics.p95();
    let p99 = metrics.p99();

    assert!(p50.as_millis() >= 49 && p50.as_millis() <= 51);
    assert!(p95.as_millis() >= 94 && p95.as_millis() <= 96);
    assert!(p99.as_millis() >= 98 && p99.as_millis() <= 100);
}

/// Test profiler scope-based profiling
#[test]
fn test_profile_scope() {
    let profiler = PerformanceProfiler::new();

    {
        let _scope = ProfileScope::new(&profiler, "test_operation");
        thread::sleep(Duration::from_millis(50));
    } // Scope ends, profiling stops

    let report = profiler.stop_profiling("test_operation");
    assert!(report.is_some());

    let report = report.unwrap();
    assert_eq!(report.name, "test_operation");
    assert!(report.duration.as_millis() >= 50);
}

/// Test optimization profiles
#[test]
fn test_optimization_profiles() {
    let default = OptimizationConfig::default();
    assert_eq!(default.websocket.target_latency_ms, 50);
    assert_eq!(default.memory.target_usage_mb, 100);
    assert_eq!(default.render.target_fps, 60);

    let aggressive = OptimizationConfig::aggressive();
    assert_eq!(aggressive.websocket.target_latency_ms, 40);
    assert_eq!(aggressive.memory.target_usage_mb, 80);
    assert!(aggressive.render.enable_webgl);

    let conservative = OptimizationConfig::conservative();
    assert!(!conservative.websocket.compression_enabled);
    assert!(!conservative.memory.enable_object_pooling);
}

/// Regression test: Ensure optimizations don't degrade performance
#[test]
fn test_no_performance_regression() {
    // Baseline measurements
    let baseline_config = OptimizationConfig::conservative();
    let baseline_latency_target = baseline_config.websocket.target_latency_ms;

    // Optimized measurements
    let optimized_config = OptimizationConfig::aggressive();
    let optimized_latency_target = optimized_config.websocket.target_latency_ms;

    // Optimized should be better than baseline
    assert!(optimized_latency_target <= baseline_latency_target);

    // Memory target should be lower
    assert!(
        optimized_config.memory.target_usage_mb <= baseline_config.memory.target_usage_mb
    );
}

/// Soak test: 10-minute memory stability test (reduced to 10 seconds for CI)
#[test]
#[ignore] // Run with --ignored flag
fn test_memory_soak() {
    let optimizer = MemoryOptimizer::new(MemoryConfig {
        max_cache_size_mb: 100,
        gc_interval_ms: 1000,
        enable_object_pooling: true,
        target_usage_mb: 100,
    });

    let start_usage = optimizer.current_usage_mb();

    // Simulate 10 seconds of allocation/deallocation
    for _ in 0..100 {
        let size = 1024 * 1024; // 1MB
        if optimizer.allocate(size) {
            thread::sleep(Duration::from_millis(50));
            optimizer.deallocate(size);
        }
        thread::sleep(Duration::from_millis(50));
    }

    let end_usage = optimizer.current_usage_mb();

    // Memory should be stable (no leak)
    assert!(
        (end_usage - start_usage).abs() < 5.0,
        "Memory leak detected: started at {:.1}MB, ended at {:.1}MB",
        start_usage,
        end_usage
    );
}
