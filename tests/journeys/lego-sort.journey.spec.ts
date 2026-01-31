/**
 * Journey Test: J-HELP-LEGO-SORT
 * Issue: #32 - STORY-HELP-006: LEGO Sort Journey Test
 *
 * Scenario: User experiences complete LEGO sorting with personality behaviors
 *   Given a white paper sorting surface with scattered LEGOs
 *   When user starts the sorting session
 *   Then mBot systematically detects and sorts each piece by color
 *   And shows personality behaviors (excitement for rare pieces, celebration)
 *   And achieves >= 90% accuracy
 *   And completes within reasonable time (< 10 minutes)
 *
 * DOD Criticality: CRITICAL
 * Journey Contract: J-HELP-LEGO-SORT (CONTRACT_INDEX.yml)
 * Covers Requirements: HELP-001 (Color Detection System)
 */

import { test, expect } from '@playwright/test';

/**
 * Journey test result structure for comprehensive reporting
 */
interface JourneyTestResult {
  journey_id: 'J-HELP-LEGO-SORT';
  status: 'passed' | 'failed' | 'partial';
  total_pieces: number;
  correctly_sorted: number;
  accuracy_percent: number;
  special_pieces_detected: string[];
  personality_behaviors_triggered: string[];
  duration_ms: number;
  steps_completed: JourneyStep[];
  video_path: string | null;
}

interface JourneyStep {
  step_number: number;
  description: string;
  user_action: string;
  robot_response: string;
  verification: string;
  passed: boolean;
  notes: string;
}

test.describe('@HELP-006 J-HELP-LEGO-SORT: LEGO Sort Journey', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to HelperBot sorting interface
    await page.goto('/helperbot/lego-sort');
    await page.waitForLoadState('networkidle');
  });

  test('@HELP-006-full-journey: Complete LEGO sorting journey with personality', async ({ page }) => {
    const startTime = Date.now();
    const stepsCompleted: JourneyStep[] = [];

    // ========================================
    // STEP 1: Robot observes area (precondition check)
    // ========================================
    const step1: JourneyStep = {
      step_number: 1,
      description: 'Robot observes sorting area',
      user_action: 'Spreads 10+ LEGOs on white paper',
      robot_response: 'Shows ready behavior',
      verification: 'Robot status is Ready',
      passed: false,
      notes: ''
    };

    // Given: White paper sorting surface is prepared
    await page.getByTestId('setup-sorting-surface').click();
    await page.getByTestId('surface-type').selectOption('white-paper');

    // And: 10+ LEGO bricks are available (red, blue, green, yellow, + 1 gold)
    await page.getByTestId('setup-pieces').click();
    await page.getByTestId('piece-count').fill('12');
    await page.getByTestId('include-colors').selectOption(['red', 'blue', 'green', 'yellow', 'gold']);
    await page.getByTestId('apply-piece-setup').click();

    // And: Color zones are marked
    await page.getByTestId('setup-color-zones').click();
    await page.getByTestId('zone-red').fill('100,100');
    await page.getByTestId('zone-blue').fill('200,100');
    await page.getByTestId('zone-green').fill('300,100');
    await page.getByTestId('zone-yellow').fill('400,100');
    await page.getByTestId('zone-special').fill('250,200');
    await page.getByTestId('apply-zone-setup').click();

    // Then: Robot should show ready behavior
    await expect(page.getByTestId('robot-status')).toHaveText('Ready', { timeout: 5000 });
    await expect(page.getByTestId('calibration-status')).toHaveText('Calibrated');

    step1.passed = true;
    step1.notes = 'Setup complete, robot ready';
    stepsCompleted.push(step1);

    // ========================================
    // STEP 2: User starts sorting
    // ========================================
    const step2: JourneyStep = {
      step_number: 2,
      description: 'Start sorting session',
      user_action: 'Clicks start sorting button',
      robot_response: 'Approaches first piece',
      verification: 'Robot begins movement',
      passed: false,
      notes: ''
    };

    // When: Sorting is started via app command
    await page.getByTestId('start-sorting').click();

    // Then: Robot should approach the first piece
    await expect(page.getByTestId('sorting-status')).toHaveText('Active', { timeout: 3000 });
    await expect(page.getByTestId('robot-movement')).toHaveText('Moving', { timeout: 5000 });

    step2.passed = true;
    step2.notes = 'Sorting initiated successfully';
    stepsCompleted.push(step2);

    // ========================================
    // STEPS 3-5: Robot sorts each piece (repeated loop)
    // ========================================
    const step3to5: JourneyStep = {
      step_number: 3,
      description: 'Sort pieces systematically',
      user_action: 'Observes robot working',
      robot_response: 'Detects color, displays on LED, pushes to zone, proceeds to next',
      verification: 'Each piece sorted correctly',
      passed: false,
      notes: ''
    };

    // Track sorting progress
    const totalPieces = 12;
    let piecesSorted = 0;
    let correctlySorted = 0;
    const specialPiecesDetected: string[] = [];
    const personalityBehaviors: string[] = [];

    // Wait for first piece detection
    await expect(page.getByTestId('detected-color')).not.toBeEmpty({ timeout: 10000 });

    // Verify LED shows detected color
    const firstColor = await page.getByTestId('detected-color').textContent();
    await expect(page.getByTestId('led-color')).toHaveText(firstColor || '', { timeout: 2000 });
    step3to5.notes = `First piece detected: ${firstColor}`;

    // Monitor sorting loop (pieces should decrease)
    const initialRemaining = await page.getByTestId('pieces-remaining').textContent();
    expect(parseInt(initialRemaining || '12')).toBe(12);

    // Wait for at least 3 pieces to be sorted to verify loop is working
    await expect(page.getByTestId('pieces-remaining')).not.toHaveText('12', { timeout: 60000 });
    await expect(page.getByTestId('pieces-remaining')).not.toHaveText('11', { timeout: 60000 });
    await expect(page.getByTestId('pieces-remaining')).not.toHaveText('10', { timeout: 60000 });

    step3to5.passed = true;
    stepsCompleted.push(step3to5);

    // ========================================
    // STEP 6: Gold piece special behavior
    // ========================================
    const step6: JourneyStep = {
      step_number: 6,
      description: 'Rare piece detection',
      user_action: 'Robot encounters gold piece',
      robot_response: 'Shows excitement behavior',
      verification: 'Special behavior triggered',
      passed: false,
      notes: ''
    };

    // When: A gold piece is detected
    // Wait for gold detection event (should happen during sorting)
    const goldDetected = await Promise.race([
      page.waitForSelector('[data-testid="special-piece-detected"]', { timeout: 300000 }),
      page.waitForSelector('[data-testid="pieces-remaining"][data-value="0"]', { timeout: 300000 })
    ]);

    if (await page.isVisible('[data-testid="special-piece-detected"]')) {
      const specialPiece = await page.getByTestId('special-piece-type').textContent();
      specialPiecesDetected.push(specialPiece || 'unknown');

      // Then: Robot should show excitement behavior
      await expect(page.getByTestId('personality-behavior')).toHaveText('excitement', { timeout: 3000 });
      personalityBehaviors.push('excitement');

      // And: Pause to "admire" the piece
      const behaviorDuration = await page.getByTestId('behavior-duration').textContent();
      expect(parseInt(behaviorDuration || '0')).toBeGreaterThan(500); // At least 500ms pause

      step6.passed = true;
      step6.notes = `Gold piece detected and celebrated: ${specialPiece}`;
    } else {
      step6.passed = false;
      step6.notes = 'Gold piece not detected (may have been sorted before monitoring started)';
    }
    stepsCompleted.push(step6);

    // ========================================
    // STEP 7: Completion celebration
    // ========================================
    const step7: JourneyStep = {
      step_number: 7,
      description: 'Sorting completion',
      user_action: 'All pieces sorted',
      robot_response: 'Performs celebration behavior',
      verification: 'Victory behavior shown',
      passed: false,
      notes: ''
    };

    // Wait for all pieces to be sorted (max 10 minutes)
    await expect(page.getByTestId('pieces-remaining')).toHaveText('0', { timeout: 600000 });

    // When: All pieces are sorted
    // Then: Robot should perform celebration behavior
    await expect(page.getByTestId('sorting-status')).toHaveText('Complete', { timeout: 5000 });
    await expect(page.getByTestId('completion-behavior')).toBeVisible({ timeout: 5000 });

    const celebrationBehavior = await page.getByTestId('completion-behavior-type').textContent();
    personalityBehaviors.push(celebrationBehavior || 'celebration');

    step7.passed = true;
    step7.notes = `Celebration behavior: ${celebrationBehavior}`;
    stepsCompleted.push(step7);

    // ========================================
    // STEP 8: Accuracy verification
    // ========================================
    const step8: JourneyStep = {
      step_number: 8,
      description: 'Verify accuracy',
      user_action: 'Reviews sorting results',
      robot_response: 'Displays accuracy metrics',
      verification: 'Accuracy >= 90%',
      passed: false,
      notes: ''
    };

    // Collect final metrics
    const redBin = parseInt(await page.getByTestId('bin-red-count').textContent() || '0');
    const blueBin = parseInt(await page.getByTestId('bin-blue-count').textContent() || '0');
    const greenBin = parseInt(await page.getByTestId('bin-green-count').textContent() || '0');
    const yellowBin = parseInt(await page.getByTestId('bin-yellow-count').textContent() || '0');
    const specialBin = parseInt(await page.getByTestId('bin-special-count').textContent() || '0');

    piecesSorted = redBin + blueBin + greenBin + yellowBin + specialBin;

    // Get accuracy from system
    const accuracyText = await page.getByTestId('sorting-accuracy').textContent();
    const accuracyPercent = parseFloat(accuracyText?.replace('%', '') || '0');

    correctlySorted = Math.round(piecesSorted * (accuracyPercent / 100));

    // And: The final accuracy should be >= 90%
    expect(accuracyPercent).toBeGreaterThanOrEqual(90);
    expect(piecesSorted).toBe(totalPieces);

    step8.passed = true;
    step8.notes = `Accuracy: ${accuracyPercent}%, Pieces: ${piecesSorted}/${totalPieces}`;
    stepsCompleted.push(step8);

    // ========================================
    // Journey completion and reporting
    // ========================================
    const endTime = Date.now();
    const durationMs = endTime - startTime;

    const journeyResult: JourneyTestResult = {
      journey_id: 'J-HELP-LEGO-SORT',
      status: 'passed',
      total_pieces: totalPieces,
      correctly_sorted: correctlySorted,
      accuracy_percent: accuracyPercent,
      special_pieces_detected: specialPiecesDetected,
      personality_behaviors_triggered: personalityBehaviors,
      duration_ms: durationMs,
      steps_completed: stepsCompleted,
      video_path: null // Could be set if video recording is enabled
    };

    // Log journey result for reporting
    console.log('Journey Test Result:', JSON.stringify(journeyResult, null, 2));

    // Verify journey completed within reasonable time (< 10 minutes)
    expect(durationMs).toBeLessThan(600000);

    // All steps should have passed
    const allStepsPassed = stepsCompleted.every(step => step.passed);
    expect(allStepsPassed).toBe(true);
  });

  test('@HELP-006-accuracy: Measure sorting accuracy', async ({ page }) => {
    // Given: 10 LEGO pieces of known colors
    await page.getByTestId('setup-test-mode').click();
    await page.getByTestId('known-pieces').selectOption('10-standard-colors');
    await page.getByTestId('apply-test-setup').click();

    // When: Sorting completes
    await page.getByTestId('start-sorting').click();
    await expect(page.getByTestId('sorting-status')).toHaveText('Complete', { timeout: 600000 });

    // Then: correctly_sorted should be >= 9
    const correctlySorted = parseInt(await page.getByTestId('correctly-sorted').textContent() || '0');
    expect(correctlySorted).toBeGreaterThanOrEqual(9);

    // And: accuracy_percent should be >= 90
    const accuracyText = await page.getByTestId('sorting-accuracy').textContent();
    const accuracyPercent = parseFloat(accuracyText?.replace('%', '') || '0');
    expect(accuracyPercent).toBeGreaterThanOrEqual(90);
  });

  test('@HELP-006-personality: Verify personality behaviors during sorting', async ({ page }) => {
    // Given: Sorting is configured to track personality
    await page.getByTestId('setup-sorting-surface').click();
    await page.getByTestId('piece-count').fill('10');
    await page.getByTestId('include-colors').selectOption(['red', 'blue', 'gold']);
    await page.getByTestId('enable-personality-tracking').check();
    await page.getByTestId('apply-piece-setup').click();

    // When: Sorting is in progress
    await page.getByTestId('start-sorting').click();

    // Wait for gold piece to be detected
    await page.waitForSelector('[data-testid="special-piece-detected"]', { timeout: 300000 });

    // Then: The following behaviors should trigger
    const behaviors = await page.getByTestId('personality-events').allTextContents();

    // rare_piece_reactions (for gold piece)
    expect(behaviors.some(b => b.includes('excitement') || b.includes('rare'))).toBe(true);

    // Wait for completion
    await expect(page.getByTestId('sorting-status')).toHaveText('Complete', { timeout: 600000 });

    // completion_celebration (at end)
    expect(behaviors.some(b => b.includes('celebration') || b.includes('victory'))).toBe(true);

    // And: personality_consistent should be true
    const personalityConsistent = await page.getByTestId('personality-consistent').textContent();
    expect(personalityConsistent).toBe('true');
  });

  test('@HELP-006-edge-cases: Handle difficult-to-move pieces', async ({ page }) => {
    // Given: A difficult-to-move piece is present
    await page.getByTestId('setup-test-mode').click();
    await page.getByTestId('difficult-piece').check();
    await page.getByTestId('apply-test-setup').click();

    // When: The robot fails to push it on first attempt
    await page.getByTestId('start-sorting').click();

    // Wait for frustration behavior
    await page.waitForSelector('[data-testid="frustration-behavior"]', { timeout: 120000 });

    // Then: Frustration behavior should trigger
    const behaviorType = await page.getByTestId('current-behavior').textContent();
    expect(behaviorType).toContain('frustration');

    // And: The robot should try an alternative approach
    await expect(page.getByTestId('retry-strategy')).toBeVisible({ timeout: 10000 });

    // And: The piece should eventually be sorted
    await expect(page.getByTestId('difficult-piece-sorted')).toBeVisible({ timeout: 60000 });
  });

  test('@HELP-006-missed-piece: Handle missed pieces with verification scan', async ({ page }) => {
    // Given: Setup with possibility of missed piece
    await page.getByTestId('setup-test-mode').click();
    await page.getByTestId('simulate-missed-piece').check();
    await page.getByTestId('apply-test-setup').click();

    // When: Initial sorting "completes" but verification detects missed piece
    await page.getByTestId('start-sorting').click();

    // Wait for verification scan
    await page.waitForSelector('[data-testid="verification-scan-running"]', { timeout: 300000 });

    // Then: The missed piece should be detected
    await expect(page.getByTestId('missed-piece-found')).toBeVisible({ timeout: 30000 });

    // And: Sorted correctly
    await expect(page.getByTestId('sorting-status')).toHaveText('Complete', { timeout: 60000 });
    const remaining = await page.getByTestId('pieces-remaining').textContent();
    expect(parseInt(remaining || '1')).toBe(0);
  });

  test('@HELP-006-timing: Complete journey within reasonable time', async ({ page }) => {
    // Given: 10 LEGO pieces to sort
    await page.getByTestId('setup-sorting-surface').click();
    await page.getByTestId('piece-count').fill('10');
    await page.getByTestId('apply-piece-setup').click();

    const startTime = Date.now();

    // When: Sorting completes
    await page.getByTestId('start-sorting').click();
    await expect(page.getByTestId('sorting-status')).toHaveText('Complete', { timeout: 600000 });

    const endTime = Date.now();
    const durationMs = endTime - startTime;

    // Then: duration_ms should be < 600000 (10 minutes)
    expect(durationMs).toBeLessThan(600000);

    // Verify system reported time matches
    const reportedDuration = parseInt(await page.getByTestId('session-duration').textContent() || '0');
    expect(Math.abs(reportedDuration - durationMs)).toBeLessThan(5000); // Within 5 second margin
  });

  test('Performance: Sorting 10 pieces completes in < 5 minutes', async ({ page }) => {
    // Setup standard test
    await page.getByTestId('setup-sorting-surface').click();
    await page.getByTestId('piece-count').fill('10');
    await page.getByTestId('include-colors').selectOption(['red', 'blue', 'green', 'yellow']);
    await page.getByTestId('apply-piece-setup').click();

    const startTime = Date.now();

    // Execute sorting
    await page.getByTestId('start-sorting').click();
    await expect(page.getByTestId('sorting-status')).toHaveText('Complete', { timeout: 300000 });

    const durationMs = Date.now() - startTime;

    // Should complete in < 5 minutes for good performance
    expect(durationMs).toBeLessThan(300000);
  });
});
