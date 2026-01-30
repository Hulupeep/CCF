/**
 * Journey Test: J-PERS-MEET-PERSONALITY
 * Issue: #29 - STORY-PERS-007: Meet Personality Journey Test
 *
 * Scenario: User experiences personality switch
 *   Given mBot is in Calm personality
 *   When user triggers a stressful situation
 *   Then mBot switches to Protect mode
 *   And displays protective behaviors
 *   When situation calms down
 *   Then mBot returns to Calm mode
 */

import { test, expect } from '@playwright/test';

test.describe('J-PERS-MEET-PERSONALITY: Meet Personality Journey', () => {
  test('user observes personality switch from calm to protective', async ({ page }) => {
    // Given: mBot is in Calm personality
    await page.goto('/');
    await expect(page.getByTestId('personality-mode')).toHaveText('Calm');
    await expect(page.getByTestId('tension-level')).toContainText('Low');

    // When: User triggers a stressful situation (simulate close obstacle)
    await page.getByTestId('simulate-obstacle')).click();
    await page.getByTestId('obstacle-distance')).fill('5'); // 5cm - very close
    await page.getByTestId('apply-simulation')).click();

    // Then: mBot switches to Protect mode
    await expect(page.getByTestId('personality-mode')).toHaveText('Protect', { timeout: 10000 });
    await expect(page.getByTestId('tension-level')).toContainText('High');

    // And: Displays protective behaviors
    await expect(page.getByTestId('motor-left')).toContainText('-'); // Backing up
    await expect(page.getByTestId('motor-right')).toContainText('-'); // Backing up
    await expect(page.getByTestId('led-color')).toContainText('Red'); // Warning color

    // When: Situation calms down
    await page.getByTestId('obstacle-distance')).fill('100'); // Move obstacle far away
    await page.getByTestId('apply-simulation')).click();

    // Then: mBot returns to Calm mode
    await expect(page.getByTestId('personality-mode')).toHaveText('Calm', { timeout: 15000 });
    await expect(page.getByTestId('tension-level')).toContainText('Low');
  });

  test('personality persists across sessions', async ({ page }) => {
    // Set personality to Active
    await page.goto('/');
    await page.getByTestId('personality-selector')).selectOption('Active');
    await expect(page.getByTestId('personality-mode')).toHaveText('Active');

    // Refresh page
    await page.reload();

    // Should remember Active personality
    await expect(page.getByTestId('personality-mode')).toHaveText('Active');
  });
});
