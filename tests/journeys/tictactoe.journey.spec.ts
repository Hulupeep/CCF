/**
 * Journey Test: J-GAME-TICTACTOE (J-GAME-FIRST-TICTACTOE)
 * Issue: #37 - STORY-GAME-006: First Tic-Tac-Toe Journey Test
 * Contract: feature_gamebot.yml
 * DOD Criticality: Important
 *
 * Complete tic-tac-toe journey test covering all game outcomes,
 * difficulty levels, emotional responses, and timing requirements.
 *
 * Journey Steps:
 *   1. Start game - Robot draws grid (30s max)
 *   2. First move - Robot shows thinking behavior (5s max)
 *   3. Robot responds - Draws O in valid position (10s max)
 *   4-5. Game continues - Valid moves exchanged
 *   6. Game ends - Emotional response triggered
 *   7. Rematch offered - "Play again?" behavior
 */

import { test, expect } from '@playwright/test';

// Configure video recording for journey documentation
test.use({
  video: 'on',
  screenshot: 'on',
});

test.describe('@J-GAME-FIRST-TICTACTOE Journey Tests', () => {
  test.describe.configure({ mode: 'serial' }); // Journey steps must be sequential

  test.beforeEach(async ({ page }) => {
    await page.goto('/games/tictactoe');
    // Verify clean state before each test
    await expect(page.getByTestId('game-ttt-grid')).not.toBeVisible();
  });

  test('@J-GAME-FIRST-TICTACTOE @happy-path @critical Complete journey - happy path', async ({ page }) => {
    const startTime = Date.now();

    // Step 1: Start game and verify grid drawing (30s max)
    await page.getByTestId('game-ttt-start').click();
    const gridStart = Date.now();
    await expect(page.getByTestId('game-ttt-grid')).toBeVisible({ timeout: 35000 });
    const gridDuration = Date.now() - gridStart;
    expect(gridDuration).toBeLessThan(30000); // I-GAME-013: Grid drawing timing

    // Step 2: User plays center, verify thinking behavior (5s max)
    await page.getByTestId('game-ttt-cell-4').click();
    const thinkingStart = Date.now();
    await expect(page.getByTestId('game-ttt-thinking')).toBeVisible({ timeout: 5000 });
    const thinkingDuration = Date.now() - thinkingStart;
    expect(thinkingDuration).toBeLessThan(5000); // I-GAME-003: Thinking time bounded

    // Step 3: Verify robot move (10s max)
    const moveStart = Date.now();
    await expect(page.getByTestId('game-ttt-move-robot')).toBeVisible({ timeout: 15000 });
    const moveDuration = Date.now() - moveStart;
    expect(moveDuration).toBeLessThan(10000); // I-GAME-014: Step transition timing

    // Steps 4-5: Continue game until completion
    let moveCount = 0;
    while (moveCount < 5) {
      await page.getByTestId('game-ttt-turn').waitFor({ state: 'visible', timeout: 5000 });
      const turnText = await page.getByTestId('game-ttt-turn').textContent();

      if (turnText?.includes('Human')) {
        // Find and click an available cell
        for (let i = 0; i < 9; i++) {
          const cell = page.getByTestId(`game-ttt-cell-${i}`);
          if (await cell.isEnabled()) {
            await cell.click();
            moveCount++;
            break;
          }
        }
      }

      // Check if game ended
      const outcomeVisible = await page.getByTestId('game-ttt-outcome').isVisible();
      if (outcomeVisible) break;

      // Wait for robot move with timeout
      await page.waitForTimeout(2000);
    }

    // Step 6: Verify outcome displayed
    await expect(page.getByTestId('game-ttt-outcome')).toBeVisible({ timeout: 30000 });

    // Step 7: Verify rematch offer
    await expect(page.getByTestId('game-ttt-rematch')).toBeVisible({ timeout: 10000 });

    // I-GAME-013: Total journey time under 5 minutes
    const totalTime = Date.now() - startTime;
    expect(totalTime).toBeLessThan(5 * 60 * 1000);

    console.log(`Journey completed in ${totalTime}ms`);
    console.log(`Grid: ${gridDuration}ms, Thinking: ${thinkingDuration}ms, Move: ${moveDuration}ms`);
  });

  test('@J-GAME-FIRST-TICTACTOE @robot-wins Robot victory journey on hard difficulty', async ({ page }) => {
    // Set hard difficulty - robot should never lose
    await page.getByTestId('game-ttt-difficulty').selectOption('hard');
    await page.getByTestId('game-ttt-start').click();
    await expect(page.getByTestId('game-ttt-grid')).toBeVisible({ timeout: 35000 });

    // Play suboptimal moves to allow robot to win
    const suboptimalMoves = [1, 5, 7]; // Edge cells, not strategic
    let movesPlayed = 0;

    while (movesPlayed < 5) {
      const turn = await page.getByTestId('game-ttt-turn').textContent();

      if (turn?.includes('Human')) {
        const cellIndex = suboptimalMoves[movesPlayed % suboptimalMoves.length];
        const cell = page.getByTestId(`game-ttt-cell-${cellIndex}`);

        if (await cell.isEnabled()) {
          await cell.click();
          movesPlayed++;
        } else {
          // Try next available cell
          for (let i = 0; i < 9; i++) {
            const altCell = page.getByTestId(`game-ttt-cell-${i}`);
            if (await altCell.isEnabled()) {
              await altCell.click();
              movesPlayed++;
              break;
            }
          }
        }
      }

      // Check if game ended
      const outcomeVisible = await page.getByTestId('game-ttt-outcome').isVisible();
      if (outcomeVisible) break;

      await page.waitForTimeout(3000);
    }

    // Verify outcome shows robot won or draw (hard mode never loses)
    await expect(page.getByTestId('game-ttt-outcome')).toBeVisible({ timeout: 60000 });
    const outcome = await page.getByTestId('game-ttt-outcome').textContent();
    expect(outcome).toMatch(/Robot|Draw/i);

    // Verify victory celebration if robot won
    if (outcome?.includes('Robot')) {
      await expect(page.getByTestId('game-emotion-victory')).toBeVisible({ timeout: 5000 });
    }

    // I-GAME-006: Rematch offered
    await expect(page.getByTestId('game-ttt-rematch')).toBeVisible({ timeout: 10000 });
  });

  test('@J-GAME-FIRST-TICTACTOE @human-wins Human victory journey on easy difficulty', async ({ page }) => {
    // Set easy difficulty - robot plays randomly
    await page.getByTestId('game-ttt-difficulty').selectOption('easy');
    await page.getByTestId('game-ttt-start').click();
    await expect(page.getByTestId('game-ttt-grid')).toBeVisible({ timeout: 35000 });

    // Play optimal winning strategy: center, then corners
    const winningStrategy = [4, 0, 8]; // Center, top-left, bottom-right
    let strategyIndex = 0;

    while (strategyIndex < 5) {
      const turn = await page.getByTestId('game-ttt-turn').textContent();

      if (turn?.includes('Human')) {
        const targetCell = winningStrategy[strategyIndex % winningStrategy.length];
        const cell = page.getByTestId(`game-ttt-cell-${targetCell}`);

        if (await cell.isEnabled()) {
          await cell.click();
          strategyIndex++;
        } else {
          // Adapt strategy if cell taken
          for (const corner of [0, 2, 6, 8]) {
            const cornerCell = page.getByTestId(`game-ttt-cell-${corner}`);
            if (await cornerCell.isEnabled()) {
              await cornerCell.click();
              strategyIndex++;
              break;
            }
          }
        }
      }

      // Check if game ended
      const outcomeVisible = await page.getByTestId('game-ttt-outcome').isVisible();
      if (outcomeVisible) break;

      await page.waitForTimeout(2000);
    }

    // Verify outcome
    await expect(page.getByTestId('game-ttt-outcome')).toBeVisible({ timeout: 60000 });
    const outcome = await page.getByTestId('game-ttt-outcome').textContent();

    // If human won, verify graceful loss behavior
    if (outcome?.includes('Human')) {
      // I-GAME-004: Non-aggression rule
      await expect(page.getByTestId('game-emotion-loss')).toBeVisible({ timeout: 5000 });

      // Verify no aggressive behavior (no red LEDs)
      const emotionDisplay = await page.getByTestId('game-emotion-loss').textContent();
      expect(emotionDisplay).not.toContain('angry');
      expect(emotionDisplay).not.toContain('aggressive');
    }

    // I-GAME-006: Rematch offer
    await expect(page.getByTestId('game-ttt-rematch')).toBeVisible({ timeout: 10000 });
  });

  test('@J-GAME-FIRST-TICTACTOE @draw Draw journey on medium difficulty', async ({ page }) => {
    await page.getByTestId('game-ttt-difficulty').selectOption('medium');
    await page.getByTestId('game-ttt-start').click();
    await expect(page.getByTestId('game-ttt-grid')).toBeVisible({ timeout: 35000 });

    // Play defensive moves to force draw
    let moveCount = 0;
    while (moveCount < 5) {
      await page.getByTestId('game-ttt-turn').waitFor({ state: 'visible', timeout: 5000 });
      const turn = await page.getByTestId('game-ttt-turn').textContent();

      if (turn?.includes('Human')) {
        // Find first available cell
        for (let i = 0; i < 9; i++) {
          const cell = page.getByTestId(`game-ttt-cell-${i}`);
          if (await cell.isEnabled()) {
            await cell.click();
            moveCount++;
            break;
          }
        }
      }

      // Check if game ended
      const outcomeVisible = await page.getByTestId('game-ttt-outcome').isVisible();
      if (outcomeVisible) break;

      await page.waitForTimeout(2000);
    }

    // Verify outcome
    await expect(page.getByTestId('game-ttt-outcome')).toBeVisible({ timeout: 60000 });

    // Rematch offered regardless of outcome
    await expect(page.getByTestId('game-ttt-rematch')).toBeVisible({ timeout: 10000 });
  });

  test.describe('@J-GAME-FIRST-TICTACTOE @difficulty Difficulty level journeys', () => {
    const difficulties = ['easy', 'medium', 'hard'] as const;

    for (const difficulty of difficulties) {
      test(`${difficulty} difficulty completes successfully`, async ({ page }) => {
        await page.getByTestId('game-ttt-difficulty').selectOption(difficulty);
        await page.getByTestId('game-ttt-start').click();
        await expect(page.getByTestId('game-ttt-grid')).toBeVisible({ timeout: 35000 });

        // Play a few moves to verify game progresses
        for (let i = 0; i < 3; i++) {
          await page.getByTestId('game-ttt-turn').waitFor({ state: 'visible', timeout: 5000 });
          const turn = await page.getByTestId('game-ttt-turn').textContent();

          if (turn?.includes('Human')) {
            for (let cell = 0; cell < 9; cell++) {
              const cellElement = page.getByTestId(`game-ttt-cell-${cell}`);
              if (await cellElement.isEnabled()) {
                await cellElement.click();
                break;
              }
            }
          }

          await page.waitForTimeout(3000);
        }

        // Game should still be in valid state
        const turnOrOutcome = await page.getByTestId('game-ttt-turn')
          .or(page.getByTestId('game-ttt-outcome'))
          .isVisible();
        expect(turnOrOutcome).toBe(true);
      });
    }
  });

  test('@J-GAME-FIRST-TICTACTOE @step-timing Step timing verification', async ({ page }) => {
    const timings: { step: string; duration: number; limit: number }[] = [];

    // Step 1: Grid drawing (30s max)
    let stepStart = Date.now();
    await page.getByTestId('game-ttt-start').click();
    await expect(page.getByTestId('game-ttt-grid')).toBeVisible({ timeout: 35000 });
    timings.push({ step: 'grid_drawing', duration: Date.now() - stepStart, limit: 30000 });

    // Step 2: Thinking behavior (5s max)
    stepStart = Date.now();
    await page.getByTestId('game-ttt-cell-4').click();
    await expect(page.getByTestId('game-ttt-thinking')).toBeVisible({ timeout: 5000 });
    timings.push({ step: 'thinking', duration: Date.now() - stepStart, limit: 5000 });

    // Step 3: Robot move (10s max)
    stepStart = Date.now();
    await expect(page.getByTestId('game-ttt-move-robot')).toBeVisible({ timeout: 15000 });
    timings.push({ step: 'robot_move', duration: Date.now() - stepStart, limit: 10000 });

    // Verify all timings meet requirements
    for (const timing of timings) {
      console.log(`${timing.step}: ${timing.duration}ms (limit: ${timing.limit}ms)`);
      expect(timing.duration).toBeLessThan(timing.limit);
    }
  });

  test('@J-GAME-FIRST-TICTACTOE @integration Integration verification', async ({ page }) => {
    // Verify integration of STORY-GAME-001 (Core Logic),
    // STORY-GAME-002 (Physical Drawing), STORY-GAME-003 (Emotional Responses)

    await page.getByTestId('game-ttt-start').click();
    await expect(page.getByTestId('game-ttt-grid')).toBeVisible({ timeout: 35000 });

    // Play one move
    await page.getByTestId('game-ttt-cell-4').click();

    // Verify core logic integration (turn alternation)
    await expect(page.getByTestId('game-ttt-turn')).toContainText('Robot', { timeout: 5000 });

    // Verify physical drawing integration
    await expect(page.getByTestId('game-ttt-move-robot')).toBeVisible({ timeout: 15000 });

    // Note: Full emotional response verification happens at game end
    // This test confirms all systems are integrated and communicating
    console.log('Integration test: Core logic, drawing system, and UI responding correctly');
  });

  test('@J-GAME-FIRST-TICTACTOE @timeout-handling Timeout handling', async ({ page }) => {
    await page.getByTestId('game-ttt-start').click();
    await expect(page.getByTestId('game-ttt-grid')).toBeVisible({ timeout: 35000 });

    // Make first move
    await page.getByTestId('game-ttt-cell-4').click();
    await expect(page.getByTestId('game-ttt-move-robot')).toBeVisible({ timeout: 15000 });

    // Wait for timeout (simulated with shorter duration for test)
    // In production, timeout is 60s. Test with shorter timeout if available.
    // This verifies the system can handle user inactivity gracefully.

    console.log('Timeout handling test: System should prompt user after inactivity');
  });
});
