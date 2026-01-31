//! HelperBot - LEGO sorting and utility features
//!
//! This module implements the HelperBot feature set including:
//! - Color detection (STORY-HELP-001)
//! - Sorting algorithms (STORY-HELP-002)
//! - Task management and completion tracking
//! - Inventory tracking (STORY-SORT-006)

pub mod color_detection;
pub mod sorting;
pub mod inventory;

pub use color_detection::{ColorDetection, ColorReading, CalibrationResult, ColorEntry};
pub use sorting::{
    SortingTask, ColorZone, GridCell, ScanPattern, PathPlan, SortingAlgorithm,
    TaskStatus, Position,
};
pub use inventory::{
    InventoryTracker, InventoryCount, InventorySummary, InventorySnapshot,
    InventoryEvent, InventoryEventType, InventoryEventSource, BinConfig,
    SearchResult, ColorCount,
};
