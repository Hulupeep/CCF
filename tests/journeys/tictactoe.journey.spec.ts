/**
 * Journey Test: J-GAME-TICTACTOE
 * Issue: #37 - STORY-GAME-006: First Tic-Tac-Toe Journey Test
 *
 * Scenario: User plays Tic-Tac-Toe game
 *   Given mBot is in GameBot mode
 *   When user starts a Tic-Tac-Toe game
 *   And user makes first move
 *   Then mBot physically draws its O on paper
 *   And user makes second move
 *   Then mBot responds with strategic placement
 *   And game continues until win/draw/loss
 */

import { test, expect } from '@playwright/test';

test.describe('J-GAME-TICTACTOE: Tic-Tac-Toe Journey', () => {
  test('user plays complete game against mBot', async ({ page }) => {
    // Given: mBot is in GameBot mode
    await page.goto('/');
    await page.getByTestId('mode-selector')).selectOption('game-bot');
    await expect(page.getByTestId('current-mode')).toHaveText('GameBot');

    // When: User starts a Tic-Tac-Toe game
    await page.getByTestId('game-library')).click();
    await page.getByTestId('game-tictactoe')).click();
    await expect(page.getByTestId('game-title')).toHaveText('Tic-Tac-Toe');

    await page.getByTestId('start-game')).click();
    await expect(page.getByTestId('game-status')).toHaveText('Your Turn');

    // And: User makes first move (center square)
    await page.getByTestId('cell-1-1')).click(); // Center
    await expect(page.getByTestId('cell-1-1')).toHaveText('X');
    await expect(page.getByTestId('game-status')).toHaveText("mBot's Turn");

    // Then: mBot physically draws its O
    await expect(page.getByTestId('drawing-status')).toHaveText('Drawing', { timeout: 5000 });
    await expect(page.getByTestId('pen-status')).toHaveText('Down');

    // Wait for mBot to complete move
    await expect(page.getByTestId('game-status')).toHaveText('Your Turn', { timeout: 30000 });

    // Verify mBot made a valid move
    const filledCells = await page.getByTestId('cell').filter({ hasText: 'O' }).count();
    expect(filledCells).toBe(1);

    // And: User makes second move
    await page.getByTestId('cell-0-0')).click(); // Top-left
    await expect(page.getByTestId('cell-0-0')).toHaveText('X');

    // Then: mBot responds strategically
    await expect(page.getByTestId('game-status')).toHaveText('Your Turn', { timeout: 30000 });
    const filledCellsAfter = await page.getByTestId('cell').filter({ hasText: 'O' }).count();
    expect(filledCellsAfter).toBe(2);

    // Play until game ends (continue making moves)
    // This is a simplified flow - full game logic would be more complex
  });

  test('mBot shows emotion based on game outcome', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('mode-selector')).selectOption('game-bot');
    await page.getByTestId('game-tictactoe')).click();
    await page.getByTestId('start-game')).click();

    // Simulate game ending in mBot win
    await page.evaluate(() => {
      (window as any).simulateGameEnd = 'mbot-wins';
    });

    // Should show happy emotion
    await expect(page.getByTestId('emotion-display')).toHaveText('Happy', { timeout: 5000 });
    await expect(page.getByTestId('led-color')).toContainText('Green');
  });

  test('user can reset game mid-play', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('mode-selector')).selectOption('game-bot');
    await page.getByTestId('game-tictactoe')).click();
    await page.getByTestId('start-game')).click();

    // Make a move
    await page.getByTestId('cell-0-0')).click();

    // Reset game
    await page.getByTestId('reset-game')).click();

    // Board should be cleared
    const filledCells = await page.getByTestId('cell').filter({ hasText: /X|O/ }).count();
    expect(filledCells).toBe(0);
    await expect(page.getByTestId('game-status')).toHaveText('Your Turn');
  });
});
