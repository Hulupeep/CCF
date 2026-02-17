//! Spatial Memory for Autonomous Exploration
//!
//! Provides coarse spatial mapping suitable for CyberPi's limited sensors
//! (ultrasonic distance + IMU yaw, no lidar/camera).
//!
//! - `SectorMap`: 12-sector bearing map (like a clock face, 30° each)
//! - `GridMap`: 10x10 room grid (each cell ~30cm)
//!
//! All types are no_std compatible (ARCH-001).

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
use core::f32::consts::PI;
#[cfg(feature = "std")]
use std::f32::consts::PI;

// ─── Sector Map (bearing-based) ──────────────────────────────────────

/// Reading for a single 30° sector of the robot's surroundings.
#[derive(Clone, Debug)]
pub struct SectorReading {
    /// Closest object distance seen in this sector (cm). 999.0 = nothing seen.
    pub min_distance_cm: f32,
    /// Number of times the robot has looked at this sector.
    pub visit_count: u16,
    /// Tick when this sector was last updated.
    pub last_update_tick: u64,
    /// Computed interest score (novelty × personality curiosity).
    pub interest_score: f32,
}

impl Default for SectorReading {
    fn default() -> Self {
        Self {
            min_distance_cm: 999.0,
            visit_count: 0,
            last_update_tick: 0,
            interest_score: 1.0, // unknown sectors are maximally interesting
        }
    }
}

/// 12-sector bearing map. Index 0 = 0°(ahead), 1 = 30°, … 11 = 330°.
pub struct SectorMap {
    pub sectors: [SectorReading; 12],
}

impl SectorMap {
    pub fn new() -> Self {
        Self {
            sectors: core::array::from_fn(|_| SectorReading::default()),
        }
    }

    /// Convert a heading in degrees (0–360) to sector index (0–11).
    pub fn heading_to_sector(heading_deg: f32) -> usize {
        let normalized = ((heading_deg % 360.0) + 360.0) % 360.0;
        ((normalized / 30.0) as usize).min(11)
    }

    /// Update a sector with a new distance reading.
    pub fn update_sector(&mut self, heading_deg: f32, distance_cm: f32, tick: u64) {
        let idx = Self::heading_to_sector(heading_deg);
        let sector = &mut self.sectors[idx];
        if distance_cm < sector.min_distance_cm {
            sector.min_distance_cm = distance_cm;
        }
        sector.visit_count = sector.visit_count.saturating_add(1);
        sector.last_update_tick = tick;
    }

    /// Recompute interest scores based on visit counts and curiosity drive.
    pub fn recompute_interest(&mut self, curiosity_drive: f32, current_tick: u64) {
        for sector in &mut self.sectors {
            // Novelty: fewer visits = more interesting
            let novelty = 1.0 / (1.0 + sector.visit_count as f32);
            // Staleness: older data = more interesting to revisit
            let staleness = if sector.last_update_tick == 0 {
                1.0 // never visited
            } else {
                let age = current_tick.saturating_sub(sector.last_update_tick) as f32;
                (age / 100.0).min(1.0)
            };
            // Curiosity amplifies interest
            sector.interest_score = (novelty * 0.6 + staleness * 0.4) * (0.5 + curiosity_drive);
        }
    }

    /// Find the sector with the highest interest score.
    pub fn most_interesting_sector(&self) -> usize {
        let mut best_idx = 0;
        let mut best_score = f32::NEG_INFINITY;
        for (i, sector) in self.sectors.iter().enumerate() {
            if sector.interest_score > best_score {
                best_score = sector.interest_score;
                best_idx = i;
            }
        }
        best_idx
    }

    /// Sector index → center bearing in degrees.
    pub fn sector_to_heading(sector_idx: usize) -> f32 {
        (sector_idx as f32) * 30.0 + 15.0 // center of 30° sector
    }

    /// Number of sectors that have been visited at least once.
    pub fn mapped_count(&self) -> usize {
        self.sectors.iter().filter(|s| s.visit_count > 0).count()
    }

    /// Summary string for LLM prompt context.
    pub fn summary(&self) -> SectorMapSummary {
        SectorMapSummary {
            mapped: self.mapped_count(),
            total: 12,
            most_interesting: self.most_interesting_sector(),
        }
    }
}

impl Default for SectorMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary data for LLM prompt context.
pub struct SectorMapSummary {
    pub mapped: usize,
    pub total: usize,
    pub most_interesting: usize,
}

// ─── Grid Map (position-based) ───────────────────────────────────────

/// What we know about a grid cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Occupancy {
    /// Never visited or observed.
    Unknown,
    /// Confirmed navigable.
    Free,
    /// Object detected here.
    Obstacle,
    /// Something noteworthy (sudden distance change, interesting reading).
    Interesting,
}

impl Default for Occupancy {
    fn default() -> Self {
        Occupancy::Unknown
    }
}

/// A single cell in the 10×10 grid.
#[derive(Clone, Debug)]
pub struct GridCell {
    pub occupancy: Occupancy,
    pub visit_count: u16,
    pub discovery_tick: u64,
}

impl Default for GridCell {
    fn default() -> Self {
        Self {
            occupancy: Occupancy::Unknown,
            visit_count: 0,
            discovery_tick: 0,
        }
    }
}

/// 10×10 room grid. Each cell represents ~30cm × 30cm.
pub const GRID_SIZE: usize = 10;
/// Cell size in centimeters.
pub const CELL_SIZE_CM: f32 = 30.0;

pub struct GridMap {
    pub cells: [[GridCell; GRID_SIZE]; GRID_SIZE],
    /// Robot's estimated grid column (0–9).
    pub robot_x: usize,
    /// Robot's estimated grid row (0–9).
    pub robot_y: usize,
    /// Robot heading in degrees from IMU yaw.
    pub robot_heading_deg: f32,
}

impl GridMap {
    /// Create a new grid with the robot starting at the center.
    pub fn new() -> Self {
        Self {
            cells: core::array::from_fn(|_| core::array::from_fn(|_| GridCell::default())),
            robot_x: GRID_SIZE / 2,
            robot_y: GRID_SIZE / 2,
            robot_heading_deg: 0.0,
        }
    }

    /// Mark the robot's current cell as Free and visited.
    pub fn mark_current_visited(&mut self, tick: u64) {
        let cell = &mut self.cells[self.robot_y][self.robot_x];
        if cell.occupancy == Occupancy::Unknown {
            cell.occupancy = Occupancy::Free;
        }
        cell.visit_count = cell.visit_count.saturating_add(1);
        cell.discovery_tick = tick;
    }

    /// Mark a cell at an estimated offset from the robot as an obstacle.
    /// `distance_cm`: ultrasonic reading. `heading_deg`: robot heading.
    pub fn mark_obstacle_ahead(&mut self, distance_cm: f32, heading_deg: f32, tick: u64) {
        let (ox, oy) = self.offset_from_heading(distance_cm, heading_deg);
        let nx = self.robot_x as i32 + ox;
        let ny = self.robot_y as i32 + oy;
        if let Some(cell) = self.cell_at_mut(nx, ny) {
            cell.occupancy = Occupancy::Obstacle;
            cell.discovery_tick = tick;
        }
    }

    /// Mark a cell as Interesting.
    pub fn mark_interesting(&mut self, x: usize, y: usize, tick: u64) {
        if x < GRID_SIZE && y < GRID_SIZE {
            self.cells[y][x].occupancy = Occupancy::Interesting;
            self.cells[y][x].discovery_tick = tick;
        }
    }

    /// Update robot position. Expects position in cm from origin.
    pub fn update_robot_position(&mut self, x_cm: f32, y_cm: f32, heading_deg: f32) {
        // Convert cm position to grid coords. Origin is at grid center.
        let center = (GRID_SIZE as f32 / 2.0) * CELL_SIZE_CM;
        let gx = ((x_cm + center) / CELL_SIZE_CM) as usize;
        let gy = ((y_cm + center) / CELL_SIZE_CM) as usize;
        self.robot_x = gx.min(GRID_SIZE - 1);
        self.robot_y = gy.min(GRID_SIZE - 1);
        self.robot_heading_deg = heading_deg;
    }

    /// Advance the robot by one grid cell in its current heading direction.
    /// Used for dead-reckoning when no encoder data is available.
    pub fn advance_one_cell(&mut self) {
        let rad = self.robot_heading_deg * PI / 180.0;
        #[cfg(not(feature = "std"))]
        let (sin_v, cos_v) = (libm::sinf(rad), libm::cosf(rad));
        #[cfg(feature = "std")]
        let (sin_v, cos_v) = (rad.sin(), rad.cos());
        let nx = (self.robot_x as f32 + cos_v).round() as i32;
        let ny = (self.robot_y as f32 + sin_v).round() as i32;
        if nx >= 0 && nx < GRID_SIZE as i32 && ny >= 0 && ny < GRID_SIZE as i32 {
            self.robot_x = nx as usize;
            self.robot_y = ny as usize;
        }
    }

    /// How many cells have been visited at least once.
    pub fn visited_count(&self) -> usize {
        self.cells.iter().flatten().filter(|c| c.visit_count > 0).count()
    }

    /// How many cells are still Unknown.
    pub fn unknown_count(&self) -> usize {
        self.cells
            .iter()
            .flatten()
            .filter(|c| c.occupancy == Occupancy::Unknown)
            .count()
    }

    /// Find the nearest unvisited cell (Manhattan distance). Returns (x, y).
    pub fn nearest_unvisited(&self) -> Option<(usize, usize)> {
        let mut best: Option<(usize, usize, u32)> = None;
        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                if self.cells[y][x].occupancy == Occupancy::Unknown {
                    let dist = (x as i32 - self.robot_x as i32).unsigned_abs()
                        + (y as i32 - self.robot_y as i32).unsigned_abs();
                    match best {
                        Some((_, _, d)) if dist < d => best = Some((x, y, dist)),
                        None => best = Some((x, y, dist)),
                        _ => {}
                    }
                }
            }
        }
        best.map(|(x, y, _)| (x, y))
    }

    /// Summary data for LLM prompt context.
    pub fn summary(&self) -> GridMapSummary {
        GridMapSummary {
            visited: self.visited_count(),
            total: GRID_SIZE * GRID_SIZE,
            robot_x: self.robot_x,
            robot_y: self.robot_y,
            heading_deg: self.robot_heading_deg,
        }
    }

    // ─── Helpers ─────────────────────────────────────────────────────

    fn offset_from_heading(&self, distance_cm: f32, heading_deg: f32) -> (i32, i32) {
        let cells = (distance_cm / CELL_SIZE_CM).round() as i32;
        let rad = heading_deg * PI / 180.0;
        #[cfg(not(feature = "std"))]
        let (sin_v, cos_v) = (libm::sinf(rad), libm::cosf(rad));
        #[cfg(feature = "std")]
        let (sin_v, cos_v) = (rad.sin(), rad.cos());
        let dx = (cos_v * cells as f32).round() as i32;
        let dy = (sin_v * cells as f32).round() as i32;
        (dx, dy)
    }

    fn cell_at_mut(&mut self, x: i32, y: i32) -> Option<&mut GridCell> {
        if x >= 0 && x < GRID_SIZE as i32 && y >= 0 && y < GRID_SIZE as i32 {
            Some(&mut self.cells[y as usize][x as usize])
        } else {
            None
        }
    }
}

impl Default for GridMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary data for LLM prompt context.
pub struct GridMapSummary {
    pub visited: usize,
    pub total: usize,
    pub robot_x: usize,
    pub robot_y: usize,
    pub heading_deg: f32,
}

// ─── Exploration Event (for event bus / narration) ───────────────────

/// Events emitted during exploration, consumed by narration + dashboard.
#[derive(Clone, Debug)]
pub enum ExplorationEvent {
    /// New cell discovered.
    CellDiscovered { x: usize, y: usize, occupancy: Occupancy },
    /// Obstacle detected.
    ObstacleFound { distance_cm: f32, heading_deg: f32 },
    /// Scan complete.
    ScanComplete { sectors_updated: usize },
    /// Navigation target chosen.
    TargetChosen { sector: usize, heading_deg: f32 },
    /// Arrived at target.
    Arrived { x: usize, y: usize },
    /// Exploration paused for reflection.
    ReflectionPause,
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sector_map_heading_to_sector() {
        assert_eq!(SectorMap::heading_to_sector(0.0), 0);
        assert_eq!(SectorMap::heading_to_sector(15.0), 0);
        assert_eq!(SectorMap::heading_to_sector(30.0), 1);
        assert_eq!(SectorMap::heading_to_sector(359.0), 11);
        assert_eq!(SectorMap::heading_to_sector(360.0), 0); // wraps
        assert_eq!(SectorMap::heading_to_sector(-30.0), 11); // negative wraps
    }

    #[test]
    fn test_sector_map_update() {
        let mut map = SectorMap::new();
        map.update_sector(45.0, 50.0, 10);
        let idx = SectorMap::heading_to_sector(45.0);
        assert_eq!(map.sectors[idx].min_distance_cm, 50.0);
        assert_eq!(map.sectors[idx].visit_count, 1);
        assert_eq!(map.sectors[idx].last_update_tick, 10);

        // Second reading with closer distance
        map.update_sector(45.0, 30.0, 20);
        assert_eq!(map.sectors[idx].min_distance_cm, 30.0);
        assert_eq!(map.sectors[idx].visit_count, 2);
    }

    #[test]
    fn test_sector_map_interest() {
        let mut map = SectorMap::new();
        // Visit sector 0 many times
        for i in 0..10 {
            map.update_sector(5.0, 100.0, i);
        }
        map.recompute_interest(0.8, 15);
        // Unvisited sectors should be more interesting than heavily visited ones
        assert!(map.sectors[6].interest_score > map.sectors[0].interest_score);
    }

    #[test]
    fn test_sector_map_most_interesting() {
        let mut map = SectorMap::new();
        // Visit all sectors except #5
        for i in 0..12 {
            if i != 5 {
                map.update_sector(i as f32 * 30.0, 100.0, 1);
            }
        }
        map.recompute_interest(0.5, 10);
        assert_eq!(map.most_interesting_sector(), 5);
    }

    #[test]
    fn test_grid_map_defaults() {
        let map = GridMap::new();
        assert_eq!(map.robot_x, 5);
        assert_eq!(map.robot_y, 5);
        assert_eq!(map.visited_count(), 0);
        assert_eq!(map.unknown_count(), 100);
    }

    #[test]
    fn test_grid_map_mark_visited() {
        let mut map = GridMap::new();
        map.mark_current_visited(1);
        assert_eq!(map.visited_count(), 1);
        assert_eq!(map.cells[5][5].occupancy, Occupancy::Free);
        assert_eq!(map.cells[5][5].visit_count, 1);
    }

    #[test]
    fn test_grid_map_nearest_unvisited() {
        let mut map = GridMap::new();
        // Visit center cell
        map.mark_current_visited(1);
        let nearest = map.nearest_unvisited();
        assert!(nearest.is_some());
        // Should be adjacent to center (distance 1)
        let (nx, ny) = nearest.unwrap();
        let dist = (nx as i32 - 5).abs() + (ny as i32 - 5).abs();
        assert_eq!(dist, 1);
    }

    #[test]
    fn test_grid_map_summary() {
        let map = GridMap::new();
        let summary = map.summary();
        assert_eq!(summary.visited, 0);
        assert_eq!(summary.total, 100);
        assert_eq!(summary.robot_x, 5);
        assert_eq!(summary.robot_y, 5);
    }

    #[test]
    fn test_occupancy_default() {
        assert_eq!(Occupancy::default(), Occupancy::Unknown);
    }
}
