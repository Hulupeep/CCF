/**
 * Journey Test: J-LEARN-FIRST-EXPERIMENT
 * Issue: #33 - STORY-LEARN-006: First Experiment Journey Test
 *
 * Scenario: Student runs first experiment
 *   Given mBot is in LearningLab mode
 *   When student selects "Reflex Response" experiment
 *   And triggers ultrasonic sensor with hand
 *   Then mBot displays real-time tension graph
 *   And student sees reflex mode changes
 *   And experiment data is saved for review
 */

import { test, expect } from '@playwright/test';

test.describe('J-LEARN-FIRST-EXPERIMENT: First Experiment Journey', () => {
  test('student runs reflex response experiment', async ({ page }) => {
    // Given: mBot is in LearningLab mode
    await page.goto('/');
    await page.getByTestId('mode-selector')).selectOption('learning-lab');
    await expect(page.getByTestId('current-mode')).toHaveText('LearningLab');

    // When: Student selects "Reflex Response" experiment
    await page.getByTestId('experiment-library')).click();
    await page.getByTestId('experiment-reflex-response')).click();
    await expect(page.getByTestId('experiment-title')).toHaveText('Reflex Response');

    // Start experiment
    await page.getByTestId('start-experiment')).click();
    await expect(page.getByTestId('experiment-status')).toHaveText('Running');

    // And: Trigger ultrasonic sensor (simulate hand approaching)
    await page.getByTestId('simulate-sensor')).click();
    await page.getByTestId('ultrasonic-distance')).fill('20');
    await page.getByTestId('apply-sensor')).click();

    // Then: mBot displays real-time tension graph
    await expect(page.getByTestId('tension-graph')).toBeVisible();
    const tensionValue = page.getByTestId('tension-value');
    await expect(tensionValue).not.toHaveText('0.00'); // Should have changed

    // And: Student sees reflex mode changes
    await expect(page.getByTestId('reflex-mode')).toContainText('Active');

    // Simulate closer approach
    await page.getByTestId('ultrasonic-distance')).fill('5');
    await page.getByTestId('apply-sensor')).click();

    await expect(page.getByTestId('reflex-mode')).toContainText('Spike', { timeout: 5000 });

    // Stop experiment
    await page.getByTestId('stop-experiment')).click();

    // And: Experiment data is saved
    await expect(page.getByTestId('save-status')).toHaveText('Saved');
    await expect(page.getByTestId('experiment-history')).toContainText('Reflex Response');
  });

  test('experiment data can be exported', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('mode-selector')).selectOption('learning-lab');
    await page.getByTestId('experiment-history')).click();

    // Should have at least one saved experiment
    const experimentRow = page.getByTestId('experiment-row').first();
    await experimentRow.click();

    // Export data
    const downloadPromise = page.waitForEvent('download');
    await page.getByTestId('export-csv')).click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toContain('.csv');
  });
});
