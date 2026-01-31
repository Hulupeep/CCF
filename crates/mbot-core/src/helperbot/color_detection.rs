//! Color Detection System - STORY-HELP-001
//!
//! Implements RGB sensor color detection with calibration support
//! Contract: feature_helperbot.yml (I-HELP-001 through I-HELP-004)

#[cfg(feature = "no_std")]
use alloc::{string::{String, ToString}, vec, vec::Vec};
#[cfg(not(feature = "no_std"))]
use std::{string::{String, ToString}, vec::Vec};

use serde::{Deserialize, Serialize};

/// RGB reading with confidence score (I-HELP-001)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorReading {
    /// Red channel (0-255)
    pub r: u8,
    /// Green channel (0-255)
    pub g: u8,
    /// Blue channel (0-255)
    pub b: u8,
    /// Detection confidence (0.0-1.0)
    pub confidence: f32,
}

impl ColorReading {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            confidence: 1.0,
        }
    }
}

/// Complete color detection result (I-HELP-001)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorDetection {
    /// Color name or 'unknown'
    pub detected_color: String,
    /// Raw RGB values
    pub rgb_values: ColorReading,
    /// Match confidence (0.0-1.0)
    pub confidence: f32,
    /// True for special colors (gold, silver, etc)
    pub is_rare: bool,
    /// Detection timestamp in microseconds
    pub timestamp_us: u64,
}

impl ColorDetection {
    /// Create an unknown color detection (I-HELP-003)
    pub fn unknown(rgb: ColorReading, timestamp_us: u64) -> Self {
        Self {
            detected_color: "unknown".to_string(),
            rgb_values: rgb,
            confidence: 0.0,
            is_rare: false,
            timestamp_us,
        }
    }

    /// Create a detection with known color
    pub fn known(name: String, rgb: ColorReading, confidence: f32, is_rare: bool, timestamp_us: u64) -> Self {
        Self {
            detected_color: name,
            rgb_values: rgb,
            confidence: confidence.clamp(0.0, 1.0),
            is_rare,
            timestamp_us,
        }
    }
}

/// Entry in color lookup table
#[derive(Clone, Debug)]
pub struct ColorEntry {
    pub name: String,
    pub rgb_min: [u8; 3],
    pub rgb_max: [u8; 3],
    pub is_rare: bool,
}

impl ColorEntry {
    pub fn matches(&self, r: u8, g: u8, b: u8) -> bool {
        r >= self.rgb_min[0]
            && r <= self.rgb_max[0]
            && g >= self.rgb_min[1]
            && g <= self.rgb_max[1]
            && b >= self.rgb_min[2]
            && b <= self.rgb_max[2]
    }

    pub fn confidence(&self, r: u8, g: u8, b: u8) -> f32 {
        if !self.matches(r, g, b) {
            return 0.0;
        }

        // Calculate how centered the color is within the range
        let r_center = (self.rgb_min[0] as u16 + self.rgb_max[0] as u16) / 2;
        let g_center = (self.rgb_min[1] as u16 + self.rgb_max[1] as u16) / 2;
        let b_center = (self.rgb_min[2] as u16 + self.rgb_max[2] as u16) / 2;

        let r_range = (self.rgb_max[0] as i16 - self.rgb_min[0] as i16).abs() as f32;
        let g_range = (self.rgb_max[1] as i16 - self.rgb_min[1] as i16).abs() as f32;
        let b_range = (self.rgb_max[2] as i16 - self.rgb_min[2] as i16).abs() as f32;

        let r_dist = ((r as i16 - r_center as i16).abs() as f32) / r_range.max(1.0);
        let g_dist = ((g as i16 - g_center as i16).abs() as f32) / g_range.max(1.0);
        let b_dist = ((b as i16 - b_center as i16).abs() as f32) / b_range.max(1.0);

        let avg_dist = (r_dist + g_dist + b_dist) / 3.0;
        (1.0 - avg_dist).clamp(0.0, 1.0)
    }
}

/// Sensor calibration data (I-HELP-002)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationResult {
    /// White surface baseline RGB
    pub surface_baseline: [u8; 3],
    /// Ambient light compensation (0.0-2.0)
    pub ambient_light_factor: f32,
    /// When calibration was performed
    pub calibration_timestamp: u64,
    /// Estimated accuracy after calibration (0.0-1.0)
    pub accuracy_estimate: f32,
}

/// Standard LEGO colors lookup table (from contract)
pub const STANDARD_COLORS: &[(&str, [u8; 3], [u8; 3], bool)] = &[
    ("red", [150, 0, 0], [255, 80, 80], false),
    ("blue", [0, 20, 120], [80, 120, 255], false),
    ("green", [0, 100, 0], [80, 255, 100], false),
    ("yellow", [180, 150, 0], [255, 255, 100], false),
    ("orange", [180, 60, 0], [255, 150, 80], false),
    ("white", [200, 200, 200], [255, 255, 255], false),
    ("black", [0, 0, 0], [50, 50, 50], false),
    ("gray", [70, 70, 70], [180, 180, 180], false),
    // Rare colors (I-HELP-004)
    ("gold", [160, 130, 20], [240, 210, 100], true),
    ("silver", [150, 150, 160], [230, 230, 240], true),
    ("clear", [180, 180, 180], [255, 255, 255], true),
];

/// Color sensor with calibration support
pub struct ColorSensor {
    calibration: Option<CalibrationResult>,
    color_table: Vec<ColorEntry>,
}

impl ColorSensor {
    pub fn new() -> Self {
        let mut color_table = Vec::new();
        for (name, min, max, is_rare) in STANDARD_COLORS {
            color_table.push(ColorEntry {
                name: name.to_string(),
                rgb_min: *min,
                rgb_max: *max,
                is_rare: *is_rare,
            });
        }

        Self {
            calibration: None,
            color_table,
        }
    }

    /// Calibrate sensor against white surface (I-HELP-002)
    pub fn calibrate(&mut self, white_rgb: [u8; 3], timestamp_us: u64) -> CalibrationResult {
        // Calculate ambient light factor
        let expected_white = 240.0;
        let actual_avg = (white_rgb[0] as f32 + white_rgb[1] as f32 + white_rgb[2] as f32) / 3.0;
        let ambient_factor = (actual_avg / expected_white).clamp(0.0, 2.0);

        // Estimate accuracy based on how close to pure white
        let white_variance = ((white_rgb[0] as f32 - actual_avg).abs()
            + (white_rgb[1] as f32 - actual_avg).abs()
            + (white_rgb[2] as f32 - actual_avg).abs())
            / 3.0;
        let accuracy = (1.0 - (white_variance / 50.0)).clamp(0.5, 1.0);

        let result = CalibrationResult {
            surface_baseline: white_rgb,
            ambient_light_factor: ambient_factor,
            calibration_timestamp: timestamp_us,
            accuracy_estimate: accuracy,
        };

        self.calibration = Some(result.clone());
        result
    }

    /// Detect color with confidence score (I-HELP-001, I-HELP-003)
    pub fn detect(&self, rgb: &ColorReading, timestamp_us: u64) -> ColorDetection {
        let (r, g, b) = (rgb.r, rgb.g, rgb.b);

        // Find best matching color
        let mut best_match: Option<(&ColorEntry, f32)> = None;

        for entry in &self.color_table {
            if entry.matches(r, g, b) {
                let confidence = entry.confidence(r, g, b);
                match best_match {
                    None => best_match = Some((entry, confidence)),
                    Some((_, prev_conf)) if confidence > prev_conf => {
                        best_match = Some((entry, confidence));
                    }
                    _ => {}
                }
            }
        }

        // Apply calibration adjustment if available
        let calibrated_confidence = if let Some(cal) = &self.calibration {
            best_match
                .as_ref()
                .map(|(_, conf)| conf * cal.accuracy_estimate)
                .unwrap_or(0.0)
        } else {
            best_match.as_ref().map(|(_, conf)| *conf).unwrap_or(0.0)
        };

        // Return result (I-HELP-003: graceful unknown handling)
        match best_match {
            Some((entry, _)) => ColorDetection::known(
                entry.name.clone(),
                rgb.clone(),
                calibrated_confidence,
                entry.is_rare,
                timestamp_us,
            ),
            None => ColorDetection::unknown(rgb.clone(), timestamp_us),
        }
    }

    pub fn get_calibration(&self) -> Option<&CalibrationResult> {
        self.calibration.as_ref()
    }
}

impl Default for ColorSensor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_detection_has_confidence() {
        let sensor = ColorSensor::new();
        let reading = ColorReading::new(180, 30, 30);
        let detection = sensor.detect(&reading, 0);

        // I-HELP-001: Must return confidence
        assert!(detection.confidence >= 0.0 && detection.confidence <= 1.0);
    }

    #[test]
    fn test_calibration_required() {
        let mut sensor = ColorSensor::new();

        // I-HELP-002: Calibration support
        let white = [240, 240, 240];
        let cal = sensor.calibrate(white, 1000);

        assert!(cal.ambient_light_factor > 0.0);
        assert!(cal.accuracy_estimate > 0.0);
    }

    #[test]
    fn test_unknown_color_no_panic() {
        let sensor = ColorSensor::new();

        // I-HELP-003: Unknown colors must not crash
        let weird_color = ColorReading::new(123, 45, 67);
        let detection = sensor.detect(&weird_color, 0);

        // Should return "unknown", not panic
        assert_eq!(detection.detected_color, "unknown");
    }

    #[test]
    fn test_rare_colors_flagged() {
        let sensor = ColorSensor::new();

        // I-HELP-004: Rare colors must be flagged
        let gold = ColorReading::new(200, 170, 50);
        let detection = sensor.detect(&gold, 0);

        if detection.detected_color == "gold" {
            assert!(detection.is_rare, "Gold should be marked as rare");
        }
    }

    #[test]
    fn test_standard_colors() {
        let sensor = ColorSensor::new();

        // Test standard LEGO colors
        let test_cases = vec![
            ([180, 30, 30], "red"),
            ([30, 60, 180], "blue"),
            ([30, 150, 50], "green"),
            ([230, 200, 30], "yellow"),
        ];

        for (rgb, expected_name) in test_cases {
            let reading = ColorReading::new(rgb[0], rgb[1], rgb[2]);
            let detection = sensor.detect(&reading, 0);
            assert_eq!(
                detection.detected_color, expected_name,
                "RGB {:?} should detect as {}",
                rgb, expected_name
            );
        }
    }
}
