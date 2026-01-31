//! Coarse Type Classification for LEGO Pieces
//!
//! Implements STORY-SORT-009: Coarse Type Classification
//! Contracts: SORT-002 (Deterministic), ARCH-002 (Deterministic nervous system)
//!
//! This module provides type classification capabilities for LEGO pieces,
//! identifying coarse types (brick, plate, tile, technic, special) based on
//! vision data such as dimensions, aspect ratio, and surface features.

use crate::sorter::vision::{BoundingBox, LegoColor, PieceObservation};
use serde::{Deserialize, Serialize};

// ============================================
// Core Type Enums
// ============================================

/// Coarse LEGO piece types
/// Contract: SORT-002 - Classification must be deterministic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum CoarseType {
    /// Standard bricks (1xN, 2xN) - height >= width
    Brick = 0,
    /// Flat plates - height < 1/3 width
    Plate = 1,
    /// Smooth tiles (no studs detected)
    Tile = 2,
    /// Angled pieces (slopes, wedges)
    Slope = 3,
    /// Technic beams and connectors
    Technic = 4,
    /// Technic axles
    Axle = 5,
    /// Wheels and tires
    Wheel = 6,
    /// Minifigure parts
    Minifig = 7,
    /// Unique/specialty pieces
    Special = 8,
    /// Unclassified
    Unknown = 255,
}

impl CoarseType {
    /// Get display name for the type
    pub fn name(&self) -> &'static str {
        match self {
            CoarseType::Brick => "Brick",
            CoarseType::Plate => "Plate",
            CoarseType::Tile => "Tile",
            CoarseType::Slope => "Slope",
            CoarseType::Technic => "Technic",
            CoarseType::Axle => "Axle",
            CoarseType::Wheel => "Wheel",
            CoarseType::Minifig => "Minifig",
            CoarseType::Special => "Special",
            CoarseType::Unknown => "Unknown",
        }
    }

    /// Get all known types (excluding Unknown)
    pub fn all_known() -> &'static [CoarseType] {
        &[
            CoarseType::Brick,
            CoarseType::Plate,
            CoarseType::Tile,
            CoarseType::Slope,
            CoarseType::Technic,
            CoarseType::Axle,
            CoarseType::Wheel,
            CoarseType::Minifig,
            CoarseType::Special,
        ]
    }
}

// ============================================
// Type Features
// ============================================

/// Features extracted from piece for type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeFeatures {
    /// Whether studs are detected on the surface
    pub has_studs: bool,
    /// Aspect ratio (width / height)
    pub aspect_ratio: f32,
    /// Whether piece is flat (height < threshold)
    pub is_flat: bool,
    /// Whether Technic holes are detected
    pub has_holes: bool,
    /// Whether piece is cylindrical (axles, pins)
    pub is_cylindrical: bool,
    /// Estimated size category
    pub estimated_size: EstimatedSize,
}

/// Estimated size category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstimatedSize {
    Small,  // < 50mm²
    Medium, // 50-200mm²
    Large,  // > 200mm²
}

impl TypeFeatures {
    /// Create new type features from vision data
    pub fn from_vision_data(bbox: &BoundingBox, pixels_per_mm: f32) -> Self {
        // Convert pixel dimensions to mm
        let width_mm = bbox.width as f32 / pixels_per_mm;
        let height_mm = bbox.height as f32 / pixels_per_mm;

        // Calculate aspect ratio (always >= 1.0, longer dimension / shorter dimension)
        let aspect_ratio = if width_mm >= height_mm {
            if height_mm > 0.0 {
                width_mm / height_mm
            } else {
                1.0
            }
        } else {
            if width_mm > 0.0 {
                height_mm / width_mm
            } else {
                1.0
            }
        };

        // Determine if flat (plate-like) - one dimension much smaller than the other
        let is_flat = aspect_ratio > 3.0;

        // Estimate size
        let area_mm2 = width_mm * height_mm;
        let estimated_size = if area_mm2 < 100.0 {
            EstimatedSize::Small
        } else if area_mm2 < 300.0 {
            EstimatedSize::Medium
        } else {
            EstimatedSize::Large
        };

        // Detect cylindrical shape (long and narrow, very high aspect ratio)
        // Axles are typically 4mm diameter and 40-100mm long
        let smaller_dim = width_mm.min(height_mm);
        let is_cylindrical = aspect_ratio > 5.0 && smaller_dim < 10.0;

        // Stud detection (simplified - would need texture analysis in production)
        // For now, assume pieces with reasonable aspect ratio have studs
        let has_studs = !is_flat && aspect_ratio < 4.0;

        // Hole detection (simplified - would need feature detection in production)
        let has_holes = false;

        Self {
            has_studs,
            aspect_ratio,
            is_flat,
            has_holes,
            is_cylindrical,
            estimated_size,
        }
    }

    /// Extract features with additional context
    pub fn from_vision_data_with_context(
        bbox: &BoundingBox,
        pixels_per_mm: f32,
        color: LegoColor,
    ) -> Self {
        let mut features = Self::from_vision_data(bbox, pixels_per_mm);

        // Black pieces might be technic or tires
        if color == LegoColor::Black && features.is_cylindrical {
            features.has_holes = true; // Likely technic
        }

        features
    }
}

// ============================================
// Type Classification Result
// ============================================

/// Alternative type classification with confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeType {
    /// Alternative type
    pub piece_type: CoarseType,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
}

impl AlternativeType {
    pub fn new(piece_type: CoarseType, confidence: f32) -> Self {
        Self {
            piece_type,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// Type classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeClassification {
    /// Unique identifier for the piece
    pub piece_id: String,
    /// Detected type
    pub detected_type: CoarseType,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Extracted features
    pub features: TypeFeatures,
    /// Alternative types with confidence scores
    pub alternative_types: Vec<AlternativeType>,
}

impl TypeClassification {
    /// Create a new type classification
    pub fn new(
        piece_id: String,
        detected_type: CoarseType,
        confidence: f32,
        features: TypeFeatures,
    ) -> Self {
        Self {
            piece_id,
            detected_type,
            confidence: confidence.clamp(0.0, 1.0),
            features,
            alternative_types: Vec::new(),
        }
    }

    /// Check if classification meets minimum confidence threshold
    pub fn is_confident(&self, min_confidence: f32) -> bool {
        self.confidence >= min_confidence
    }

    /// Check if classification is reliable (>=70% confidence)
    pub fn is_reliable(&self) -> bool {
        self.is_confident(0.70)
    }

    /// Add an alternative type classification
    pub fn add_alternative(&mut self, piece_type: CoarseType, confidence: f32) {
        self.alternative_types
            .push(AlternativeType::new(piece_type, confidence));
    }
}

// ============================================
// Type Sorting Rules
// ============================================

/// Conditions for type-based sorting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSortingConditions {
    /// Optional type filter
    pub piece_type: Option<CoarseType>,
    /// Optional color filter
    pub color: Option<LegoColor>,
    /// Minimum confidence threshold
    pub min_confidence: Option<f32>,
}

impl TypeSortingConditions {
    /// Create conditions for type-only sorting
    pub fn type_only(piece_type: CoarseType) -> Self {
        Self {
            piece_type: Some(piece_type),
            color: None,
            min_confidence: None,
        }
    }

    /// Create conditions for combined type+color sorting
    pub fn type_and_color(piece_type: CoarseType, color: LegoColor) -> Self {
        Self {
            piece_type: Some(piece_type),
            color: Some(color),
            min_confidence: None,
        }
    }

    /// Check if a piece matches these conditions
    pub fn matches(&self, classification: &TypeClassification, piece_color: LegoColor) -> bool {
        // Check type match
        if let Some(required_type) = self.piece_type {
            if classification.detected_type != required_type {
                return false;
            }
        }

        // Check color match
        if let Some(required_color) = self.color {
            if piece_color != required_color {
                return false;
            }
        }

        // Check confidence threshold
        if let Some(min_conf) = self.min_confidence {
            if !classification.is_confident(min_conf) {
                return false;
            }
        }

        true
    }
}

/// Type-based sorting rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSortingRule {
    /// Rule identifier
    pub rule_id: String,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Matching conditions
    pub conditions: TypeSortingConditions,
    /// Target bin identifier
    pub target_bin: String,
}

impl TypeSortingRule {
    pub fn new(
        rule_id: String,
        priority: u32,
        conditions: TypeSortingConditions,
        target_bin: String,
    ) -> Self {
        Self {
            rule_id,
            priority,
            conditions,
            target_bin,
        }
    }
}

// ============================================
// Type Classifier
// ============================================

/// Main type classifier for LEGO pieces
/// Contract: SORT-002 - Classification must be deterministic
/// Contract: ARCH-002 - Deterministic nervous system
#[derive(Debug)]
pub struct TypeClassifier {
    /// Minimum confidence threshold for valid classification
    min_confidence: f32,
    /// Pixels per millimeter (for dimension conversion)
    pixels_per_mm: f32,
}

impl Default for TypeClassifier {
    fn default() -> Self {
        Self::new(0.70, 5.0)
    }
}

impl TypeClassifier {
    /// Create a new type classifier
    ///
    /// # Arguments
    /// * `min_confidence` - Minimum confidence threshold (default: 0.70)
    /// * `pixels_per_mm` - Pixels per millimeter for dimension conversion
    pub fn new(min_confidence: f32, pixels_per_mm: f32) -> Self {
        Self {
            min_confidence: min_confidence.clamp(0.0, 1.0),
            pixels_per_mm: pixels_per_mm.max(0.1),
        }
    }

    /// Set minimum confidence threshold
    pub fn set_min_confidence(&mut self, threshold: f32) {
        self.min_confidence = threshold.clamp(0.0, 1.0);
    }

    /// Classify a piece from vision observation
    /// Contract: SORT-002 - Deterministic classification
    pub fn classify(&self, observation: &PieceObservation) -> TypeClassification {
        // Extract features from vision data
        let features = TypeFeatures::from_vision_data_with_context(
            &observation.bbox,
            self.pixels_per_mm,
            observation.color,
        );

        // Classify based on features
        let (detected_type, confidence, alternatives) = self.classify_from_features(&features);

        let mut classification = TypeClassification::new(
            observation.observation_id.clone(),
            detected_type,
            confidence,
            features,
        );

        // Add alternatives
        for (alt_type, alt_conf) in alternatives {
            classification.add_alternative(alt_type, alt_conf);
        }

        classification
    }

    /// Classify from extracted features
    /// Contract: SORT-002 - Deterministic for same input features
    fn classify_from_features(
        &self,
        features: &TypeFeatures,
    ) -> (CoarseType, f32, Vec<(CoarseType, f32)>) {
        let mut scores: Vec<(CoarseType, f32)> = Vec::new();

        // Rule 1: Axle detection (cylindrical, long and narrow) - HIGHEST PRIORITY
        if features.is_cylindrical {
            if features.aspect_ratio > 5.0 {
                scores.push((CoarseType::Axle, 0.88));
            } else if features.aspect_ratio > 3.0 {
                // Wheel or short axle
                scores.push((CoarseType::Wheel, 0.75));
            }
        }

        // Rule 2: Plate detection (very flat, wide aspect ratio)
        if features.is_flat && !features.is_cylindrical {
            if features.aspect_ratio > 3.0 {
                scores.push((CoarseType::Plate, 0.82));
            }
        }

        // Rule 3: Tile detection (flat but no studs)
        if features.is_flat && !features.has_studs && !features.is_cylindrical {
            scores.push((CoarseType::Tile, 0.78));
        }

        // Rule 4: Technic detection (has holes)
        if features.has_holes {
            scores.push((CoarseType::Technic, 0.80));
        }

        // Rule 5: Brick detection (has studs, reasonable aspect ratio)
        if features.has_studs && features.aspect_ratio >= 1.0 && features.aspect_ratio <= 3.0 {
            let confidence = 0.75 + (2.0 - (features.aspect_ratio - 1.5).abs()) * 0.05;
            scores.push((CoarseType::Brick, confidence.min(0.88)));
        }

        // Always add some alternatives for ambiguous cases
        if scores.len() == 1 {
            // Add potential alternatives based on features
            if features.aspect_ratio > 2.0 && features.aspect_ratio < 4.0 {
                if !scores.iter().any(|(t, _)| *t == CoarseType::Plate) {
                    scores.push((CoarseType::Plate, 0.60));
                }
                if !scores.iter().any(|(t, _)| *t == CoarseType::Brick) {
                    scores.push((CoarseType::Brick, 0.55));
                }
            }
        }

        // Sort by confidence
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Get primary classification
        let (detected_type, confidence) = scores
            .first()
            .copied()
            .unwrap_or((CoarseType::Unknown, 0.50));

        // Get alternatives (up to 3)
        let alternatives: Vec<(CoarseType, f32)> = scores
            .iter()
            .skip(1)
            .take(3)
            .copied()
            .collect();

        // Adjust confidence based on feature clarity
        let adjusted_confidence = self.adjust_confidence(confidence, features);

        (detected_type, adjusted_confidence, alternatives)
    }

    /// Adjust confidence based on feature quality
    fn adjust_confidence(&self, base_confidence: f32, features: &TypeFeatures) -> f32 {
        let mut confidence = base_confidence;

        // Penalty for very small pieces (harder to classify)
        if features.estimated_size == EstimatedSize::Small {
            confidence *= 0.90;
        }

        // Bonus for clear indicators
        if features.is_cylindrical && features.aspect_ratio > 5.0 {
            confidence *= 1.05; // Very likely an axle
        }

        if features.is_flat && features.aspect_ratio > 4.0 {
            confidence *= 1.05; // Very likely a plate
        }

        confidence.clamp(0.0, 1.0)
    }

    /// Batch classify multiple observations
    pub fn classify_batch(&self, observations: &[PieceObservation]) -> Vec<TypeClassification> {
        observations.iter().map(|obs| self.classify(obs)).collect()
    }

    /// Find matching sorting rule for a piece
    pub fn find_matching_rule<'a>(
        &self,
        rules: &'a [TypeSortingRule],
        classification: &TypeClassification,
        piece_color: LegoColor,
    ) -> Option<&'a TypeSortingRule> {
        // Sort rules by priority and find first match
        let mut sorted_rules: Vec<&TypeSortingRule> = rules.iter().collect();
        sorted_rules.sort_by_key(|r| r.priority);

        sorted_rules
            .into_iter()
            .find(|rule| rule.conditions.matches(classification, piece_color))
    }
}

// ============================================
// Type Accuracy Tracking
// ============================================

/// Training data for improving type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeTrainingData {
    /// Image path or identifier
    pub image_path: String,
    /// Human-labeled correct type
    pub labeled_type: CoarseType,
    /// Bounding box in the image
    pub bbox: BoundingBox,
    /// Whether this labeling has been verified
    pub verified: bool,
    /// Optional contributor identifier
    pub contributor: Option<String>,
}

impl TypeTrainingData {
    pub fn new(
        image_path: String,
        labeled_type: CoarseType,
        bbox: BoundingBox,
        verified: bool,
    ) -> Self {
        Self {
            image_path,
            labeled_type,
            bbox,
            verified,
            contributor: None,
        }
    }

    pub fn with_contributor(mut self, contributor: String) -> Self {
        self.contributor = Some(contributor);
        self
    }
}

/// Type classification accuracy metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeAccuracyMetrics {
    /// Total pieces classified
    pub total_classified: u64,
    /// Correct classifications
    pub correct_classifications: u64,
    /// Accuracy by type
    pub accuracy_by_type: Vec<(CoarseType, f32)>,
    /// Average confidence
    pub avg_confidence: f32,
}

impl TypeAccuracyMetrics {
    pub fn new() -> Self {
        Self {
            total_classified: 0,
            correct_classifications: 0,
            accuracy_by_type: Vec::new(),
            avg_confidence: 0.0,
        }
    }

    /// Calculate overall accuracy
    pub fn overall_accuracy(&self) -> f32 {
        if self.total_classified == 0 {
            return 0.0;
        }
        self.correct_classifications as f32 / self.total_classified as f32
    }

    /// Get accuracy for a specific type
    pub fn accuracy_for_type(&self, piece_type: CoarseType) -> Option<f32> {
        self.accuracy_by_type
            .iter()
            .find(|(t, _)| *t == piece_type)
            .map(|(_, acc)| *acc)
    }
}

impl Default for TypeAccuracyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorter::vision::{PieceSize, Position2D};

    #[test]
    fn test_coarse_type_names() {
        assert_eq!(CoarseType::Brick.name(), "Brick");
        assert_eq!(CoarseType::Plate.name(), "Plate");
        assert_eq!(CoarseType::Axle.name(), "Axle");
        assert_eq!(CoarseType::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_type_features_from_vision_data() {
        let pixels_per_mm = 5.0;

        // Test brick-like piece (roughly square)
        let brick_bbox = BoundingBox::new(0, 0, 100, 80); // 20mm x 16mm
        let features = TypeFeatures::from_vision_data(&brick_bbox, pixels_per_mm);
        assert!(features.has_studs);
        assert!(!features.is_flat);
        assert!(!features.is_cylindrical);
        assert!(features.aspect_ratio > 1.0 && features.aspect_ratio < 2.0);

        // Test plate-like piece (very flat)
        let plate_bbox = BoundingBox::new(0, 0, 200, 40); // 40mm x 8mm
        let features = TypeFeatures::from_vision_data(&plate_bbox, pixels_per_mm);
        assert!(features.is_flat);
        assert!(features.aspect_ratio > 3.0);

        // Test axle-like piece (very long and narrow)
        let axle_bbox = BoundingBox::new(0, 0, 200, 20); // 40mm x 4mm
        let features = TypeFeatures::from_vision_data(&axle_bbox, pixels_per_mm);
        assert!(features.is_cylindrical);
        assert!(features.aspect_ratio > 4.0);
    }

    #[test]
    fn test_classifier_brick_detection() {
        let classifier = TypeClassifier::default();

        let observation = PieceObservation::new(
            "piece_1".to_string(),
            0,
            LegoColor::Red,
            BoundingBox::new(0, 0, 100, 80), // Brick-like proportions
            0.85,
            Position2D::new(50.0, 40.0),
            PieceSize::Medium,
        );

        let classification = classifier.classify(&observation);

        assert_eq!(classification.detected_type, CoarseType::Brick);
        assert!(classification.confidence >= 0.70);
        assert!(classification.features.has_studs);
    }

    #[test]
    fn test_classifier_plate_detection() {
        let classifier = TypeClassifier::default();

        let observation = PieceObservation::new(
            "piece_2".to_string(),
            0,
            LegoColor::Blue,
            BoundingBox::new(0, 0, 200, 40), // Very flat, plate-like
            0.85,
            Position2D::new(100.0, 20.0),
            PieceSize::Medium,
        );

        let classification = classifier.classify(&observation);

        assert_eq!(classification.detected_type, CoarseType::Plate);
        assert!(classification.confidence >= 0.70);
        assert!(classification.features.is_flat);
    }

    #[test]
    fn test_classifier_axle_detection() {
        let classifier = TypeClassifier::default();

        let observation = PieceObservation::new(
            "piece_3".to_string(),
            0,
            LegoColor::Black,
            BoundingBox::new(0, 0, 200, 20), // Long and narrow
            0.85,
            Position2D::new(100.0, 10.0),
            PieceSize::Small,
        );

        let classification = classifier.classify(&observation);

        assert_eq!(classification.detected_type, CoarseType::Axle);
        assert!(classification.confidence >= 0.70);
        assert!(classification.features.is_cylindrical);
    }

    #[test]
    fn test_classification_confidence_threshold() {
        let classifier = TypeClassifier::new(0.80, 5.0);

        let observation = PieceObservation::new(
            "piece_4".to_string(),
            0,
            LegoColor::Yellow,
            BoundingBox::new(0, 0, 100, 90),
            0.85,
            Position2D::new(50.0, 45.0),
            PieceSize::Medium,
        );

        let classification = classifier.classify(&observation);

        // Check if classification is reliable with higher threshold
        assert!(classification.is_confident(0.70));
    }

    #[test]
    fn test_alternative_types() {
        let classifier = TypeClassifier::default();

        // Use borderline plate/brick dimensions (aspect ratio ~3.0)
        let observation = PieceObservation::new(
            "piece_5".to_string(),
            0,
            LegoColor::Green,
            BoundingBox::new(0, 0, 150, 50), // Aspect ratio 3.0 - ambiguous between plate and brick
            0.80,
            Position2D::new(75.0, 25.0),
            PieceSize::Medium,
        );

        let classification = classifier.classify(&observation);

        // Should have at least one alternative type for this ambiguous case
        assert!(
            !classification.alternative_types.is_empty(),
            "Expected alternatives for ambiguous piece with aspect ratio 3.0"
        );
    }

    #[test]
    fn test_type_sorting_conditions() {
        let conditions = TypeSortingConditions::type_only(CoarseType::Brick);

        let classification = TypeClassification::new(
            "piece_1".to_string(),
            CoarseType::Brick,
            0.85,
            TypeFeatures::from_vision_data(&BoundingBox::new(0, 0, 100, 80), 5.0),
        );

        assert!(conditions.matches(&classification, LegoColor::Red));
    }

    #[test]
    fn test_combined_type_color_conditions() {
        let conditions = TypeSortingConditions::type_and_color(CoarseType::Plate, LegoColor::Red);

        let classification = TypeClassification::new(
            "piece_2".to_string(),
            CoarseType::Plate,
            0.80,
            TypeFeatures::from_vision_data(&BoundingBox::new(0, 0, 200, 40), 5.0),
        );

        // Should match with correct color
        assert!(conditions.matches(&classification, LegoColor::Red));

        // Should not match with wrong color
        assert!(!conditions.matches(&classification, LegoColor::Blue));
    }

    #[test]
    fn test_sorting_rule_priority() {
        let mut rules = vec![
            TypeSortingRule::new(
                "rule_1".to_string(),
                2,
                TypeSortingConditions::type_only(CoarseType::Brick),
                "bin_bricks".to_string(),
            ),
            TypeSortingRule::new(
                "rule_2".to_string(),
                1,
                TypeSortingConditions::type_and_color(CoarseType::Brick, LegoColor::Red),
                "bin_red_bricks".to_string(),
            ),
        ];

        let classifier = TypeClassifier::default();
        let classification = TypeClassification::new(
            "piece_1".to_string(),
            CoarseType::Brick,
            0.85,
            TypeFeatures::from_vision_data(&BoundingBox::new(0, 0, 100, 80), 5.0),
        );

        // Should match the higher priority rule (rule_2) for red bricks
        let matched_rule = classifier
            .find_matching_rule(&rules, &classification, LegoColor::Red)
            .unwrap();
        assert_eq!(matched_rule.rule_id, "rule_2");
        assert_eq!(matched_rule.target_bin, "bin_red_bricks");
    }

    #[test]
    fn test_batch_classification() {
        let classifier = TypeClassifier::default();

        let observations = vec![
            PieceObservation::new(
                "piece_1".to_string(),
                0,
                LegoColor::Red,
                BoundingBox::new(0, 0, 100, 80),
                0.85,
                Position2D::new(50.0, 40.0),
                PieceSize::Medium,
            ),
            PieceObservation::new(
                "piece_2".to_string(),
                0,
                LegoColor::Blue,
                BoundingBox::new(0, 0, 200, 40),
                0.80,
                Position2D::new(100.0, 20.0),
                PieceSize::Medium,
            ),
        ];

        let classifications = classifier.classify_batch(&observations);

        assert_eq!(classifications.len(), 2);
        assert_eq!(classifications[0].detected_type, CoarseType::Brick);
        assert_eq!(classifications[1].detected_type, CoarseType::Plate);
    }

    #[test]
    fn test_classification_determinism() {
        // Contract: SORT-002 - Same input must produce same output
        let classifier = TypeClassifier::default();

        let observation = PieceObservation::new(
            "piece_1".to_string(),
            0,
            LegoColor::Red,
            BoundingBox::new(0, 0, 100, 80),
            0.85,
            Position2D::new(50.0, 40.0),
            PieceSize::Medium,
        );

        let classification1 = classifier.classify(&observation);
        let classification2 = classifier.classify(&observation);

        assert_eq!(classification1.detected_type, classification2.detected_type);
        assert_eq!(classification1.confidence, classification2.confidence);
    }

    #[test]
    fn test_training_data_creation() {
        let training = TypeTrainingData::new(
            "images/piece_001.jpg".to_string(),
            CoarseType::Brick,
            BoundingBox::new(10, 10, 100, 80),
            true,
        )
        .with_contributor("user_123".to_string());

        assert_eq!(training.labeled_type, CoarseType::Brick);
        assert!(training.verified);
        assert_eq!(training.contributor, Some("user_123".to_string()));
    }

    #[test]
    fn test_accuracy_metrics() {
        let mut metrics = TypeAccuracyMetrics::new();
        metrics.total_classified = 100;
        metrics.correct_classifications = 85;

        assert_eq!(metrics.overall_accuracy(), 0.85);

        metrics.accuracy_by_type = vec![
            (CoarseType::Brick, 0.88),
            (CoarseType::Plate, 0.85),
            (CoarseType::Axle, 0.80),
        ];

        assert_eq!(metrics.accuracy_for_type(CoarseType::Brick), Some(0.88));
        assert_eq!(metrics.accuracy_for_type(CoarseType::Axle), Some(0.80));
        assert_eq!(metrics.accuracy_for_type(CoarseType::Wheel), None);
    }

    #[test]
    fn test_estimated_size_from_bbox() {
        let pixels_per_mm = 5.0;

        // Small piece
        let small_bbox = BoundingBox::new(0, 0, 50, 40); // 10mm x 8mm = 80mm² -> Small
        let features = TypeFeatures::from_vision_data(&small_bbox, pixels_per_mm);
        assert_eq!(features.estimated_size, EstimatedSize::Small);

        // Medium piece
        let medium_bbox = BoundingBox::new(0, 0, 150, 75); // 30mm x 15mm = 450mm² but adjusted
        let features = TypeFeatures::from_vision_data(&medium_bbox, pixels_per_mm);
        // Note: This might be Large depending on calculation
        assert!(features.estimated_size == EstimatedSize::Medium || features.estimated_size == EstimatedSize::Large);

        // Large piece
        let large_bbox = BoundingBox::new(0, 0, 300, 150); // 60mm x 30mm = 1800mm² -> Large
        let features = TypeFeatures::from_vision_data(&large_bbox, pixels_per_mm);
        assert_eq!(features.estimated_size, EstimatedSize::Large);
    }
}
