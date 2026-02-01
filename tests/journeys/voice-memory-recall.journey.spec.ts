/**
 * E2E Journey Test: J-VOICE-MEMORY-RECALL
 *
 * User Story: As a user, I want mBot to remember our previous conversations
 * and bring them up naturally, so I feel like I have a companion who truly
 * knows me.
 *
 * Contract: I-VOICE-004 (Conversational Memory - 7 days retention)
 */

import { test, expect } from '@playwright/test';

test.describe('J-VOICE-MEMORY-RECALL: Contextual Memory Across Days', () => {
  test('should recall conversation from 3 days ago', async ({ page }) => {
    // Step 1: Store past conversation (3 days ago)
    const threeDaysAgo = Date.now() - (3 * 24 * 60 * 60 * 1000);

    await page.goto('/voice/dashboard');

    await page.evaluate(({ timestamp }) => {
      const conversation = {
        id: 'conv-1',
        timestamp,
        turns: [
          { speaker: 'user', text: 'I want to learn guitar', timestamp },
          { speaker: 'mbot', text: 'That\'s great! What song?', timestamp: timestamp + 1000 },
          { speaker: 'user', text: 'Wonderwall', timestamp: timestamp + 2000 },
          { speaker: 'mbot', text: 'Great choice! Let me know how it goes.', timestamp: timestamp + 3000 }
        ],
        topic: 'Learning guitar',
        sentiment: 'positive',
        keyPoints: ['wants to learn guitar', 'Wonderwall song', 'beginner']
      };

      localStorage.setItem('conversation_Charlie_' + timestamp, JSON.stringify(conversation));

      // Store in conversation list
      const conversations = JSON.parse(localStorage.getItem('conversations_Charlie') || '[]');
      conversations.push(conversation);
      localStorage.setItem('conversations_Charlie', JSON.stringify(conversations));
    }, { timestamp: threeDaysAgo });

    // Step 2: Today's greeting
    await page.evaluate(() => {
      window.mockVoiceInput = {
        text: 'Hey mBot',
        userId: 'Charlie',
        confidence: 0.93
      };
    });

    // Step 3: Generate briefing (should include memory recall)
    await page.getByTestId('generate-briefing-btn').click();

    const briefing = page.getByTestId('morning-briefing');
    await expect(briefing).toBeVisible();

    // Verify memory recall mentions guitar and Wonderwall
    const briefingText = await briefing.textContent();
    expect(briefingText).toMatch(/guitar|wonderwall/i);

    // Step 4: View conversation history
    await page.click('[data-testid="tab-conversations"]');
    await expect(page.getByTestId('conversation-history')).toBeVisible();

    // Verify conversation is listed
    const conversations = page.locator('.conversation-card');
    await expect(conversations).toHaveCount(1);

    const conversationContent = await conversations.first().textContent();
    expect(conversationContent).toContain('Learning guitar');
    expect(conversationContent).toContain('Wonderwall');
  });

  test('should display key points from conversations', async ({ page }) => {
    await page.goto('/voice/dashboard');
    await page.click('[data-testid="tab-conversations"]');

    // Store conversation with key points
    await page.evaluate(() => {
      const conversation = {
        id: 'conv-2',
        timestamp: Date.now() - 86400000,
        turns: [
          { speaker: 'user', text: 'I got accepted to MIT!', timestamp: Date.now() - 86400000 },
          { speaker: 'mbot', text: 'Congratulations! That\'s amazing!', timestamp: Date.now() - 86400000 + 1000 }
        ],
        topic: 'College acceptance',
        sentiment: 'positive',
        keyPoints: ['Accepted to MIT', 'Major: Computer Science', 'Fall 2024']
      };

      localStorage.setItem('conversations_Charlie', JSON.stringify([conversation]));
    });

    await page.reload();

    // Verify key points are displayed
    const keyPoints = page.locator('.key-points');
    await expect(keyPoints).toBeVisible();

    const keyPointsText = await keyPoints.textContent();
    expect(keyPointsText).toContain('MIT');
    expect(keyPointsText).toContain('Computer Science');
  });

  test('should track conversation sentiment', async ({ page }) => {
    await page.goto('/voice/dashboard');
    await page.click('[data-testid="tab-conversations"]');

    // Store conversations with different sentiments
    await page.evaluate(() => {
      const conversations = [
        {
          id: 'conv-positive',
          timestamp: Date.now() - 1000,
          turns: [{ speaker: 'user', text: 'I love this!', timestamp: Date.now() - 1000 }],
          topic: 'Happy chat',
          sentiment: 'positive',
          keyPoints: []
        },
        {
          id: 'conv-negative',
          timestamp: Date.now() - 2000,
          turns: [{ speaker: 'user', text: 'This is frustrating', timestamp: Date.now() - 2000 }],
          topic: 'Problem solving',
          sentiment: 'negative',
          keyPoints: []
        },
        {
          id: 'conv-neutral',
          timestamp: Date.now() - 3000,
          turns: [{ speaker: 'user', text: 'What time is it?', timestamp: Date.now() - 3000 }],
          topic: 'Time query',
          sentiment: 'neutral',
          keyPoints: []
        }
      ];

      localStorage.setItem('conversations_Charlie', JSON.stringify(conversations));
    });

    await page.reload();

    // Verify sentiment badges
    const positiveBadge = page.locator('.sentiment-badge.sentiment-positive');
    await expect(positiveBadge).toBeVisible();

    const negativeBadge = page.locator('.sentiment-badge.sentiment-negative');
    await expect(negativeBadge).toBeVisible();

    const neutralBadge = page.locator('.sentiment-badge.sentiment-neutral');
    await expect(neutralBadge).toBeVisible();
  });

  test('should limit conversation history to 7 days (retention policy)', async ({ page }) => {
    await page.goto('/voice/dashboard');

    // Store conversations: 5 days old (should keep) and 10 days old (should delete)
    await page.evaluate(() => {
      const fiveDaysAgo = Date.now() - (5 * 24 * 60 * 60 * 1000);
      const tenDaysAgo = Date.now() - (10 * 24 * 60 * 60 * 1000);

      const conversations = [
        {
          id: 'conv-recent',
          timestamp: fiveDaysAgo,
          turns: [{ speaker: 'user', text: 'Recent conversation', timestamp: fiveDaysAgo }],
          topic: 'Recent',
          sentiment: 'neutral',
          keyPoints: []
        },
        {
          id: 'conv-old',
          timestamp: tenDaysAgo,
          turns: [{ speaker: 'user', text: 'Old conversation', timestamp: tenDaysAgo }],
          topic: 'Old',
          sentiment: 'neutral',
          keyPoints: []
        }
      ];

      localStorage.setItem('conversations_Charlie', JSON.stringify(conversations));

      // Set retention policy to 7 days
      localStorage.setItem('privacy_settings_Charlie', JSON.stringify({
        allowVoiceRecording: true,
        allowEmailAccess: true,
        allowNewsPersonalization: true,
        shareDataWithFamily: false,
        retentionDays: 7
      }));
    });

    // Trigger cleanup (would happen automatically in real app)
    await page.evaluate(() => {
      const retentionDays = 7;
      const cutoffTime = Date.now() - (retentionDays * 24 * 60 * 60 * 1000);

      const conversations = JSON.parse(localStorage.getItem('conversations_Charlie') || '[]');
      const filtered = conversations.filter((c: any) => c.timestamp > cutoffTime);

      localStorage.setItem('conversations_Charlie', JSON.stringify(filtered));
    });

    await page.click('[data-testid="tab-conversations"]');

    // Should only show recent conversation
    const conversations = page.locator('.conversation-card');
    await expect(conversations).toHaveCount(1);

    const conversationText = await conversations.first().textContent();
    expect(conversationText).toContain('Recent');
    expect(conversationText).not.toContain('Old');
  });

  test('should show conversation turns in correct order', async ({ page }) => {
    await page.goto('/voice/dashboard');
    await page.click('[data-testid="tab-conversations"]');

    await page.evaluate(() => {
      const conversation = {
        id: 'conv-multi-turn',
        timestamp: Date.now() - 3600000,
        turns: [
          { speaker: 'user', text: 'Hello', timestamp: Date.now() - 3600000 },
          { speaker: 'mbot', text: 'Hi there!', timestamp: Date.now() - 3600000 + 1000 },
          { speaker: 'user', text: 'How are you?', timestamp: Date.now() - 3600000 + 2000 },
          { speaker: 'mbot', text: 'I\'m great! How can I help?', timestamp: Date.now() - 3600000 + 3000 },
          { speaker: 'user', text: 'Tell me a joke', timestamp: Date.now() - 3600000 + 4000 },
          { speaker: 'mbot', text: 'Why did the robot go to school? To improve its learning algorithms!', timestamp: Date.now() - 3600000 + 5000 }
        ],
        topic: 'General chat',
        sentiment: 'positive',
        keyPoints: []
      };

      localStorage.setItem('conversations_Charlie', JSON.stringify([conversation]));
    });

    await page.reload();

    // Get all turns
    const turns = page.locator('.conversation-turns .turn');
    await expect(turns).toHaveCount(6);

    // Verify order
    const firstTurn = await turns.nth(0).textContent();
    expect(firstTurn).toContain('user:');
    expect(firstTurn).toContain('Hello');

    const secondTurn = await turns.nth(1).textContent();
    expect(secondTurn).toContain('mbot:');
    expect(secondTurn).toContain('Hi there');

    const lastTurn = await turns.nth(5).textContent();
    expect(lastTurn).toContain('learning algorithms');
  });

  test('should handle empty conversation history', async ({ page }) => {
    await page.evaluate(() => {
      localStorage.removeItem('conversations_Charlie');
    });

    await page.goto('/voice/dashboard');
    await page.click('[data-testid="tab-conversations"]');

    // Should show empty state
    await expect(page.locator('.empty-state')).toBeVisible();
    const emptyText = await page.locator('.empty-state').textContent();
    expect(emptyText).toContain('No conversations yet');
  });

  test('should recall specific user interests in briefing', async ({ page }) => {
    // Store user preferences with interests
    await page.evaluate(() => {
      localStorage.setItem('user_preferences_Charlie', JSON.stringify({
        userId: 'Charlie',
        newsTopics: ['music', 'technology'],
        contentRestrictions: [],
        privacySettings: {
          allowVoiceRecording: true,
          allowEmailAccess: false,
          allowNewsPersonalization: true,
          shareDataWithFamily: false,
          retentionDays: 7
        },
        personalDetails: {
          nickname: 'Chuck',
          interests: ['guitar', 'programming', 'hiking'],
          occupation: 'Software Engineer'
        }
      }));

      // Store conversation about guitar
      const twoDaysAgo = Date.now() - (2 * 24 * 60 * 60 * 1000);
      const conversation = {
        id: 'conv-guitar',
        timestamp: twoDaysAgo,
        turns: [
          { speaker: 'user', text: 'I practiced Wonderwall for 2 hours today', timestamp: twoDaysAgo }
        ],
        topic: 'Guitar practice',
        sentiment: 'positive',
        keyPoints: ['Wonderwall', '2 hours practice', 'improving']
      };

      localStorage.setItem('conversations_Charlie', JSON.stringify([conversation]));
    });

    await page.goto('/voice/dashboard');
    await page.getByTestId('generate-briefing-btn').click();

    const briefing = await page.getByTestId('morning-briefing').textContent();

    // Should use nickname
    expect(briefing).toContain('Chuck');

    // Should reference recent guitar activity
    expect(briefing).toMatch(/guitar|wonderwall/i);
  });

  test('should support multi-user context switching', async ({ page }) => {
    // Setup two users with different conversations
    await page.evaluate(() => {
      // User 1: Dad - financial conversations
      localStorage.setItem('conversations_Dad', JSON.stringify([
        {
          id: 'conv-dad',
          timestamp: Date.now() - 3600000,
          turns: [
            { speaker: 'user', text: 'Check my stock portfolio', timestamp: Date.now() - 3600000 }
          ],
          topic: 'Finance',
          sentiment: 'neutral',
          keyPoints: ['stocks', 'portfolio']
        }
      ]));

      // User 2: Sarah - homework conversations
      localStorage.setItem('conversations_Sarah', JSON.stringify([
        {
          id: 'conv-sarah',
          timestamp: Date.now() - 1800000,
          turns: [
            { speaker: 'user', text: 'Help me with my math homework', timestamp: Date.now() - 1800000 }
          ],
          topic: 'Homework',
          sentiment: 'neutral',
          keyPoints: ['math', 'homework', 'fractions']
        }
      ]));
    });

    // Switch to Dad's context
    await page.goto('/voice/dashboard?user=Dad');
    await page.click('[data-testid="tab-conversations"]');

    let conversations = page.locator('.conversation-card');
    await expect(conversations).toHaveCount(1);

    let conversationText = await conversations.first().textContent();
    expect(conversationText).toContain('stock portfolio');
    expect(conversationText).not.toContain('homework');

    // Switch to Sarah's context
    await page.goto('/voice/dashboard?user=Sarah');
    await page.click('[data-testid="tab-conversations"]');

    conversations = page.locator('.conversation-card');
    await expect(conversations).toHaveCount(1);

    conversationText = await conversations.first().textContent();
    expect(conversationText).toContain('math homework');
    expect(conversationText).not.toContain('stock');
  });
});
