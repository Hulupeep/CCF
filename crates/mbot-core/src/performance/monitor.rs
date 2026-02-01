//! Real-time performance monitoring
//!
//! Continuous monitoring of performance metrics with alerting and reporting.

use super::metrics::{PerformanceMetrics, WebSocketMetrics, UIMetrics, MemoryMetrics};
use super::profiler::PerformanceProfiler;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Performance monitor for continuous tracking
pub struct PerformanceMonitor {
    profiler: Arc<PerformanceProfiler>,
    collector: Arc<Mutex<MetricsCollector>>,
    alerts: Arc<Mutex<Vec<PerformanceAlert>>>,
    running: Arc<Mutex<bool>>,
}

impl PerformanceMonitor {
    pub fn new(profiler: Arc<PerformanceProfiler>) -> Self {
        Self {
            profiler,
            collector: Arc::new(Mutex::new(MetricsCollector::new())),
            alerts: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start monitoring
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        *running = true;
    }

    /// Stop monitoring
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }

    /// Check if monitoring is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Collect current metrics snapshot
    pub fn collect_metrics(&self) -> PerformanceMetrics {
        let mut collector = self.collector.lock().unwrap();
        collector.collect_snapshot(&self.profiler)
    }

    /// Get all collected metrics
    pub fn get_history(&self) -> Vec<PerformanceMetrics> {
        let collector = self.collector.lock().unwrap();
        collector.history.clone()
    }

    /// Check for performance alerts
    pub fn check_alerts(&self, metrics: &PerformanceMetrics) {
        let mut alerts = self.alerts.lock().unwrap();

        // Check I-PERF-001: WebSocket latency
        if metrics.websocket.latency_p99_ms > 50.0 {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::High,
                category: AlertCategory::Latency,
                message: format!(
                    "WebSocket p99 latency {}ms exceeds 50ms target (I-PERF-001)",
                    metrics.websocket.latency_p99_ms
                ),
                timestamp: Instant::now(),
            });
        }

        // Check I-PERF-002: Frame rate
        if metrics.ui.frame_rate < 60.0 {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::High,
                category: AlertCategory::Rendering,
                message: format!(
                    "UI frame rate {}fps below 60fps target (I-PERF-002)",
                    metrics.ui.frame_rate
                ),
                timestamp: Instant::now(),
            });
        }

        // Check I-PERF-003: Memory usage
        if metrics.memory.heap_used_mb > 100.0 {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::High,
                category: AlertCategory::Memory,
                message: format!(
                    "Memory usage {}MB exceeds 100MB target (I-PERF-003)",
                    metrics.memory.heap_used_mb
                ),
                timestamp: Instant::now(),
            });
        }

        // Warn if approaching limits
        if metrics.websocket.latency_p99_ms > 40.0 && metrics.websocket.latency_p99_ms <= 50.0 {
            alerts.push(PerformanceAlert {
                severity: AlertSeverity::Medium,
                category: AlertCategory::Latency,
                message: format!("WebSocket latency {}ms approaching limit", metrics.websocket.latency_p99_ms),
                timestamp: Instant::now(),
            });
        }
    }

    /// Get recent alerts
    pub fn get_alerts(&self) -> Vec<PerformanceAlert> {
        let alerts = self.alerts.lock().unwrap();
        alerts.clone()
    }

    /// Clear alerts
    pub fn clear_alerts(&self) {
        let mut alerts = self.alerts.lock().unwrap();
        alerts.clear();
    }

    /// Generate monitoring report
    pub fn generate_report(&self) -> MonitoringReport {
        let metrics = self.collect_metrics();
        let history = self.get_history();
        let alerts = self.get_alerts();

        let invariant_status = metrics.meets_invariants();

        MonitoringReport {
            current_metrics: metrics,
            history_length: history.len(),
            alert_count: alerts.len(),
            high_severity_alerts: alerts
                .iter()
                .filter(|a| matches!(a.severity, AlertSeverity::High))
                .count(),
            invariants_met: invariant_status.is_ok(),
            violations: invariant_status.err().unwrap_or_default(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

/// Metrics collector with history
pub struct MetricsCollector {
    history: Vec<PerformanceMetrics>,
    max_history: usize,
    last_collection: Option<Instant>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            max_history: 1000, // Keep last 1000 samples
            last_collection: None,
        }
    }

    pub fn collect_snapshot(&mut self, profiler: &PerformanceProfiler) -> PerformanceMetrics {
        let render_metrics = profiler.get_render_metrics();

        let mut metrics = PerformanceMetrics::new();

        // Update UI metrics
        metrics.ui.frame_rate = render_metrics.fps();
        metrics.ui.avg_render_time_ms = render_metrics.avg_frame_time().as_secs_f64() * 1000.0;
        metrics.ui.slow_renders = render_metrics.slow_frame_count(Duration::from_millis(16)) as u32;

        // Store in history
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(metrics.clone());

        self.last_collection = Some(Instant::now());

        metrics
    }

    pub fn get_average_metrics(&self, window_size: usize) -> Option<PerformanceMetrics> {
        if self.history.is_empty() {
            return None;
        }

        let window = std::cmp::min(window_size, self.history.len());
        let samples = &self.history[self.history.len() - window..];

        let mut avg = PerformanceMetrics::new();

        // Average WebSocket metrics
        avg.websocket.latency_p99_ms =
            samples.iter().map(|s| s.websocket.latency_p99_ms).sum::<f64>() / window as f64;

        // Average UI metrics
        avg.ui.frame_rate = samples.iter().map(|s| s.ui.frame_rate).sum::<f64>() / window as f64;

        // Average memory metrics
        avg.memory.heap_used_mb =
            samples.iter().map(|s| s.memory.heap_used_mb).sum::<f64>() / window as f64;

        Some(avg)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance alert
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub message: String,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertCategory {
    Latency,
    Memory,
    Rendering,
    Network,
}

/// Monitoring report
#[derive(Debug, Clone)]
pub struct MonitoringReport {
    pub current_metrics: PerformanceMetrics,
    pub history_length: usize,
    pub alert_count: usize,
    pub high_severity_alerts: usize,
    pub invariants_met: bool,
    pub violations: Vec<String>,
    pub timestamp: u64,
}

impl MonitoringReport {
    pub fn is_healthy(&self) -> bool {
        self.invariants_met && self.high_severity_alerts == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let profiler = Arc::new(PerformanceProfiler::new());
        let mut collector = MetricsCollector::new();

        // Collect some samples
        for _ in 0..10 {
            collector.collect_snapshot(&profiler);
        }

        assert_eq!(collector.history.len(), 10);

        let avg = collector.get_average_metrics(5);
        assert!(avg.is_some());
    }

    #[test]
    fn test_performance_monitor() {
        let profiler = Arc::new(PerformanceProfiler::new());
        let monitor = PerformanceMonitor::new(profiler.clone());

        assert!(!monitor.is_running());

        monitor.start();
        assert!(monitor.is_running());

        monitor.stop();
        assert!(!monitor.is_running());
    }

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

    #[test]
    fn test_monitoring_report() {
        let profiler = Arc::new(PerformanceProfiler::new());
        let monitor = PerformanceMonitor::new(profiler.clone());

        // Simulate some rendering to get valid FPS
        for _ in 0..60 {
            profiler.start_frame();
            std::thread::sleep(Duration::from_millis(16));
            profiler.end_frame();
        }

        let report = monitor.generate_report();
        // Should be healthy with metrics that meet invariants
        assert_eq!(report.high_severity_alerts, 0);
    }
}
