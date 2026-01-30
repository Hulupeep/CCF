/**
 * Journey Test: J-ART-FIRST-DRAWING
 * Issue: #16 - STORY-ART-005: First Drawing Journey Test
 *
 * Scenario: User creates their first artwork with mBot
 *   Given the mBot is powered on and pen is ready
 *   When the user starts a drawing session
 *   And selects a simple shape (circle)
 *   Then mBot draws the shape smoothly
 *   And the pen lifts when complete
 *   And user sees the completed artwork
 */

import { test, expect } from '@playwright/test';

test.describe('J-ART-FIRST-DRAWING: First Drawing Journey', () => {
  test('user creates first simple drawing', async ({ page }) => {
    // Given: mBot is powered on and pen is ready
    await page.goto('/');
    await expect(page.getByTestId('mbot-status')).toHaveText('Ready');
    await expect(page.getByTestId('pen-status')).toHaveText('Up');

    // When: User starts a drawing session
    await page.getByTestId('start-drawing').click();
    await expect(page.getByTestId('drawing-mode')).toBeVisible();

    // And: Selects a simple shape (circle)
    await page.getByTestId('shape-selector').click();
    await page.getByTestId('shape-circle').click();

    // Then: mBot draws the shape smoothly
    await page.getByTestId('draw-shape').click();
    await expect(page.getByTestId('pen-status')).toHaveText('Down', { timeout: 5000 });

    // Wait for drawing to complete
    await expect(page.getByTestId('drawing-progress')).toHaveText('100%', { timeout: 30000 });

    // And: Pen lifts when complete
    await expect(page.getByTestId('pen-status')).toHaveText('Up');

    // And: User sees the completed artwork
    const canvas = page.getByTestId('drawing-canvas');
    await expect(canvas).toBeVisible();

    // Verify canvas has drawing (non-empty)
    const canvasContent = await canvas.screenshot();
    expect(canvasContent.length).toBeGreaterThan(1000); // Has content
  });

  test('drawing fails gracefully if pen servo error', async ({ page }) => {
    // Test error handling
    await page.goto('/');

    // Simulate servo error
    await page.evaluate(() => {
      (window as any).simulateServoError = true;
    });

    await page.getByTestId('start-drawing').click();
    await page.getByTestId('shape-circle').click();
    await page.getByTestId('draw-shape').click();

    // Should show error message
    await expect(page.getByTestId('error-message')).toContainText('servo');
    await expect(page.getByTestId('pen-status')).toHaveText('Error');
  });
});
