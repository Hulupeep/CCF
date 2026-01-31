//! LEGOSorter module - Complete sorting system for LEGO pieces
//!
//! This module implements the LEGO sorting system for mBot2, including:
//! - Servo calibration (SORT-001)
//! - Carousel configuration (SORT-002, SORT-004)
//! - Color detection vision (SORT-002, SORT-003, SORT-005)
//! - Type classification (SORT-009)
//! - Sorting algorithm (STORY-HELP-002)
//! - Verification & error handling (STORY-SORT-005)
//! - NFC storage integration (STORY-SORT-007)

pub mod calibration;
pub mod carousel;
pub mod color_detection;
pub mod error_handling;
pub mod nfc_storage;
pub mod sorting_algorithm;
pub mod sorting_loop;
pub mod type_classifier;
pub mod verification;
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
pub use error_handling::{
    Action, ErrorEvent, ErrorHandler, ErrorStatistics, Operation as ErrorOperation,
    RecoveryAction, Resolution,
};
pub use sorting_algorithm::{
    ColorZone, GridCell, PathPlan, ScanGrid, ScanPattern, SortingAlgorithm, SortingTask,
    TaskStatus,
};
pub use sorting_loop::{
    DropOperation, GripForce, LoopMetrics, LoopState, PickOperation, SortingLoop, SortingStep,
};
pub use type_classifier::{
    AlternativeType, CoarseType, EstimatedSize, TypeAccuracyMetrics, TypeClassification,
    TypeClassifier, TypeFeatures, TypeSortingConditions, TypeSortingRule, TypeTrainingData,
};
pub use verification::{
    FailureReason, JamDetection, Operation as VerificationOperation, RetryPolicy,
    VerificationResult, VerificationSystem,
};
pub use nfc_storage::{
    BoxInventory, BoxLocator, ColorCount as NfcColorCount, NFCAction, NFCReadEvent, NfcStorageSystem,
    NfcTagData, NfcTagId, RetrievalRequest, RetrievalResult, SmartBox, TypeCount,
};
pub use vision::{
    BoundingBox, ColorCalibration, HsvColor, HsvRange, LegoColor, LightingMode, LightingStatus,
    PieceObservation, PieceSize, Position2D, TrayRegion, VisionAnalysisResult, VisionConfig,
    VisionDetector, VisionWarning,
};
