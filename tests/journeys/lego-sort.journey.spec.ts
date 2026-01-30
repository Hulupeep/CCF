/**
 * Journey Test: J-SORT-RESET / J-HELP-LEGO-SORT
 * Issue: #32 - STORY-HELP-006: LEGO Sort Journey Test
 * Issue: #55 - STORY-SORT-008: Reset Play Area Journey Test
 *
 * Scenario: User sorts LEGO pieces by color
 *   Given a mixed pile of LEGO pieces in the play area
 *   When user starts the sorting session
 *   Then mBot picks up each piece
 *   And identifies its color
 *   And places it in the correct bin
 *   And all pieces are sorted within 5 minutes
 */

import { test, expect } from '@playwright/test';

test.describe('J-SORT-RESET: LEGO Sort Journey', () => {
  test('user sorts mixed LEGO pile successfully', async ({ page }) => {
    // Given: Mixed pile of LEGO pieces
    await page.goto('/');
    await page.getByTestId('mode-selector')).selectOption('sorter');

    // Set up test scenario with 10 mixed pieces
    await page.getByTestId('test-setup')).click();
    await page.getByTestId('piece-count')).fill('10');
    await page.getByTestId('colors')).selectOption(['red', 'blue', 'green', 'yellow']);
    await page.getByTestId('apply-test-setup')).click();

    await expect(page.getByTestId('pieces-remaining')).toHaveText('10');

    // When: User starts sorting session
    await page.getByTestId('start-sorting')).click();
    await expect(page.getByTestId('sorting-status')).toHaveText('Active');

    // Then: mBot picks up each piece and sorts
    // Wait for first pickup
    await expect(page.getByTestId('gripper-status')).toHaveText('Holding', { timeout: 10000 });

    // Color should be detected
    await expect(page.getByTestId('detected-color')).not.toBeEmpty();

    // Piece should be placed in bin
    await expect(page.getByTestId('pieces-remaining')).toHaveText('9', { timeout: 20000 });

    // All pieces sorted within 5 minutes
    await expect(page.getByTestId('pieces-remaining')).toHaveText('0', { timeout: 300000 });
    await expect(page.getByTestId('sorting-status')).toHaveText('Complete');

    // Verify bin counts
    const totalBinned = await page.getByTestId('bin-red')).textContent();
    expect(parseInt(totalBinned || '0')).toBeGreaterThan(0);
  });

  test('sorting handles piece detection errors gracefully', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('mode-selector')).selectOption('sorter');

    // Simulate detection error
    await page.evaluate(() => {
      (window as any).simulateDetectionError = true;
    });

    await page.getByTestId('start-sorting')).click();

    // Should show error and skip piece
    await expect(page.getByTestId('error-message')).toContainText('detection', { timeout: 10000 });
    await expect(page.getByTestId('error-count')).toContainText('1');
  });
});
