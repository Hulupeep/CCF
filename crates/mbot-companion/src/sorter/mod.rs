//! LEGOSorter module - Complete sorting system for LEGO pieces
//!
//! This module implements the LEGO sorting system for mBot2, including:
//! - Servo calibration (SORT-001)
//! - Carousel configuration (SORT-002, SORT-004)
//! - Color detection vision (SORT-002, SORT-003, SORT-005)

pub mod calibration;
pub mod carousel;
pub mod color_detection;
pub mod vision;

// Re-export main types for convenience
pub use calibration::{
    CalibrationProfile, CalibrationStep, ServoCalibration, ServoCalibrationWizard,
};
pub use carousel::{Bin, CarouselConfig, MarkerDetection, MarkerType};
pub use color_detection::{
    ColorDetectionResult, ColorLookupTable, DetectionStatistics, RareColor, RgbColorDetector,
    RgbReading, SurfaceCalibration,
};
pub use vision::{
    BoundingBox, ColorCalibration, HsvColor, HsvRange, LegoColor, LightingMode, LightingStatus,
    PieceObservation, PieceSize, Position2D, TrayRegion, VisionAnalysisResult, VisionConfig,
    VisionDetector, VisionWarning,
};
