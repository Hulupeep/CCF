//! Performance profiling utilities
//!
//! Provides tools for profiling operation latency, memory usage, and rendering performance.

use super::metrics::{PerformanceMetrics, LatencyMetrics, RenderMetrics};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Main performance profiler
pub struct PerformanceProfiler {
    profiles: Arc<Mutex<HashMap<String, ProfileData>>>,
    latency_tracker: Arc<Mutex<HashMap<String, LatencyMetrics>>>,
    render_tracker: Arc<Mutex<RenderMetrics>>,
    enabled: bool,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(Mutex::new(HashMap::new())),
            latency_tracker: Arc::new(Mutex::new(HashMap::new())),
            render_tracker: Arc::new(Mutex::new(RenderMetrics::default())),
            enabled: true,
        }
    }

    /// Start profiling an operation
    pub fn start_profiling(&self, name: &str) {
        if !self.enabled {
            return;
        }

        let mut profiles = self.profiles.lock().unwrap();
        profiles.insert(
            name.to_string(),
            ProfileData {
                name: name.to_string(),
                start_time: Instant::now(),
                end_time: None,
                duration: Duration::from_secs(0),
            },
        );
    }

    /// Stop profiling and return report
    pub fn stop_profiling(&self, name: &str) -> Option<ProfileReport> {
        if !self.enabled {
            return None;
        }

        let mut profiles = self.profiles.lock().unwrap();
        if let Some(profile) = profiles.get_mut(name) {
            profile.end_time = Some(Instant::now());
            profile.duration = profile.end_time.unwrap() - profile.start_time;

            Some(ProfileReport {
                name: profile.name.clone(),
                duration: profile.duration,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            })
        } else {
            None
        }
    }

    /// Measure latency of an operation
    pub fn measure_latency<F, R>(&self, operation_name: &str, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();

        if self.enabled {
            let mut tracker = self.latency_tracker.lock().unwrap();
            tracker
                .entry(operation_name.to_string())
                .or_insert_with(LatencyMetrics::default)
                .record(duration);
        }

        result
    }

    /// Get latency metrics for an operation
    pub fn get_latency_metrics(&self, operation_name: &str) -> Option<LatencyMetrics> {
        let tracker = self.latency_tracker.lock().unwrap();
        tracker.get(operation_name).cloned()
    }

    /// Start rendering frame
    pub fn start_frame(&self) {
        if self.enabled {
            let mut tracker = self.render_tracker.lock().unwrap();
            tracker.start_frame();
        }
    }

    /// End rendering frame
    pub fn end_frame(&self) {
        if self.enabled {
            let mut tracker = self.render_tracker.lock().unwrap();
            tracker.end_frame();
        }
    }

    /// Get current FPS
    pub fn get_fps(&self) -> f64 {
        let tracker = self.render_tracker.lock().unwrap();
        tracker.fps()
    }

    /// Get render metrics
    pub fn get_render_metrics(&self) -> RenderMetrics {
        let tracker = self.render_tracker.lock().unwrap();
        tracker.clone()
    }

    /// Enable/disable profiling
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Clear all collected metrics
    pub fn clear(&self) {
        let mut profiles = self.profiles.lock().unwrap();
        profiles.clear();

        let mut latency = self.latency_tracker.lock().unwrap();
        latency.clear();

        let mut render = self.render_tracker.lock().unwrap();
        *render = RenderMetrics::default();
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII scope-based profiling
pub struct ProfileScope<'a> {
    profiler: &'a PerformanceProfiler,
    name: String,
}

impl<'a> ProfileScope<'a> {
    pub fn new(profiler: &'a PerformanceProfiler, name: &str) -> Self {
        profiler.start_profiling(name);
        Self {
            profiler,
            name: name.to_string(),
        }
    }
}

impl<'a> Drop for ProfileScope<'a> {
    fn drop(&mut self) {
        self.profiler.stop_profiling(&self.name);
    }
}

#[derive(Debug, Clone)]
struct ProfileData {
    name: String,
    start_time: Instant,
    end_time: Option<Instant>,
    duration: Duration,
}

/// Performance profiling report
#[derive(Debug, Clone)]
pub struct ProfileReport {
    pub name: String,
    pub duration: Duration,
    pub timestamp: u64,
}

impl ProfileReport {
    /// Check if operation is within acceptable latency
    pub fn is_acceptable(&self, max_latency_ms: u64) -> bool {
        self.duration.as_millis() as u64 <= max_latency_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profiling_basic() {
        let profiler = PerformanceProfiler::new();

        profiler.start_profiling("test_operation");
        thread::sleep(Duration::from_millis(50));
        let report = profiler.stop_profiling("test_operation");

        assert!(report.is_some());
        let report = report.unwrap();
        assert_eq!(report.name, "test_operation");
        assert!(report.duration.as_millis() >= 50);
    }

    #[test]
    fn test_profile_scope() {
        let profiler = PerformanceProfiler::new();

        {
            let _scope = ProfileScope::new(&profiler, "scoped_operation");
            thread::sleep(Duration::from_millis(30));
        } // scope ends, profiling stops

        // Profile should have been recorded
        let profiles = profiler.profiles.lock().unwrap();
        assert!(profiles.contains_key("scoped_operation"));
    }

    #[test]
    fn test_latency_measurement() {
        let profiler = PerformanceProfiler::new();

        // Measure some operations
        for _ in 0..10 {
            profiler.measure_latency("test_op", || {
                thread::sleep(Duration::from_millis(10));
            });
        }

        let metrics = profiler.get_latency_metrics("test_op");
        assert!(metrics.is_some());

        let metrics = metrics.unwrap();
        let avg = metrics.avg();
        assert!(avg.as_millis() >= 10 && avg.as_millis() <= 20);
    }

    #[test]
    fn test_fps_tracking() {
        let profiler = PerformanceProfiler::new();

        // Simulate 60fps rendering
        for _ in 0..60 {
            profiler.start_frame();
            thread::sleep(Duration::from_millis(16));
            profiler.end_frame();
        }

        let fps = profiler.get_fps();
        assert!(fps >= 55.0 && fps <= 65.0, "Expected ~60 FPS, got {}", fps);
    }
}
