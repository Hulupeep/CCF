/**
 * E2E Journey Test: J-VOICE-MORNING-BRIEFING
 *
 * User Story: As a busy professional, I want mBot to give me a personalized
 * morning briefing when I walk into the room, so I can quickly catch up on
 * what matters most to me.
 *
 * Contract: I-VOICE-003 (Personalized Content)
 */

import { test, expect } from '@playwright/test';

test.describe('J-VOICE-MORNING-BRIEFING: Personalized Morning Briefing', () => {
  test('should deliver complete personalized morning briefing', async ({ page }) => {
    // Step 1: Enroll user voice
    await page.goto('/voice/enroll');
    await page.getByTestId('voice-enroll-btn').click();

    await page.fill('[name="user-name"]', 'Alice');

    // Record voice samples (mocked)
    for (let i = 0; i < 3; i++) {
      await page.getByTestId('voice-recorder').click();
      // Mock voice input
      await page.evaluate(() => {
        window.mockVoiceInput = { phrase: `Alice sample ${i}`, duration: 3000 };
      });
      await page.waitForTimeout(2000);
    }

    await expect(page.getByTestId('enrollment-status')).toContainText('completed');

    // Step 2: Configure preferences
    await page.goto('/voice/preferences');
    await page.check('[data-topic="technology"]');
    await page.check('[data-topic="science"]');
    await page.click('button:has-text("Save Preferences")');

    // Step 3: Connect email (mock OAuth2 flow)
    await page.goto('/voice/email');
    await page.getByTestId('connect-email-btn').click();

    // Mock OAuth2 callback
    await page.evaluate(() => {
      localStorage.setItem('email_accounts_user-1', JSON.stringify([
        {
          userId: 'user-1',
          provider: 'gmail',
          email: 'alice@example.com',
          accessToken: 'mock-token',
          refreshToken: 'mock-refresh',
          expiresAt: Date.now() + 3600000,
          lastSynced: Date.now()
        }
      ]));
    });

    // Step 4: Navigate to dashboard
    await page.goto('/voice/dashboard');
    await expect(page.getByTestId('voice-assistant-dashboard')).toBeVisible();

    // Step 5: Trigger morning briefing via voice (mocked)
    await page.evaluate(() => {
      window.mockVoiceInput = {
        text: 'Good morning',
        userId: 'user-1',
        confidence: 0.92
      };
    });

    // Step 6: Verify briefing is delivered
    await expect(page.getByTestId('morning-briefing')).toBeVisible();

    // Verify greeting section
    const briefingText = await page.getByTestId('morning-briefing').textContent();
    expect(briefingText).toContain('Good morning, Alice');

    // Verify news section
    await expect(page.getByTestId('briefing-news')).toBeVisible();
    const newsContent = await page.getByTestId('briefing-news').textContent();
    expect(newsContent).toMatch(/technology|science/i);

    // Verify email section
    await expect(page.getByTestId('briefing-email')).toBeVisible();
    const emailContent = await page.getByTestId('briefing-email').textContent();
    expect(emailContent).toContain('unread');

    // Step 7: Follow-up question
    await page.getByTestId('play-briefing-btn').click();

    // Wait for briefing to play
    await expect(page.getByTestId('playing-indicator')).toBeVisible();
    await page.waitForTimeout(2000);

    // Step 8: Ask for more details
    await page.evaluate(() => {
      window.mockVoiceInput = {
        text: 'Tell me more about the first story',
        userId: 'user-1',
        confidence: 0.95
      };
    });

    // Verify news detail is shown
    await expect(page.locator('[data-testid="news-detail"]')).toBeVisible();

    // Step 9: Verify preference learning
    // After asking for "more about tech story", tech preference should increase
    await page.goto('/voice/preferences');
    const techWeight = await page.evaluate(() => {
      const prefs = JSON.parse(localStorage.getItem('mbot_news_preferences') || '{}');
      return prefs.topicWeights?.technology || 0;
    });

    expect(techWeight).toBeGreaterThan(0);
  });

  test('should handle anonymous mode gracefully', async ({ page }) => {
    await page.goto('/voice/dashboard');

    // Trigger voice without enrollment
    await page.evaluate(() => {
      window.mockVoiceInput = {
        text: 'Hello',
        userId: null,
        confidence: 0.45,
        isAnonymous: true
      };
    });

    // Should show anonymous mode indicator
    await expect(page.getByTestId('anonymous-mode')).toBeVisible();

    // Should NOT show personal data
    await expect(page.getByTestId('briefing-email')).not.toBeVisible();
  });

  test('should play briefing with TTS', async ({ page }) => {
    // Setup: user enrolled and configured
    await page.goto('/voice/dashboard');

    // Generate briefing
    await page.getByTestId('generate-briefing-btn').click();
    await expect(page.getByTestId('morning-briefing')).toBeVisible();

    // Play briefing
    await page.getByTestId('play-briefing-btn').click();

    // Verify playing state
    await expect(page.getByTestId('play-briefing-btn')).toContainText('Playing');
    await expect(page.getByTestId('stop-briefing-btn')).toBeVisible();

    // Verify sections are highlighted as they play
    const sections = page.locator('[data-testid^="briefing-"]');
    const firstSection = sections.first();
    await expect(firstSection).toHaveClass(/active/);

    // Stop briefing
    await page.getByTestId('stop-briefing-btn').click();
    await expect(page.getByTestId('play-briefing-btn')).not.toContainText('Playing');
  });

  test('should respect privacy settings', async ({ page }) => {
    // Setup: user enrolled
    await page.goto('/voice/privacy');
    await expect(page.getByTestId('voice-privacy-settings')).toBeVisible();

    // Disable email access
    await page.uncheck('input[type="checkbox"]:has-text("Allow Email Access")');
    await page.click('button:has-text("Save Settings")');

    // Go to dashboard
    await page.goto('/voice/dashboard');
    await page.getByTestId('generate-briefing-btn').click();

    // Email section should NOT be present
    const briefing = page.getByTestId('morning-briefing');
    await expect(briefing).toBeVisible();
    await expect(page.getByTestId('briefing-email')).not.toBeVisible();
  });

  test('should show correct duration estimate', async ({ page }) => {
    await page.goto('/voice/dashboard');
    await page.getByTestId('generate-briefing-btn').click();

    // Verify duration is displayed
    const durationText = await page.locator('.briefing-duration').textContent();
    expect(durationText).toMatch(/\d+ seconds/);
  });

  test('should regenerate briefing on demand', async ({ page }) => {
    await page.goto('/voice/dashboard');

    // Generate first briefing
    await page.getByTestId('generate-briefing-btn').click();
    const firstBriefing = await page.getByTestId('morning-briefing').textContent();

    // Mock news update
    await page.evaluate(() => {
      localStorage.setItem('mock_news_articles', JSON.stringify([
        {
          id: 'new-1',
          headline: 'Updated News Headline',
          summary: 'New content',
          source: 'Reuters',
          category: 'technology',
          publishedAt: Date.now(),
          url: 'https://example.com/new',
          relevanceScore: 0.92
        }
      ]));
    });

    // Regenerate
    await page.getByTestId('generate-briefing-btn').click();
    await page.waitForTimeout(500);

    const secondBriefing = await page.getByTestId('morning-briefing').textContent();

    // Briefings should differ (due to new news)
    expect(secondBriefing).toContain('Updated News Headline');
  });
});
