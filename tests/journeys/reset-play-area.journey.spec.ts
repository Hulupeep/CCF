/**
 * Journey Test: J-SORT-RESET
 * Issue: #55 - STORY-SORT-008: Reset Play Area Journey Test
 *
 * Journey: Reset the Play Area
 * Actor: Family with mixed LEGO mess
 * Goal: Restore order fast after play
 *
 * Contract References:
 * - SORT-001: Servo calibration repeatable (±2°)
 * - SORT-002: Sorting loop deterministic
 * - SORT-003: Safety - no pinch events
 * - SORT-004: Inventory must persist
 *
 * Preconditions:
 * - mBot is calibrated (STORY-SORT-001)
 * - Carousel is configured with 6+ bins (STORY-SORT-002)
 * - Vision system is active (STORY-SORT-003)
 * - Inventory tracking is enabled (STORY-SORT-006)
 */

import { test, expect } from '@playwright/test';

test.describe('J-SORT-RESET: Reset Play Area Journey', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to sorting interface
    await page.goto('/sorter');
    await page.waitForLoadState('networkidle');

    // Verify preconditions
    await expect(page.getByTestId('calibration-status')).toHaveText('Calibrated');
    await expect(page.getByTestId('carousel-status')).toContainText('Ready');
    await expect(page.getByTestId('vision-status')).toHaveText('Active');
  });

  test('@critical @journey: Complete happy path - mixed pile to sorted bins', async ({ page }) => {
    /**
     * Scenario: Complete happy path - mixed pile to sorted bins
     * Given the mBot is ready (calibrated and configured)
     * And the tray is empty
     * And all bins have known inventory counts
     * When I dump 20 mixed-color pieces into the tray
     * And I press the "Start Sorting" button
     * Then the robot should begin the sorting loop
     * And I should see progress: "Sorting piece X of Y"
     * And each piece should go to its color-matched bin
     * When all pieces are sorted
     * Then I should see "Sorting Complete" message
     * And the summary should show pieces sorted, time taken, bins used
     * And inventory counts should be updated
     */

    // Given: mBot is ready, tray is empty, bins have known inventory
    await expect(page.getByTestId('tray-status')).toHaveText('Empty');

    // Record initial inventory counts
    const initialInventory = await page.evaluate(() => {
      const bins = {};
      document.querySelectorAll('[data-testid^="bin-count-"]').forEach(el => {
        const binId = el.getAttribute('data-testid')?.replace('bin-count-', '');
        const count = parseInt(el.textContent || '0');
        if (binId) bins[binId] = count;
      });
      return bins;
    });

    // When: Dump 20 mixed-color pieces into the tray
    await page.getByTestId('test-setup').click();
    await page.getByTestId('piece-count').fill('20');
    await page.getByTestId('colors').selectOption(['red', 'blue', 'yellow', 'green']);
    await page.getByTestId('apply-test-setup').click();

    // Verify tray has pieces
    await expect(page.getByTestId('pieces-remaining')).toHaveText('20');
    await expect(page.getByTestId('tray-status')).toHaveText('Ready');

    // And: Press "Start Sorting" button
    await page.getByTestId('journey-start').click();

    // Then: Robot begins sorting loop
    await expect(page.getByTestId('sorting-status')).toHaveText('Active', { timeout: 5000 });

    // And: See progress updates
    await expect(page.getByTestId('journey-progress')).toContainText('Sorting');

    // Monitor progress (should see piece count decreasing)
    await page.waitForFunction(
      () => {
        const remaining = document.querySelector('[data-testid="pieces-remaining"]');
        return remaining && parseInt(remaining.textContent || '20') < 20;
      },
      { timeout: 30000 }
    );

    // Wait for completion (2 minutes max for 20 pieces)
    await expect(page.getByTestId('journey-complete')).toBeVisible({ timeout: 120000 });
    await expect(page.getByTestId('sorting-status')).toHaveText('Complete');

    // And: Summary shows correct data
    const sortedCount = await page.getByTestId('summary-sorted').textContent();
    expect(parseInt(sortedCount || '0')).toBeGreaterThanOrEqual(16); // ≥80% success rate

    const skippedCount = await page.getByTestId('summary-skipped').textContent();
    expect(parseInt(skippedCount || '0')).toBeLessThanOrEqual(4); // ≤20% failure rate

    const duration = await page.getByTestId('summary-duration').textContent();
    expect(duration).toBeTruthy();

    const binsUsed = await page.getByTestId('summary-bins').textContent();
    expect(parseInt(binsUsed || '0')).toBeGreaterThan(0);

    // And: Inventory counts updated
    const finalInventory = await page.evaluate(() => {
      const bins = {};
      document.querySelectorAll('[data-testid^="bin-count-"]').forEach(el => {
        const binId = el.getAttribute('data-testid')?.replace('bin-count-', '');
        const count = parseInt(el.textContent || '0');
        if (binId) bins[binId] = count;
      });
      return bins;
    });

    // At least one bin should have increased
    let totalIncrease = 0;
    Object.keys(finalInventory).forEach(binId => {
      const increase = finalInventory[binId] - (initialInventory[binId] || 0);
      totalIncrease += increase;
    });
    expect(totalIncrease).toBeGreaterThanOrEqual(16);
  });

  test('@critical @journey: Handle partial completion with skips', async ({ page }) => {
    /**
     * Scenario: Handle partial completion with skips
     * Given the tray has 15 pieces including 2 problematic ones
     * When I start sorting
     * And 2 pieces fail after max retries
     * Then 13 pieces should be sorted successfully
     * And 2 pieces should be skipped
     * And the summary should show "13 sorted, 2 skipped"
     * And skipped pieces should remain in tray
     */

    // Given: Tray has 15 pieces with 2 problematic ones
    await page.getByTestId('test-setup').click();
    await page.getByTestId('piece-count').fill('15');
    await page.getByTestId('problematic-pieces').fill('2');
    await page.getByTestId('apply-test-setup').click();

    await expect(page.getByTestId('pieces-remaining')).toHaveText('15');

    // When: Start sorting
    await page.getByTestId('journey-start').click();
    await expect(page.getByTestId('sorting-status')).toHaveText('Active', { timeout: 5000 });

    // Wait for completion (pieces fail after max retries)
    await expect(page.getByTestId('journey-complete')).toBeVisible({ timeout: 120000 });

    // Then: 13 sorted, 2 skipped
    const sortedCount = await page.getByTestId('summary-sorted').textContent();
    expect(parseInt(sortedCount || '0')).toBe(13);

    const skippedCount = await page.getByTestId('summary-skipped').textContent();
    expect(parseInt(skippedCount || '0')).toBe(2);

    // And: Summary shows correct message
    await expect(page.getByTestId('journey-summary')).toContainText('13 sorted, 2 skipped');

    // And: Skipped pieces remain in tray
    const remainingPieces = await page.getByTestId('pieces-remaining').textContent();
    expect(parseInt(remainingPieces || '0')).toBe(2);
  });

  test('@critical @journey: User pauses mid-sort', async ({ page }) => {
    /**
     * Scenario: User pauses mid-sort
     * Given sorting is in progress (10 of 20 pieces done)
     * When I press "Pause"
     * Then the current operation should complete safely
     * And the robot should stop at home position
     * And I should see "Paused - 10 sorted, 10 remaining"
     * When I press "Resume"
     * Then sorting should continue from piece 11
     */

    // Setup 20 pieces
    await page.getByTestId('test-setup').click();
    await page.getByTestId('piece-count').fill('20');
    await page.getByTestId('apply-test-setup').click();

    // Start sorting
    await page.getByTestId('journey-start').click();
    await expect(page.getByTestId('sorting-status')).toHaveText('Active', { timeout: 5000 });

    // Wait for approximately half to be sorted
    await page.waitForFunction(
      () => {
        const remaining = document.querySelector('[data-testid="pieces-remaining"]');
        return remaining && parseInt(remaining.textContent || '20') <= 10;
      },
      { timeout: 60000 }
    );

    // Record pieces sorted before pause
    const sortedBeforePause = await page.getByTestId('summary-sorted').textContent();
    const pauseSortedCount = parseInt(sortedBeforePause || '0');

    // When: Press "Pause"
    await page.getByTestId('journey-pause').click();

    // Then: Robot completes current operation safely
    await expect(page.getByTestId('sorting-status')).toHaveText('Paused', { timeout: 10000 });
    await expect(page.getByTestId('robot-position')).toContainText('Home');

    // And: Status shows paused state
    const pauseMessage = await page.getByTestId('journey-progress').textContent();
    expect(pauseMessage).toContain('Paused');

    // When: Press "Resume"
    await page.getByTestId('journey-resume').click();

    // Then: Sorting continues
    await expect(page.getByTestId('sorting-status')).toHaveText('Active', { timeout: 5000 });

    // Wait for more pieces to be sorted
    await page.waitForFunction(
      (beforeCount) => {
        const sorted = document.querySelector('[data-testid="summary-sorted"]');
        return sorted && parseInt(sorted.textContent || '0') > beforeCount;
      },
      pauseSortedCount,
      { timeout: 30000 }
    );

    // Verify sorting continues to completion
    await expect(page.getByTestId('journey-complete')).toBeVisible({ timeout: 120000 });
  });

  test('@critical @journey: Recovery from jam during journey', async ({ page }) => {
    /**
     * Scenario: Recovery from jam during journey
     * Given sorting is in progress
     * When a jam is detected (3 consecutive failures)
     * Then the robot should pause
     * And display "Needs help - piece stuck"
     * When I clear the jam manually
     * And press "Resume"
     * Then sorting should continue
     */

    // Setup with jam simulation
    await page.getByTestId('test-setup').click();
    await page.getByTestId('piece-count').fill('15');
    await page.getByTestId('simulate-jam').check();
    await page.getByTestId('jam-after-pieces').fill('5');
    await page.getByTestId('apply-test-setup').click();

    // Start sorting
    await page.getByTestId('journey-start').click();
    await expect(page.getByTestId('sorting-status')).toHaveText('Active', { timeout: 5000 });

    // When: Jam detected (3 consecutive failures)
    await expect(page.getByTestId('error-message')).toContainText('Needs help', { timeout: 60000 });
    await expect(page.getByTestId('error-message')).toContainText('stuck');

    // Then: Robot pauses
    await expect(page.getByTestId('sorting-status')).toHaveText('Paused');

    // When: Clear jam manually
    await page.getByTestId('clear-jam').click();

    // And: Press "Resume"
    await page.getByTestId('journey-resume').click();

    // Then: Sorting continues
    await expect(page.getByTestId('sorting-status')).toHaveText('Active', { timeout: 5000 });
    await expect(page.getByTestId('error-message')).not.toBeVisible();

    // Wait for completion
    await expect(page.getByTestId('journey-complete')).toBeVisible({ timeout: 120000 });
  });

  test('@journey: Empty tray detection', async ({ page }) => {
    /**
     * Scenario: Empty tray detection
     * Given the tray is empty
     * When I press "Start Sorting"
     * Then the system should report "Tray is empty"
     * And prompt "Add pieces to begin sorting"
     */

    // Given: Tray is empty (default state)
    await expect(page.getByTestId('tray-status')).toHaveText('Empty');
    await expect(page.getByTestId('pieces-remaining')).toHaveText('0');

    // When: Press "Start Sorting"
    await page.getByTestId('journey-start').click();

    // Then: System reports empty tray
    await expect(page.getByTestId('error-message')).toContainText('Tray is empty', { timeout: 5000 });
    await expect(page.getByTestId('error-message')).toContainText('Add pieces to begin sorting');

    // And: Sorting does not start
    await expect(page.getByTestId('sorting-status')).not.toHaveText('Active');
  });

  test('@journey: View completion summary details', async ({ page }) => {
    /**
     * Scenario: View completion summary details
     * Given sorting just completed (25 pieces)
     * When I tap the summary for details
     * Then I should see breakdown by bin with before/after/delta
     */

    // Setup and complete sorting
    await page.getByTestId('test-setup').click();
    await page.getByTestId('piece-count').fill('25');
    await page.getByTestId('colors').selectOption(['red', 'blue', 'yellow', 'green']);
    await page.getByTestId('apply-test-setup').click();

    // Record initial bin counts
    const initialBins = await page.evaluate(() => {
      const bins = {};
      document.querySelectorAll('[data-testid^="bin-count-"]').forEach(el => {
        const binId = el.getAttribute('data-testid')?.replace('bin-count-', '') || '';
        bins[binId] = parseInt(el.textContent || '0');
      });
      return bins;
    });

    // Start and wait for completion
    await page.getByTestId('journey-start').click();
    await expect(page.getByTestId('journey-complete')).toBeVisible({ timeout: 120000 });

    // When: Tap summary for details
    await page.getByTestId('summary-details').click();

    // Then: See breakdown by bin
    await expect(page.getByTestId('delta-breakdown')).toBeVisible();

    // Verify each bin shows before/after/delta
    for (const binId of Object.keys(initialBins)) {
      const deltaRow = page.getByTestId(`delta-${binId}`);
      await expect(deltaRow).toBeVisible();

      const beforeCell = deltaRow.locator('[data-column="before"]');
      const afterCell = deltaRow.locator('[data-column="after"]');
      const addedCell = deltaRow.locator('[data-column="added"]');

      await expect(beforeCell).toBeVisible();
      await expect(afterCell).toBeVisible();
      await expect(addedCell).toBeVisible();
    }
  });

  test('@journey @safety: Emergency stop preserves state', async ({ page }) => {
    /**
     * Scenario: Emergency stop preserves state
     * Given sorting is at piece 15 of 30
     * When I hit emergency stop
     * Then all motion should halt immediately
     * And the session should save state
     * And I should be able to resume later
     */

    // Setup 30 pieces
    await page.getByTestId('test-setup').click();
    await page.getByTestId('piece-count').fill('30');
    await page.getByTestId('apply-test-setup').click();

    // Start sorting
    await page.getByTestId('journey-start').click();
    await expect(page.getByTestId('sorting-status')).toHaveText('Active', { timeout: 5000 });

    // Wait for approximately half to be sorted
    await page.waitForFunction(
      () => {
        const remaining = document.querySelector('[data-testid="pieces-remaining"]');
        return remaining && parseInt(remaining.textContent || '30') <= 15;
      },
      { timeout: 90000 }
    );

    // Record state before emergency stop
    const sortedBeforeStop = await page.getByTestId('summary-sorted').textContent();
    const stopSortedCount = parseInt(sortedBeforeStop || '0');

    // When: Hit emergency stop
    await page.getByTestId('emergency-stop').click();

    // Then: Motion halts immediately
    await expect(page.getByTestId('sorting-status')).toHaveText('Stopped', { timeout: 2000 });
    await expect(page.getByTestId('robot-position')).toContainText('Halted');

    // And: Session saves state
    await expect(page.getByTestId('session-saved')).toBeVisible({ timeout: 5000 });

    // And: Can resume later
    await expect(page.getByTestId('journey-resume')).toBeEnabled();

    // Verify state is preserved
    const sortedAfterStop = await page.getByTestId('summary-sorted').textContent();
    expect(parseInt(sortedAfterStop || '0')).toBeGreaterThanOrEqual(stopSortedCount);

    // Resume and verify continuation
    await page.getByTestId('journey-resume').click();
    await expect(page.getByTestId('sorting-status')).toHaveText('Active', { timeout: 5000 });
  });

  test('Performance: Sorting rate meets requirements', async ({ page }) => {
    /**
     * Performance test: Verify sorting meets minimum rate
     * - Should sort at least 1 piece per 6 seconds
     * - 20 pieces should complete in ≤120 seconds
     */

    await page.getByTestId('test-setup').click();
    await page.getByTestId('piece-count').fill('20');
    await page.getByTestId('apply-test-setup').click();

    // Start and measure time
    const startTime = Date.now();
    await page.getByTestId('journey-start').click();
    await expect(page.getByTestId('journey-complete')).toBeVisible({ timeout: 120000 });
    const endTime = Date.now();

    const duration = endTime - startTime;
    const sortedCount = await page.getByTestId('summary-sorted').textContent();
    const sorted = parseInt(sortedCount || '0');

    // Verify performance
    expect(sorted).toBeGreaterThanOrEqual(16); // ≥80% success
    expect(duration).toBeLessThan(120000); // ≤120 seconds

    const avgTimePerPiece = duration / sorted;
    expect(avgTimePerPiece).toBeLessThan(6000); // ≤6 seconds per piece
  });

  test('Contract SORT-001: Calibration persists throughout journey', async ({ page }) => {
    /**
     * Verify SORT-001: Servo calibration repeatable (±2°)
     * Calibration should remain accurate throughout the entire sorting session
     */

    await page.getByTestId('test-setup').click();
    await page.getByTestId('piece-count').fill('15');
    await page.getByTestId('apply-test-setup').click();

    // Check calibration before
    await expect(page.getByTestId('calibration-status')).toHaveText('Calibrated');
    const calibrationBefore = await page.getByTestId('calibration-accuracy').textContent();

    // Run sorting
    await page.getByTestId('journey-start').click();
    await expect(page.getByTestId('journey-complete')).toBeVisible({ timeout: 120000 });

    // Check calibration after
    await expect(page.getByTestId('calibration-status')).toHaveText('Calibrated');
    const calibrationAfter = await page.getByTestId('calibration-accuracy').textContent();

    // Verify calibration drift is within ±2°
    const beforeDegrees = parseFloat(calibrationBefore || '0');
    const afterDegrees = parseFloat(calibrationAfter || '0');
    const drift = Math.abs(afterDegrees - beforeDegrees);

    expect(drift).toBeLessThanOrEqual(2);
  });

  test('Contract SORT-004: Inventory persists after journey', async ({ page }) => {
    /**
     * Verify SORT-004: Inventory must persist
     * Inventory changes should be persisted to storage
     */

    // Setup and complete sorting
    await page.getByTestId('test-setup').click();
    await page.getByTestId('piece-count').fill('20');
    await page.getByTestId('apply-test-setup').click();

    await page.getByTestId('journey-start').click();
    await expect(page.getByTestId('journey-complete')).toBeVisible({ timeout: 120000 });

    // Record inventory after completion
    const inventoryAfter = await page.evaluate(() => {
      const bins = {};
      document.querySelectorAll('[data-testid^="bin-count-"]').forEach(el => {
        const binId = el.getAttribute('data-testid')?.replace('bin-count-', '') || '';
        bins[binId] = parseInt(el.textContent || '0');
      });
      return bins;
    });

    // Reload page (simulates app restart)
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify inventory persisted
    const inventoryReloaded = await page.evaluate(() => {
      const bins = {};
      document.querySelectorAll('[data-testid^="bin-count-"]').forEach(el => {
        const binId = el.getAttribute('data-testid')?.replace('bin-count-', '') || '';
        bins[binId] = parseInt(el.textContent || '0');
      });
      return bins;
    });

    // Compare inventories
    Object.keys(inventoryAfter).forEach(binId => {
      expect(inventoryReloaded[binId]).toBe(inventoryAfter[binId]);
    });
  });
});
