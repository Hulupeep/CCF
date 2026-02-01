/**
 * E2E Journey Test: Browse and Download Marketplace Personality
 * Issue #85 - J-CLOUD-MARKETPLACE
 *
 * Tests:
 * - I-CLOUD-005: One rating per user
 * - I-CLOUD-006: Search results within 500ms
 * - Browse, search, filter, download, rate
 */

import { test, expect } from '@playwright/test';

test.describe('Journey: Browse and Download Marketplace Personality', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to dashboard
    await page.goto('http://localhost:3000');

    // Wait for marketplace to load
    await page.getByTestId('marketplace-tab').click();
    await expect(page.getByTestId('marketplace-browser')).toBeVisible();
  });

  test('Scenario: Download Personality', async ({ page }) => {
    // When I browse marketplace
    await expect(page.getByText('Trending Personalities')).toBeVisible();

    // And I search for "creative"
    await page.getByTestId('marketplace-search').fill('creative');

    // Then matching personalities appear within 500ms (I-CLOUD-006)
    const startTime = Date.now();
    await expect(page.getByTestId('personality-card-*').first()).toBeVisible({ timeout: 500 });
    const elapsedMs = Date.now() - startTime;
    expect(elapsedMs).toBeLessThan(500);

    // When I click "Download" on "CoolBot"
    const firstCard = page.getByTestId('personality-card-*').first();
    await firstCard.getByTestId('download-personality-*').click();

    // Then "CoolBot" downloads to my device
    // And "CoolBot" added to my presets
    await expect(page.getByText(/Downloaded.*successfully/i)).toBeVisible();

    // And download count increments (verify in UI update)
    await expect(firstCard.getByText(/📥 \d+/)).toBeVisible();
  });

  test('Scenario: Rate Personality (I-CLOUD-005)', async ({ page }) => {
    // Given I downloaded "CoolBot"
    const firstCard = page.getByTestId('personality-card-*').first();
    await firstCard.getByTestId('download-personality-*').click();
    await expect(page.getByText(/Downloaded.*successfully/i)).toBeVisible();

    // Close alert
    await page.on('dialog', (dialog) => dialog.accept());

    // When I rate it 5 stars
    await firstCard.getByTestId('rate-star-*-5').click();

    // Then rating saves to database
    await expect(page.getByText(/Rating submitted successfully/i)).toBeVisible();

    // And average rating updates (would need to reload or have real-time update)
    // And I cannot rate again (second click should fail)
    await page.on('dialog', (dialog) => dialog.accept());
    await firstCard.getByTestId('rate-star-*-4').click();

    // Then error message appears
    await expect(page.getByText(/Already rated/i)).toBeVisible();
  });

  test('Scenario: Search and Filter', async ({ page }) => {
    // When I search marketplace for "energetic"
    await page.getByTestId('marketplace-search').fill('energetic');

    // And I filter by "4+ stars"
    await page.getByTestId('marketplace-rating-filter').selectOption({ label: '4+ stars' });

    // And I sort by "Most Downloaded"
    await page.getByTestId('marketplace-sort').selectOption({ value: 'downloads' });

    // Then results match criteria
    // And results appear within 500ms (I-CLOUD-006)
    const startTime = Date.now();
    await expect(page.getByTestId('personality-card-*').first()).toBeVisible({ timeout: 500 });
    const elapsedMs = Date.now() - startTime;
    expect(elapsedMs).toBeLessThan(500);

    // Verify results contain "energetic" in name or description
    const firstCardText = await page.getByTestId('personality-card-*').first().textContent();
    expect(firstCardText?.toLowerCase()).toMatch(/energetic/);
  });

  test('Scenario: Filter by Tags', async ({ page }) => {
    // When I click on "playful" tag
    await page.getByTestId('tag-playful').click();

    // Then only personalities with "playful" tag appear
    await expect(page.getByTestId('personality-card-*').first()).toBeVisible();

    // Verify tag is active
    await expect(page.getByTestId('tag-playful')).toHaveClass(/active/);

    // When I click "creative" tag (multiple tags)
    await page.getByTestId('tag-creative').click();

    // Then personalities with either tag appear
    const cards = page.getByTestId('personality-card-*');
    await expect(cards.first()).toBeVisible();
  });

  test('Scenario: Preview Before Download', async ({ page }) => {
    // When I browse marketplace
    const firstCard = page.getByTestId('personality-card-*').first();

    // And I click "Preview"
    await firstCard.getByTestId('preview-personality-*').click();

    // Then preview modal opens
    await expect(page.getByTestId('preview-modal-*')).toBeVisible();

    // And I see all 9 personality parameters
    await expect(page.getByText(/tension_baseline/i)).toBeVisible();
    await expect(page.getByText(/coherence_baseline/i)).toBeVisible();
    await expect(page.getByText(/energy_baseline/i)).toBeVisible();
    await expect(page.getByText(/startle_sensitivity/i)).toBeVisible();
    await expect(page.getByText(/recovery_speed/i)).toBeVisible();
    await expect(page.getByText(/curiosity_drive/i)).toBeVisible();
    await expect(page.getByText(/movement_expressiveness/i)).toBeVisible();
    await expect(page.getByText(/sound_expressiveness/i)).toBeVisible();
    await expect(page.getByText(/light_expressiveness/i)).toBeVisible();

    // When I click "Download" in preview
    await page.getByText('Download').click();

    // Then personality downloads
    await expect(page.getByText(/Downloaded.*successfully/i)).toBeVisible();
  });

  test('Scenario: Report Personality', async ({ page }) => {
    // Given I see an inappropriate personality
    const firstCard = page.getByTestId('personality-card-*').first();

    // When I click "Report"
    await firstCard.getByTestId('report-personality-*').click();

    // Then report dialog opens
    await expect(page.getByTestId('report-dialog')).toBeVisible();

    // And I enter reason
    await page.getByTestId('report-reason').fill(
      'This personality contains inappropriate content and violates community guidelines.'
    );

    // And I click "Submit Report"
    await page.getByTestId('report-submit').click();

    // Then report is saved
    await expect(page.getByText(/Report submitted successfully/i)).toBeVisible();

    // And thank you message appears
    await expect(page.getByText(/Thank you/i)).toBeVisible();
  });

  test('Scenario: Pagination', async ({ page }) => {
    // Given there are more than 20 personalities
    // When I scroll to bottom
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));

    // Then "Load More" button appears
    await expect(page.getByTestId('load-more')).toBeVisible();

    // When I click "Load More"
    await page.getByTestId('load-more').click();

    // Then next page loads
    // And more personalities appear
    const cardCount = await page.getByTestId('personality-card-*').count();
    expect(cardCount).toBeGreaterThan(20);
  });

  test('Scenario: View Trending', async ({ page }) => {
    // When I open marketplace
    // Then "Trending Personalities" section appears
    await expect(page.getByText('Trending Personalities')).toBeVisible();

    // And up to 10 personalities shown
    const trendingCards = page.getByTestId('personality-card-*');
    const count = await trendingCards.count();
    expect(count).toBeLessThanOrEqual(10);

    // Verify sorted by popularity (downloads + rating)
    // (Visual check - in real test would verify API response)
  });
});
