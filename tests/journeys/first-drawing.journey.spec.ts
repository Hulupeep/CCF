/**
 * Journey Test: J-ART-FIRST-DRAWING
 * Issue: #16 - STORY-ART-005: First Drawing Journey Test
 * Contract: J-ART-FIRST-DRAWING (CRITICAL)
 *
 * Complete first drawing experience with mood transitions:
 *   1. Robot indicates ready state
 *   2. Drawing begins with calm spiral
 *   3. Loud noise triggers spike mode (jagged drawing)
 *   4. Quiet environment restores calm drawing
 *   5. Session ends with signing and pen lift
 *   6. Physical artwork shows emotional journey
 */

import { test, expect, Page } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

// Data contract types
interface MoodTransition {
  from_mode: 'Calm' | 'Active' | 'Spike' | 'Protect';
  to_mode: 'Calm' | 'Active' | 'Spike' | 'Protect';
  timestamp: number;
  trigger: 'stimulus' | 'natural_decay' | 'session_event';
}

interface JourneyArtifact {
  type: 'photo' | 'video' | 'log' | 'session_export';
  path: string;
  description: string;
}

interface JourneyTestResult {
  journey_id: 'J-ART-FIRST-DRAWING';
  passed: boolean;
  steps_completed: number;
  total_steps: number;
  mood_transitions: MoodTransition[];
  duration_ms: number;
  artifacts: JourneyArtifact[];
}

// Test helpers
class StimulusInjector {
  constructor(private page: Page) {}

  async injectLoudNoise(intensity: number = 0.9, duration_ms: number = 1000): Promise<void> {
    await this.page.evaluate(
      ({ intensity, duration_ms }) => {
        // Inject stimulus event into robot nervous system
        (window as any).mBot?.stimulusSystem?.inject({
          type: 'sound',
          intensity,
          duration_ms,
          timestamp: Date.now()
        });
      },
      { intensity, duration_ms }
    );
  }

  async injectQuietPeriod(duration_ms: number = 5000): Promise<void> {
    // Simply wait - no stimuli during this time
    await this.page.waitForTimeout(duration_ms);
  }
}

class JourneyRecorder {
  public artifacts: JourneyArtifact[] = [];
  private moodTransitions: MoodTransition[] = [];
  private startTime: number = 0;

  constructor(private page: Page, private testInfo: any) {
    this.startTime = Date.now();
  }

  async captureScreenshot(description: string): Promise<void> {
    const timestamp = Date.now() - this.startTime;
    const filename = `step-${this.artifacts.length + 1}-${timestamp}ms.png`;
    const screenshotPath = path.join(
      this.testInfo.outputDir,
      filename
    );

    await this.page.screenshot({ path: screenshotPath, fullPage: true });

    this.artifacts.push({
      type: 'photo',
      path: screenshotPath,
      description
    });
  }

  async captureVideo(description: string): Promise<void> {
    // Video capture handled by Playwright config
    const videoPath = await this.testInfo.outputPath('video.webm');
    this.artifacts.push({
      type: 'video',
      path: videoPath,
      description
    });
  }

  async recordMoodTransition(
    from_mode: MoodTransition['from_mode'],
    to_mode: MoodTransition['to_mode'],
    trigger: MoodTransition['trigger']
  ): Promise<void> {
    this.moodTransitions.push({
      from_mode,
      to_mode,
      timestamp: Date.now() - this.startTime,
      trigger
    });
  }

  getResult(passed: boolean, steps_completed: number, total_steps: number): JourneyTestResult {
    return {
      journey_id: 'J-ART-FIRST-DRAWING',
      passed,
      steps_completed,
      total_steps,
      mood_transitions: this.moodTransitions,
      duration_ms: Date.now() - this.startTime,
      artifacts: this.artifacts
    };
  }

  async exportResult(result: JourneyTestResult): Promise<void> {
    const resultPath = path.join(
      this.testInfo.outputDir,
      'journey-result.json'
    );
    fs.writeFileSync(resultPath, JSON.stringify(result, null, 2));

    this.artifacts.push({
      type: 'session_export',
      path: resultPath,
      description: 'Journey test result with mood transitions'
    });
  }
}

test.describe('J-ART-FIRST-DRAWING: First Drawing Journey', () => {
  test.setTimeout(120000); // 120 second timeout as per acceptance criteria

  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('/');

    // Wait for app to be ready
    await page.waitForLoadState('networkidle');
  });

  /**
   * @ART-JOURNEY-STEP-1
   * Robot indicates ready state when placed on paper
   */
  test('Step 1: Robot indicates ready state', async ({ page }, testInfo) => {
    const recorder = new JourneyRecorder(page, testInfo);

    // Given the robot is placed on paper
    // When the companion app connects
    await expect(page.getByTestId('status-ready')).toBeVisible({ timeout: 10000 });

    // Then the LED should pulse blue
    const ledStatus = await page.getByTestId('led-status').textContent();
    expect(ledStatus).toContain('blue');
    expect(ledStatus).toContain('pulse');

    // And the pen should be in the up position
    await expect(page.getByTestId('pen-status')).toHaveText('Up');

    await recorder.captureScreenshot('Robot ready state - LED blue pulse, pen up');

    const result = recorder.getResult(true, 1, 1);
    await recorder.exportResult(result);
  });

  /**
   * @ART-JOURNEY-STEP-2
   * Drawing begins with calm spiral on start
   */
  test('Step 2: Drawing begins with calm spiral', async ({ page }, testInfo) => {
    const recorder = new JourneyRecorder(page, testInfo);

    // Given the robot indicates ready state
    await expect(page.getByTestId('status-ready')).toBeVisible();

    // When user taps "Start Drawing"
    await page.getByTestId('start-drawing').click();
    await recorder.captureScreenshot('User tapped Start Drawing button');

    // Then the pen should lower to paper
    await expect(page.getByTestId('pen-status')).toHaveText('Down', { timeout: 5000 });

    // And the robot should begin drawing a spiral
    await expect(page.getByTestId('drawing-mode')).toHaveText('Calm');
    await expect(page.getByTestId('pattern-type')).toHaveText('spiral');

    // And the drawing style should match Calm mode (smooth curves)
    const drawingStyle = await page.getByTestId('drawing-style').textContent();
    expect(drawingStyle).toContain('smooth');

    await page.waitForTimeout(10000); // Wait 10s for Calm drawing

    await recorder.captureScreenshot('Calm spiral drawing - smooth curves');
    await recorder.recordMoodTransition('Calm', 'Calm', 'session_event');

    const result = recorder.getResult(true, 2, 2);
    await recorder.exportResult(result);
  });

  /**
   * @ART-JOURNEY-STEP-3
   * Loud noise triggers spike mode and jagged drawing
   */
  test('Step 3: Loud noise triggers spike mode', async ({ page }, testInfo) => {
    const recorder = new JourneyRecorder(page, testInfo);
    const stimulator = new StimulusInjector(page);

    // Given the robot is actively drawing in Calm mode
    await expect(page.getByTestId('status-ready')).toBeVisible();
    await page.getByTestId('start-drawing').click();
    await expect(page.getByTestId('drawing-mode')).toHaveText('Calm');
    await page.waitForTimeout(10000); // Draw in Calm for 10s

    await recorder.captureScreenshot('Before stimulus - Calm mode');

    // When a loud noise stimulus is injected
    await stimulator.injectLoudNoise(0.9, 1000);

    // Then the robot should transition to Spike reflex mode
    await expect(page.getByTestId('drawing-mode')).toHaveText('Spike', { timeout: 2000 });
    await recorder.recordMoodTransition('Calm', 'Spike', 'stimulus');

    // And the drawing should become jagged
    const drawingStyle = await page.getByTestId('drawing-style').textContent();
    expect(drawingStyle).toContain('jagged');

    // And sharp angles greater than 45 degrees should appear
    const maxAngle = await page.getByTestId('max-angle-drawn').textContent();
    expect(parseInt(maxAngle || '0')).toBeGreaterThan(45);

    await page.waitForTimeout(5000); // Draw in Spike for 5s

    await recorder.captureScreenshot('After stimulus - Spike mode with jagged lines');

    const result = recorder.getResult(true, 3, 3);
    await recorder.exportResult(result);
  });

  /**
   * @ART-JOURNEY-STEP-4
   * Quiet environment restores calm drawing
   */
  test('Step 4: Quiet environment restores calm', async ({ page }, testInfo) => {
    const recorder = new JourneyRecorder(page, testInfo);
    const stimulator = new StimulusInjector(page);

    // Given the robot is drawing in Spike mode
    await expect(page.getByTestId('status-ready')).toBeVisible();
    await page.getByTestId('start-drawing').click();
    await page.waitForTimeout(10000); // Calm drawing
    await stimulator.injectLoudNoise(0.9, 1000); // Trigger Spike
    await expect(page.getByTestId('drawing-mode')).toHaveText('Spike');
    await recorder.recordMoodTransition('Calm', 'Spike', 'stimulus');

    await recorder.captureScreenshot('Robot in Spike mode');

    // When the environment remains quiet for 5 seconds
    await stimulator.injectQuietPeriod(5000);

    // Then the robot should return to Calm reflex mode
    await expect(page.getByTestId('drawing-mode')).toHaveText('Calm', { timeout: 6000 });
    await recorder.recordMoodTransition('Spike', 'Calm', 'natural_decay');

    // And the drawing should return to smooth curves
    const drawingStyle = await page.getByTestId('drawing-style').textContent();
    expect(drawingStyle).toContain('smooth');

    // And the tension level should decrease below 0.3
    const tensionLevel = await page.getByTestId('tension-level').textContent();
    expect(parseFloat(tensionLevel || '1.0')).toBeLessThan(0.3);

    await recorder.captureScreenshot('Returned to Calm mode - smooth curves restored');

    const result = recorder.getResult(true, 4, 4);
    await recorder.exportResult(result);
  });

  /**
   * @ART-JOURNEY-STEP-5
   * Session ends with signing and pen lift
   */
  test('Step 5: Session ends with signing', async ({ page }, testInfo) => {
    const recorder = new JourneyRecorder(page, testInfo);

    // Given the robot is actively drawing
    await expect(page.getByTestId('status-ready')).toBeVisible();
    await page.getByTestId('start-drawing').click();
    await page.waitForTimeout(15000); // Draw for 15s

    await recorder.captureScreenshot('Before finishing - active drawing');

    // When user taps "Finish"
    await page.getByTestId('finish-drawing').click();

    // Then the robot should execute signing behavior
    await expect(page.getByTestId('robot-action')).toHaveText('Signing', { timeout: 5000 });
    await page.waitForTimeout(3000); // Wait for signature

    // And the pen should lift after signing
    await expect(page.getByTestId('pen-status')).toHaveText('Up', { timeout: 5000 });

    // And the session should be marked complete
    await expect(page.getByTestId('session-status')).toHaveText('Complete');

    await recorder.captureScreenshot('Session complete - pen up, signature visible');

    const result = recorder.getResult(true, 5, 5);
    await recorder.exportResult(result);
  });

  /**
   * @ART-JOURNEY-STEP-6
   * Physical artwork shows emotional journey
   */
  test('Step 6: Physical artwork inspection protocol', async ({ page }, testInfo) => {
    const recorder = new JourneyRecorder(page, testInfo);
    const stimulator = new StimulusInjector(page);

    // Complete a full drawing session
    await expect(page.getByTestId('status-ready')).toBeVisible();
    await page.getByTestId('start-drawing').click();

    // Calm phase
    await page.waitForTimeout(10000);
    await recorder.captureScreenshot('Section 1: Smooth spirals (Calm)');

    // Spike phase
    await stimulator.injectLoudNoise(0.9, 1000);
    await expect(page.getByTestId('drawing-mode')).toHaveText('Spike');
    await page.waitForTimeout(5000);
    await recorder.captureScreenshot('Section 2: Jagged transitions (Spike)');

    // Return to Calm
    await stimulator.injectQuietPeriod(5000);
    await expect(page.getByTestId('drawing-mode')).toHaveText('Calm');
    await page.waitForTimeout(5000);
    await recorder.captureScreenshot('Section 3: Return to smooth');

    // Finish
    await page.getByTestId('finish-drawing').click();
    await expect(page.getByTestId('session-status')).toHaveText('Complete');
    await recorder.captureScreenshot('Section 4: Signature');

    // Given the drawing session has completed
    // Then the physical paper should show expected patterns
    const canvas = page.getByTestId('drawing-canvas');
    await expect(canvas).toBeVisible();

    // Capture final artwork
    const artworkPath = path.join(testInfo.outputDir, 'final-artwork.png');
    await canvas.screenshot({ path: artworkPath });

    recorder.artifacts.push({
      type: 'photo',
      path: artworkPath,
      description: 'Complete artwork showing emotional journey: Calm -> Spike -> Calm -> Signature'
    });

    // Visual inspection checklist (manual verification required)
    console.log('\n=== VISUAL INSPECTION PROTOCOL ===');
    console.log('Verify physical paper shows:');
    console.log('  [  ] Start: Smooth spirals (Calm mode)');
    console.log('  [  ] Middle: Jagged transitions (Spike mode)');
    console.log('  [  ] End: Return to smooth curves');
    console.log('  [  ] Signature present');
    console.log('\nArtifacts captured:', recorder.artifacts.length);
    console.log('See:', testInfo.outputDir);

    const result = recorder.getResult(true, 6, 6);
    await recorder.exportResult(result);
  });

  /**
   * @ART-JOURNEY-COMPLETE
   * Complete first drawing journey end-to-end
   */
  test('Complete journey: Full emotional arc', async ({ page }, testInfo) => {
    const recorder = new JourneyRecorder(page, testInfo);
    const stimulator = new StimulusInjector(page);
    const totalSteps = 6;
    let stepsCompleted = 0;

    try {
      // Step 1: Robot ready
      await expect(page.getByTestId('status-ready')).toBeVisible({ timeout: 10000 });
      await expect(page.getByTestId('pen-status')).toHaveText('Up');
      stepsCompleted++;
      await recorder.captureScreenshot('Step 1: Robot ready');

      // Step 2: Start drawing (Calm)
      await page.getByTestId('start-drawing').click();
      await expect(page.getByTestId('pen-status')).toHaveText('Down', { timeout: 5000 });
      await expect(page.getByTestId('drawing-mode')).toHaveText('Calm');
      stepsCompleted++;

      // Wait 10 seconds for Calm drawing
      await page.waitForTimeout(10000);
      await recorder.captureScreenshot('Step 2: Calm drawing');
      await recorder.recordMoodTransition('Calm', 'Calm', 'session_event');

      // Step 3: Inject loud noise stimulus
      await stimulator.injectLoudNoise(0.9, 1000);
      await expect(page.getByTestId('drawing-mode')).toHaveText('Spike', { timeout: 2000 });
      stepsCompleted++;

      // Wait 5 seconds for Spike reaction
      await page.waitForTimeout(5000);
      await recorder.captureScreenshot('Step 3: Spike reaction');
      await recorder.recordMoodTransition('Calm', 'Spike', 'stimulus');

      // Step 4: Stay quiet for 5 seconds
      await stimulator.injectQuietPeriod(5000);
      await expect(page.getByTestId('drawing-mode')).toHaveText('Calm', { timeout: 6000 });
      stepsCompleted++;
      await recorder.captureScreenshot('Step 4: Returned to Calm');
      await recorder.recordMoodTransition('Spike', 'Calm', 'natural_decay');

      // Step 5: Finish
      await page.getByTestId('finish-drawing').click();
      await expect(page.getByTestId('robot-action')).toHaveText('Signing', { timeout: 5000 });
      await expect(page.getByTestId('pen-status')).toHaveText('Up', { timeout: 5000 });
      await expect(page.getByTestId('session-status')).toHaveText('Complete');
      stepsCompleted++;
      await recorder.captureScreenshot('Step 5: Session complete with signature');

      // Step 6: Verify session saved
      const sessionId = await page.getByTestId('session-id').textContent();
      expect(sessionId).toBeTruthy();
      stepsCompleted++;

      // Verify mood transitions
      expect(recorder['moodTransitions'].length).toBeGreaterThanOrEqual(2);

      const moodSequence = recorder['moodTransitions'].map(t => `${t.from_mode}->${t.to_mode}`);
      console.log('Mood transitions:', moodSequence);

      // Should include Calm->Spike->Calm
      const hasCalmToSpike = recorder['moodTransitions'].some(
        t => t.from_mode === 'Calm' && t.to_mode === 'Spike'
      );
      const hasSpikeToCalmDecay = recorder['moodTransitions'].some(
        t => t.from_mode === 'Spike' && t.to_mode === 'Calm' && t.trigger === 'natural_decay'
      );

      expect(hasCalmToSpike).toBeTruthy();
      expect(hasSpikeToCalmDecay).toBeTruthy();

      // Capture final canvas
      const canvas = page.getByTestId('drawing-canvas');
      const artworkPath = path.join(testInfo.outputDir, 'complete-journey-artwork.png');
      await canvas.screenshot({ path: artworkPath });

      // Export final result
      const result = recorder.getResult(true, stepsCompleted, totalSteps);
      await recorder.exportResult(result);

      console.log('\n=== JOURNEY COMPLETED SUCCESSFULLY ===');
      console.log(`Duration: ${result.duration_ms}ms`);
      console.log(`Steps: ${result.steps_completed}/${result.total_steps}`);
      console.log(`Mood transitions: ${result.mood_transitions.length}`);
      console.log(`Artifacts: ${result.artifacts.length}`);
      console.log('\nExpected mood flow: Calm -> Spike (stimulus) -> Calm (decay) ✓');

    } catch (error) {
      const result = recorder.getResult(false, stepsCompleted, totalSteps);
      await recorder.exportResult(result);
      throw error;
    }
  });

  /**
   * Error handling test
   */
  test('Journey fails gracefully on servo error', async ({ page }, testInfo) => {
    const recorder = new JourneyRecorder(page, testInfo);

    // Navigate and wait for ready
    await expect(page.getByTestId('status-ready')).toBeVisible();

    // Simulate servo error
    await page.evaluate(() => {
      (window as any).mBot = {
        ...(window as any).mBot,
        simulateServoError: true
      };
    });

    // Attempt to start drawing
    await page.getByTestId('start-drawing').click();

    // Should show error message
    await expect(page.getByTestId('error-message')).toContainText('servo', { timeout: 5000 });
    await expect(page.getByTestId('pen-status')).toHaveText('Error');

    await recorder.captureScreenshot('Servo error - graceful failure');

    const result = recorder.getResult(false, 0, 6);
    await recorder.exportResult(result);
  });
});
