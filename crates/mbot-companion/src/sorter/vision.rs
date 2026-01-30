//! Color Detection Vision System for LEGO Sorter
//!
//! Implements SORT-002 (Deterministic), SORT-003 (Color Detection), SORT-005 (Offline-First)
//!
//! This module provides computer vision capabilities for detecting LEGO pieces
//! in a sorting tray, identifying their color and position for pick operations.

use std::collections::HashMap;

// ============================================
// Core Enums
// ============================================

/// LEGO piece colors supported by the vision system
/// Contract: SORT-003 LegoColor enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LegoColor {
    Red = 0,
    Blue = 1,
    Yellow = 2,
    Green = 3,
    Black = 4,
    White = 5,
    Orange = 6,
    Purple = 7,
    Tan = 8,
    Gray = 9,
    Brown = 10,
    Unknown = 255,
}

impl LegoColor {
    /// Get display name for the color
    pub fn name(&self) -> &'static str {
        match self {
            LegoColor::Red => "Red",
            LegoColor::Blue => "Blue",
            LegoColor::Yellow => "Yellow",
            LegoColor::Green => "Green",
            LegoColor::Black => "Black",
            LegoColor::White => "White",
            LegoColor::Orange => "Orange",
            LegoColor::Purple => "Purple",
            LegoColor::Tan => "Tan",
            LegoColor::Gray => "Gray",
            LegoColor::Brown => "Brown",
            LegoColor::Unknown => "Unknown",
        }
    }

    /// Get all known colors (excluding Unknown)
    pub fn all_known() -> &'static [LegoColor] {
        &[
            LegoColor::Red,
            LegoColor::Blue,
            LegoColor::Yellow,
            LegoColor::Green,
            LegoColor::Black,
            LegoColor::White,
            LegoColor::Orange,
            LegoColor::Purple,
            LegoColor::Tan,
            LegoColor::Gray,
            LegoColor::Brown,
        ]
    }
}

/// Estimated piece size category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceSize {
    Small,  // 1x1, 1x2 pieces
    Medium, // 2x2, 2x3, 1x4 pieces
    Large,  // 2x4 and larger
}

impl PieceSize {
    /// Estimate size from bounding box area in pixels
    pub fn from_pixel_area(area: u32, pixels_per_mm: f32) -> Self {
        // Convert to approximate mm^2
        let area_mm2 = area as f32 / (pixels_per_mm * pixels_per_mm);

        if area_mm2 < 100.0 {
            PieceSize::Small
        } else if area_mm2 < 300.0 {
            PieceSize::Medium
        } else {
            PieceSize::Large
        }
    }
}

/// Lighting mode for vision capture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingMode {
    Bright,
    Dim,
    Auto,
}

/// Lighting quality assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingStatus {
    Adequate,
    TooDim,
    TooBright,
    Uneven,
}

/// Warning types that can be flagged during detection
#[derive(Debug, Clone, PartialEq)]
pub enum VisionWarning {
    LightingInadequate { status: LightingStatus, suggestion: String },
    LowContrast { background: String, piece_color: LegoColor },
    ClusterDetected { piece_ids: Vec<String> },
    NeedsReposition { observation_id: String, reason: String },
    ReflectiveSurface { observation_id: String },
}

// ============================================
// Core Structs
// ============================================

/// 2D position in millimeters from tray origin
/// Contract: SORT-003 Position2D
#[derive(Debug, Clone, Copy, Default)]
pub struct Position2D {
    /// X coordinate in millimeters
    pub x: f32,
    /// Y coordinate in millimeters
    pub y: f32,
}

impl Position2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Calculate Euclidean distance to another position
    pub fn distance_to(&self, other: &Position2D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Bounding box for piece detection
/// Contract: SORT-003 BoundingBox
#[derive(Debug, Clone, Copy, Default)]
pub struct BoundingBox {
    /// Top-left X coordinate in pixels
    pub x: u32,
    /// Top-left Y coordinate in pixels
    pub y: u32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl BoundingBox {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// Calculate area in pixels
    pub fn area(&self) -> u32 {
        self.width * self.height
    }

    /// Get center point in pixels
    pub fn center(&self) -> (u32, u32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    /// Check if this bounding box overlaps with another
    pub fn overlaps(&self, other: &BoundingBox) -> bool {
        !(self.x + self.width < other.x
            || other.x + other.width < self.x
            || self.y + self.height < other.y
            || other.y + other.height < self.y)
    }

    /// Check if a point is inside this bounding box
    pub fn contains_point(&self, px: u32, py: u32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }
}

/// HSV color representation for detection
#[derive(Debug, Clone, Copy, Default)]
pub struct HsvColor {
    /// Hue (0-360 degrees)
    pub h: f32,
    /// Saturation (0.0-1.0)
    pub s: f32,
    /// Value/Brightness (0.0-1.0)
    pub v: f32,
}

impl HsvColor {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self { h, s, v }
    }

    /// Convert from RGB (0-255 per channel)
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };

        let s = if max == 0.0 { 0.0 } else { delta / max };

        Self { h, s, v: max }
    }
}

/// HSV range for color matching
#[derive(Debug, Clone, Copy)]
pub struct HsvRange {
    /// Hue range (min, max) in degrees (0-360)
    pub h: (f32, f32),
    /// Saturation range (min, max) (0.0-1.0)
    pub s: (f32, f32),
    /// Value range (min, max) (0.0-1.0)
    pub v: (f32, f32),
}

impl HsvRange {
    pub fn new(h: (f32, f32), s: (f32, f32), v: (f32, f32)) -> Self {
        Self { h, s, v }
    }

    /// Check if an HSV color falls within this range
    /// Handles hue wraparound (e.g., red spans 350-10 degrees)
    pub fn contains(&self, hsv: &HsvColor) -> bool {
        let h_match = if self.h.0 <= self.h.1 {
            // Normal range
            hsv.h >= self.h.0 && hsv.h <= self.h.1
        } else {
            // Wraparound range (e.g., red: 350-10)
            hsv.h >= self.h.0 || hsv.h <= self.h.1
        };

        h_match
            && hsv.s >= self.s.0
            && hsv.s <= self.s.1
            && hsv.v >= self.v.0
            && hsv.v <= self.v.1
    }
}

/// Color sample for calibration
#[derive(Debug, Clone)]
pub struct ColorSample {
    /// The LEGO color this sample represents
    pub color: LegoColor,
    /// HSV range for this color
    pub hsv_range: HsvRange,
}

/// Color calibration data
/// Contract: SORT-003 ColorCalibration
#[derive(Debug, Clone)]
pub struct ColorCalibration {
    /// Reference samples with HSV ranges per color
    pub reference_samples: Vec<ColorSample>,
    /// Timestamp of last calibration (microseconds)
    pub last_calibrated: u64,
}

impl Default for ColorCalibration {
    fn default() -> Self {
        Self::new_with_defaults()
    }
}

impl ColorCalibration {
    /// Create a new calibration with default HSV ranges for common LEGO colors
    pub fn new_with_defaults() -> Self {
        let reference_samples = vec![
            ColorSample {
                color: LegoColor::Red,
                hsv_range: HsvRange::new((350.0, 10.0), (0.6, 1.0), (0.3, 1.0)),
            },
            ColorSample {
                color: LegoColor::Blue,
                hsv_range: HsvRange::new((200.0, 240.0), (0.5, 1.0), (0.3, 1.0)),
            },
            ColorSample {
                color: LegoColor::Yellow,
                hsv_range: HsvRange::new((45.0, 65.0), (0.6, 1.0), (0.5, 1.0)),
            },
            ColorSample {
                color: LegoColor::Green,
                hsv_range: HsvRange::new((80.0, 150.0), (0.4, 1.0), (0.3, 1.0)),
            },
            ColorSample {
                color: LegoColor::Black,
                hsv_range: HsvRange::new((0.0, 360.0), (0.0, 0.3), (0.0, 0.2)),
            },
            ColorSample {
                color: LegoColor::White,
                hsv_range: HsvRange::new((0.0, 360.0), (0.0, 0.15), (0.8, 1.0)),
            },
            ColorSample {
                color: LegoColor::Orange,
                hsv_range: HsvRange::new((15.0, 40.0), (0.7, 1.0), (0.5, 1.0)),
            },
            ColorSample {
                color: LegoColor::Purple,
                hsv_range: HsvRange::new((270.0, 310.0), (0.3, 1.0), (0.2, 0.8)),
            },
            ColorSample {
                color: LegoColor::Tan,
                hsv_range: HsvRange::new((30.0, 50.0), (0.2, 0.5), (0.5, 0.85)),
            },
            ColorSample {
                color: LegoColor::Gray,
                hsv_range: HsvRange::new((0.0, 360.0), (0.0, 0.15), (0.25, 0.75)),
            },
            ColorSample {
                color: LegoColor::Brown,
                hsv_range: HsvRange::new((10.0, 30.0), (0.4, 0.8), (0.2, 0.5)),
            },
        ];

        Self {
            reference_samples,
            last_calibrated: 0,
        }
    }

    /// Classify an HSV color to a LegoColor
    /// Contract: SORT-002 - Deterministic classification
    pub fn classify_hsv(&self, hsv: &HsvColor) -> LegoColor {
        for sample in &self.reference_samples {
            if sample.hsv_range.contains(hsv) {
                return sample.color;
            }
        }
        LegoColor::Unknown
    }

    /// Update the HSV range for a specific color
    pub fn update_color_range(&mut self, color: LegoColor, range: HsvRange, timestamp: u64) {
        if let Some(sample) = self.reference_samples.iter_mut().find(|s| s.color == color) {
            sample.hsv_range = range;
        } else {
            self.reference_samples.push(ColorSample {
                color,
                hsv_range: range,
            });
        }
        self.last_calibrated = timestamp;
    }
}

/// Tray region definition
/// Contract: SORT-003 TrayRegion
#[derive(Debug, Clone)]
pub struct TrayRegion {
    /// Top-left corner position in mm
    pub top_left: Position2D,
    /// Bottom-right corner position in mm
    pub bottom_right: Position2D,
    /// Background color name for contrast detection
    pub background_color: String,
}

impl TrayRegion {
    pub fn new(top_left: Position2D, bottom_right: Position2D, background_color: &str) -> Self {
        Self {
            top_left,
            bottom_right,
            background_color: background_color.to_string(),
        }
    }

    /// Get tray width in mm
    pub fn width(&self) -> f32 {
        self.bottom_right.x - self.top_left.x
    }

    /// Get tray height in mm
    pub fn height(&self) -> f32 {
        self.bottom_right.y - self.top_left.y
    }

    /// Check if a position is within the tray region
    pub fn contains(&self, pos: &Position2D) -> bool {
        pos.x >= self.top_left.x
            && pos.x <= self.bottom_right.x
            && pos.y >= self.top_left.y
            && pos.y <= self.bottom_right.y
    }
}

/// Vision system configuration
/// Contract: SORT-003 VisionConfig
#[derive(Debug, Clone)]
pub struct VisionConfig {
    /// Camera identifier
    pub camera_id: String,
    /// Defined tray region for detection
    pub tray_region: TrayRegion,
    /// Minimum confidence threshold for valid detection (0.0-1.0)
    pub min_confidence: f32,
    /// Lighting mode setting
    pub lighting_mode: LightingMode,
    /// Color calibration data
    pub color_calibration: ColorCalibration,
    /// Pixels per millimeter (for coordinate conversion)
    pub pixels_per_mm: f32,
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            camera_id: "default_camera".to_string(),
            tray_region: TrayRegion::new(
                Position2D::new(0.0, 0.0),
                Position2D::new(200.0, 150.0),
                "white",
            ),
            min_confidence: 0.75,
            lighting_mode: LightingMode::Auto,
            color_calibration: ColorCalibration::default(),
            pixels_per_mm: 5.0, // Default: 5 pixels per mm
        }
    }
}

/// Detected piece observation
/// Contract: SORT-003 PieceObservation
#[derive(Debug, Clone)]
pub struct PieceObservation {
    /// Unique identifier for this observation
    pub observation_id: String,
    /// Timestamp in microseconds
    pub timestamp: u64,
    /// Detected color
    pub color: LegoColor,
    /// Future: coarse piece type (brick, plate, axle)
    pub coarse_type: Option<String>,
    /// Bounding box in pixels
    pub bbox: BoundingBox,
    /// Detection confidence (0.0-1.0)
    pub confidence: f32,
    /// Center position in millimeters from tray origin
    pub center_position: Position2D,
    /// Estimated piece size category
    pub estimated_size: PieceSize,
}

impl PieceObservation {
    /// Create a new piece observation
    pub fn new(
        observation_id: String,
        timestamp: u64,
        color: LegoColor,
        bbox: BoundingBox,
        confidence: f32,
        center_position: Position2D,
        estimated_size: PieceSize,
    ) -> Self {
        Self {
            observation_id,
            timestamp,
            color,
            coarse_type: None,
            bbox,
            confidence: confidence.clamp(0.0, 1.0),
            center_position,
            estimated_size,
        }
    }

    /// Check if this observation meets the minimum confidence threshold
    pub fn is_valid(&self, min_confidence: f32) -> bool {
        self.confidence >= min_confidence
    }
}

/// Result of a vision analysis
#[derive(Debug, Clone)]
pub struct VisionAnalysisResult {
    /// Detected pieces
    pub observations: Vec<PieceObservation>,
    /// Warnings generated during analysis
    pub warnings: Vec<VisionWarning>,
    /// Lighting status assessment
    pub lighting_status: LightingStatus,
    /// Analysis timestamp
    pub timestamp: u64,
}

impl VisionAnalysisResult {
    /// Get only valid observations above min_confidence
    pub fn valid_observations(&self, min_confidence: f32) -> Vec<&PieceObservation> {
        self.observations
            .iter()
            .filter(|o| o.is_valid(min_confidence))
            .collect()
    }

    /// Check if tray is empty (no valid detections)
    pub fn is_empty(&self, min_confidence: f32) -> bool {
        self.valid_observations(min_confidence).is_empty()
    }

    /// Get piece count
    pub fn piece_count(&self) -> usize {
        self.observations.len()
    }

    /// Get the next piece to pick (closest to pick position)
    pub fn next_to_pick(&self, pick_position: &Position2D, min_confidence: f32) -> Option<&PieceObservation> {
        self.valid_observations(min_confidence)
            .into_iter()
            .min_by(|a, b| {
                let dist_a = a.center_position.distance_to(pick_position);
                let dist_b = b.center_position.distance_to(pick_position);
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

// ============================================
// Vision Detector
// ============================================

/// Main vision detector for LEGO piece detection
/// Contract: SORT-005 - Offline-first (no cloud dependencies)
#[derive(Debug)]
pub struct VisionDetector {
    config: VisionConfig,
    observation_counter: u64,
}

impl VisionDetector {
    /// Create a new vision detector with the given configuration
    pub fn new(config: VisionConfig) -> Self {
        Self {
            config,
            observation_counter: 0,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(VisionConfig::default())
    }

    /// Get the current configuration
    pub fn config(&self) -> &VisionConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: VisionConfig) {
        self.config = config;
    }

    /// Update color calibration
    pub fn update_calibration(&mut self, calibration: ColorCalibration) {
        self.config.color_calibration = calibration;
    }

    /// Classify color from HSV values
    /// Contract: SORT-002 - Deterministic classification
    pub fn classify_color(&self, hsv: &HsvColor) -> LegoColor {
        self.config.color_calibration.classify_hsv(hsv)
    }

    /// Analyze a frame of RGB pixel data
    /// Contract: SORT-005 - Offline-first (synchronous, no network calls)
    ///
    /// # Arguments
    /// * `rgb_data` - Raw RGB pixel data (width * height * 3 bytes)
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    /// * `timestamp` - Frame timestamp in microseconds
    pub fn analyze_frame(
        &mut self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        timestamp: u64,
    ) -> VisionAnalysisResult {
        let mut observations = Vec::new();
        let mut warnings = Vec::new();

        // Assess lighting quality
        let lighting_status = self.assess_lighting(rgb_data);

        if lighting_status == LightingStatus::TooDim {
            warnings.push(VisionWarning::LightingInadequate {
                status: lighting_status,
                suggestion: "improve_lighting_or_recalibrate".to_string(),
            });
        }

        // Find connected regions (simplified blob detection)
        let blobs = self.find_blobs(rgb_data, width, height);

        // Process each blob
        for blob in blobs {
            self.observation_counter += 1;
            let obs_id = format!("obs_{}", self.observation_counter);

            // Get average color of blob
            let avg_hsv = self.average_hsv_in_bbox(rgb_data, width, &blob);
            let color = self.classify_color(&avg_hsv);

            // Calculate confidence based on color saturation and blob quality
            let confidence = self.calculate_confidence(&avg_hsv, &blob);

            // Convert pixel position to mm
            let center_px = blob.center();
            let center_mm = Position2D::new(
                center_px.0 as f32 / self.config.pixels_per_mm,
                center_px.1 as f32 / self.config.pixels_per_mm,
            );

            // Estimate piece size
            let size = PieceSize::from_pixel_area(blob.area(), self.config.pixels_per_mm);

            let observation = PieceObservation::new(
                obs_id.clone(),
                timestamp,
                color,
                blob,
                confidence,
                center_mm,
                size,
            );

            // Check for low confidence
            if !observation.is_valid(self.config.min_confidence) {
                warnings.push(VisionWarning::NeedsReposition {
                    observation_id: obs_id.clone(),
                    reason: "low_confidence".to_string(),
                });
            }

            // Check for contrast issues
            if color == LegoColor::White && self.config.tray_region.background_color == "white" {
                warnings.push(VisionWarning::LowContrast {
                    background: "white".to_string(),
                    piece_color: LegoColor::White,
                });
            }

            // Check for reflective surface
            if color == LegoColor::Unknown && avg_hsv.v > 0.9 {
                warnings.push(VisionWarning::ReflectiveSurface {
                    observation_id: obs_id,
                });
            }

            observations.push(observation);
        }

        // Check for touching/overlapping pieces
        self.detect_clusters(&observations, &mut warnings);

        VisionAnalysisResult {
            observations,
            warnings,
            lighting_status,
            timestamp,
        }
    }

    /// Assess lighting quality from image brightness
    fn assess_lighting(&self, rgb_data: &[u8]) -> LightingStatus {
        if rgb_data.is_empty() {
            return LightingStatus::TooDim;
        }

        // Calculate average brightness (simple V channel approximation)
        let total_brightness: u64 = rgb_data
            .chunks(3)
            .map(|rgb| rgb.iter().map(|&c| c as u64).max().unwrap_or(0))
            .sum();

        let pixel_count = rgb_data.len() / 3;
        if pixel_count == 0 {
            return LightingStatus::TooDim;
        }

        let avg_brightness = total_brightness / pixel_count as u64;

        if avg_brightness < 50 {
            LightingStatus::TooDim
        } else if avg_brightness > 230 {
            LightingStatus::TooBright
        } else {
            LightingStatus::Adequate
        }
    }

    /// Find blobs (connected regions) in the image
    /// This is a simplified implementation - real version would use proper CV
    fn find_blobs(&self, rgb_data: &[u8], width: u32, height: u32) -> Vec<BoundingBox> {
        // Simplified: divide image into grid and detect high-saturation regions
        // In production, use proper connected component analysis

        let mut blobs = Vec::new();
        let cell_size = 20u32; // Grid cell size in pixels
        let mut visited: HashMap<(u32, u32), bool> = HashMap::new();

        for cy in (0..height).step_by(cell_size as usize) {
            for cx in (0..width).step_by(cell_size as usize) {
                if visited.get(&(cx, cy)).copied().unwrap_or(false) {
                    continue;
                }

                // Check if this cell has significant color saturation
                let hsv = self.sample_hsv_at(rgb_data, width, height, cx, cy);

                // Skip if too low saturation (background) or too dark/bright
                if hsv.s < 0.2 || hsv.v < 0.1 || hsv.v > 0.95 {
                    continue;
                }

                // Flood fill to find blob extent
                let bbox = self.flood_fill_bbox(
                    rgb_data, width, height,
                    cx, cy, cell_size,
                    &mut visited, &hsv,
                );

                // Only add blobs of reasonable size
                if bbox.width >= cell_size && bbox.height >= cell_size {
                    blobs.push(bbox);
                }
            }
        }

        blobs
    }

    /// Sample HSV at a position
    fn sample_hsv_at(&self, rgb_data: &[u8], width: u32, _height: u32, x: u32, y: u32) -> HsvColor {
        let idx = ((y * width + x) * 3) as usize;
        if idx + 2 < rgb_data.len() {
            HsvColor::from_rgb(rgb_data[idx], rgb_data[idx + 1], rgb_data[idx + 2])
        } else {
            HsvColor::default()
        }
    }

    /// Flood fill to find bounding box of a blob
    fn flood_fill_bbox(
        &self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        start_x: u32,
        start_y: u32,
        cell_size: u32,
        visited: &mut HashMap<(u32, u32), bool>,
        reference_hsv: &HsvColor,
    ) -> BoundingBox {
        let mut min_x = start_x;
        let mut min_y = start_y;
        let mut max_x = start_x + cell_size;
        let mut max_y = start_y + cell_size;

        let mut stack = vec![(start_x, start_y)];

        while let Some((cx, cy)) = stack.pop() {
            if visited.get(&(cx, cy)).copied().unwrap_or(false) {
                continue;
            }
            if cx >= width || cy >= height {
                continue;
            }

            let hsv = self.sample_hsv_at(rgb_data, width, height, cx, cy);

            // Check if this cell is similar to reference color
            let h_diff = (hsv.h - reference_hsv.h).abs();
            let h_similar = h_diff < 30.0 || h_diff > 330.0;
            let s_similar = (hsv.s - reference_hsv.s).abs() < 0.3;

            if !h_similar || !s_similar || hsv.s < 0.2 {
                continue;
            }

            visited.insert((cx, cy), true);

            // Expand bounding box
            min_x = min_x.min(cx);
            min_y = min_y.min(cy);
            max_x = max_x.max(cx + cell_size);
            max_y = max_y.max(cy + cell_size);

            // Add neighbors
            if cx >= cell_size {
                stack.push((cx - cell_size, cy));
            }
            if cx + cell_size < width {
                stack.push((cx + cell_size, cy));
            }
            if cy >= cell_size {
                stack.push((cx, cy - cell_size));
            }
            if cy + cell_size < height {
                stack.push((cx, cy + cell_size));
            }
        }

        BoundingBox::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// Calculate average HSV within a bounding box
    fn average_hsv_in_bbox(&self, rgb_data: &[u8], width: u32, bbox: &BoundingBox) -> HsvColor {
        let mut total_h = 0.0f64;
        let mut total_s = 0.0f64;
        let mut total_v = 0.0f64;
        let mut count = 0u64;

        // Sample points within bbox
        let step = 4u32; // Sample every 4th pixel
        for y in (bbox.y..bbox.y + bbox.height).step_by(step as usize) {
            for x in (bbox.x..bbox.x + bbox.width).step_by(step as usize) {
                let idx = ((y * width + x) * 3) as usize;
                if idx + 2 < rgb_data.len() {
                    let hsv = HsvColor::from_rgb(rgb_data[idx], rgb_data[idx + 1], rgb_data[idx + 2]);
                    total_h += hsv.h as f64;
                    total_s += hsv.s as f64;
                    total_v += hsv.v as f64;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return HsvColor::default();
        }

        HsvColor {
            h: (total_h / count as f64) as f32,
            s: (total_s / count as f64) as f32,
            v: (total_v / count as f64) as f32,
        }
    }

    /// Calculate detection confidence
    fn calculate_confidence(&self, hsv: &HsvColor, bbox: &BoundingBox) -> f32 {
        // Base confidence from color saturation
        let saturation_factor = hsv.s.clamp(0.0, 1.0);

        // Bonus for reasonable blob size
        let area = bbox.area() as f32;
        let size_factor = if area > 400.0 && area < 50000.0 { 1.0 } else { 0.7 };

        // Penalty for very bright (possibly reflective) or very dark
        let brightness_factor = if hsv.v > 0.9 || hsv.v < 0.15 { 0.6 } else { 1.0 };

        (saturation_factor * size_factor * brightness_factor).clamp(0.0, 1.0)
    }

    /// Detect clusters of touching pieces
    fn detect_clusters(&self, observations: &[PieceObservation], warnings: &mut Vec<VisionWarning>) {
        for i in 0..observations.len() {
            for j in (i + 1)..observations.len() {
                if observations[i].bbox.overlaps(&observations[j].bbox) {
                    warnings.push(VisionWarning::ClusterDetected {
                        piece_ids: vec![
                            observations[i].observation_id.clone(),
                            observations[j].observation_id.clone(),
                        ],
                    });
                }
            }
        }
    }

    /// Run color calibration with a reference piece
    /// Returns the detected HSV range for the piece
    pub fn calibrate_color(
        &mut self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        color: LegoColor,
        timestamp: u64,
    ) -> Option<HsvRange> {
        // Find the main blob in the image
        let blobs = self.find_blobs(rgb_data, width, height);

        if blobs.is_empty() {
            return None;
        }

        // Use the largest blob
        let largest_blob = blobs.iter().max_by_key(|b| b.area())?;

        // Get HSV statistics within the blob
        let (min_hsv, max_hsv) = self.hsv_range_in_bbox(rgb_data, width, largest_blob);

        // Create range with some margin
        let margin_h = 10.0;
        let margin_s = 0.1;
        let margin_v = 0.1;

        let range = HsvRange::new(
            ((min_hsv.h - margin_h).max(0.0), (max_hsv.h + margin_h).min(360.0)),
            ((min_hsv.s - margin_s).max(0.0), (max_hsv.s + margin_s).min(1.0)),
            ((min_hsv.v - margin_v).max(0.0), (max_hsv.v + margin_v).min(1.0)),
        );

        // Update calibration
        self.config.color_calibration.update_color_range(color, range, timestamp);

        Some(range)
    }

    /// Get min/max HSV values within a bounding box
    fn hsv_range_in_bbox(
        &self,
        rgb_data: &[u8],
        width: u32,
        bbox: &BoundingBox,
    ) -> (HsvColor, HsvColor) {
        let mut min_h = 360.0f32;
        let mut max_h = 0.0f32;
        let mut min_s = 1.0f32;
        let mut max_s = 0.0f32;
        let mut min_v = 1.0f32;
        let mut max_v = 0.0f32;

        let step = 4u32;
        for y in (bbox.y..bbox.y + bbox.height).step_by(step as usize) {
            for x in (bbox.x..bbox.x + bbox.width).step_by(step as usize) {
                let idx = ((y * width + x) * 3) as usize;
                if idx + 2 < rgb_data.len() {
                    let hsv = HsvColor::from_rgb(rgb_data[idx], rgb_data[idx + 1], rgb_data[idx + 2]);
                    min_h = min_h.min(hsv.h);
                    max_h = max_h.max(hsv.h);
                    min_s = min_s.min(hsv.s);
                    max_s = max_s.max(hsv.s);
                    min_v = min_v.min(hsv.v);
                    max_v = max_v.max(hsv.v);
                }
            }
        }

        (
            HsvColor::new(min_h, min_s, min_v),
            HsvColor::new(max_h, max_s, max_v),
        )
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lego_color_names() {
        assert_eq!(LegoColor::Red.name(), "Red");
        assert_eq!(LegoColor::Blue.name(), "Blue");
        assert_eq!(LegoColor::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_hsv_from_rgb() {
        // Pure red
        let hsv = HsvColor::from_rgb(255, 0, 0);
        assert!((hsv.h - 0.0).abs() < 1.0 || (hsv.h - 360.0).abs() < 1.0);
        assert!((hsv.s - 1.0).abs() < 0.01);
        assert!((hsv.v - 1.0).abs() < 0.01);

        // Pure green
        let hsv = HsvColor::from_rgb(0, 255, 0);
        assert!((hsv.h - 120.0).abs() < 1.0);

        // Pure blue
        let hsv = HsvColor::from_rgb(0, 0, 255);
        assert!((hsv.h - 240.0).abs() < 1.0);

        // White
        let hsv = HsvColor::from_rgb(255, 255, 255);
        assert!(hsv.s < 0.01);
        assert!((hsv.v - 1.0).abs() < 0.01);

        // Black
        let hsv = HsvColor::from_rgb(0, 0, 0);
        assert!(hsv.v < 0.01);
    }

    #[test]
    fn test_hsv_range_contains() {
        // Normal range
        let range = HsvRange::new((100.0, 150.0), (0.5, 1.0), (0.3, 0.9));
        assert!(range.contains(&HsvColor::new(125.0, 0.7, 0.5)));
        assert!(!range.contains(&HsvColor::new(200.0, 0.7, 0.5)));

        // Wraparound range (red)
        let red_range = HsvRange::new((350.0, 10.0), (0.5, 1.0), (0.3, 1.0));
        assert!(red_range.contains(&HsvColor::new(355.0, 0.8, 0.5)));
        assert!(red_range.contains(&HsvColor::new(5.0, 0.8, 0.5)));
        assert!(!red_range.contains(&HsvColor::new(180.0, 0.8, 0.5)));
    }

    #[test]
    fn test_bounding_box_operations() {
        let bbox = BoundingBox::new(10, 20, 100, 50);

        assert_eq!(bbox.area(), 5000);
        assert_eq!(bbox.center(), (60, 45));
        assert!(bbox.contains_point(50, 30));
        assert!(!bbox.contains_point(5, 30));

        let bbox2 = BoundingBox::new(50, 30, 100, 50);
        assert!(bbox.overlaps(&bbox2));

        let bbox3 = BoundingBox::new(200, 200, 50, 50);
        assert!(!bbox.overlaps(&bbox3));
    }

    #[test]
    fn test_position_distance() {
        let p1 = Position2D::new(0.0, 0.0);
        let p2 = Position2D::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_piece_size_from_area() {
        let ppm = 5.0; // pixels per mm

        // Small piece (1x1 stud ~= 8mm x 8mm = 64 mm^2)
        let small = PieceSize::from_pixel_area(64 * 25, ppm); // 64 mm^2 * (5^2) px/mm^2
        assert_eq!(small, PieceSize::Small);

        // Medium piece (2x2 ~= 16x16 = 256 mm^2)
        let medium = PieceSize::from_pixel_area(200 * 25, ppm);
        assert_eq!(medium, PieceSize::Medium);

        // Large piece (2x4 ~= 16x32 = 512 mm^2)
        let large = PieceSize::from_pixel_area(500 * 25, ppm);
        assert_eq!(large, PieceSize::Large);
    }

    #[test]
    fn test_color_calibration_defaults() {
        let calibration = ColorCalibration::default();

        // Test red detection (hue ~0)
        let red_hsv = HsvColor::new(5.0, 0.9, 0.7);
        assert_eq!(calibration.classify_hsv(&red_hsv), LegoColor::Red);

        // Test blue detection (hue ~220)
        let blue_hsv = HsvColor::new(220.0, 0.8, 0.6);
        assert_eq!(calibration.classify_hsv(&blue_hsv), LegoColor::Blue);

        // Test yellow detection (hue ~55)
        let yellow_hsv = HsvColor::new(55.0, 0.8, 0.8);
        assert_eq!(calibration.classify_hsv(&yellow_hsv), LegoColor::Yellow);

        // Test gray (low saturation, mid value)
        let gray_hsv = HsvColor::new(180.0, 0.05, 0.5); // Low saturation, mid value
        assert_eq!(calibration.classify_hsv(&gray_hsv), LegoColor::Gray);
    }

    #[test]
    fn test_vision_detector_determinism() {
        // Contract: SORT-002 - Same input must produce same output
        let detector = VisionDetector::with_defaults();

        let hsv1 = HsvColor::new(5.0, 0.9, 0.7);
        let hsv2 = HsvColor::new(5.0, 0.9, 0.7);

        let color1 = detector.classify_color(&hsv1);
        let color2 = detector.classify_color(&hsv2);

        assert_eq!(color1, color2, "Determinism: Same HSV must produce same color");
    }

    #[test]
    fn test_piece_observation_validity() {
        let obs = PieceObservation::new(
            "test_1".to_string(),
            0,
            LegoColor::Red,
            BoundingBox::new(10, 10, 50, 50),
            0.85,
            Position2D::new(35.0, 35.0),
            PieceSize::Medium,
        );

        assert!(obs.is_valid(0.75));
        assert!(obs.is_valid(0.85));
        assert!(!obs.is_valid(0.90));
    }

    #[test]
    fn test_vision_analysis_result() {
        let obs1 = PieceObservation::new(
            "obs_1".to_string(),
            0,
            LegoColor::Red,
            BoundingBox::new(10, 10, 50, 50),
            0.90,
            Position2D::new(20.0, 20.0),
            PieceSize::Medium,
        );

        let obs2 = PieceObservation::new(
            "obs_2".to_string(),
            0,
            LegoColor::Blue,
            BoundingBox::new(100, 10, 50, 50),
            0.60, // Below typical threshold
            Position2D::new(60.0, 20.0),
            PieceSize::Medium,
        );

        let result = VisionAnalysisResult {
            observations: vec![obs1, obs2],
            warnings: vec![],
            lighting_status: LightingStatus::Adequate,
            timestamp: 0,
        };

        assert_eq!(result.piece_count(), 2);
        assert_eq!(result.valid_observations(0.75).len(), 1);
        assert!(!result.is_empty(0.75));

        // Next to pick should be obs1 (closer to origin)
        let pick_pos = Position2D::new(0.0, 0.0);
        let next = result.next_to_pick(&pick_pos, 0.75).unwrap();
        assert_eq!(next.observation_id, "obs_1");
    }

    #[test]
    fn test_tray_region() {
        let tray = TrayRegion::new(
            Position2D::new(10.0, 10.0),
            Position2D::new(200.0, 150.0),
            "white",
        );

        assert!((tray.width() - 190.0).abs() < 0.001);
        assert!((tray.height() - 140.0).abs() < 0.001);
        assert!(tray.contains(&Position2D::new(100.0, 100.0)));
        assert!(!tray.contains(&Position2D::new(5.0, 5.0)));
    }

    #[test]
    fn test_lighting_status_assessment() {
        let detector = VisionDetector::with_defaults();

        // Test dim lighting
        let dim_data: Vec<u8> = vec![30; 100]; // Very dark
        let status = detector.assess_lighting(&dim_data);
        assert_eq!(status, LightingStatus::TooDim);

        // Test bright lighting
        let bright_data: Vec<u8> = vec![250; 100];
        let status = detector.assess_lighting(&bright_data);
        assert_eq!(status, LightingStatus::TooBright);

        // Test adequate lighting
        let good_data: Vec<u8> = vec![128; 100];
        let status = detector.assess_lighting(&good_data);
        assert_eq!(status, LightingStatus::Adequate);
    }

    #[test]
    fn test_empty_frame_analysis() {
        let mut detector = VisionDetector::with_defaults();

        // Empty/black frame
        let empty_data: Vec<u8> = vec![0; 640 * 480 * 3];
        let result = detector.analyze_frame(&empty_data, 640, 480, 0);

        assert!(result.is_empty(0.5));
    }
}
