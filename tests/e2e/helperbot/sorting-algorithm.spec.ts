/**
 * E2E Tests for STORY-HELP-002: Sorting Algorithm
 *
 * Tests the complete sorting workflow including:
 * - Grid-based scanning
 * - Piece detection
 * - Path planning
 * - Completion detection
 * - Pause/resume functionality
 */

import { test, expect } from '@playwright/test';

test.describe('HELP-002: Sorting Algorithm', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to HelperBot sorting interface
    await page.goto('/helperbot/sorting');
    await page.waitForLoadState('networkidle');
  });

  test('@HELP-002-grid-scan: Systematic grid scanning', async ({ page }) => {
    // Given: 10 LEGO pieces are scattered on the surface
    await page.click('[data-testid="setup-sorting-surface"]');
    await page.fill('[data-testid="piece-count-input"]', '10');
    await page.selectOption('[data-testid="scan-pattern-select"]', 'zigzag');

    // When: the robot starts the sorting scan
    await page.click('[data-testid="start-scan-button"]');

    // Then: it should follow a zigzag pattern
    const scanPattern = await page.textContent('[data-testid="scan-pattern-display"]');
    expect(scanPattern).toBe('zigzag');

    // And: every grid cell should be visited
    await page.waitForSelector('[data-testid="scan-complete"]', { timeout: 30000 });
    const coverage = await page.textContent('[data-testid="coverage-percent"]');
    expect(parseFloat(coverage || '0')).toBeGreaterThanOrEqual(100);
  });

  test('@HELP-002-piece-detection: Detect piece presence in cell', async ({ page }) => {
    // Given: the robot is scanning a grid cell
    await page.click('[data-testid="setup-sorting-surface"]');
    await page.click('[data-testid="place-piece-button"]');
    await page.click('[data-testid="grid-cell-2-3"]'); // Place piece at (2,3)

    // When: a LEGO piece is present in the cell
    await page.click('[data-testid="start-scan-button"]');

    // Then: has_piece should be true
    await page.waitForSelector('[data-testid="cell-2-3-detected"]');
    const cellStatus = await page.getAttribute('[data-testid="grid-cell-2-3"]', 'data-has-piece');
    expect(cellStatus).toBe('true');

    // And: piece_color should be detected
    const pieceColor = await page.getAttribute('[data-testid="grid-cell-2-3"]', 'data-color');
    expect(pieceColor).toBeTruthy();
  });

  test('@HELP-002-path-planning: Plan path to color zone', async ({ page }) => {
    // Given: a red piece is detected at position (2, 3)
    await page.click('[data-testid="place-red-piece"]');
    await page.click('[data-testid="grid-cell-2-3"]');

    // And: the red zone is at position (10, 5)
    await page.fill('[data-testid="red-zone-x"]', '10');
    await page.fill('[data-testid="red-zone-y"]', '5');

    // When: path planning is invoked
    await page.click('[data-testid="plan-path-button"]');

    // Then: a PathPlan should be generated
    await page.waitForSelector('[data-testid="path-plan-display"]');
    const pathExists = await page.isVisible('[data-testid="path-plan-display"]');
    expect(pathExists).toBe(true);

    // And: estimated_time_ms should be calculated
    const estimatedTime = await page.textContent('[data-testid="estimated-time"]');
    expect(parseFloat(estimatedTime || '0')).toBeGreaterThan(0);
  });

  test('@HELP-002-completion: Detect sorting completion', async ({ page }) => {
    // Given: 10 pieces have been sorted
    await page.click('[data-testid="setup-test-scenario"]');
    await page.selectOption('[data-testid="scenario-select"]', 'all-sorted');

    // And: 0 pieces remain on the surface
    await page.click('[data-testid="verify-surface-empty"]');

    // When: completion check runs
    await page.click('[data-testid="check-completion-button"]');

    // Then: status should be "complete"
    const status = await page.textContent('[data-testid="task-status"]');
    expect(status).toBe('complete');

    // And: items_remaining should be 0
    const remaining = await page.textContent('[data-testid="items-remaining"]');
    expect(remaining).toBe('0');
  });

  test('@HELP-002-missed-piece: Handle missed pieces with rescan', async ({ page }) => {
    // Given: the initial scan is complete
    await page.click('[data-testid="setup-sorting-surface"]');
    await page.fill('[data-testid="piece-count-input"]', '10');
    await page.click('[data-testid="start-scan-button"]');
    await page.waitForSelector('[data-testid="scan-complete"]');

    // And: 1 piece was missed
    const itemsRemaining = await page.textContent('[data-testid="items-remaining"]');
    expect(parseInt(itemsRemaining || '0')).toBeGreaterThan(0);

    // When: the robot performs a verification scan
    await page.click('[data-testid="rescan-button"]');

    // Then: the missed piece should be detected
    await page.waitForSelector('[data-testid="rescan-complete"]');
    const newRemaining = await page.textContent('[data-testid="items-remaining"]');
    expect(parseInt(newRemaining || '0')).toBe(0);
  });

  test('@HELP-002-pause-resume: Pause and resume sorting task', async ({ page }) => {
    // Given: sorting is in progress with 5 pieces sorted
    await page.click('[data-testid="setup-sorting-surface"]');
    await page.fill('[data-testid="piece-count-input"]', '10');
    await page.click('[data-testid="start-scan-button"]');

    // Wait for some progress
    await page.waitForTimeout(2000);

    // When: the user pauses the task
    await page.click('[data-testid="pause-button"]');

    // Then: status should be "paused"
    const status = await page.textContent('[data-testid="task-status"]');
    expect(status).toBe('paused');

    // And: current state should be preserved
    const sortedCount = await page.textContent('[data-testid="items-sorted"]');
    const sortedBeforePause = parseInt(sortedCount || '0');

    // When: the user resumes the task
    await page.click('[data-testid="resume-button"]');

    // Then: sorting should continue from the saved position
    const newStatus = await page.textContent('[data-testid="task-status"]');
    expect(newStatus).toBe('active');

    // And: items_sorted should not have reset
    const newSortedCount = await page.textContent('[data-testid="items-sorted"]');
    expect(parseInt(newSortedCount || '0')).toBeGreaterThanOrEqual(sortedBeforePause);
  });

  test('@HELP-002-timeout: Timeout prevents infinite sorting', async ({ page }) => {
    // Given: the maximum sorting time is set
    await page.click('[data-testid="setup-sorting-surface"]');
    await page.fill('[data-testid="max-duration-input"]', '5'); // 5 seconds for test
    await page.fill('[data-testid="piece-count-input"]', '100'); // Many pieces

    // When: sorting runs for the maximum time
    await page.click('[data-testid="start-scan-button"]');

    // Then: the task should timeout gracefully
    await page.waitForSelector('[data-testid="task-timeout"]', { timeout: 10000 });
    const status = await page.textContent('[data-testid="task-status"]');
    expect(status).toBe('timeout');

    // And: partial results should be available
    const sortedCount = await page.textContent('[data-testid="items-sorted"]');
    expect(parseInt(sortedCount || '0')).toBeGreaterThan(0);
  });

  test('@HELP-002-special-finds: Track special finds during sorting', async ({ page }) => {
    // Given: a gold piece is placed
    await page.click('[data-testid="place-gold-piece"]');
    await page.click('[data-testid="grid-cell-5-5"]');

    // When: sorting detects and sorts the piece
    await page.click('[data-testid="start-scan-button"]');
    await page.waitForSelector('[data-testid="gold-detected"]');

    // Then: special_finds should contain "gold"
    const specialFinds = await page.textContent('[data-testid="special-finds-list"]');
    expect(specialFinds).toContain('gold');

    // And: the piece should still be sorted correctly
    const goldZoneCount = await page.textContent('[data-testid="gold-zone-count"]');
    expect(parseInt(goldZoneCount || '0')).toBe(1);
  });

  test('Performance: path planning completes in < 50ms', async ({ page }) => {
    // Setup
    await page.click('[data-testid="place-red-piece"]');
    await page.click('[data-testid="grid-cell-2-3"]');
    await page.fill('[data-testid="red-zone-x"]', '10');
    await page.fill('[data-testid="red-zone-y"]', '5');

    // Measure path planning time
    const startTime = Date.now();
    await page.click('[data-testid="plan-path-button"]');
    await page.waitForSelector('[data-testid="path-plan-display"]');
    const endTime = Date.now();

    const duration = endTime - startTime;
    expect(duration).toBeLessThan(50);
  });
});
