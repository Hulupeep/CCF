//! Performance benchmarking suite
//!
//! Comprehensive benchmarks for measuring and validating performance improvements.

use super::metrics::PerformanceMetrics;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Benchmark suite containing multiple tests
#[derive(Debug, Clone)]
pub struct BenchmarkSuite {
    pub name: String,
    pub tests: Vec<BenchmarkTest>,
}

impl BenchmarkSuite {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tests: Vec::new(),
        }
    }

    pub fn add_test(&mut self, test: BenchmarkTest) {
        self.tests.push(test);
    }

    /// Run all benchmarks in the suite
    pub fn run(&self) -> Vec<BenchmarkResult> {
        self.tests.iter().map(|test| test.run()).collect()
    }

    /// Run benchmarks and generate report
    pub fn run_with_report(&self) -> BenchmarkReport {
        let results = self.run();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;

        BenchmarkReport {
            suite_name: self.name.clone(),
            total_tests: results.len(),
            passed,
            failed,
            results,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

/// Individual benchmark test
#[derive(Debug, Clone)]
pub struct BenchmarkTest {
    pub name: String,
    pub iterations: usize,
    pub expected_max_duration_ms: Option<f64>,
    pub expected_min_ops_per_sec: Option<f64>,
}

impl BenchmarkTest {
    pub fn new(name: &str, iterations: usize) -> Self {
        Self {
            name: name.to_string(),
            iterations,
            expected_max_duration_ms: None,
            expected_min_ops_per_sec: None,
        }
    }

    pub fn with_max_duration(mut self, max_ms: f64) -> Self {
        self.expected_max_duration_ms = Some(max_ms);
        self
    }

    pub fn with_min_ops_per_sec(mut self, min_ops: f64) -> Self {
        self.expected_min_ops_per_sec = Some(min_ops);
        self
    }

    /// Run the benchmark (placeholder for actual operation)
    pub fn run(&self) -> BenchmarkResult {
        let mut durations = Vec::with_capacity(self.iterations);

        for _ in 0..self.iterations {
            let start = Instant::now();
            // Simulate work - in real implementation, this would be the actual operation
            std::thread::sleep(Duration::from_micros(100));
            durations.push(start.elapsed());
        }

        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / self.iterations as u32;
        let min_duration = *durations.iter().min().unwrap();
        let max_duration = *durations.iter().max().unwrap();

        let ops_per_sec = if avg_duration.as_secs_f64() > 0.0 {
            1.0 / avg_duration.as_secs_f64()
        } else {
            0.0
        };

        // Check pass/fail criteria
        let mut passed = true;
        let mut failure_reasons = Vec::new();

        if let Some(max_ms) = self.expected_max_duration_ms {
            if avg_duration.as_secs_f64() * 1000.0 > max_ms {
                passed = false;
                failure_reasons.push(format!(
                    "Average duration {:.2}ms exceeds max {:.2}ms",
                    avg_duration.as_secs_f64() * 1000.0,
                    max_ms
                ));
            }
        }

        if let Some(min_ops) = self.expected_min_ops_per_sec {
            if ops_per_sec < min_ops {
                passed = false;
                failure_reasons.push(format!(
                    "Ops/sec {:.2} below minimum {:.2}",
                    ops_per_sec, min_ops
                ));
            }
        }

        BenchmarkResult {
            name: self.name.clone(),
            iterations: self.iterations,
            avg_duration_ms: avg_duration.as_secs_f64() * 1000.0,
            min_duration_ms: min_duration.as_secs_f64() * 1000.0,
            max_duration_ms: max_duration.as_secs_f64() * 1000.0,
            ops_per_sec,
            passed,
            failure_reasons,
        }
    }
}

/// Benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub avg_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
    pub ops_per_sec: f64,
    pub passed: bool,
    pub failure_reasons: Vec<String>,
}

/// Complete benchmark report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub suite_name: String,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<BenchmarkResult>,
    pub timestamp: u64,
}

impl BenchmarkReport {
    /// Check if all benchmarks passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "{}: {}/{} passed, {}/{} failed",
            self.suite_name, self.passed, self.total_tests, self.failed, self.total_tests
        )
    }
}

/// Pre-defined benchmark suites

/// WebSocket latency benchmark
pub fn websocket_latency_benchmark() -> BenchmarkSuite {
    let mut suite = BenchmarkSuite::new("WebSocket Latency");

    suite.add_test(
        BenchmarkTest::new("message_send", 1000)
            .with_max_duration(50.0), // I-PERF-001: <50ms p99
    );

    suite.add_test(
        BenchmarkTest::new("message_receive", 1000)
            .with_max_duration(50.0),
    );

    suite.add_test(
        BenchmarkTest::new("round_trip", 100)
            .with_max_duration(100.0),
    );

    suite
}

/// Memory usage benchmark
pub fn memory_usage_benchmark() -> BenchmarkSuite {
    let mut suite = BenchmarkSuite::new("Memory Usage");

    suite.add_test(
        BenchmarkTest::new("allocation_1kb", 10000)
            .with_max_duration(0.01),
    );

    suite.add_test(
        BenchmarkTest::new("allocation_100kb", 1000)
            .with_max_duration(0.1),
    );

    suite.add_test(
        BenchmarkTest::new("deallocation", 10000)
            .with_max_duration(0.01),
    );

    suite
}

/// Rendering performance benchmark
pub fn rendering_performance_benchmark() -> BenchmarkSuite {
    let mut suite = BenchmarkSuite::new("Rendering Performance");

    // I-PERF-002: Must maintain 60fps = 16.67ms per frame
    suite.add_test(
        BenchmarkTest::new("frame_render", 60)
            .with_max_duration(16.0),
    );

    // I-PERF-004: Smooth playback for 1000+ strokes
    suite.add_test(
        BenchmarkTest::new("drawing_1000_strokes", 10)
            .with_max_duration(16.0),
    );

    suite.add_test(
        BenchmarkTest::new("canvas_clear", 1000)
            .with_max_duration(1.0),
    );

    suite
}

/// Comprehensive performance benchmark
pub fn full_performance_benchmark() -> Vec<BenchmarkSuite> {
    vec![
        websocket_latency_benchmark(),
        memory_usage_benchmark(),
        rendering_performance_benchmark(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_execution() {
        let test = BenchmarkTest::new("test_op", 10);
        let result = test.run();

        assert_eq!(result.name, "test_op");
        assert_eq!(result.iterations, 10);
        assert!(result.ops_per_sec > 0.0);
    }

    #[test]
    fn test_benchmark_pass_fail() {
        let test = BenchmarkTest::new("fast_op", 10)
            .with_max_duration(1.0); // 1ms max

        let result = test.run();
        // Should pass since sleep is only 100µs
        assert!(result.passed);

        let test = BenchmarkTest::new("slow_op", 10)
            .with_max_duration(0.01); // 0.01ms max (10µs)

        let result = test.run();
        // Should fail since sleep is 100µs
        assert!(!result.passed);
        assert!(!result.failure_reasons.is_empty());
    }

    #[test]
    fn test_benchmark_suite() {
        let mut suite = BenchmarkSuite::new("Test Suite");
        suite.add_test(BenchmarkTest::new("test1", 10));
        suite.add_test(BenchmarkTest::new("test2", 10));

        let report = suite.run_with_report();
        assert_eq!(report.total_tests, 2);
        assert_eq!(report.suite_name, "Test Suite");
    }

    #[test]
    fn test_predefined_benchmarks() {
        let ws_suite = websocket_latency_benchmark();
        assert_eq!(ws_suite.name, "WebSocket Latency");
        assert!(!ws_suite.tests.is_empty());

        let mem_suite = memory_usage_benchmark();
        assert_eq!(mem_suite.name, "Memory Usage");

        let render_suite = rendering_performance_benchmark();
        assert_eq!(render_suite.name, "Rendering Performance");
    }

    #[test]
    fn test_full_benchmark() {
        let suites = full_performance_benchmark();
        assert_eq!(suites.len(), 3);
    }
}
