/**
 * E2E Journey Test: Publish Personality to Marketplace
 * Issue #85 - J-CLOUD-MARKETPLACE
 *
 * Tests:
 * - I-CLOUD-004: Validation before publication
 * - User can publish custom personality
 * - Validation errors displayed if invalid
 */

import { test, expect } from '@playwright/test';

test.describe('Journey: Publish Personality to Marketplace', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to dashboard
    await page.goto('http://localhost:3000');

    // Wait for PersonalityMixer to load
    await expect(page.getByTestId('personality-mixer')).toBeVisible();
  });

  test('Scenario: Publish Valid Personality', async ({ page }) => {
    // Given I created "SuperBot" personality
    await page.getByTestId('slider-energy-baseline').fill('0.9');
    await page.getByTestId('slider-curiosity-drive').fill('0.8');
    await page.getByTestId('slider-movement-expressiveness').fill('0.7');

    // When I click "Publish Current"
    await page.getByTestId('publish-personality-btn').click();

    // Then publish dialog opens
    await expect(page.getByTestId('publish-dialog')).toBeVisible();

    // And I fill in metadata
    await page.getByTestId('publish-description').fill(
      'SuperBot\n\nAn energetic and curious personality perfect for exploration!'
    );
    await page.getByTestId('publish-tags').fill('energetic, curious, playful');

    // And I click "Publish"
    await page.getByTestId('publish-confirm').click();

    // Then validation runs
    // And validation passes
    // And personality uploads to public gallery
    await expect(page.getByText('Personality published successfully!')).toBeVisible();

    // And appears in search results within 10 seconds
    await page.getByTestId('marketplace-tab').click();
    await page.getByTestId('marketplace-search').fill('SuperBot');

    await expect(page.getByTestId('personality-card-*')).toBeVisible({ timeout: 10000 });

    // And shows "Published" status
    await page.getByTestId('my-published-tab').click();
    await expect(page.getByText('SuperBot')).toBeVisible();
  });

  test('Scenario: Validation Fails', async ({ page }) => {
    // Given I have a personality configured
    await page.getByTestId('slider-energy-baseline').fill('0.5');

    // When I click "Publish Current"
    await page.getByTestId('publish-personality-btn').click();

    // And I fill in invalid metadata (description too short)
    await page.getByTestId('publish-description').fill('Bad\n\nShort');
    await page.getByTestId('publish-tags').fill('');

    // And I click "Publish"
    await page.getByTestId('publish-confirm').click();

    // Then validation errors appear
    // (In reality, client-side validation would prevent submission,
    // but server-side would also reject)
    await expect(page.getByText(/validation/i)).toBeVisible();
  });

  test('Scenario: View Published in My Published Tab', async ({ page }) => {
    // Given I published a personality
    await page.getByTestId('publish-personality-btn').click();
    await page.getByTestId('publish-description').fill(
      'Test Bot\n\nA test personality for E2E testing'
    );
    await page.getByTestId('publish-tags').fill('test');
    await page.getByTestId('publish-confirm').click();

    // Wait for success
    await expect(page.getByText('Personality published successfully!')).toBeVisible();

    // When I navigate to "My Published"
    await page.getByTestId('my-published-tab').click();

    // Then my personality appears
    await expect(page.getByText('Test Bot')).toBeVisible();

    // And metrics are visible
    await expect(page.getByText('⭐ 0.00 (0)')).toBeVisible(); // No ratings yet
    await expect(page.getByText('📥 0')).toBeVisible(); // No downloads yet
  });

  test('Scenario: Unpublish Personality', async ({ page }) => {
    // Given I published a personality
    await page.getByTestId('publish-personality-btn').click();
    await page.getByTestId('publish-description').fill(
      'Temp Bot\n\nA temporary personality'
    );
    await page.getByTestId('publish-tags').fill('temp');
    await page.getByTestId('publish-confirm').click();
    await expect(page.getByText('Personality published successfully!')).toBeVisible();

    // When I go to My Published
    await page.getByTestId('my-published-tab').click();

    // And I click "Unpublish"
    await page.getByTestId('unpublish-personality-*').first().click();

    // Then confirmation dialog appears
    await page.on('dialog', (dialog) => dialog.accept());

    // And personality is removed
    await expect(page.getByText('Personality unpublished successfully')).toBeVisible();
    await expect(page.getByText('Temp Bot')).not.toBeVisible();
  });
});
