//! Carousel Station Configuration for LEGO Sorter
//!
//! Implements SORT-002 (Deterministic) and SORT-004 (Inventory Persistence)
//!
//! This module provides carousel/bin configuration that maps physical bin positions
//! (angles from home) to sorting categories, with visual marker detection for
//! bin identification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ============================================
// Core Data Structures
// ============================================

/// Visual marker type for bin identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerType {
    /// AprilTag marker system
    AprilTag,
    /// ArUco marker system
    ArUco,
    /// NFC tag (optional backup)
    Nfc,
}

/// Bin in the carousel sorting station
/// Contract: SORT-002, SORT-004 Bin data structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bin {
    /// Unique identifier (e.g., "bin-01")
    pub bin_id: String,
    /// Visual marker ID (AprilTag/ArUco)
    pub marker_id: String,
    /// Optional NFC tag ID
    pub nfc_id: Option<String>,
    /// Position angle from home (0-360 degrees)
    pub position_angle: f32,
    /// Sorting rule (e.g., "color:red", "type:axle")
    pub category_rule: String,
    /// Estimated piece capacity
    pub capacity: u32,
    /// Current inventory count
    /// Contract: SORT-004 - Inventory must persist
    pub current_count: u32,
}

impl Bin {
    /// Create a new bin
    pub fn new(
        bin_id: impl Into<String>,
        marker_id: impl Into<String>,
        position_angle: f32,
        category_rule: impl Into<String>,
        capacity: u32,
    ) -> Self {
        Self {
            bin_id: bin_id.into(),
            marker_id: marker_id.into(),
            nfc_id: None,
            position_angle: Self::normalize_angle(position_angle),
            category_rule: category_rule.into(),
            capacity,
            current_count: 0,
        }
    }

    /// Normalize angle to 0-360 range
    fn normalize_angle(angle: f32) -> f32 {
        let mut normalized = angle % 360.0;
        if normalized < 0.0 {
            normalized += 360.0;
        }
        normalized
    }

    /// Check if bin has capacity
    pub fn has_capacity(&self) -> bool {
        self.current_count < self.capacity
    }

    /// Add a piece to inventory
    pub fn add_piece(&mut self) -> bool {
        if self.has_capacity() {
            self.current_count += 1;
            true
        } else {
            false
        }
    }

    /// Remove a piece from inventory
    pub fn remove_piece(&mut self) -> bool {
        if self.current_count > 0 {
            self.current_count -= 1;
            true
        } else {
            false
        }
    }

    /// Reset inventory count
    pub fn reset_count(&mut self) {
        self.current_count = 0;
    }

    /// Check if bin is nearly full (>=90% capacity)
    pub fn is_nearly_full(&self) -> bool {
        self.current_count as f32 / self.capacity as f32 >= 0.9
    }

    /// Check if this bin matches a category (e.g., "color:red")
    pub fn matches_category(&self, category: &str) -> bool {
        self.category_rule == category
    }
}

/// Position for the pickup/tray area
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct TrayPosition {
    /// Angle to face the tray (degrees)
    pub angle: f32,
    /// Distance to tray center (millimeters)
    pub distance_mm: f32,
}

impl Default for TrayPosition {
    fn default() -> Self {
        Self {
            angle: 0.0,
            distance_mm: 150.0,
        }
    }
}

/// Carousel configuration
/// Contract: SORT-002 CarouselConfig data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarouselConfig {
    /// Unique carousel identifier
    pub carousel_id: String,
    /// Reference home position (0° reference)
    pub home_angle: f32,
    /// Number of bins configured
    pub bin_count: usize,
    /// Bin configurations
    /// Contract: SORT-002 - Same bin_id always maps to same angle (deterministic)
    pub bins: Vec<Bin>,
    /// Tray/pickup position
    pub tray_position: TrayPosition,
    /// Safe rotation speed (degrees per second)
    /// Contract: SORT-003 - Speed limited for safety
    pub rotation_speed: f32,
    /// Minimum angular spacing between bins (degrees)
    pub min_bin_spacing: f32,
}

impl Default for CarouselConfig {
    fn default() -> Self {
        Self {
            carousel_id: "default_carousel".to_string(),
            home_angle: 0.0,
            bin_count: 0,
            bins: Vec::new(),
            tray_position: TrayPosition::default(),
            rotation_speed: 30.0, // 30°/s safe speed
            min_bin_spacing: 30.0, // 30° minimum spacing
        }
    }
}

impl CarouselConfig {
    /// Create a new carousel configuration
    pub fn new(carousel_id: impl Into<String>) -> Self {
        Self {
            carousel_id: carousel_id.into(),
            ..Default::default()
        }
    }

    /// Add a bin to the configuration
    pub fn add_bin(&mut self, bin: Bin) -> Result<(), String> {
        // Check for overlapping positions
        for existing in &self.bins {
            let angle_diff = (bin.position_angle - existing.position_angle).abs();
            let angle_diff = angle_diff.min(360.0 - angle_diff); // Handle wraparound

            if angle_diff < self.min_bin_spacing {
                return Err(format!(
                    "Bins too close ({:.1}° apart, need {:.1}°)",
                    angle_diff, self.min_bin_spacing
                ));
            }
        }

        // Check for duplicate bin_id
        if self.bins.iter().any(|b| b.bin_id == bin.bin_id) {
            return Err(format!("Bin ID '{}' already exists", bin.bin_id));
        }

        self.bins.push(bin);
        self.bin_count = self.bins.len();
        Ok(())
    }

    /// Remove a bin by ID
    pub fn remove_bin(&mut self, bin_id: &str) -> Result<(), String> {
        let initial_len = self.bins.len();
        self.bins.retain(|b| b.bin_id != bin_id);

        if self.bins.len() == initial_len {
            return Err(format!("Bin '{}' not found", bin_id));
        }

        self.bin_count = self.bins.len();
        Ok(())
    }

    /// Get a bin by ID
    pub fn get_bin(&self, bin_id: &str) -> Option<&Bin> {
        self.bins.iter().find(|b| b.bin_id == bin_id)
    }

    /// Get a mutable bin by ID
    pub fn get_bin_mut(&mut self, bin_id: &str) -> Option<&mut Bin> {
        self.bins.iter_mut().find(|b| b.bin_id == bin_id)
    }

    /// Find bin by marker ID
    pub fn find_by_marker(&self, marker_id: &str) -> Option<&Bin> {
        self.bins.iter().find(|b| b.marker_id == marker_id)
    }

    /// Find bin by category rule
    /// Contract: SORT-002 - Deterministic mapping
    pub fn find_by_category(&self, category: &str) -> Option<&Bin> {
        self.bins.iter().find(|b| b.matches_category(category))
    }

    /// Get all bins sorted by angle
    pub fn bins_by_angle(&self) -> Vec<&Bin> {
        let mut sorted: Vec<&Bin> = self.bins.iter().collect();
        sorted.sort_by(|a, b| {
            a.position_angle
                .partial_cmp(&b.position_angle)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    /// Calculate rotation time from current angle to target bin
    pub fn rotation_time(&self, current_angle: f32, target_bin: &Bin) -> f32 {
        let angle_diff = (target_bin.position_angle - current_angle).abs();
        let angle_diff = angle_diff.min(360.0 - angle_diff); // Shortest path
        angle_diff / self.rotation_speed
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.bins.is_empty() {
            return Err("No bins configured".to_string());
        }

        // Check for position conflicts
        for i in 0..self.bins.len() {
            for j in (i + 1)..self.bins.len() {
                let diff = (self.bins[i].position_angle - self.bins[j].position_angle).abs();
                let diff = diff.min(360.0 - diff);

                if diff < self.min_bin_spacing {
                    return Err(format!(
                        "Bins '{}' and '{}' too close ({:.1}° apart)",
                        self.bins[i].bin_id, self.bins[j].bin_id, diff
                    ));
                }
            }
        }

        // Check for duplicate marker IDs
        let mut markers = HashMap::new();
        for bin in &self.bins {
            if let Some(existing) = markers.insert(&bin.marker_id, &bin.bin_id) {
                return Err(format!(
                    "Duplicate marker '{}' for bins '{}' and '{}'",
                    bin.marker_id, existing, bin.bin_id
                ));
            }
        }

        Ok(())
    }

    /// Save to a JSON file
    /// Contract: SORT-004 - Configuration persists across power cycles
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), String> {
        self.validate()?;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(path, json).map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Load from a JSON file
    /// Contract: SORT-004 - Restore configuration on startup
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let json =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: CarouselConfig = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        config.validate()?;

        Ok(config)
    }

    /// Get default configuration file path
    pub fn default_path() -> PathBuf {
        PathBuf::from("config/carousel_config.json")
    }

    /// Load or create default configuration
    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        if path.exists() {
            Self::load_from_file(&path).unwrap_or_else(|_| Self::default())
        } else {
            Self::default()
        }
    }

    /// Save to default location
    pub fn save(&self) -> Result<(), String> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        self.save_to_file(path)
    }

    /// Get inventory summary
    pub fn inventory_summary(&self) -> HashMap<String, u32> {
        self.bins
            .iter()
            .map(|b| (b.bin_id.clone(), b.current_count))
            .collect()
    }

    /// Get bins that are nearly full
    pub fn nearly_full_bins(&self) -> Vec<&Bin> {
        self.bins.iter().filter(|b| b.is_nearly_full()).collect()
    }

    /// Reset all inventory counts
    pub fn reset_all_inventory(&mut self) {
        for bin in &mut self.bins {
            bin.reset_count();
        }
    }
}

// ============================================
// Marker Detection
// ============================================

/// Marker detection result from camera scan
/// Contract: SORT-002 MarkerDetection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkerDetection {
    /// Detected marker ID
    pub marker_id: String,
    /// Angle where marker was detected (0-360°)
    pub detected_angle: f32,
    /// Detection confidence (0.0-1.0)
    pub confidence: f32,
    /// Detection timestamp (microseconds)
    pub timestamp: u64,
}

impl MarkerDetection {
    /// Create a new marker detection
    pub fn new(marker_id: impl Into<String>, angle: f32, confidence: f32, timestamp: u64) -> Self {
        Self {
            marker_id: marker_id.into(),
            detected_angle: angle,
            confidence,
            timestamp,
        }
    }

    /// Check if detection meets minimum confidence threshold
    pub fn is_valid(&self, min_confidence: f32) -> bool {
        self.confidence >= min_confidence
    }
}

/// Bin detection system
pub struct BinDetector {
    config: CarouselConfig,
    min_confidence: f32,
}

impl BinDetector {
    /// Create a new bin detector
    pub fn new(config: CarouselConfig) -> Self {
        Self {
            config,
            min_confidence: 0.75,
        }
    }

    /// Set minimum confidence threshold
    pub fn set_min_confidence(&mut self, confidence: f32) {
        self.min_confidence = confidence.clamp(0.0, 1.0);
    }

    /// Process detected markers and update bin positions
    pub fn update_from_detections(&mut self, detections: &[MarkerDetection]) -> Result<usize, String> {
        let mut updated_count = 0;

        for detection in detections {
            if !detection.is_valid(self.min_confidence) {
                continue;
            }

            // Find bin with this marker
            if let Some(bin) = self.config.bins.iter_mut().find(|b| b.marker_id == detection.marker_id) {
                // Update position
                bin.position_angle = detection.detected_angle;
                updated_count += 1;
            }
        }

        // Validate after updates
        self.config.validate()?;

        Ok(updated_count)
    }

    /// Get bins with missing markers (not detected in scan)
    pub fn missing_markers(&self, detections: &[MarkerDetection]) -> Vec<String> {
        let detected: HashMap<_, _> = detections
            .iter()
            .map(|d| (d.marker_id.as_str(), true))
            .collect();

        self.config
            .bins
            .iter()
            .filter(|b| !detected.contains_key(b.marker_id.as_str()))
            .map(|b| b.marker_id.clone())
            .collect()
    }

    /// Get the updated configuration
    pub fn config(&self) -> &CarouselConfig {
        &self.config
    }

    /// Consume detector and return configuration
    pub fn into_config(self) -> CarouselConfig {
        self.config
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bin_creation() {
        let bin = Bin::new("bin-01", "marker-01", 45.0, "color:red", 50);

        assert_eq!(bin.bin_id, "bin-01");
        assert_eq!(bin.position_angle, 45.0);
        assert_eq!(bin.category_rule, "color:red");
        assert_eq!(bin.capacity, 50);
        assert_eq!(bin.current_count, 0);
        assert!(bin.has_capacity());
    }

    #[test]
    fn test_bin_inventory() {
        let mut bin = Bin::new("bin-01", "marker-01", 45.0, "color:red", 3);

        assert!(bin.add_piece());
        assert_eq!(bin.current_count, 1);

        assert!(bin.add_piece());
        assert!(bin.add_piece());
        assert_eq!(bin.current_count, 3);

        // At capacity
        assert!(!bin.has_capacity());
        assert!(!bin.add_piece());

        // Remove pieces
        assert!(bin.remove_piece());
        assert_eq!(bin.current_count, 2);
        assert!(bin.has_capacity());
    }

    #[test]
    fn test_bin_nearly_full() {
        let mut bin = Bin::new("bin-01", "marker-01", 45.0, "color:red", 10);

        assert!(!bin.is_nearly_full());

        bin.current_count = 9; // 90%
        assert!(bin.is_nearly_full());

        bin.current_count = 10; // 100%
        assert!(bin.is_nearly_full());

        bin.current_count = 8; // 80%
        assert!(!bin.is_nearly_full());
    }

    #[test]
    fn test_carousel_add_bins() {
        let mut config = CarouselConfig::new("test_carousel");

        let bin1 = Bin::new("bin-01", "marker-01", 45.0, "color:red", 50);
        assert!(config.add_bin(bin1).is_ok());

        let bin2 = Bin::new("bin-02", "marker-02", 135.0, "color:blue", 50);
        assert!(config.add_bin(bin2).is_ok());

        assert_eq!(config.bin_count, 2);
    }

    #[test]
    fn test_carousel_rejects_close_bins() {
        let mut config = CarouselConfig::new("test_carousel");
        config.min_bin_spacing = 30.0;

        let bin1 = Bin::new("bin-01", "marker-01", 45.0, "color:red", 50);
        assert!(config.add_bin(bin1).is_ok());

        // Too close (only 20° apart)
        let bin2 = Bin::new("bin-02", "marker-02", 65.0, "color:blue", 50);
        assert!(config.add_bin(bin2).is_err());

        // Far enough (90° apart)
        let bin3 = Bin::new("bin-03", "marker-03", 135.0, "color:yellow", 50);
        assert!(config.add_bin(bin3).is_ok());
    }

    #[test]
    fn test_carousel_find_by_category() {
        let mut config = CarouselConfig::new("test_carousel");

        config
            .add_bin(Bin::new("bin-01", "marker-01", 45.0, "color:red", 50))
            .unwrap();
        config
            .add_bin(Bin::new("bin-02", "marker-02", 135.0, "color:blue", 50))
            .unwrap();
        config
            .add_bin(Bin::new("bin-03", "marker-03", 225.0, "color:yellow", 50))
            .unwrap();

        // Contract: SORT-002 - Same category always maps to same bin (deterministic)
        let red_bin = config.find_by_category("color:red");
        assert!(red_bin.is_some());
        assert_eq!(red_bin.unwrap().bin_id, "bin-01");

        let blue_bin = config.find_by_category("color:blue");
        assert!(blue_bin.is_some());
        assert_eq!(blue_bin.unwrap().bin_id, "bin-02");

        // Unknown category
        let unknown = config.find_by_category("color:purple");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_carousel_validation() {
        let config = CarouselConfig::new("test_carousel");

        // Empty config is invalid
        assert!(config.validate().is_err());

        let mut config = CarouselConfig::new("test_carousel");
        config
            .add_bin(Bin::new("bin-01", "marker-01", 45.0, "color:red", 50))
            .unwrap();

        // Valid with at least one bin
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_carousel_bins_by_angle() {
        let mut config = CarouselConfig::new("test_carousel");

        config
            .add_bin(Bin::new("bin-01", "marker-01", 135.0, "color:red", 50))
            .unwrap();
        config
            .add_bin(Bin::new("bin-02", "marker-02", 45.0, "color:blue", 50))
            .unwrap();
        config
            .add_bin(Bin::new("bin-03", "marker-03", 225.0, "color:yellow", 50))
            .unwrap();

        let sorted = config.bins_by_angle();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].bin_id, "bin-02"); // 45°
        assert_eq!(sorted[1].bin_id, "bin-01"); // 135°
        assert_eq!(sorted[2].bin_id, "bin-03"); // 225°
    }

    #[test]
    fn test_rotation_time_calculation() {
        let mut config = CarouselConfig::new("test_carousel");
        config.rotation_speed = 30.0; // 30°/s

        let bin = Bin::new("bin-01", "marker-01", 90.0, "color:red", 50);
        config.add_bin(bin).unwrap();

        // From 0° to 90° = 90° / 30°/s = 3 seconds
        let time = config.rotation_time(0.0, config.get_bin("bin-01").unwrap());
        assert!((time - 3.0).abs() < 0.01);

        // From 350° to 10° = 20° / 30°/s = 0.667 seconds (shortest path)
        let bin2 = Bin::new("bin-02", "marker-02", 10.0, "color:blue", 50);
        config.add_bin(bin2).unwrap();
        let time = config.rotation_time(350.0, config.get_bin("bin-02").unwrap());
        assert!(time < 1.0);
    }

    #[test]
    fn test_marker_detection() {
        let detection = MarkerDetection::new("marker-01", 45.0, 0.92, 12345);

        assert!(detection.is_valid(0.75));
        assert!(detection.is_valid(0.90));
        assert!(!detection.is_valid(0.95));
    }

    #[test]
    fn test_bin_detector_updates() {
        let mut config = CarouselConfig::new("test_carousel");
        config
            .add_bin(Bin::new("bin-01", "marker-01", 30.0, "color:red", 50))
            .unwrap();
        config
            .add_bin(Bin::new("bin-02", "marker-02", 150.0, "color:blue", 50))
            .unwrap();

        let mut detector = BinDetector::new(config);

        let detections = vec![
            MarkerDetection::new("marker-01", 45.0, 0.95, 12345),
            MarkerDetection::new("marker-02", 135.0, 0.88, 12345),
        ];

        let updated = detector.update_from_detections(&detections).unwrap();
        assert_eq!(updated, 2);

        let config = detector.config();
        assert_eq!(config.get_bin("bin-01").unwrap().position_angle, 45.0);
        assert_eq!(config.get_bin("bin-02").unwrap().position_angle, 135.0);
    }

    #[test]
    fn test_missing_markers_detection() {
        let mut config = CarouselConfig::new("test_carousel");
        config
            .add_bin(Bin::new("bin-01", "marker-01", 45.0, "color:red", 50))
            .unwrap();
        config
            .add_bin(Bin::new("bin-02", "marker-02", 135.0, "color:blue", 50))
            .unwrap();
        config
            .add_bin(Bin::new("bin-03", "marker-03", 225.0, "color:yellow", 50))
            .unwrap();

        let detector = BinDetector::new(config);

        // Only detect markers 1 and 2
        let detections = vec![
            MarkerDetection::new("marker-01", 45.0, 0.95, 12345),
            MarkerDetection::new("marker-02", 135.0, 0.88, 12345),
        ];

        let missing = detector.missing_markers(&detections);
        assert_eq!(missing.len(), 1);
        assert!(missing.contains(&"marker-03".to_string()));
    }

    #[test]
    fn test_carousel_persistence() {
        use std::fs;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_carousel.json");

        let mut config = CarouselConfig::new("test_carousel");
        config
            .add_bin(Bin::new("bin-01", "marker-01", 45.0, "color:red", 50))
            .unwrap();

        // Save
        assert!(config.save_to_file(&test_file).is_ok());
        assert!(test_file.exists());

        // Load
        let loaded = CarouselConfig::load_from_file(&test_file).unwrap();
        assert_eq!(loaded.carousel_id, "test_carousel");
        assert_eq!(loaded.bin_count, 1);
        assert_eq!(loaded.bins[0].bin_id, "bin-01");

        // Cleanup
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_inventory_summary() {
        let mut config = CarouselConfig::new("test_carousel");

        let mut bin1 = Bin::new("bin-01", "marker-01", 45.0, "color:red", 50);
        bin1.current_count = 10;
        config.add_bin(bin1).unwrap();

        let mut bin2 = Bin::new("bin-02", "marker-02", 135.0, "color:blue", 50);
        bin2.current_count = 5;
        config.add_bin(bin2).unwrap();

        let summary = config.inventory_summary();
        assert_eq!(summary.get("bin-01"), Some(&10));
        assert_eq!(summary.get("bin-02"), Some(&5));
    }
}
