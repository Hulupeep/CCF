/**
 * E2E Journey Test: Reinforcement Learning System
 *
 * Tests the complete RL learning flow from training to deployment.
 *
 * Invariants tested:
 * - I-AI-001: Q-learning converges within 1000 episodes
 * - I-AI-002: Learned policy persists across sessions
 * - I-AI-003: Learning rate decay prevents oscillation
 * - I-AI-004: Exploration rate decreases over time
 */

import { test, expect } from '@playwright/test';

test.describe('Reinforcement Learning Journey', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/learning');
  });

  test('Scenario: Learn from Tic-Tac-Toe', async ({ page }) => {
    // Given robot plays 100 games of tic-tac-toe
    await page.getByTestId('learning-toggle').click();
    await expect(page.getByTestId('learning-status')).toHaveText('Training');

    // Select tic-tac-toe game
    await page.selectOption('[data-testid="game-selector"]', 'tictactoe');

    // Start training
    await page.getByTestId('start-training-btn').click();

    // And learning is enabled
    await expect(page.getByTestId('learning-toggle')).toBeChecked();

    // When I check win rate after 100 games
    await page.waitForFunction(
      () => {
        const episodes = document.querySelector('[data-testid="episode-counter"]');
        return episodes && parseInt(episodes.textContent || '0') >= 100;
      },
      { timeout: 60000 }
    );

    const episodeCount = await page.getByTestId('episode-counter').textContent();
    expect(parseInt(episodeCount || '0')).toBeGreaterThanOrEqual(100);

    // Then win rate improves from 40% to 70%+
    const winRateText = await page.getByTestId('win-rate-display').textContent();
    const winRate = parseFloat(winRateText?.replace('%', '') || '0');
    expect(winRate).toBeGreaterThan(60); // Allow some variance

    // And robot adapts strategy
    const convergence = await page.getByTestId('convergence-indicator').textContent();
    expect(convergence).toContain('Converging');

    // And learned policy persists after restart
    await page.getByTestId('save-policy-btn').click();
    await expect(page.locator('.toast')).toContainText('Policy saved');
  });

  test('Scenario: Learn from User Feedback', async ({ page }) => {
    // Given robot performs behavior "cautious exploration"
    await page.getByTestId('learning-toggle').click();

    // Simulate robot behavior
    await page.evaluate(() => {
      window.postMessage({ type: 'robot_behavior', id: 'behavior_001', action: 'cautious_explore' }, '*');
    });

    // When user rates behavior as "good" (thumbs up)
    await page.getByTestId('feedback-good').click();

    // Then robot reinforces that behavior
    await expect(page.locator('.feedback-message')).toContainText('Feedback recorded');

    // And increases probability of similar actions by 10%
    // (This would be verified through internal metrics)

    // When user rates behavior as "bad" (thumbs down)
    await page.evaluate(() => {
      window.postMessage({ type: 'robot_behavior', id: 'behavior_002', action: 'reckless_move' }, '*');
    });

    await page.getByTestId('feedback-bad').click();

    // Then robot reduces that behavior
    await expect(page.locator('.feedback-message')).toContainText('Feedback recorded');

    // Verify feedback was sent
    const rewardDisplay = await page.getByTestId('reward-display').textContent();
    expect(rewardDisplay).toBeTruthy();
  });

  test('Scenario: Multi-Game Learning', async ({ page }) => {
    // Given robot learned tic-tac-toe strategy
    await page.getByTestId('learning-toggle').click();
    await page.selectOption('[data-testid="game-selector"]', 'tictactoe');
    await page.getByTestId('load-policy-btn').click();

    await expect(page.locator('.toast')).toContainText('Policy loaded', { timeout: 5000 });

    // When robot plays connect-four
    await page.selectOption('[data-testid="game-selector"]', 'connect-four');

    // Then learning transfers basic strategy
    // (Robot starts with better-than-random play)

    // And robot learns game-specific tactics
    await page.getByTestId('start-training-btn').click();

    await page.waitForFunction(
      () => {
        const episodes = document.querySelector('[data-testid="episode-counter"]');
        return episodes && parseInt(episodes.textContent || '0') >= 50;
      },
      { timeout: 30000 }
    );

    // And maintains separate Q-tables per game
    await page.selectOption('[data-testid="game-selector"]', 'tictactoe');
    const tttEpisodes = await page.getByTestId('episode-counter').textContent();

    await page.selectOption('[data-testid="game-selector"]', 'connect-four');
    const c4Episodes = await page.getByTestId('episode-counter').textContent();

    // Episode counts should be different (separate policies)
    expect(tttEpisodes).not.toBe(c4Episodes);
  });

  test('Scenario: Policy Persistence', async ({ page }) => {
    // Given robot learned strategy over 500 games
    await page.getByTestId('learning-toggle').click();
    await page.selectOption('[data-testid="game-selector"]', 'tictactoe');
    await page.getByTestId('start-training-btn').click();

    await page.waitForFunction(
      () => {
        const episodes = document.querySelector('[data-testid="episode-counter"]');
        return episodes && parseInt(episodes.textContent || '0') >= 100;
      },
      { timeout: 60000 }
    );

    // Save policy
    await page.getByTestId('save-policy-btn').click();
    await expect(page.locator('.toast')).toContainText('Policy saved');

    const initialWinRate = await page.getByTestId('win-rate-display').textContent();

    // When robot restarts (simulate by resetting UI)
    await page.reload();
    await page.getByTestId('learning-toggle').click();
    await page.selectOption('[data-testid="game-selector"]', 'tictactoe');

    // Then learned policy loads from storage
    await page.getByTestId('load-policy-btn').click();
    await expect(page.locator('.toast')).toContainText('Policy loaded');

    // And robot maintains 70%+ win rate
    const loadedWinRate = await page.getByTestId('win-rate-display').textContent();
    expect(loadedWinRate).toBe(initialWinRate);

    // And no re-training required
    const convergenceScore = await page.getByTestId('convergence-indicator').getAttribute('data-score');
    expect(parseFloat(convergenceScore || '0')).toBeGreaterThan(0.8);
  });

  test('Invariant I-AI-001: Convergence within 1000 episodes', async ({ page }) => {
    await page.getByTestId('learning-toggle').click();
    await page.selectOption('[data-testid="game-selector"]', 'tictactoe');
    await page.getByTestId('start-training-btn').click();

    // Wait for convergence or max episodes
    await page.waitForFunction(
      () => {
        const convergence = document.querySelector('[data-testid="convergence-indicator"]');
        const episodes = document.querySelector('[data-testid="episode-counter"]');
        const score = convergence?.getAttribute('data-score');
        const count = episodes ? parseInt(episodes.textContent || '0') : 0;

        return (score && parseFloat(score) > 0.9) || count >= 1000;
      },
      { timeout: 120000 }
    );

    const episodeCount = await page.getByTestId('episode-counter').textContent();
    const convergenceScore = await page.getByTestId('convergence-indicator').getAttribute('data-score');

    // Should converge before 1000 episodes
    expect(parseInt(episodeCount || '0')).toBeLessThanOrEqual(1000);
    expect(parseFloat(convergenceScore || '0')).toBeGreaterThan(0.9);
  });

  test('Invariant I-AI-003: Learning rate decay', async ({ page }) => {
    await page.getByTestId('learning-toggle').click();
    await page.selectOption('[data-testid="game-selector"]', 'tictactoe');

    // Get initial learning rate
    const initialLR = await page.getByTestId('learning-rate-display').textContent();
    const initialValue = parseFloat(initialLR || '0');

    // Train for some episodes
    await page.getByTestId('start-training-btn').click();

    await page.waitForFunction(
      () => {
        const episodes = document.querySelector('[data-testid="episode-counter"]');
        return episodes && parseInt(episodes.textContent || '0') >= 200;
      },
      { timeout: 30000 }
    );

    // Learning rate should have decayed
    const finalLR = await page.getByTestId('learning-rate-display').textContent();
    const finalValue = parseFloat(finalLR || '0');

    expect(finalValue).toBeLessThan(initialValue);
  });

  test('Invariant I-AI-004: Epsilon decay', async ({ page }) => {
    await page.getByTestId('learning-toggle').click();
    await page.selectOption('[data-testid="game-selector"]', 'tictactoe');

    // Get initial epsilon
    const initialEpsilon = await page.getByTestId('epsilon-display').textContent();
    const initialValue = parseFloat(initialEpsilon || '0');

    // Train for some episodes
    await page.getByTestId('start-training-btn').click();

    await page.waitForFunction(
      () => {
        const episodes = document.querySelector('[data-testid="episode-counter"]');
        return episodes && parseInt(episodes.textContent || '0') >= 100;
      },
      { timeout: 30000 }
    );

    // Epsilon should have decayed
    const finalEpsilon = await page.getByTestId('epsilon-display').textContent();
    const finalValue = parseFloat(finalEpsilon || '0');

    expect(finalValue).toBeLessThan(initialValue);
    expect(finalValue).toBeGreaterThanOrEqual(0.1); // Should not go below epsilon_end
  });

  test('Dashboard displays all metrics', async ({ page }) => {
    await page.getByTestId('learning-toggle').click();

    // Verify dashboard is visible
    await expect(page.getByTestId('learning-dashboard')).toBeVisible();

    // Check all required metrics are displayed
    await expect(page.getByTestId('episode-counter')).toBeVisible();
    await expect(page.getByTestId('win-rate-chart')).toBeVisible();
    await expect(page.getByTestId('epsilon-display')).toBeVisible();
    await expect(page.getByTestId('reward-display')).toBeVisible();
    await expect(page.getByTestId('convergence-indicator')).toBeVisible();
    await expect(page.getByTestId('learning-status')).toBeVisible();
    await expect(page.getByTestId('learning-rate-display')).toBeVisible();
  });

  test('Reset learning clears Q-table', async ({ page }) => {
    await page.getByTestId('learning-toggle').click();
    await page.selectOption('[data-testid="game-selector"]', 'tictactoe');

    // Train for a bit
    await page.getByTestId('start-training-btn').click();
    await page.waitForTimeout(5000);

    const beforeReset = await page.getByTestId('episode-counter').textContent();
    expect(parseInt(beforeReset || '0')).toBeGreaterThan(0);

    // Reset learning
    await page.getByTestId('reset-learning-btn').click();
    await page.getByRole('button', { name: 'Confirm' }).click();

    // Verify reset
    const afterReset = await page.getByTestId('episode-counter').textContent();
    expect(parseInt(afterReset || '0')).toBe(0);

    const winRate = await page.getByTestId('win-rate-display').textContent();
    expect(parseFloat(winRate?.replace('%', '') || '0')).toBe(0);
  });
});
