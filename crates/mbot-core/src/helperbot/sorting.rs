//! Sorting Algorithm - STORY-HELP-002
//!
//! Implements systematic LEGO sorting with grid-based scanning and path planning
//! Contract: feature_helperbot.yml (I-HELP-010 through I-HELP-013)

#[cfg(feature = "no_std")]
use alloc::{string::{String, ToString}, vec, vec::Vec};
#[cfg(not(feature = "no_std"))]
use std::{string::{String, ToString}, vec, vec::Vec};

use serde::{Deserialize, Serialize};

/// Position on the sorting surface
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        #[cfg(feature = "no_std")]
        return libm::sqrtf(dx * dx + dy * dy);
        #[cfg(not(feature = "no_std"))]
        return (dx * dx + dy * dy).sqrt();
    }
}

/// Color zone for sorted pieces
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorZone {
    pub color: String,
    pub position: Position,
    pub count: u32,
}

impl ColorZone {
    pub fn new(color: String, position: Position) -> Self {
        Self {
            color,
            position,
            count: 0,
        }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }
}

/// Task status (I-HELP-012: must be interruptible)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Active,
    Paused,
    Complete,
    Timeout,
}

/// Grid cell state (I-HELP-013: systematic coverage)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GridCell {
    pub x: usize,
    pub y: usize,
    pub scanned: bool,
    pub has_piece: bool,
    pub piece_color: Option<String>,
}

impl GridCell {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            scanned: false,
            has_piece: false,
            piece_color: None,
        }
    }

    pub fn mark_scanned(&mut self, has_piece: bool, color: Option<String>) {
        self.scanned = true;
        self.has_piece = has_piece;
        self.piece_color = color;
    }
}

/// Scan pattern type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanOrder {
    Zigzag,
    Spiral,
    Linear,
}

/// Scan pattern state (I-HELP-013: systematic coverage)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanPattern {
    pub grid_width: usize,
    pub grid_height: usize,
    pub current_cell: Position,
    pub scan_order: ScanOrder,
    pub coverage_percent: f32,
    pub cells: Vec<GridCell>,
}

impl ScanPattern {
    pub fn new(width: usize, height: usize, scan_order: ScanOrder) -> Self {
        let mut cells = Vec::new();
        for y in 0..height {
            for x in 0..width {
                cells.push(GridCell::new(x, y));
            }
        }

        Self {
            grid_width: width,
            grid_height: height,
            current_cell: Position::new(0.0, 0.0),
            scan_order,
            coverage_percent: 0.0,
            cells,
        }
    }

    /// Get cell at grid position
    pub fn get_cell(&self, x: usize, y: usize) -> Option<&GridCell> {
        if x < self.grid_width && y < self.grid_height {
            self.cells.get(y * self.grid_width + x)
        } else {
            None
        }
    }

    /// Get mutable cell at grid position
    pub fn get_cell_mut(&mut self, x: usize, y: usize) -> Option<&mut GridCell> {
        if x < self.grid_width && y < self.grid_height {
            self.cells.get_mut(y * self.grid_width + x)
        } else {
            None
        }
    }

    /// Update coverage percentage
    pub fn update_coverage(&mut self) {
        let scanned = self.cells.iter().filter(|c| c.scanned).count();
        let total = self.cells.len();
        self.coverage_percent = if total > 0 {
            (scanned as f32 / total as f32) * 100.0
        } else {
            0.0
        };
    }

    /// Get next cell to scan based on pattern
    pub fn next_cell(&self) -> Option<(usize, usize)> {
        match self.scan_order {
            ScanOrder::Zigzag => self.next_zigzag(),
            ScanOrder::Spiral => self.next_spiral(),
            ScanOrder::Linear => self.next_linear(),
        }
    }

    fn next_zigzag(&self) -> Option<(usize, usize)> {
        // Find first unscanned cell in zigzag pattern
        for y in 0..self.grid_height {
            if y % 2 == 0 {
                // Left to right on even rows
                for x in 0..self.grid_width {
                    if let Some(cell) = self.get_cell(x, y) {
                        if !cell.scanned {
                            return Some((x, y));
                        }
                    }
                }
            } else {
                // Right to left on odd rows
                for x in (0..self.grid_width).rev() {
                    if let Some(cell) = self.get_cell(x, y) {
                        if !cell.scanned {
                            return Some((x, y));
                        }
                    }
                }
            }
        }
        None
    }

    fn next_spiral(&self) -> Option<(usize, usize)> {
        // Find first unscanned cell (simplified spiral)
        // TODO: Implement proper spiral pattern
        self.next_linear()
    }

    fn next_linear(&self) -> Option<(usize, usize)> {
        // Simple left-to-right, top-to-bottom
        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                if let Some(cell) = self.get_cell(x, y) {
                    if !cell.scanned {
                        return Some((x, y));
                    }
                }
            }
        }
        None
    }
}

/// Path plan to navigate to a zone (I-HELP-010: clear completion)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PathPlan {
    pub from: Position,
    pub to: Position,
    pub waypoints: Vec<Position>,
    pub estimated_time_ms: u64,
}

impl PathPlan {
    pub fn new(from: Position, to: Position) -> Self {
        // Simple direct path for now
        let distance = from.distance_to(&to);
        let estimated_time_ms = (distance * 100.0) as u64; // ~10 cm/s

        Self {
            from,
            to,
            waypoints: Vec::new(),
            estimated_time_ms,
        }
    }

    pub fn add_waypoint(&mut self, waypoint: Position) {
        self.waypoints.push(waypoint);
        // Recalculate time
        self.recalculate_time();
    }

    fn recalculate_time(&mut self) {
        let mut total_distance = self.from.distance_to(&self.to);
        let mut prev = self.from;
        for wp in &self.waypoints {
            total_distance += prev.distance_to(wp);
            prev = *wp;
        }
        if !self.waypoints.is_empty() {
            total_distance += self.waypoints.last().unwrap().distance_to(&self.to);
        }
        self.estimated_time_ms = (total_distance * 100.0) as u64;
    }
}

/// Sorting task state (I-HELP-012: interruptible)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SortingTask {
    pub id: String,
    pub task_type: String,
    pub zones: Vec<ColorZone>,
    pub items_sorted: u32,
    pub items_remaining: u32,
    pub special_finds: Vec<String>,
    pub status: TaskStatus,
    pub scan_pattern: ScanPattern,
    pub start_time_us: u64,
    pub max_duration_us: u64,
}

impl SortingTask {
    /// Create new sorting task (I-HELP-010: completion criteria)
    pub fn new(
        id: String,
        zones: Vec<ColorZone>,
        grid_width: usize,
        grid_height: usize,
        scan_order: ScanOrder,
        start_time_us: u64,
        max_duration_us: u64,
    ) -> Self {
        Self {
            id,
            task_type: "lego".to_string(),
            zones,
            items_sorted: 0,
            items_remaining: 0,
            special_finds: Vec::new(),
            status: TaskStatus::Active,
            scan_pattern: ScanPattern::new(grid_width, grid_height, scan_order),
            start_time_us,
            max_duration_us,
        }
    }

    /// Pause task (I-HELP-012: interruptible)
    pub fn pause(&mut self) {
        if self.status == TaskStatus::Active {
            self.status = TaskStatus::Paused;
        }
    }

    /// Resume task (I-HELP-012: interruptible)
    pub fn resume(&mut self) {
        if self.status == TaskStatus::Paused {
            self.status = TaskStatus::Active;
        }
    }

    /// Check if task should timeout (I-HELP-011: no infinite loops)
    pub fn check_timeout(&mut self, current_time_us: u64) -> bool {
        if self.status == TaskStatus::Active {
            let elapsed = current_time_us.saturating_sub(self.start_time_us);
            if elapsed >= self.max_duration_us {
                self.status = TaskStatus::Timeout;
                return true;
            }
        }
        false
    }

    /// Record piece sorted
    pub fn record_sorted(&mut self, color: &str) {
        self.items_sorted += 1;
        if self.items_remaining > 0 {
            self.items_remaining -= 1;
        }

        // Update zone count
        for zone in &mut self.zones {
            if zone.color == color {
                zone.increment();
                break;
            }
        }
    }

    /// Record special find (I-HELP-004: rare colors)
    pub fn record_special_find(&mut self, color: String) {
        if !self.special_finds.contains(&color) {
            self.special_finds.push(color);
        }
    }

    /// Check completion (I-HELP-010: clear completion criteria)
    pub fn check_completion(&mut self) -> bool {
        // Complete if no items remaining OR coverage >= 95%
        let coverage_complete = self.scan_pattern.coverage_percent >= 95.0;
        let items_complete = self.items_remaining == 0;

        if coverage_complete || items_complete {
            if self.status == TaskStatus::Active {
                self.status = TaskStatus::Complete;
            }
            return true;
        }
        false
    }

    /// Scan current cell
    pub fn scan_cell(&mut self, x: usize, y: usize, has_piece: bool, color: Option<String>) {
        if let Some(cell) = self.scan_pattern.get_cell_mut(x, y) {
            cell.mark_scanned(has_piece, color);
            if has_piece {
                self.items_remaining += 1;
            }
        }
        self.scan_pattern.update_coverage();
    }

    /// Get next scan position
    pub fn next_scan_position(&self) -> Option<(usize, usize)> {
        self.scan_pattern.next_cell()
    }
}

/// Sorting algorithm coordinator
pub struct SortingAlgorithm {
    active_task: Option<SortingTask>,
    iteration_count: u32,
    max_iterations: u32,
}

impl SortingAlgorithm {
    pub fn new(max_iterations: u32) -> Self {
        Self {
            active_task: None,
            iteration_count: 0,
            max_iterations,
        }
    }

    /// Start a sorting task
    pub fn start_task(&mut self, task: SortingTask) {
        self.active_task = Some(task);
        self.iteration_count = 0;
    }

    /// Process one iteration of sorting (I-HELP-011: no infinite loops)
    pub fn tick(&mut self, current_time_us: u64) -> Option<&SortingTask> {
        if let Some(task) = &mut self.active_task {
            // Check timeout
            task.check_timeout(current_time_us);

            // Check iteration limit (I-HELP-011)
            self.iteration_count += 1;
            if self.iteration_count >= self.max_iterations {
                task.status = TaskStatus::Timeout;
            }

            // Check completion
            task.check_completion();

            return Some(task);
        }
        None
    }

    /// Get current task
    pub fn get_task(&self) -> Option<&SortingTask> {
        self.active_task.as_ref()
    }

    /// Get mutable task
    pub fn get_task_mut(&mut self) -> Option<&mut SortingTask> {
        self.active_task.as_mut()
    }

    /// Clear completed task
    pub fn clear_task(&mut self) {
        self.active_task = None;
        self.iteration_count = 0;
    }

    /// Plan path to color zone
    pub fn plan_path(&self, from: Position, color: &str) -> Option<PathPlan> {
        if let Some(task) = &self.active_task {
            for zone in &task.zones {
                if zone.color == color {
                    return Some(PathPlan::new(from, zone.position));
                }
            }
        }
        None
    }
}

impl Default for SortingAlgorithm {
    fn default() -> Self {
        // 10 minutes at ~10Hz = 6000 iterations
        Self::new(6000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_scan_coverage() {
        let mut pattern = ScanPattern::new(3, 3, ScanOrder::Zigzag);
        assert_eq!(pattern.coverage_percent, 0.0);

        // Scan some cells
        pattern.get_cell_mut(0, 0).unwrap().scanned = true;
        pattern.get_cell_mut(1, 0).unwrap().scanned = true;
        pattern.update_coverage();

        assert!(pattern.coverage_percent > 0.0);
        assert!(pattern.coverage_percent < 100.0);
    }

    #[test]
    fn test_zigzag_pattern() {
        let pattern = ScanPattern::new(3, 2, ScanOrder::Zigzag);

        // First cell should be (0, 0)
        let next = pattern.next_cell();
        assert_eq!(next, Some((0, 0)));
    }

    #[test]
    fn test_task_pause_resume() {
        let zones = vec![ColorZone::new("red".to_string(), Position::new(0.0, 0.0))];
        let mut task = SortingTask::new(
            "task1".to_string(),
            zones,
            3,
            3,
            ScanOrder::Zigzag,
            0,
            600_000_000, // 10 minutes
        );

        assert_eq!(task.status, TaskStatus::Active);

        // I-HELP-012: Must be interruptible
        task.pause();
        assert_eq!(task.status, TaskStatus::Paused);

        task.resume();
        assert_eq!(task.status, TaskStatus::Active);
    }

    #[test]
    fn test_completion_criteria() {
        let zones = vec![ColorZone::new("red".to_string(), Position::new(0.0, 0.0))];
        let mut task = SortingTask::new(
            "task1".to_string(),
            zones,
            2,
            2,
            ScanOrder::Linear,
            0,
            600_000_000,
        );

        // I-HELP-010: Clear completion criteria
        task.items_remaining = 0;
        assert!(task.check_completion());
        assert_eq!(task.status, TaskStatus::Complete);
    }

    #[test]
    fn test_timeout_prevention() {
        let zones = vec![ColorZone::new("red".to_string(), Position::new(0.0, 0.0))];
        let mut task = SortingTask::new(
            "task1".to_string(),
            zones,
            2,
            2,
            ScanOrder::Linear,
            0,
            1000, // 1ms timeout
        );

        // I-HELP-011: No infinite loops
        task.check_timeout(2000); // 2ms elapsed
        assert_eq!(task.status, TaskStatus::Timeout);
    }

    #[test]
    fn test_iteration_limit() {
        let mut algo = SortingAlgorithm::new(10); // Very low limit for testing

        let zones = vec![ColorZone::new("red".to_string(), Position::new(0.0, 0.0))];
        let task = SortingTask::new(
            "task1".to_string(),
            zones,
            2,
            2,
            ScanOrder::Linear,
            0,
            600_000_000,
        );

        algo.start_task(task);

        // I-HELP-011: Iteration limit prevents infinite loops
        for _ in 0..15 {
            algo.tick(0);
        }

        let task = algo.get_task().unwrap();
        assert_eq!(task.status, TaskStatus::Timeout);
    }

    #[test]
    fn test_path_planning() {
        let from = Position::new(0.0, 0.0);
        let to = Position::new(10.0, 10.0);
        let path = PathPlan::new(from, to);

        assert!(path.estimated_time_ms > 0);
        assert_eq!(path.from, from);
        assert_eq!(path.to, to);
    }

    #[test]
    fn test_special_finds() {
        let zones = vec![ColorZone::new("red".to_string(), Position::new(0.0, 0.0))];
        let mut task = SortingTask::new(
            "task1".to_string(),
            zones,
            2,
            2,
            ScanOrder::Linear,
            0,
            600_000_000,
        );

        // Record special find
        task.record_special_find("gold".to_string());
        assert!(task.special_finds.contains(&"gold".to_string()));

        // Should not duplicate
        task.record_special_find("gold".to_string());
        assert_eq!(task.special_finds.len(), 1);
    }
}
