# Performance Optimization Guide

## Overview

This guide documents the performance optimization system for mBot RuVector, targeting significant improvements in latency, memory usage, and rendering performance.

## Performance Targets

| Metric | Baseline | Target | Improvement |
|--------|----------|--------|-------------|
| WebSocket Latency (p99) | 80ms | <50ms | 37.5% |
| Memory Usage | 120MB | <100MB | 20% |
| UI Frame Rate | 45fps | 60fps | 33% |
| Drawing Playback | Variable | 60fps @ 1000+ strokes | - |

## Invariants

### I-PERF-001: WebSocket Latency
**Requirement:** WebSocket latency must be <50ms at p99

**Why:** Real-time control requires low-latency communication between companion and robot.

**How to Meet:**
- Enable message batching (batch_size: 10-20)
- Use compression for large payloads
- Implement connection pooling
- Optimize message serialization

**Testing:**
```rust
let mut latency_metrics = LatencyMetrics::new(1000);
// Record 1000 samples
assert!(latency_metrics.p99().as_millis() < 50);
```

### I-PERF-002: UI Frame Rate
**Requirement:** UI must maintain 60fps during all interactions

**Why:** Smooth UI ensures good user experience and real-time feedback.

**How to Meet:**
- Memoize expensive React components
- Use offscreen canvas for rendering
- Optimize canvas operations
- Implement request animation frame

**Testing:**
```rust
let render_metrics = profiler.get_render_metrics();
assert!(render_metrics.maintains_60fps());
```

### I-PERF-003: Memory Usage
**Requirement:** Memory usage must not exceed 100MB

**Why:** Prevents memory leaks and ensures stable long-term operation.

**How to Meet:**
- Apply quantization (50-75% reduction)
- Implement object pooling
- Regular garbage collection
- Fix event listener leaks

**Testing:**
```rust
let optimizer = MemoryOptimizer::new(config);
assert!(optimizer.current_usage_mb() <= 100.0);
```

### I-PERF-004: Drawing Playback
**Requirement:** Smooth 60fps playback for 1000+ strokes

**Why:** Complex drawings need smooth animation for good UX.

**How to Meet:**
- Use offscreen canvas pre-rendering
- Optimize stroke rendering algorithms
- Enable WebGL for complex scenes
- Reduce unnecessary redraws

**Testing:**
```rust
let render_metrics = profiler.get_render_metrics();
assert!(render_metrics.smooth_for_strokes(1200));
```

## Architecture

### Performance Module Structure

```
mbot-core/src/performance/
├── mod.rs              # Module exports and high-level API
├── metrics.rs          # Performance metrics collection
├── profiler.rs         # Profiling utilities
├── optimizations.rs    # Optimization implementations
├── benchmarks.rs       # Benchmark suites
└── monitor.rs          # Real-time monitoring
```

### Key Components

#### 1. PerformanceProfiler
Tracks operation latency, frame rate, and provides RAII-based scoped profiling.

```rust
let profiler = PerformanceProfiler::new();

// Method 1: Manual profiling
profiler.start_profiling("operation");
// ... do work ...
let report = profiler.stop_profiling("operation");

// Method 2: Scope-based (RAII)
{
    let _scope = ProfileScope::new(&profiler, "operation");
    // ... do work ...
} // Automatically stops profiling

// Method 3: Latency measurement
let result = profiler.measure_latency("op", || {
    // ... do work ...
    42
});
```

#### 2. WebSocketOptimizer
Batches messages to reduce round trips and latency.

```rust
let config = WebSocketConfig {
    batching_enabled: true,
    batch_size: 10,
    batch_delay_ms: 16,
    compression_enabled: true,
    target_latency_ms: 50,
};

let optimizer = WebSocketOptimizer::new(config);

// Queue messages
if let Some(batch) = optimizer.queue_message(msg) {
    // Batch ready, send it
    send_websocket_batch(batch);
}

// Force flush
let remaining = optimizer.flush();
```

#### 3. MemoryOptimizer
Enforces memory limits and tracks allocation.

```rust
let optimizer = MemoryOptimizer::new(MemoryConfig {
    max_cache_size_mb: 100,
    gc_interval_ms: 5000,
    enable_object_pooling: true,
    target_usage_mb: 100,
});

// Check before allocating
if optimizer.can_allocate(size_bytes) {
    optimizer.allocate(size_bytes);
    // ... use memory ...
    optimizer.deallocate(size_bytes);
}

// Monitor usage
let usage_mb = optimizer.current_usage_mb();
```

#### 4. RenderOptimizer
Ensures rendering stays within frame budget.

```rust
let optimizer = RenderOptimizer::new(RenderConfig {
    use_offscreen_canvas: true,
    enable_webgl: false,
    use_request_animation_frame: true,
    target_fps: 60,
});

let frame_budget = optimizer.frame_budget_ms(); // ~16.67ms for 60fps

// Check if operation fits
if optimizer.fits_in_budget(operation_duration) {
    // Safe to render
}
```

#### 5. PerformanceMonitor
Continuous monitoring with alerting.

```rust
let profiler = Arc::new(PerformanceProfiler::new());
let monitor = PerformanceMonitor::new(profiler.clone());

monitor.start();

// Periodically collect metrics
let metrics = monitor.collect_metrics();

// Check for violations
monitor.check_alerts(&metrics);

// Get alerts
let alerts = monitor.get_alerts();
for alert in alerts {
    match alert.severity {
        AlertSeverity::High => eprintln!("HIGH: {}", alert.message),
        _ => println!("INFO: {}", alert.message),
    }
}

// Generate report
let report = monitor.generate_report();
if !report.is_healthy() {
    eprintln!("Performance degradation detected!");
}
```

## Optimization Strategies

### WebSocket Optimization

**Message Batching:**
- Reduces round trips by combining multiple messages
- Configurable batch size and delay
- Automatic flushing when batch size or delay reached

**Benefits:**
- 30-50% latency reduction
- Increased throughput
- Lower overhead

**Implementation:**
```rust
// Enable batching
let config = WebSocketConfig {
    batching_enabled: true,
    batch_size: 10,
    batch_delay_ms: 16,
    ..Default::default()
};
```

### Memory Optimization

**Quantization:**
- Reduces memory by 50-75%
- Convert float32 to int8/int4
- Negligible quality loss for sensor data

**Object Pooling:**
- Reuse allocated objects
- Reduces GC pressure
- Faster allocation

**Garbage Collection:**
- Scheduled GC to prevent pauses
- Incremental collection
- Idle-time collection

**Implementation:**
```rust
let config = MemoryConfig {
    max_cache_size_mb: 100,
    gc_interval_ms: 5000,
    enable_object_pooling: true,
    target_usage_mb: 100,
};
```

### Rendering Optimization

**Offscreen Canvas:**
- Pre-render complex scenes
- Reduce main thread work
- Better parallelization

**Memoization:**
- Cache expensive computations
- React.memo for components
- useMemo for calculations

**WebGL Acceleration:**
- GPU-accelerated rendering
- Better for complex drawings
- Maintain 60fps with 1000+ strokes

**Implementation:**
```rust
let config = RenderConfig {
    use_offscreen_canvas: true,
    enable_webgl: true,
    use_request_animation_frame: true,
    target_fps: 60,
};
```

## Benchmarking

### Running Benchmarks

```bash
# Run all performance tests
cargo test --test performance_tests

# Run benchmarks only
cargo test --test performance_tests --release

# Run soak test (10-minute stability)
cargo test --test performance_tests --ignored -- test_memory_soak
```

### Pre-defined Benchmark Suites

```rust
// WebSocket latency
let suite = benchmarks::websocket_latency_benchmark();
let report = suite.run_with_report();

// Memory usage
let suite = benchmarks::memory_usage_benchmark();
let report = suite.run_with_report();

// Rendering performance
let suite = benchmarks::rendering_performance_benchmark();
let report = suite.run_with_report();

// Run all
let suites = benchmarks::full_performance_benchmark();
for suite in suites {
    let report = suite.run_with_report();
    println!("{}", report.summary());
}
```

### Creating Custom Benchmarks

```rust
let mut suite = BenchmarkSuite::new("Custom Suite");

suite.add_test(
    BenchmarkTest::new("my_operation", 1000)
        .with_max_duration(10.0)  // 10ms max
        .with_min_ops_per_sec(100.0)
);

let report = suite.run_with_report();
assert!(report.all_passed());
```

## Monitoring

### Real-time Monitoring

```rust
let profiler = Arc::new(PerformanceProfiler::new());
let monitor = PerformanceMonitor::new(profiler.clone());

monitor.start();

// In your main loop
loop {
    // Do work...

    // Periodically check performance
    if tick_count % 100 == 0 {
        let metrics = monitor.collect_metrics();

        // Check invariants
        if let Err(violations) = metrics.meets_invariants() {
            for violation in violations {
                eprintln!("VIOLATION: {}", violation);
            }
        }

        // Check alerts
        monitor.check_alerts(&metrics);
    }
}
```

### Metrics Collection

```rust
let metrics = monitor.collect_metrics();

println!("WebSocket p99: {:.1}ms", metrics.websocket.latency_p99_ms);
println!("UI FPS: {:.1}", metrics.ui.frame_rate);
println!("Memory: {:.1}MB", metrics.memory.heap_used_mb);
```

### Alerting

```rust
let alerts = monitor.get_alerts();

for alert in alerts {
    match alert.category {
        AlertCategory::Latency => {
            // Handle latency alert
            eprintln!("Latency alert: {}", alert.message);
        }
        AlertCategory::Memory => {
            // Handle memory alert
            eprintln!("Memory alert: {}", alert.message);
        }
        AlertCategory::Rendering => {
            // Handle rendering alert
            eprintln!("Rendering alert: {}", alert.message);
        }
        _ => {}
    }
}
```

## Optimization Profiles

### Default Profile
Balanced optimization for general use.

```rust
let config = OptimizationConfig::default();
```

### Aggressive Profile
Maximum performance, higher resource usage.

```rust
let config = OptimizationConfig::aggressive();
// - Smaller batches (8ms delay)
// - More aggressive memory limits (80MB)
// - WebGL enabled
```

### Conservative Profile
Minimal optimization, maximum compatibility.

```rust
let config = OptimizationConfig::conservative();
// - Larger batches (32ms delay)
// - Higher memory limits (100MB)
// - No compression, no WebGL
```

## Best Practices

### 1. Profile Before Optimizing
Always measure baseline performance before applying optimizations.

```rust
// Measure baseline
let profiler = PerformanceProfiler::new();
let baseline = profiler.measure_latency("operation", || {
    do_work();
});

// Apply optimization
optimize();

// Measure optimized
let optimized = profiler.measure_latency("operation", || {
    do_work();
});

// Calculate improvement
let improvement = (baseline - optimized) / baseline * 100.0;
```

### 2. Test Continuously
Run performance tests in CI to catch regressions.

```bash
# In CI pipeline
cargo test --test performance_tests --release
```

### 3. Monitor in Production
Deploy with monitoring enabled to catch real-world issues.

```rust
// Enable monitoring in production
let monitor = PerformanceMonitor::new(profiler);
monitor.start();
```

### 4. Document Optimizations
Keep track of what optimizations were applied and why.

### 5. Validate Invariants
Always check that optimizations don't violate performance invariants.

```rust
let metrics = monitor.collect_metrics();
assert!(metrics.meets_invariants().is_ok());
```

## Troubleshooting

### High WebSocket Latency
**Symptoms:** p99 > 50ms

**Solutions:**
1. Enable batching
2. Increase batch size
3. Check network conditions
4. Optimize message serialization

### Low Frame Rate
**Symptoms:** FPS < 60

**Solutions:**
1. Enable offscreen canvas
2. Memoize React components
3. Reduce render complexity
4. Use WebGL for complex scenes

### High Memory Usage
**Symptoms:** Heap > 100MB

**Solutions:**
1. Apply quantization
2. Enable object pooling
3. Fix memory leaks
4. Increase GC frequency

### Slow Drawing Playback
**Symptoms:** Stuttering with 1000+ strokes

**Solutions:**
1. Use offscreen canvas
2. Simplify stroke rendering
3. Enable GPU acceleration
4. Reduce stroke complexity

## References

- Performance Contract: `docs/contracts/feature_performance.yml`
- Tests: `tests/performance/performance_tests.rs`
- Source: `crates/mbot-core/src/performance/`
- Issue: #90 (STORY-PERF-001)
