//! Sorting Algorithm for LEGO Piece Organization
//!
//! Implements STORY-HELP-002: Sorting Algorithm
//! Contracts: I-HELP-010, I-HELP-011, I-HELP-012, I-HELP-013
//!
//! This module provides systematic sorting capabilities with:
//! - Grid-based scanning patterns
//! - Piece detection and prioritization
//! - Path planning to color zones
//! - Completion detection
//! - Missed piece handling

use crate::sorter::carousel::{Bin, CarouselConfig};
use crate::sorter::vision::{LegoColor, Position2D};
use std::collections::HashMap;

// ============================================
// Core Data Structures
// ============================================

/// Grid cell in the scanning pattern
/// Contract: I-HELP-013 - Systematic coverage required
#[derive(Debug, Clone, Copy)]
pub struct GridCell {
    /// X coordinate in grid
    pub x: usize,
    /// Y coordinate in grid
    pub y: usize,
    /// Whether this cell has been scanned
    pub scanned: bool,
    /// Whether a piece is present
    pub has_piece: bool,
    /// Color of the piece (if detected)
    pub piece_color: Option<LegoColor>,
    /// Timestamp of scan (microseconds)
    pub scan_timestamp: u64,
}

impl GridCell {
    /// Create a new unscanned grid cell
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            scanned: false,
            has_piece: false,
            piece_color: None,
            scan_timestamp: 0,
        }
    }

    /// Mark cell as scanned with optional piece detection
    pub fn mark_scanned(
        &mut self,
        has_piece: bool,
        piece_color: Option<LegoColor>,
        timestamp: u64,
    ) {
        self.scanned = true;
        self.has_piece = has_piece;
        self.piece_color = piece_color;
        self.scan_timestamp = timestamp;
    }

    /// Reset cell to unscanned state
    pub fn reset(&mut self) {
        self.scanned = false;
        self.has_piece = false;
        self.piece_color = None;
        self.scan_timestamp = 0;
    }
}

/// Scanning pattern type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanPattern {
    /// Zigzag pattern (efficient for rectangular areas)
    Zigzag,
    /// Spiral pattern (efficient for centered areas)
    Spiral,
    /// Linear row-by-row (simple but less efficient)
    Linear,
}

/// Grid-based scanning pattern
/// Contract: I-HELP-013 - Systematic coverage ensures no area skipped
#[derive(Debug, Clone)]
pub struct ScanGrid {
    /// Grid cells
    grid: Vec<Vec<GridCell>>,
    /// Grid dimensions
    width: usize,
    height: usize,
    /// Current cell position
    current_cell: (usize, usize),
    /// Scan pattern type
    pattern: ScanPattern,
    /// Coverage percentage (0.0-100.0)
    coverage_percent: f32,
}

impl ScanGrid {
    /// Create a new scan grid
    pub fn new(width: usize, height: usize, pattern: ScanPattern) -> Self {
        let mut grid = Vec::with_capacity(height);
        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                row.push(GridCell::new(x, y));
            }
            grid.push(row);
        }

        Self {
            grid,
            width,
            height,
            current_cell: (0, 0),
            pattern,
            coverage_percent: 0.0,
        }
    }

    /// Get cell at position
    pub fn get_cell(&self, x: usize, y: usize) -> Option<&GridCell> {
        self.grid.get(y).and_then(|row| row.get(x))
    }

    /// Get mutable cell at position
    pub fn get_cell_mut(&mut self, x: usize, y: usize) -> Option<&mut GridCell> {
        self.grid.get_mut(y).and_then(|row| row.get_mut(x))
    }

    /// Get current cell position
    pub fn current_position(&self) -> (usize, usize) {
        self.current_cell
    }

    /// Get coverage percentage
    pub fn coverage(&self) -> f32 {
        self.coverage_percent
    }

    /// Calculate coverage percentage
    pub fn update_coverage(&mut self) {
        let total_cells = self.width * self.height;
        let scanned_cells = self
            .grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.scanned)
            .count();

        self.coverage_percent = (scanned_cells as f32 / total_cells as f32) * 100.0;
    }

    /// Get next cell to scan based on pattern
    /// Contract: I-HELP-013 - Systematic pattern
    pub fn next_cell(&mut self) -> Option<(usize, usize)> {
        match self.pattern {
            ScanPattern::Zigzag => self.next_zigzag(),
            ScanPattern::Spiral => self.next_spiral(),
            ScanPattern::Linear => self.next_linear(),
        }
    }

    /// Zigzag pattern: left-to-right on even rows, right-to-left on odd rows
    fn next_zigzag(&mut self) -> Option<(usize, usize)> {
        let (mut x, mut y) = self.current_cell;

        // Even row: left to right
        if y % 2 == 0 {
            if x + 1 < self.width {
                x += 1;
            } else if y + 1 < self.height {
                y += 1;
            } else {
                return None; // Completed
            }
        }
        // Odd row: right to left
        else {
            if x > 0 {
                x -= 1;
            } else if y + 1 < self.height {
                y += 1;
            } else {
                return None; // Completed
            }
        }

        self.current_cell = (x, y);
        Some((x, y))
    }

    /// Linear pattern: simple row-by-row left-to-right
    fn next_linear(&mut self) -> Option<(usize, usize)> {
        let (mut x, mut y) = self.current_cell;

        if x + 1 < self.width {
            x += 1;
        } else if y + 1 < self.height {
            x = 0;
            y += 1;
        } else {
            return None; // Completed
        }

        self.current_cell = (x, y);
        Some((x, y))
    }

    /// Spiral pattern: spiral inward from edges
    fn next_spiral(&mut self) -> Option<(usize, usize)> {
        // Simple spiral implementation
        // For now, fall back to zigzag for complexity
        self.next_zigzag()
    }

    /// Reset grid to unscanned state
    pub fn reset(&mut self) {
        for row in &mut self.grid {
            for cell in row {
                cell.reset();
            }
        }
        self.current_cell = (0, 0);
        self.coverage_percent = 0.0;
    }

    /// Get all cells with detected pieces
    pub fn pieces(&self) -> Vec<(usize, usize, LegoColor)> {
        let mut pieces = Vec::new();
        for row in &self.grid {
            for cell in row {
                if cell.has_piece {
                    if let Some(color) = cell.piece_color {
                        pieces.push((cell.x, cell.y, color));
                    }
                }
            }
        }
        pieces
    }
}

/// Color zone mapping (bin position for each color)
#[derive(Debug, Clone)]
pub struct ColorZone {
    /// Color this zone handles
    pub color: LegoColor,
    /// Position of the zone (bin angle)
    pub position: Position2D,
    /// Number of pieces in this zone
    pub count: usize,
}

impl ColorZone {
    pub fn new(color: LegoColor, position: Position2D) -> Self {
        Self {
            color,
            position,
            count: 0,
        }
    }

    /// Increment piece count
    pub fn add_piece(&mut self) {
        self.count += 1;
    }
}

/// Path plan from current position to target
#[derive(Debug, Clone)]
pub struct PathPlan {
    /// Starting position
    pub from: Position2D,
    /// Target position
    pub to: Position2D,
    /// Waypoints along the path
    pub waypoints: Vec<Position2D>,
    /// Estimated time to complete (milliseconds)
    pub estimated_time_ms: u64,
}

impl PathPlan {
    /// Create a simple direct path
    pub fn direct(from: Position2D, to: Position2D, speed_mm_per_sec: f32) -> Self {
        let distance = from.distance_to(&to);
        let estimated_time_ms = ((distance / speed_mm_per_sec) * 1000.0) as u64;

        Self {
            from,
            to,
            waypoints: vec![],
            estimated_time_ms,
        }
    }

    /// Create a path with waypoints
    pub fn with_waypoints(
        from: Position2D,
        to: Position2D,
        waypoints: Vec<Position2D>,
        speed_mm_per_sec: f32,
    ) -> Self {
        // Calculate total distance
        let mut total_distance = from.distance_to(waypoints.first().unwrap_or(&to));
        for i in 0..waypoints.len() - 1 {
            total_distance += waypoints[i].distance_to(&waypoints[i + 1]);
        }
        if !waypoints.is_empty() {
            total_distance += waypoints.last().unwrap().distance_to(&to);
        }

        let estimated_time_ms = ((total_distance / speed_mm_per_sec) * 1000.0) as u64;

        Self {
            from,
            to,
            waypoints,
            estimated_time_ms,
        }
    }
}

/// Sorting task status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is actively running
    Active,
    /// Task is paused
    /// Contract: I-HELP-012 - Tasks must be interruptible
    Paused,
    /// Task completed successfully
    Complete,
    /// Task timed out
    /// Contract: I-HELP-011 - No infinite loops
    Timeout,
}

/// Sorting task state
/// Contract: I-HELP-010, I-HELP-012, I-HELP-013
#[derive(Debug, Clone)]
pub struct SortingTask {
    /// Unique task identifier
    pub id: String,
    /// Task type
    pub task_type: String,
    /// Color zones for sorting
    pub zones: Vec<ColorZone>,
    /// Number of pieces sorted
    pub items_sorted: usize,
    /// Estimated items remaining
    pub items_remaining: usize,
    /// Special finds (rare colors, etc.)
    pub special_finds: Vec<String>,
    /// Task status
    /// Contract: I-HELP-012 - Interruptible
    pub status: TaskStatus,
    /// Scan grid
    scan_grid: ScanGrid,
    /// Task start time (microseconds)
    start_time: u64,
    /// Task timeout (microseconds)
    /// Contract: I-HELP-011 - Prevents infinite operation
    timeout_us: u64,
    /// Maximum iterations
    /// Contract: I-HELP-011 - Failsafe
    max_iterations: usize,
    /// Current iteration count
    current_iteration: usize,
}

impl SortingTask {
    /// Create a new sorting task
    /// Contract: I-HELP-010 - Clear completion criteria
    pub fn new(
        id: impl Into<String>,
        task_type: impl Into<String>,
        grid_width: usize,
        grid_height: usize,
        scan_pattern: ScanPattern,
        timeout_us: u64,
    ) -> Self {
        Self {
            id: id.into(),
            task_type: task_type.into(),
            zones: Vec::new(),
            items_sorted: 0,
            items_remaining: 0,
            special_finds: Vec::new(),
            status: TaskStatus::Active,
            scan_grid: ScanGrid::new(grid_width, grid_height, scan_pattern),
            start_time: 0,
            timeout_us,
            max_iterations: grid_width * grid_height * 3, // 3x grid size as safety
            current_iteration: 0,
        }
    }

    /// Add a color zone to the task
    pub fn add_zone(&mut self, zone: ColorZone) {
        self.zones.push(zone);
    }

    /// Start the task
    pub fn start(&mut self, current_time: u64) {
        self.start_time = current_time;
        self.status = TaskStatus::Active;
    }

    /// Pause the task
    /// Contract: I-HELP-012 - Interruptible
    pub fn pause(&mut self) {
        if self.status == TaskStatus::Active {
            self.status = TaskStatus::Paused;
        }
    }

    /// Resume the task
    /// Contract: I-HELP-012 - Resumable
    pub fn resume(&mut self) {
        if self.status == TaskStatus::Paused {
            self.status = TaskStatus::Active;
        }
    }

    /// Check if task has timed out
    /// Contract: I-HELP-011 - Prevents infinite operation
    pub fn check_timeout(&mut self, current_time: u64) -> bool {
        if current_time - self.start_time > self.timeout_us {
            self.status = TaskStatus::Timeout;
            true
        } else {
            false
        }
    }

    /// Check if task is complete
    /// Contract: I-HELP-010 - Clear completion criteria
    pub fn is_complete(&self) -> bool {
        // Completion criteria:
        // 1. All items sorted (items_remaining == 0)
        // 2. OR coverage >= 95%
        self.items_remaining == 0 || self.scan_grid.coverage() >= 95.0
    }

    /// Update completion status
    pub fn update_completion(&mut self) {
        if self.is_complete() {
            self.status = TaskStatus::Complete;
        }
    }

    /// Increment iteration counter and check for infinite loop
    /// Contract: I-HELP-011 - No infinite loops
    pub fn increment_iteration(&mut self) -> Result<(), String> {
        self.current_iteration += 1;
        if self.current_iteration >= self.max_iterations {
            self.status = TaskStatus::Timeout;
            Err(format!(
                "Maximum iterations ({}) exceeded. Possible infinite loop.",
                self.max_iterations
            ))
        } else {
            Ok(())
        }
    }

    /// Get next cell to scan
    pub fn next_scan_cell(&mut self) -> Option<(usize, usize)> {
        self.scan_grid.next_cell()
    }

    /// Mark cell as scanned
    pub fn mark_cell_scanned(
        &mut self,
        x: usize,
        y: usize,
        has_piece: bool,
        piece_color: Option<LegoColor>,
        timestamp: u64,
    ) {
        if let Some(cell) = self.scan_grid.get_cell_mut(x, y) {
            cell.mark_scanned(has_piece, piece_color, timestamp);
            self.scan_grid.update_coverage();

            if has_piece {
                self.items_remaining += 1;
            }
        }
    }

    /// Record a piece being sorted
    pub fn record_sorted_piece(&mut self, color: LegoColor) {
        self.items_sorted += 1;
        if self.items_remaining > 0 {
            self.items_remaining -= 1;
        }

        // Update zone count
        if let Some(zone) = self.zones.iter_mut().find(|z| z.color == color) {
            zone.add_piece();
        }

        self.update_completion();
    }

    /// Record a special find
    pub fn record_special_find(&mut self, description: String) {
        self.special_finds.push(description);
    }

    /// Get current coverage percentage
    pub fn coverage(&self) -> f32 {
        self.scan_grid.coverage()
    }

    /// Get all detected pieces
    pub fn detected_pieces(&self) -> Vec<(usize, usize, LegoColor)> {
        self.scan_grid.pieces()
    }

    /// Re-scan for missed pieces
    pub fn rescan(&mut self) {
        self.scan_grid.reset();
        self.current_iteration = 0;
    }

    /// Get scan grid reference
    pub fn scan_grid(&self) -> &ScanGrid {
        &self.scan_grid
    }
}

/// Sorting algorithm engine
/// Contract: SORT-002 - Deterministic sorting
pub struct SortingAlgorithm {
    /// Carousel configuration
    carousel_config: CarouselConfig,
    /// Color-to-bin mapping
    color_bin_map: HashMap<LegoColor, String>,
    /// Movement speed (mm/s)
    movement_speed: f32,
}

impl SortingAlgorithm {
    /// Create a new sorting algorithm
    pub fn new(carousel_config: CarouselConfig) -> Self {
        Self {
            carousel_config,
            color_bin_map: HashMap::new(),
            movement_speed: 100.0, // 100 mm/s default
        }
    }

    /// Map a color to a bin
    /// Contract: SORT-002 - Deterministic mapping
    pub fn map_color_to_bin(&mut self, color: LegoColor, bin_id: impl Into<String>) {
        self.color_bin_map.insert(color, bin_id.into());
    }

    /// Get bin for a color
    pub fn get_bin_for_color(&self, color: LegoColor) -> Option<&Bin> {
        let bin_id = self.color_bin_map.get(&color)?;
        self.carousel_config
            .bins
            .iter()
            .find(|bin| &bin.bin_id == bin_id)
    }

    /// Calculate path to bin for a piece
    /// Contract: SORT-002 - Deterministic path planning
    pub fn plan_path(&self, from: Position2D, piece_color: LegoColor) -> Option<PathPlan> {
        let bin = self.get_bin_for_color(piece_color)?;

        // Convert bin angle to position
        let angle_rad = bin.position_angle.to_radians();
        let radius = 200.0; // Example radius in mm
        let bin_position = Position2D::new(angle_rad.cos() * radius, angle_rad.sin() * radius);

        Some(PathPlan::direct(from, bin_position, self.movement_speed))
    }

    /// Optimize bin rotation order to minimize movements
    /// Contract: SORT-002 - Bin optimization
    pub fn optimize_sorting_order(&self, pieces: &[(LegoColor, Position2D)]) -> Vec<usize> {
        // Simple optimization: group by color to minimize carousel rotations
        let mut order = Vec::new();
        let mut by_color: HashMap<LegoColor, Vec<usize>> = HashMap::new();

        for (idx, (color, _)) in pieces.iter().enumerate() {
            by_color.entry(*color).or_insert_with(Vec::new).push(idx);
        }

        // Process each color group together
        for color in LegoColor::all_known() {
            if let Some(indices) = by_color.get(color) {
                order.extend(indices);
            }
        }

        order
    }

    /// Determine best bin for unknown color pieces
    /// Contract: I-HELP-013 - Unknown piece handling
    pub fn get_unknown_bin(&self) -> Option<&Bin> {
        // Look for a bin with category "unknown" or "misc"
        self.carousel_config
            .bins
            .iter()
            .find(|bin| bin.category_rule.contains("unknown") || bin.category_rule.contains("misc"))
    }

    /// Calculate rotation angle needed to move from one bin to another
    /// Contract: SORT-002 - Minimize carousel rotations
    pub fn calculate_rotation(&self, from_angle: f32, to_angle: f32) -> f32 {
        let mut diff = to_angle - from_angle;

        // Normalize to -180 to 180
        while diff > 180.0 {
            diff -= 360.0;
        }
        while diff < -180.0 {
            diff += 360.0;
        }

        diff
    }

    /// Get estimated time for rotation
    pub fn estimate_rotation_time(&self, rotation_degrees: f32) -> u64 {
        let rotation_speed = self.carousel_config.rotation_speed;
        ((rotation_degrees.abs() / rotation_speed) * 1000.0) as u64
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_cell_creation() {
        let cell = GridCell::new(5, 10);
        assert_eq!(cell.x, 5);
        assert_eq!(cell.y, 10);
        assert!(!cell.scanned);
        assert!(!cell.has_piece);
        assert_eq!(cell.piece_color, None);
    }

    #[test]
    fn test_grid_cell_mark_scanned() {
        let mut cell = GridCell::new(0, 0);
        cell.mark_scanned(true, Some(LegoColor::Red), 1000);

        assert!(cell.scanned);
        assert!(cell.has_piece);
        assert_eq!(cell.piece_color, Some(LegoColor::Red));
        assert_eq!(cell.scan_timestamp, 1000);
    }

    #[test]
    fn test_scan_grid_creation() {
        let grid = ScanGrid::new(5, 5, ScanPattern::Zigzag);
        assert_eq!(grid.width, 5);
        assert_eq!(grid.height, 5);
        assert_eq!(grid.coverage(), 0.0);
    }

    #[test]
    fn test_scan_grid_zigzag_pattern() {
        let mut grid = ScanGrid::new(3, 2, ScanPattern::Zigzag);

        // Start at (0, 0)
        assert_eq!(grid.current_position(), (0, 0));

        // Row 0: left to right
        assert_eq!(grid.next_cell(), Some((1, 0)));
        assert_eq!(grid.next_cell(), Some((2, 0)));

        // Move to row 1
        assert_eq!(grid.next_cell(), Some((2, 1)));

        // Row 1: right to left
        assert_eq!(grid.next_cell(), Some((1, 1)));
        assert_eq!(grid.next_cell(), Some((0, 1)));

        // Should be done
        assert_eq!(grid.next_cell(), None);
    }

    #[test]
    fn test_scan_grid_linear_pattern() {
        let mut grid = ScanGrid::new(3, 2, ScanPattern::Linear);

        assert_eq!(grid.next_cell(), Some((1, 0)));
        assert_eq!(grid.next_cell(), Some((2, 0)));
        assert_eq!(grid.next_cell(), Some((0, 1)));
        assert_eq!(grid.next_cell(), Some((1, 1)));
        assert_eq!(grid.next_cell(), Some((2, 1)));
        assert_eq!(grid.next_cell(), None);
    }

    #[test]
    fn test_scan_grid_coverage() {
        let mut grid = ScanGrid::new(2, 2, ScanPattern::Zigzag);

        // No cells scanned
        grid.update_coverage();
        assert_eq!(grid.coverage(), 0.0);

        // Scan one cell
        grid.get_cell_mut(0, 0)
            .unwrap()
            .mark_scanned(false, None, 0);
        grid.update_coverage();
        assert_eq!(grid.coverage(), 25.0);

        // Scan all cells
        grid.get_cell_mut(1, 0)
            .unwrap()
            .mark_scanned(false, None, 0);
        grid.get_cell_mut(0, 1)
            .unwrap()
            .mark_scanned(false, None, 0);
        grid.get_cell_mut(1, 1)
            .unwrap()
            .mark_scanned(false, None, 0);
        grid.update_coverage();
        assert_eq!(grid.coverage(), 100.0);
    }

    #[test]
    fn test_scan_grid_pieces() {
        let mut grid = ScanGrid::new(3, 3, ScanPattern::Zigzag);

        // Add some pieces
        grid.get_cell_mut(0, 0)
            .unwrap()
            .mark_scanned(true, Some(LegoColor::Red), 0);
        grid.get_cell_mut(2, 1)
            .unwrap()
            .mark_scanned(true, Some(LegoColor::Blue), 0);

        let pieces = grid.pieces();
        assert_eq!(pieces.len(), 2);
        assert!(pieces.contains(&(0, 0, LegoColor::Red)));
        assert!(pieces.contains(&(2, 1, LegoColor::Blue)));
    }

    #[test]
    fn test_color_zone() {
        let mut zone = ColorZone::new(LegoColor::Red, Position2D::new(100.0, 0.0));
        assert_eq!(zone.color, LegoColor::Red);
        assert_eq!(zone.count, 0);

        zone.add_piece();
        zone.add_piece();
        assert_eq!(zone.count, 2);
    }

    #[test]
    fn test_path_plan_direct() {
        let from = Position2D::new(0.0, 0.0);
        let to = Position2D::new(100.0, 0.0);
        let plan = PathPlan::direct(from, to, 100.0);

        assert_eq!(plan.from.x, 0.0);
        assert_eq!(plan.to.x, 100.0);
        assert_eq!(plan.estimated_time_ms, 1000); // 100mm at 100mm/s = 1s
    }

    #[test]
    fn test_sorting_task_creation() {
        let task = SortingTask::new("task-1", "lego", 5, 5, ScanPattern::Zigzag, 600_000_000);

        assert_eq!(task.id, "task-1");
        assert_eq!(task.task_type, "lego");
        assert_eq!(task.status, TaskStatus::Active);
        assert_eq!(task.items_sorted, 0);
    }

    #[test]
    fn test_sorting_task_pause_resume() {
        let mut task = SortingTask::new("task-1", "lego", 5, 5, ScanPattern::Zigzag, 600_000_000);

        assert_eq!(task.status, TaskStatus::Active);

        task.pause();
        assert_eq!(task.status, TaskStatus::Paused);

        task.resume();
        assert_eq!(task.status, TaskStatus::Active);
    }

    #[test]
    fn test_sorting_task_completion() {
        let mut task = SortingTask::new("task-1", "lego", 2, 2, ScanPattern::Zigzag, 600_000_000);

        // Not complete initially (has items to scan)
        task.items_remaining = 5;
        assert!(!task.is_complete());

        // Complete when items_remaining = 0
        task.items_remaining = 0;
        assert!(task.is_complete());

        // Or when coverage >= 95%
        task.items_remaining = 5;
        task.scan_grid.coverage_percent = 96.0;
        assert!(task.is_complete());
    }

    #[test]
    fn test_sorting_task_timeout() {
        let mut task = SortingTask::new("task-1", "lego", 5, 5, ScanPattern::Zigzag, 1000);

        task.start(0);
        assert!(!task.check_timeout(500));
        assert_eq!(task.status, TaskStatus::Active);

        assert!(task.check_timeout(2000));
        assert_eq!(task.status, TaskStatus::Timeout);
    }

    #[test]
    fn test_sorting_task_max_iterations() {
        let mut task = SortingTask::new("task-1", "lego", 2, 2, ScanPattern::Zigzag, 600_000_000);

        // Should allow iterations up to max
        for _ in 0..11 {
            assert!(task.increment_iteration().is_ok());
        }

        // Should fail on 12th (max is 2*2*3 = 12)
        assert!(task.increment_iteration().is_err());
        assert_eq!(task.status, TaskStatus::Timeout);
    }

    #[test]
    fn test_sorting_task_record_sorted() {
        let mut task = SortingTask::new("task-1", "lego", 5, 5, ScanPattern::Zigzag, 600_000_000);

        let mut zone = ColorZone::new(LegoColor::Red, Position2D::new(100.0, 0.0));
        task.add_zone(zone);

        task.items_remaining = 3;
        task.record_sorted_piece(LegoColor::Red);

        assert_eq!(task.items_sorted, 1);
        assert_eq!(task.items_remaining, 2);
    }

    #[test]
    fn test_sorting_algorithm_color_mapping() {
        let config = CarouselConfig::default();
        let mut algo = SortingAlgorithm::new(config);

        algo.map_color_to_bin(LegoColor::Red, "bin-01");
        algo.map_color_to_bin(LegoColor::Blue, "bin-02");

        assert_eq!(
            algo.color_bin_map.get(&LegoColor::Red).unwrap(),
            "bin-01"
        );
        assert_eq!(
            algo.color_bin_map.get(&LegoColor::Blue).unwrap(),
            "bin-02"
        );
    }

    #[test]
    fn test_sorting_algorithm_rotation_calculation() {
        let config = CarouselConfig::default();
        let algo = SortingAlgorithm::new(config);

        // Forward rotation
        assert_eq!(algo.calculate_rotation(0.0, 90.0), 90.0);

        // Backward rotation
        assert_eq!(algo.calculate_rotation(90.0, 0.0), -90.0);

        // Wraparound (shortest path)
        assert_eq!(algo.calculate_rotation(350.0, 10.0), 20.0);
        assert_eq!(algo.calculate_rotation(10.0, 350.0), -20.0);
    }

    #[test]
    fn test_sorting_algorithm_optimize_order() {
        let config = CarouselConfig::default();
        let algo = SortingAlgorithm::new(config);

        let pieces = vec![
            (LegoColor::Red, Position2D::new(0.0, 0.0)),
            (LegoColor::Blue, Position2D::new(10.0, 0.0)),
            (LegoColor::Red, Position2D::new(20.0, 0.0)),
            (LegoColor::Blue, Position2D::new(30.0, 0.0)),
        ];

        let order = algo.optimize_sorting_order(&pieces);

        // Should group by color: reds first, then blues
        assert_eq!(order.len(), 4);
        // Indices 0 and 2 are red, indices 1 and 3 are blue
        assert!(order[0] == 0 || order[0] == 2);
        assert!(order[2] == 1 || order[2] == 3);
    }

    #[test]
    fn test_sorting_task_special_finds() {
        let mut task = SortingTask::new("task-1", "lego", 5, 5, ScanPattern::Zigzag, 600_000_000);

        task.record_special_find("gold piece detected".to_string());
        task.record_special_find("chrome brick found".to_string());

        assert_eq!(task.special_finds.len(), 2);
        assert_eq!(task.special_finds[0], "gold piece detected");
        assert_eq!(task.special_finds[1], "chrome brick found");
    }

    #[test]
    fn test_sorting_task_rescan() {
        let mut task = SortingTask::new("task-1", "lego", 3, 3, ScanPattern::Zigzag, 600_000_000);

        // Scan some cells
        task.mark_cell_scanned(0, 0, true, Some(LegoColor::Red), 1000);
        task.mark_cell_scanned(1, 0, false, None, 1100);

        task.scan_grid.update_coverage();
        assert!(task.coverage() > 0.0);

        // Rescan
        task.rescan();
        assert_eq!(task.coverage(), 0.0);
        assert_eq!(task.current_iteration, 0);
    }

    #[test]
    fn test_position_distance() {
        let p1 = Position2D::new(0.0, 0.0);
        let p2 = Position2D::new(3.0, 4.0);

        assert_eq!(p1.distance_to(&p2), 5.0);
    }
}
