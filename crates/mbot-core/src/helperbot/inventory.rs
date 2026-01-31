//! Inventory Tracking System - STORY-SORT-006 (#53)
//!
//! Implements persistent inventory tracking for sorted LEGO pieces
//! Contracts: SORT-004 (persistence), SORT-005 (offline-first), ARCH-001 (no_std)

#[cfg(feature = "no_std")]
use alloc::{string::{String, ToString}, vec, vec::Vec, format};
#[cfg(not(feature = "no_std"))]
use std::{string::{String, ToString}, vec::Vec, format};

use serde::{Deserialize, Serialize};

/// Inventory count for a specific bin and color combination
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryCount {
    pub bin_id: String,
    pub color: String,
    pub coarse_type: Option<String>,
    pub count: u32,
    /// Average detection confidence for pieces in this category
    pub confidence: f32,
    /// Last update timestamp in microseconds
    pub last_updated: u64,
}

impl InventoryCount {
    pub fn new(bin_id: String, color: String, timestamp: u64) -> Self {
        Self {
            bin_id,
            color,
            coarse_type: None,
            count: 0,
            confidence: 0.0,
            last_updated: timestamp,
        }
    }

    /// Increment count and update confidence (running average)
    pub fn increment(&mut self, confidence: f32, timestamp: u64) {
        let total = self.count as f32;
        // Running average of confidence
        self.confidence = if total > 0.0 {
            (self.confidence * total + confidence) / (total + 1.0)
        } else {
            confidence
        };
        self.count += 1;
        self.last_updated = timestamp;
    }

    /// Decrement count
    pub fn decrement(&mut self, timestamp: u64) {
        if self.count > 0 {
            self.count -= 1;
            self.last_updated = timestamp;
        }
    }

    /// Adjust count to specific value
    pub fn adjust_to(&mut self, new_count: u32, timestamp: u64) {
        self.count = new_count;
        self.last_updated = timestamp;
    }

    /// Reset count to zero
    pub fn reset(&mut self, timestamp: u64) {
        self.count = 0;
        self.last_updated = timestamp;
    }
}

/// Summary of a single bin's inventory
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventorySummary {
    pub bin_id: String,
    pub bin_name: String,
    pub category: String,
    pub piece_count: u32,
    pub capacity: u32,
    pub fill_percentage: f32,
    pub top_colors: Vec<ColorCount>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorCount {
    pub color: String,
    pub count: u32,
}

impl InventorySummary {
    pub fn new(bin_id: String, bin_name: String, category: String, capacity: u32) -> Self {
        Self {
            bin_id,
            bin_name,
            category,
            piece_count: 0,
            capacity,
            fill_percentage: 0.0,
            top_colors: Vec::new(),
        }
    }

    pub fn calculate_fill(&mut self) {
        self.fill_percentage = if self.capacity > 0 {
            (self.piece_count as f32 / self.capacity as f32) * 100.0
        } else {
            0.0
        };
    }

    /// Check if bin is approaching capacity
    pub fn needs_warning(&self, threshold_percent: f32) -> bool {
        self.fill_percentage >= threshold_percent
    }
}

/// Complete inventory snapshot at a point in time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventorySnapshot {
    pub snapshot_id: String,
    pub timestamp: u64,
    pub bins: Vec<InventorySummary>,
    pub total_pieces: u32,
    pub sorting_session: Option<String>,
}

impl InventorySnapshot {
    pub fn new(snapshot_id: String, timestamp: u64) -> Self {
        Self {
            snapshot_id,
            timestamp,
            bins: Vec::new(),
            total_pieces: 0,
            sorting_session: None,
        }
    }
}

/// Event types for inventory changes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InventoryEventType {
    Add,
    Remove,
    Adjust,
    Reset,
}

/// Source of inventory change
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InventoryEventSource {
    Sort,
    Manual,
    Recount,
}

/// Single inventory change event for audit trail
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryEvent {
    pub event_id: String,
    pub timestamp: u64,
    pub event_type: InventoryEventType,
    pub bin_id: String,
    pub color: String,
    /// Positive for add, negative for remove
    pub delta: i32,
    pub source: InventoryEventSource,
}

impl InventoryEvent {
    pub fn new(
        event_id: String,
        timestamp: u64,
        event_type: InventoryEventType,
        bin_id: String,
        color: String,
        delta: i32,
        source: InventoryEventSource,
    ) -> Self {
        Self {
            event_id,
            timestamp,
            event_type,
            bin_id,
            color,
            delta,
            source,
        }
    }
}

/// Bin configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BinConfig {
    pub bin_id: String,
    pub bin_name: String,
    pub category: String,
    pub capacity: u32,
}

impl BinConfig {
    pub fn new(bin_id: String, bin_name: String, category: String, capacity: u32) -> Self {
        Self {
            bin_id,
            bin_name,
            category,
            capacity,
        }
    }
}

/// Search result for inventory queries
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub bin_id: String,
    pub bin_name: String,
    pub match_type: String,
    pub count: u32,
    pub confidence: f32,
}

/// Complete inventory tracking system (SORT-004: persistent, SORT-005: offline-first)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryTracker {
    /// All inventory counts by bin and color
    counts: Vec<InventoryCount>,
    /// Bin configurations
    bins: Vec<BinConfig>,
    /// Event history for audit trail
    events: Vec<InventoryEvent>,
    /// Current sorting session ID
    current_session: Option<String>,
    /// Capacity warning threshold (percentage)
    warning_threshold: f32,
}

impl InventoryTracker {
    /// Create new inventory tracker
    pub fn new(warning_threshold: f32) -> Self {
        Self {
            counts: Vec::new(),
            bins: Vec::new(),
            events: Vec::new(),
            current_session: None,
            warning_threshold: warning_threshold.clamp(0.0, 100.0),
        }
    }

    /// Add a bin configuration
    pub fn add_bin(&mut self, bin: BinConfig) {
        // Remove existing bin with same ID
        self.bins.retain(|b| b.bin_id != bin.bin_id);
        self.bins.push(bin);
    }

    /// Start a new sorting session
    pub fn start_session(&mut self, session_id: String) {
        self.current_session = Some(session_id);
    }

    /// End current sorting session
    pub fn end_session(&mut self) {
        self.current_session = None;
    }

    /// Record a piece being sorted (SORT-004: persistence)
    pub fn record_sorted(
        &mut self,
        bin_id: String,
        color: String,
        confidence: f32,
        timestamp: u64,
    ) {
        // Find or create inventory count
        let mut found = false;
        for count in &mut self.counts {
            if count.bin_id == bin_id && count.color == color {
                count.increment(confidence, timestamp);
                found = true;
                break;
            }
        }

        if !found {
            let mut count = InventoryCount::new(bin_id.clone(), color.clone(), timestamp);
            count.increment(confidence, timestamp);
            self.counts.push(count);
        }

        // Log event
        let event = InventoryEvent::new(
            format!("evt-{}", timestamp),
            timestamp,
            InventoryEventType::Add,
            bin_id,
            color,
            1,
            InventoryEventSource::Sort,
        );
        self.events.push(event);
    }

    /// Manually adjust bin count
    pub fn adjust_bin(
        &mut self,
        bin_id: String,
        color: String,
        new_count: u32,
        timestamp: u64,
    ) {
        // Find or create inventory count
        let mut found = false;
        let mut delta = 0;

        for count in &mut self.counts {
            if count.bin_id == bin_id && count.color == color {
                delta = new_count as i32 - count.count as i32;
                count.adjust_to(new_count, timestamp);
                found = true;
                break;
            }
        }

        if !found && new_count > 0 {
            let mut count = InventoryCount::new(bin_id.clone(), color.clone(), timestamp);
            count.adjust_to(new_count, timestamp);
            delta = new_count as i32;
            self.counts.push(count);
        }

        // Log event
        let event = InventoryEvent::new(
            format!("evt-{}", timestamp),
            timestamp,
            InventoryEventType::Adjust,
            bin_id,
            color,
            delta,
            InventoryEventSource::Manual,
        );
        self.events.push(event);
    }

    /// Reset a bin's inventory to zero
    pub fn reset_bin(&mut self, bin_id: String, timestamp: u64) {
        for count in &mut self.counts {
            if count.bin_id == bin_id && count.count > 0 {
                let delta = -(count.count as i32);
                count.reset(timestamp);

                // Log event
                let event = InventoryEvent::new(
                    format!("evt-{}-{}", timestamp, count.color),
                    timestamp,
                    InventoryEventType::Reset,
                    bin_id.clone(),
                    count.color.clone(),
                    delta,
                    InventoryEventSource::Manual,
                );
                self.events.push(event);
            }
        }
    }

    /// Get bin contents
    pub fn get_bin_contents(&self, bin_id: &str) -> Vec<InventoryCount> {
        self.counts
            .iter()
            .filter(|c| c.bin_id == bin_id && c.count > 0)
            .cloned()
            .collect()
    }

    /// Get total count across all bins
    pub fn get_total_count(&self) -> u32 {
        self.counts.iter().map(|c| c.count).sum()
    }

    /// Get total count for a specific color
    pub fn get_color_count(&self, color: &str) -> u32 {
        self.counts
            .iter()
            .filter(|c| c.color == color)
            .map(|c| c.count)
            .sum()
    }

    /// Search inventory by query string
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for count in &self.counts {
            if count.count == 0 {
                continue;
            }

            let bin_name = self
                .bins
                .iter()
                .find(|b| b.bin_id == count.bin_id)
                .map(|b| b.bin_name.clone())
                .unwrap_or_else(|| count.bin_id.clone());

            let mut match_type = None;

            if count.color.to_lowercase().contains(&query_lower) {
                match_type = Some("color");
            } else if bin_name.to_lowercase().contains(&query_lower) {
                match_type = Some("name");
            } else if let Some(ref coarse_type) = count.coarse_type {
                if coarse_type.to_lowercase().contains(&query_lower) {
                    match_type = Some("type");
                }
            }

            if let Some(mt) = match_type {
                results.push(SearchResult {
                    bin_id: count.bin_id.clone(),
                    bin_name,
                    match_type: mt.to_string(),
                    count: count.count,
                    confidence: count.confidence,
                });
            }
        }

        results
    }

    /// Generate inventory summary
    pub fn generate_summary(&self, timestamp: u64) -> InventorySnapshot {
        let mut snapshot = InventorySnapshot::new(format!("snapshot-{}", timestamp), timestamp);
        snapshot.sorting_session = self.current_session.clone();

        for bin in &self.bins {
            let mut summary = InventorySummary::new(
                bin.bin_id.clone(),
                bin.bin_name.clone(),
                bin.category.clone(),
                bin.capacity,
            );

            // Calculate piece count and top colors
            let bin_counts: Vec<_> = self
                .counts
                .iter()
                .filter(|c| c.bin_id == bin.bin_id && c.count > 0)
                .collect();

            summary.piece_count = bin_counts.iter().map(|c| c.count).sum();

            // Get top 3 colors
            let mut color_counts: Vec<_> = bin_counts
                .iter()
                .map(|c| ColorCount {
                    color: c.color.clone(),
                    count: c.count,
                })
                .collect();
            color_counts.sort_by(|a, b| b.count.cmp(&a.count));
            summary.top_colors = color_counts.into_iter().take(3).collect();

            summary.calculate_fill();
            snapshot.bins.push(summary);
        }

        snapshot.total_pieces = self.get_total_count();
        snapshot
    }

    /// Check if any bins need capacity warnings
    pub fn check_capacity_warnings(&self) -> Vec<(String, f32)> {
        let mut warnings = Vec::new();
        let summary = self.generate_summary(0);

        for bin in summary.bins {
            if bin.needs_warning(self.warning_threshold) {
                warnings.push((bin.bin_name, bin.fill_percentage));
            }
        }

        warnings
    }

    /// Export inventory as JSON string (SORT-004: persistence, SORT-005: offline)
    pub fn to_json(&self) -> Result<String, &'static str> {
        serde_json::to_string(self).map_err(|_| "Serialization failed")
    }

    /// Load inventory from JSON string (SORT-004: persistence)
    pub fn from_json(json: &str) -> Result<Self, &'static str> {
        serde_json::from_str(json).map_err(|_| "Deserialization failed")
    }

    /// Get event history
    pub fn get_events(&self) -> &[InventoryEvent] {
        &self.events
    }

    /// Clear old events (keep last N)
    pub fn trim_events(&mut self, keep_count: usize) {
        if self.events.len() > keep_count {
            let start = self.events.len() - keep_count;
            self.events = self.events[start..].to_vec();
        }
    }
}

impl Default for InventoryTracker {
    fn default() -> Self {
        Self::new(90.0) // 90% capacity warning by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_sorted_increments_count() {
        let mut tracker = InventoryTracker::new(90.0);
        tracker.add_bin(BinConfig::new(
            "bin-01".to_string(),
            "Red Bin".to_string(),
            "color".to_string(),
            50,
        ));

        // Record 3 red pieces
        tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.95, 1000);
        tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.90, 2000);
        tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.92, 3000);

        let contents = tracker.get_bin_contents("bin-01");
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0].count, 3);
        assert_eq!(contents[0].color, "red");
    }

    #[test]
    fn test_inventory_persists_across_sessions() {
        let mut tracker = InventoryTracker::new(90.0);
        tracker.add_bin(BinConfig::new(
            "bin-01".to_string(),
            "Red Bin".to_string(),
            "color".to_string(),
            50,
        ));

        tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.95, 1000);

        // SORT-004: Serialize and deserialize
        let json = tracker.to_json().unwrap();
        let restored = InventoryTracker::from_json(&json).unwrap();

        let contents = restored.get_bin_contents("bin-01");
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0].count, 1);
    }

    #[test]
    fn test_manual_adjustment() {
        let mut tracker = InventoryTracker::new(90.0);
        tracker.add_bin(BinConfig::new(
            "bin-01".to_string(),
            "Red Bin".to_string(),
            "color".to_string(),
            50,
        ));

        // Auto-sorted 20 pieces
        for i in 0..20 {
            tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.95, 1000 + i);
        }

        // Manual recount shows 18
        tracker.adjust_bin("bin-01".to_string(), "red".to_string(), 18, 2000);

        let contents = tracker.get_bin_contents("bin-01");
        assert_eq!(contents[0].count, 18);

        // Check event logged
        let events = tracker.get_events();
        let adjust_event = events
            .iter()
            .find(|e| e.event_type == InventoryEventType::Adjust);
        assert!(adjust_event.is_some());
        assert_eq!(adjust_event.unwrap().delta, -2);
    }

    #[test]
    fn test_reset_bin() {
        let mut tracker = InventoryTracker::new(90.0);
        tracker.add_bin(BinConfig::new(
            "bin-01".to_string(),
            "Red Bin".to_string(),
            "color".to_string(),
            50,
        ));

        tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.95, 1000);
        tracker.record_sorted("bin-01".to_string(), "blue".to_string(), 0.90, 2000);

        tracker.reset_bin("bin-01".to_string(), 3000);

        let contents = tracker.get_bin_contents("bin-01");
        assert_eq!(contents.len(), 0);
    }

    #[test]
    fn test_search_by_color() {
        let mut tracker = InventoryTracker::new(90.0);
        tracker.add_bin(BinConfig::new(
            "bin-01".to_string(),
            "Red Bin".to_string(),
            "color".to_string(),
            50,
        ));
        tracker.add_bin(BinConfig::new(
            "bin-03".to_string(),
            "Mixed Bin".to_string(),
            "mixed".to_string(),
            100,
        ));

        tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.95, 1000);
        tracker.record_sorted("bin-03".to_string(), "red".to_string(), 0.90, 2000);

        let results = tracker.search("red");
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.bin_id == "bin-01"));
        assert!(results.iter().any(|r| r.bin_id == "bin-03"));
    }

    #[test]
    fn test_capacity_warning() {
        let mut tracker = InventoryTracker::new(90.0);
        tracker.add_bin(BinConfig::new(
            "bin-01".to_string(),
            "Red Bin".to_string(),
            "color".to_string(),
            50,
        ));

        // Add 46 pieces (92% full)
        for i in 0..46 {
            tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.95, 1000 + i);
        }

        let warnings = tracker.check_capacity_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].0, "Red Bin");
        assert!(warnings[0].1 >= 90.0);
    }

    #[test]
    fn test_inventory_summary() {
        let mut tracker = InventoryTracker::new(90.0);
        tracker.add_bin(BinConfig::new(
            "bin-01".to_string(),
            "Red Bin".to_string(),
            "color".to_string(),
            50,
        ));

        tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.95, 1000);
        tracker.record_sorted("bin-01".to_string(), "blue".to_string(), 0.90, 2000);

        let summary = tracker.generate_summary(3000);
        assert_eq!(summary.total_pieces, 2);
        assert_eq!(summary.bins.len(), 1);
        assert_eq!(summary.bins[0].piece_count, 2);
        assert_eq!(summary.bins[0].top_colors.len(), 2);
    }

    #[test]
    fn test_total_counts() {
        let mut tracker = InventoryTracker::new(90.0);
        tracker.add_bin(BinConfig::new(
            "bin-01".to_string(),
            "Red Bin".to_string(),
            "color".to_string(),
            50,
        ));
        tracker.add_bin(BinConfig::new(
            "bin-02".to_string(),
            "Blue Bin".to_string(),
            "color".to_string(),
            50,
        ));

        tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.95, 1000);
        tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.93, 2000);
        tracker.record_sorted("bin-02".to_string(), "blue".to_string(), 0.90, 3000);

        assert_eq!(tracker.get_total_count(), 3);
        assert_eq!(tracker.get_color_count("red"), 2);
        assert_eq!(tracker.get_color_count("blue"), 1);
    }

    #[test]
    fn test_confidence_averaging() {
        let mut tracker = InventoryTracker::new(90.0);
        tracker.add_bin(BinConfig::new(
            "bin-01".to_string(),
            "Red Bin".to_string(),
            "color".to_string(),
            50,
        ));

        tracker.record_sorted("bin-01".to_string(), "red".to_string(), 1.0, 1000);
        tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.8, 2000);

        let contents = tracker.get_bin_contents("bin-01");
        assert_eq!(contents[0].count, 2);
        // Confidence should be average of 1.0 and 0.8 = 0.9
        assert!((contents[0].confidence - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_event_history() {
        let mut tracker = InventoryTracker::new(90.0);
        tracker.add_bin(BinConfig::new(
            "bin-01".to_string(),
            "Red Bin".to_string(),
            "color".to_string(),
            50,
        ));

        tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.95, 1000);
        tracker.adjust_bin("bin-01".to_string(), "red".to_string(), 5, 2000);
        tracker.reset_bin("bin-01".to_string(), 3000);

        let events = tracker.get_events();
        assert!(events.len() >= 3);

        // Check event types
        assert!(events
            .iter()
            .any(|e| e.event_type == InventoryEventType::Add));
        assert!(events
            .iter()
            .any(|e| e.event_type == InventoryEventType::Adjust));
        assert!(events
            .iter()
            .any(|e| e.event_type == InventoryEventType::Reset));
    }

    #[test]
    fn test_session_tracking() {
        let mut tracker = InventoryTracker::new(90.0);

        tracker.start_session("session-001".to_string());
        assert_eq!(tracker.current_session, Some("session-001".to_string()));

        let summary = tracker.generate_summary(1000);
        assert_eq!(summary.sorting_session, Some("session-001".to_string()));

        tracker.end_session();
        assert_eq!(tracker.current_session, None);
    }

    #[test]
    fn test_trim_events() {
        let mut tracker = InventoryTracker::new(90.0);
        tracker.add_bin(BinConfig::new(
            "bin-01".to_string(),
            "Red Bin".to_string(),
            "color".to_string(),
            50,
        ));

        // Generate many events
        for i in 0..100 {
            tracker.record_sorted("bin-01".to_string(), "red".to_string(), 0.95, i);
        }

        assert!(tracker.get_events().len() >= 100);

        tracker.trim_events(10);
        assert_eq!(tracker.get_events().len(), 10);
    }

    // SORT-005: Offline-first - no network dependencies
    #[test]
    fn test_offline_operation() {
        let tracker = InventoryTracker::new(90.0);
        // All operations work without network
        let _ = tracker.to_json();
        let _ = tracker.get_total_count();
        let _ = tracker.search("red");
        // No async, no network calls - pure local operation
    }
}
