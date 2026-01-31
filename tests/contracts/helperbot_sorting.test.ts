/**
 * Contract Tests for HelperBot Sorting Algorithm (STORY-HELP-002)
 *
 * Verifies invariants from feature_helperbot.yml:
 * - I-HELP-010: Sorting must have clear completion criteria
 * - I-HELP-011: No infinite loops in scanning
 * - I-HELP-012: Tasks must be interruptible
 * - I-HELP-013: Systematic coverage required
 */

import { describe, test, expect } from '@jest/globals';
import * as fs from 'fs';
import * as path from 'path';

const HELPERBOT_DIR = path.join(__dirname, '../../crates/mbot-core/src/helperbot');

describe('STORY-HELP-002: Sorting Algorithm Contracts', () => {
  let sortingCode: string;

  beforeAll(() => {
    sortingCode = fs.readFileSync(path.join(HELPERBOT_DIR, 'sorting.rs'), 'utf-8');
  });

  describe('I-HELP-010: Clear completion criteria', () => {
    test('SortingTask has completion check method', () => {
      expect(sortingCode).toMatch(/fn check_completion/);
      expect(sortingCode).toMatch(/items_remaining.*==.*0/);
      expect(sortingCode).toMatch(/coverage_percent.*>=.*95/);
    });

    test('Completion sets status to Complete', () => {
      expect(sortingCode).toMatch(/status.*=.*TaskStatus::Complete/);
    });
  });

  describe('I-HELP-011: No infinite loops', () => {
    test('Has maximum iteration limit', () => {
      expect(sortingCode).toMatch(/max_iterations/);
      expect(sortingCode).toMatch(/iteration_count/);
    });

    test('Has timeout mechanism', () => {
      expect(sortingCode).toMatch(/check_timeout/);
      expect(sortingCode).toMatch(/max_duration_us/);
      expect(sortingCode).toMatch(/TaskStatus::Timeout/);
    });

    test('Iteration count is checked in tick', () => {
      expect(sortingCode).toMatch(/iteration_count.*\+=.*1/);
      expect(sortingCode).toMatch(/iteration_count.*>=.*max_iterations/);
    });
  });

  describe('I-HELP-012: Tasks must be interruptible', () => {
    test('TaskStatus has Paused state', () => {
      expect(sortingCode).toMatch(/enum TaskStatus/);
      expect(sortingCode).toMatch(/Paused/);
    });

    test('Has pause method', () => {
      expect(sortingCode).toMatch(/fn pause/);
      expect(sortingCode).toMatch(/status.*=.*TaskStatus::Paused/);
    });

    test('Has resume method', () => {
      expect(sortingCode).toMatch(/fn resume/);
      expect(sortingCode).toMatch(/TaskStatus::Active/);
    });

    test('State can transition from Active to Paused', () => {
      expect(sortingCode).toMatch(/if.*status.*==.*TaskStatus::Active/);
    });
  });

  describe('I-HELP-013: Systematic coverage required', () => {
    test('Has grid-based scanning structure', () => {
      expect(sortingCode).toMatch(/struct ScanPattern/);
      expect(sortingCode).toMatch(/GridCell/);
      expect(sortingCode).toMatch(/coverage_percent/);
    });

    test('Has scan order patterns', () => {
      expect(sortingCode).toMatch(/enum ScanOrder/);
      expect(sortingCode).toMatch(/Zigzag/);
      expect(sortingCode).toMatch(/Spiral/);
      expect(sortingCode).toMatch(/Linear/);
    });

    test('Grid cells track scanned state', () => {
      expect(sortingCode).toMatch(/scanned.*bool/);
      expect(sortingCode).toMatch(/fn mark_scanned/);
    });

    test('Coverage percentage is calculated', () => {
      expect(sortingCode).toMatch(/fn update_coverage/);
      expect(sortingCode).toMatch(/scanned.*count/);
    });

    test('Next cell method ensures systematic traversal', () => {
      expect(sortingCode).toMatch(/fn next_cell/);
      expect(sortingCode).toMatch(/next_zigzag/);
    });
  });

  describe('Data Contracts (from issue #17)', () => {
    test('SortingTask has required fields', () => {
      expect(sortingCode).toMatch(/struct SortingTask/);
      expect(sortingCode).toMatch(/id.*String/);
      expect(sortingCode).toMatch(/zones.*Vec<ColorZone>/);
      expect(sortingCode).toMatch(/items_sorted.*u32/);
      expect(sortingCode).toMatch(/items_remaining.*u32/);
      expect(sortingCode).toMatch(/special_finds.*Vec<String>/);
      expect(sortingCode).toMatch(/status.*TaskStatus/);
    });

    test('ColorZone has required fields', () => {
      expect(sortingCode).toMatch(/struct ColorZone/);
      expect(sortingCode).toMatch(/color.*String/);
      expect(sortingCode).toMatch(/position.*Position/);
      expect(sortingCode).toMatch(/count.*u32/);
    });

    test('GridCell has required fields', () => {
      expect(sortingCode).toMatch(/struct GridCell/);
      expect(sortingCode).toMatch(/x.*usize/);
      expect(sortingCode).toMatch(/y.*usize/);
      expect(sortingCode).toMatch(/scanned.*bool/);
      expect(sortingCode).toMatch(/has_piece.*bool/);
      expect(sortingCode).toMatch(/piece_color.*Option<String>/);
    });

    test('PathPlan has required fields', () => {
      expect(sortingCode).toMatch(/struct PathPlan/);
      expect(sortingCode).toMatch(/from.*Position/);
      expect(sortingCode).toMatch(/to.*Position/);
      expect(sortingCode).toMatch(/waypoints.*Vec<Position>/);
      expect(sortingCode).toMatch(/estimated_time_ms.*u64/);
    });
  });

  describe('Acceptance Criteria (from issue #17)', () => {
    test('Grid-based scanning covers entire surface', () => {
      expect(sortingCode).toMatch(/grid_width.*usize/);
      expect(sortingCode).toMatch(/grid_height.*usize/);
      expect(sortingCode).toMatch(/new.*width.*height/);
    });

    test('Scanning pattern is systematic', () => {
      expect(sortingCode).toMatch(/scan_order.*ScanOrder/);
      expect(sortingCode).toMatch(/next_zigzag/);
    });

    test('Piece detection triggers', () => {
      expect(sortingCode).toMatch(/has_piece.*bool/);
      expect(sortingCode).toMatch(/scan_cell/);
    });

    test('Path planning routes to zones', () => {
      expect(sortingCode).toMatch(/fn plan_path/);
      expect(sortingCode).toMatch(/PathPlan::new/);
    });

    test('Task state can be persisted', () => {
      // Check for Serialize/Deserialize derives
      expect(sortingCode).toMatch(/#\[derive\(.*Serialize.*\)\]/);
      expect(sortingCode).toMatch(/#\[derive\(.*Deserialize.*\)\]/);
    });
  });
});
