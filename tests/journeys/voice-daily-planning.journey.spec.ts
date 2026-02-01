/**
 * E2E Journey Test: J-VOICE-DAILY-PLANNING
 *
 * User Story: As a parent, I want mBot to engage my child in planning their day
 * and remember what they planned, so my child develops planning skills and
 * feels accountable.
 *
 * Contract: I-MEMORY-001 (Activity Tracking)
 */

import { test, expect } from '@playwright/test';

test.describe('J-VOICE-DAILY-PLANNING: Kid Daily Planning with Memory', () => {
  test('should remember yesterdays plan and ask about it today', async ({ page }) => {
    // Step 1: Setup kid profile
    await page.goto('/voice/enroll');
    await page.getByTestId('voice-enroll-btn').click();

    await page.fill('[name="user-name"]', 'Emma');
    await page.fill('[name="user-age"]', '8');

    // Record voice samples
    for (let i = 0; i < 3; i++) {
      await page.getByTestId('voice-recorder').click();
      await page.waitForTimeout(2000);
    }

    await expect(page.getByTestId('enrollment-status')).toContainText('completed');

    // Step 2: Store yesterday's activity
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    const yesterdayStr = yesterday.toISOString().split('T')[0];

    await page.evaluate(({ date }) => {
      localStorage.setItem(`daily_activity_Emma_${date}`, JSON.stringify({
        date,
        userId: 'Emma',
        plannedActivities: ['build LEGO castle'],
        completedActivities: [],
        notes: ''
      }));
    }, { date: yesterdayStr });

    // Step 3: Today's greeting
    await page.goto('/voice/dashboard');

    await page.evaluate(() => {
      window.mockVoiceInput = {
        text: 'Hi mBot',
        userId: 'Emma',
        confidence: 0.94
      };
    });

    // Step 4: Verify memory recall
    await page.getByTestId('generate-briefing-btn').click();

    const memorySection = page.getByTestId('briefing-memory');
    await expect(memorySection).toBeVisible();

    const memoryContent = await memorySection.textContent();
    expect(memoryContent).toContain('LEGO castle');
    expect(memoryContent).toMatch(/did you|how did it go/i);

    // Step 5: Respond with completion
    await page.evaluate(() => {
      window.mockVoiceInput = {
        text: 'I finished it! Today I want to draw',
        userId: 'Emma',
        confidence: 0.96
      };
    });

    // Step 6: Verify today's plan is stored
    await page.waitForTimeout(500);

    const todayStr = new Date().toISOString().split('T')[0];
    const todayActivity = await page.evaluate(({ date }) => {
      const stored = localStorage.getItem(`daily_activity_Emma_${date}`);
      return stored ? JSON.parse(stored) : null;
    }, { date: todayStr });

    expect(todayActivity).toBeTruthy();
    expect(todayActivity.plannedActivities).toContain('draw');

    // Step 7: Verify bot celebrates and offers help
    const response = await page.locator('[data-testid="bot-response"]').textContent();
    expect(response).toMatch(/awesome|great/i);
    expect(response).toMatch(/help|ideas/i);
  });

  test('should display activity timeline', async ({ page }) => {
    await page.goto('/voice/dashboard');
    await page.click('[data-testid="tab-memory"]');

    // Verify timeline is visible
    await expect(page.locator('.memory-timeline')).toBeVisible();

    // Should show activities for past 7 days
    const timelineItems = page.locator('[data-testid^="activity-timeline-"]');
    const count = await timelineItems.count();
    expect(count).toBeGreaterThan(0);

    // Check yesterday's entry
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    const yesterdayStr = yesterday.toISOString().split('T')[0];

    const yesterdayItem = page.getByTestId(`activity-timeline-${yesterdayStr}`);
    await expect(yesterdayItem).toBeVisible();

    const yesterdayContent = await yesterdayItem.textContent();
    expect(yesterdayContent).toContain('LEGO castle');
  });

  test('should handle multiple activities in one day', async ({ page }) => {
    const todayStr = new Date().toISOString().split('T')[0];

    // Store multiple activities
    await page.evaluate(({ date }) => {
      localStorage.setItem(`daily_activity_Emma_${date}`, JSON.stringify({
        date,
        userId: 'Emma',
        plannedActivities: ['draw', 'play outside', 'read a book'],
        completedActivities: ['draw'],
        notes: 'Drew a picture of a cat',
        mood: 'happy'
      }));
    }, { date: todayStr });

    await page.goto('/voice/dashboard');
    await page.click('[data-testid="tab-memory"]');

    const todayItem = page.getByTestId(`activity-timeline-${todayStr}`);
    await expect(todayItem).toBeVisible();

    const content = await todayItem.textContent();
    expect(content).toContain('draw');
    expect(content).toContain('play outside');
    expect(content).toContain('read a book');
    expect(content).toContain('✅'); // Checkmark for completed
    expect(content).toContain('happy');
  });

  test('should format dates correctly (today, yesterday, dates)', async ({ page }) => {
    await page.goto('/voice/dashboard');
    await page.click('[data-testid="tab-memory"]');

    // Today should say "Today"
    const todayStr = new Date().toISOString().split('T')[0];
    const todayItem = page.getByTestId(`activity-timeline-${todayStr}`);
    const todayLabel = await todayItem.locator('.date-label').textContent();
    expect(todayLabel).toBe('Today');

    // Yesterday should say "Yesterday"
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    const yesterdayStr = yesterday.toISOString().split('T')[0];
    const yesterdayItem = page.getByTestId(`activity-timeline-${yesterdayStr}`);
    const yesterdayLabel = await yesterdayItem.locator('.date-label').textContent();
    expect(yesterdayLabel).toBe('Yesterday');

    // Older dates should show weekday + date
    const threeDaysAgo = new Date();
    threeDaysAgo.setDate(threeDaysAgo.getDate() - 3);
    const threeDaysAgoStr = threeDaysAgo.toISOString().split('T')[0];
    const oldItem = page.getByTestId(`activity-timeline-${threeDaysAgoStr}`);
    const oldLabel = await oldItem.locator('.date-label').textContent();
    expect(oldLabel).toMatch(/Mon|Tue|Wed|Thu|Fri|Sat|Sun/);
  });

  test('should track mood over time', async ({ page }) => {
    // Store activities with moods
    for (let i = 0; i < 3; i++) {
      const date = new Date();
      date.setDate(date.getDate() - i);
      const dateStr = date.toISOString().split('T')[0];

      await page.evaluate(({ date, mood }) => {
        localStorage.setItem(`daily_activity_Emma_${date}`, JSON.stringify({
          date,
          userId: 'Emma',
          plannedActivities: ['activity'],
          completedActivities: [],
          notes: '',
          mood
        }));
      }, { date: dateStr, mood: i === 0 ? 'happy' : i === 1 ? 'neutral' : 'excited' });
    }

    await page.goto('/voice/dashboard');
    await page.click('[data-testid="tab-memory"]');

    // Verify moods are displayed
    const moodIndicators = page.locator('.mood-indicator');
    const count = await moodIndicators.count();
    expect(count).toBeGreaterThanOrEqual(3);
  });

  test('should ask proactive follow-up questions', async ({ page }) => {
    // Setup: Yesterday Emma planned to "practice piano"
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    const yesterdayStr = yesterday.toISOString().split('T')[0];

    await page.evaluate(({ date }) => {
      localStorage.setItem(`daily_activity_Emma_${date}`, JSON.stringify({
        date,
        userId: 'Emma',
        plannedActivities: ['practice piano'],
        completedActivities: [],
        notes: ''
      }));

      // Store follow-up question
      localStorage.setItem('followup_questions_Emma', JSON.stringify([
        {
          id: 'q-1',
          userId: 'Emma',
          question: 'Did you practice piano yesterday?',
          context: 'Emma planned to practice piano',
          priority: 85,
          validUntil: Date.now() + 86400000,
          answered: false
        }
      ]));
    }, { date: yesterdayStr });

    await page.goto('/voice/dashboard');
    await page.getByTestId('generate-briefing-btn').click();

    // Question section should be visible
    const questionSection = page.getByTestId('briefing-question');
    await expect(questionSection).toBeVisible();

    const questionContent = await questionSection.textContent();
    expect(questionContent).toContain('practice piano');
  });

  test('should handle empty activity history', async ({ page }) => {
    // Clear all activities
    await page.evaluate(() => {
      const keys = Object.keys(localStorage).filter(k => k.startsWith('daily_activity_'));
      keys.forEach(k => localStorage.removeItem(k));
    });

    await page.goto('/voice/dashboard');
    await page.click('[data-testid="tab-memory"]');

    // Should show empty state
    await expect(page.locator('.empty-state')).toBeVisible();
    const emptyText = await page.locator('.empty-state').textContent();
    expect(emptyText).toContain('No activity history');
  });
});
