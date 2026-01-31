/**
 * Journey Test: J-LEARN-FIRST-EXPERIMENT
 * Issue: #33 - STORY-LEARN-006: First Experiment Journey Test
 *
 * Contract: J-LEARN-FIRST-EXPERIMENT (Critical - MUST pass for release)
 *
 * Journey: Student runs their first hands-on experiment with AI
 * Scenario: Student completes all 8 steps of the first experiment
 *   Given mBot is connected and in LearningLab mode
 *   When student follows the guided experiment flow
 *   Then they learn about AI behavior through observable robot responses
 *   And they understand parameter-driven behavior changes
 *   And they successfully create their first custom AI personality
 *
 * Invariants Referenced:
 *   I-LEARN-050: Journey must complete without errors in < 10 minutes
 *   I-LEARN-051: Each step must have observable robot response
 *   I-LEARN-052: Learning outcomes must be verifiable through UI state
 *   I-LEARN-053: Journey must be completable with keyboard only
 *   I-LEARN-054: Journey must work on tablet devices
 */

import { test, expect, Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

// Test data contract interfaces
interface JourneyStepResult {
  step_number: number;
  action: string;
  expected_response: string;
  actual_response: string;
  learning_outcome: string;
  result: 'pass' | 'fail' | 'skip';
  duration_ms: number;
  screenshot_path?: string;
  notes?: string;
}

interface JourneyTestResult {
  journey_id: 'J-LEARN-FIRST-EXPERIMENT';
  executed_at: number;
  duration_ms: number;
  steps: JourneyStepResult[];
  overall_result: 'pass' | 'fail' | 'partial';
  accessibility_score: number;
  artifacts: {
    video_path?: string;
    screenshots: string[];
    logs: string[];
  };
}

// Helper functions
async function waitForRobotResponse(page: Page, timeout = 5000): Promise<boolean> {
  try {
    await page.waitForSelector('[data-testid="robot-response-indicator"]', {
      state: 'visible',
      timeout
    });
    return true;
  } catch {
    return false;
  }
}

async function getTensionValue(page: Page): Promise<number> {
  const tensionText = await page.getByTestId('tension-value').textContent();
  return parseFloat(tensionText || '0');
}

async function captureStepScreenshot(page: Page, stepNumber: number): Promise<string> {
  const path = `test-results/journey-step-${stepNumber}.png`;
  await page.screenshot({ path, fullPage: true });
  return path;
}

test.describe('J-LEARN-FIRST-EXPERIMENT: First Experiment Journey', () => {
  let journeyResult: JourneyTestResult;
  let journeyStartTime: number;

  test.beforeEach(async ({ page }) => {
    journeyStartTime = Date.now();
    journeyResult = {
      journey_id: 'J-LEARN-FIRST-EXPERIMENT',
      executed_at: journeyStartTime,
      duration_ms: 0,
      steps: [],
      overall_result: 'pass',
      accessibility_score: 0,
      artifacts: {
        screenshots: [],
        logs: []
      }
    };

    // Navigate to app and ensure robot is connected
    await page.goto('/');
    await expect(page.getByTestId('mbot-status')).toHaveText('Connected', { timeout: 10000 });
  });

  test.afterEach(async ({ page }) => {
    journeyResult.duration_ms = Date.now() - journeyStartTime;
    console.log('Journey Test Results:', JSON.stringify(journeyResult, null, 2));

    // Verify journey completed within time limit (I-LEARN-050)
    expect(journeyResult.duration_ms).toBeLessThan(10 * 60 * 1000); // 10 minutes
  });

  test('@LEARN-006-001 Step 1: Student opens visualizer', async ({ page }) => {
    const stepStartTime = Date.now();
    const stepNumber = 1;

    // Navigate to LearningLab
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await expect(page.getByTestId('current-mode')).toHaveText('LearningLab');

    // Click "Start Exploring" button
    await page.getByTestId('start-exploring').click();

    // Then: Nervous system visualizer should open
    await expect(page.getByTestId('nervous-system-visualizer')).toBeVisible({ timeout: 5000 });

    // Verify real-time data streaming (I-LEARN-051)
    const dataStreamIndicator = page.getByTestId('data-streaming-indicator');
    await expect(dataStreamIndicator).toBeVisible();
    await expect(dataStreamIndicator).toHaveAttribute('data-streaming', 'true');

    // Verify display is understandable (child-friendly labels)
    await expect(page.getByTestId('tension-label')).toBeVisible();
    await expect(page.getByTestId('energy-label')).toBeVisible();
    await expect(page.getByTestId('mode-label')).toBeVisible();

    const screenshotPath = await captureStepScreenshot(page, stepNumber);

    journeyResult.steps.push({
      step_number: stepNumber,
      action: 'Opens visualizer',
      expected_response: 'Real-time nervous system display',
      actual_response: 'Visualizer opened with streaming data',
      learning_outcome: '"The robot has a \'brain\'"',
      result: 'pass',
      duration_ms: Date.now() - stepStartTime,
      screenshot_path: screenshotPath
    });
    journeyResult.artifacts.screenshots.push(screenshotPath);
  });

  test('@LEARN-006-002 Step 2: Observe idle robot baseline', async ({ page }) => {
    const stepStartTime = Date.now();
    const stepNumber = 2;

    // Setup: Open visualizer
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await page.getByTestId('start-exploring').click();
    await expect(page.getByTestId('nervous-system-visualizer')).toBeVisible();

    // Wait for robot to settle (no stimuli)
    await page.waitForTimeout(5000);

    // Then: All gauges should be stable
    const tension = await getTensionValue(page);
    expect(tension).toBeLessThan(0.3); // Low tension

    // Verify mode is calm
    const modeText = await page.getByTestId('current-reflex-mode').textContent();
    expect(modeText?.toLowerCase()).toContain('calm');

    // Verify gauges are not fluctuating wildly
    const tension1 = await getTensionValue(page);
    await page.waitForTimeout(1000);
    const tension2 = await getTensionValue(page);
    const variability = Math.abs(tension2 - tension1);
    expect(variability).toBeLessThan(0.1); // Stable

    const screenshotPath = await captureStepScreenshot(page, stepNumber);

    journeyResult.steps.push({
      step_number: stepNumber,
      action: 'Observes idle robot',
      expected_response: 'Calm, steady gauges',
      actual_response: `Tension: ${tension.toFixed(2)}, Mode: ${modeText}`,
      learning_outcome: '"It has a baseline state"',
      result: 'pass',
      duration_ms: Date.now() - stepStartTime,
      screenshot_path: screenshotPath
    });
    journeyResult.artifacts.screenshots.push(screenshotPath);
  });

  test('@LEARN-006-003 Step 3: Robot detects stimulus', async ({ page }) => {
    const stepStartTime = Date.now();
    const stepNumber = 3;

    // Setup: Open visualizer and observe baseline
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await page.getByTestId('start-exploring').click();
    await expect(page.getByTestId('nervous-system-visualizer')).toBeVisible();

    // Capture baseline tension
    const baselineTension = await getTensionValue(page);

    // When: Student waves hand near ultrasonic sensor
    await page.getByTestId('simulate-stimulus').click();
    await page.getByTestId('stimulus-type').selectOption('ultrasonic');
    await page.getByTestId('stimulus-distance').fill('20'); // 20cm approach
    await page.getByTestId('apply-stimulus').click();

    // Then: Tension gauge should spike visibly (I-LEARN-051)
    await page.waitForTimeout(100); // Response within 100ms
    const spikedTension = await getTensionValue(page);
    expect(spikedTension).toBeGreaterThan(baselineTension + 0.2); // Visible spike

    // Mode should change to "active" or "spike"
    const modeAfterSpike = await page.getByTestId('current-reflex-mode').textContent();
    expect(modeAfterSpike?.toLowerCase()).toMatch(/active|spike/);

    // Verify response was within 100ms
    const responseIndicator = await page.getByTestId('response-time').textContent();
    const responseTime = parseInt(responseIndicator || '0');
    expect(responseTime).toBeLessThan(100);

    const screenshotPath = await captureStepScreenshot(page, stepNumber);

    journeyResult.steps.push({
      step_number: stepNumber,
      action: 'Waves hand near ultrasonic sensor',
      expected_response: 'Spike in tension, mode change',
      actual_response: `Tension: ${baselineTension.toFixed(2)} → ${spikedTension.toFixed(2)}, Mode: ${modeAfterSpike}`,
      learning_outcome: '"It detected me!"',
      result: 'pass',
      duration_ms: Date.now() - stepStartTime,
      screenshot_path: screenshotPath,
      notes: `Response time: ${responseTime}ms`
    });
    journeyResult.artifacts.screenshots.push(screenshotPath);
  });

  test('@LEARN-006-004 Step 4: Robot calms down', async ({ page }) => {
    const stepStartTime = Date.now();
    const stepNumber = 4;

    // Setup: Open visualizer and trigger startle
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await page.getByTestId('start-exploring').click();
    await expect(page.getByTestId('nervous-system-visualizer')).toBeVisible();

    // Trigger stimulus
    await page.getByTestId('simulate-stimulus').click();
    await page.getByTestId('stimulus-type').selectOption('ultrasonic');
    await page.getByTestId('stimulus-distance').fill('20');
    await page.getByTestId('apply-stimulus').click();
    await page.waitForTimeout(100);

    const elevatedTension = await getTensionValue(page);
    expect(elevatedTension).toBeGreaterThan(0.4); // Verify it's elevated

    // When: Student removes hand and waits
    await page.getByTestId('remove-stimulus').click();

    // Then: Tension should gradually decrease
    const tensionReadings: number[] = [];
    for (let i = 0; i < 5; i++) {
      await page.waitForTimeout(1000);
      tensionReadings.push(await getTensionValue(page));
    }

    // Verify downward trend
    for (let i = 1; i < tensionReadings.length; i++) {
      expect(tensionReadings[i]).toBeLessThanOrEqual(tensionReadings[i - 1] + 0.05); // Decreasing or stable
    }

    // Mode should return to calm
    const finalMode = await page.getByTestId('current-reflex-mode').textContent();
    expect(finalMode?.toLowerCase()).toContain('calm');

    // Recovery should take 5-15 seconds
    const recoveryTime = Date.now() - stepStartTime;
    expect(recoveryTime).toBeGreaterThan(5000);
    expect(recoveryTime).toBeLessThan(15000);

    const screenshotPath = await captureStepScreenshot(page, stepNumber);

    journeyResult.steps.push({
      step_number: stepNumber,
      action: 'Removes hand and waits',
      expected_response: 'Tension gradually decreases',
      actual_response: `Tension: ${elevatedTension.toFixed(2)} → ${tensionReadings[4].toFixed(2)}`,
      learning_outcome: '"It calms down over time"',
      result: 'pass',
      duration_ms: Date.now() - stepStartTime,
      screenshot_path: screenshotPath,
      notes: `Recovery time: ${recoveryTime}ms`
    });
    journeyResult.artifacts.screenshots.push(screenshotPath);
  });

  test('@LEARN-006-005 Step 5: Open personality mixer', async ({ page }) => {
    const stepStartTime = Date.now();
    const stepNumber = 5;

    // Setup: Open visualizer
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await page.getByTestId('start-exploring').click();
    await expect(page.getByTestId('nervous-system-visualizer')).toBeVisible();

    // When: Student clicks "Adjust Personality" button
    await page.getByTestId('adjust-personality').click();

    // Then: Personality mixer panel should open
    await expect(page.getByTestId('personality-mixer-panel')).toBeVisible({ timeout: 2000 });

    // Sliders should be visible for all parameters
    await expect(page.getByTestId('slider-startle-sensitivity')).toBeVisible();
    await expect(page.getByTestId('slider-tension-decay')).toBeVisible();
    await expect(page.getByTestId('slider-curiosity-gain')).toBeVisible();
    await expect(page.getByTestId('slider-energy-baseline')).toBeVisible();
    await expect(page.getByTestId('slider-social-response')).toBeVisible();

    // Verify sliders have labels
    await expect(page.getByText('Startle Sensitivity')).toBeVisible();
    await expect(page.getByText('Tension Decay')).toBeVisible();

    const screenshotPath = await captureStepScreenshot(page, stepNumber);

    journeyResult.steps.push({
      step_number: stepNumber,
      action: 'Opens personality mixer',
      expected_response: 'Slider controls appear',
      actual_response: 'Personality mixer panel opened with 5 parameter sliders',
      learning_outcome: '"I can adjust its \'personality\'"',
      result: 'pass',
      duration_ms: Date.now() - stepStartTime,
      screenshot_path: screenshotPath
    });
    journeyResult.artifacts.screenshots.push(screenshotPath);
  });

  test('@LEARN-006-006 Step 6: Adjust startle sensitivity', async ({ page }) => {
    const stepStartTime = Date.now();
    const stepNumber = 6;

    // Setup: Open personality mixer
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await page.getByTestId('start-exploring').click();
    await page.getByTestId('adjust-personality').click();
    await expect(page.getByTestId('personality-mixer-panel')).toBeVisible();

    // When: Student moves startle_sensitivity slider to high (0.8)
    const slider = page.getByTestId('slider-startle-sensitivity');
    await slider.fill('0.8'); // Set to high sensitivity

    // Then: Slider value should update
    const sliderValue = await slider.inputValue();
    expect(parseFloat(sliderValue)).toBeCloseTo(0.8, 1);

    // Parameter should be transmitted to robot
    await page.getByTestId('apply-parameters').click();
    await expect(page.getByTestId('parameter-sync-status')).toHaveText('Synced', { timeout: 3000 });

    // Verify the value is reflected in the display
    const displayedValue = await page.getByTestId('startle-sensitivity-value').textContent();
    expect(parseFloat(displayedValue || '0')).toBeCloseTo(0.8, 1);

    const screenshotPath = await captureStepScreenshot(page, stepNumber);

    journeyResult.steps.push({
      step_number: stepNumber,
      action: 'Increases startle sensitivity to 0.8',
      expected_response: 'Slider updates, parameter transmitted',
      actual_response: `Slider value: ${sliderValue}, Status: Synced`,
      learning_outcome: '"Let\'s see what happens"',
      result: 'pass',
      duration_ms: Date.now() - stepStartTime,
      screenshot_path: screenshotPath
    });
    journeyResult.artifacts.screenshots.push(screenshotPath);
  });

  test('@LEARN-006-007 Step 7: Test with new sensitivity', async ({ page }) => {
    const stepStartTime = Date.now();
    const stepNumber = 7;

    // Setup: Adjust startle sensitivity to high
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await page.getByTestId('start-exploring').click();
    await page.getByTestId('adjust-personality').click();
    await page.getByTestId('slider-startle-sensitivity').fill('0.8');
    await page.getByTestId('apply-parameters').click();
    await expect(page.getByTestId('parameter-sync-status')).toHaveText('Synced');

    // Close mixer to see visualizer
    await page.getByTestId('close-mixer').click();

    // Capture baseline with high sensitivity
    await page.waitForTimeout(2000);
    const baselineTension = await getTensionValue(page);

    // When: Student waves hand (same stimulus as Step 3)
    await page.getByTestId('simulate-stimulus').click();
    await page.getByTestId('stimulus-type').selectOption('ultrasonic');
    await page.getByTestId('stimulus-distance').fill('20'); // Same distance as Step 3
    await page.getByTestId('apply-stimulus').click();
    await page.waitForTimeout(100);

    // Then: Startle response should be visibly LARGER
    const spikedTension = await getTensionValue(page);
    const tensionIncrease = spikedTension - baselineTension;

    // With high sensitivity, spike should be substantial
    expect(tensionIncrease).toBeGreaterThan(0.4); // Much larger than Step 3

    // Tension should spike higher than Step 3 baseline response
    expect(spikedTension).toBeGreaterThan(0.6); // High spike

    const screenshotPath = await captureStepScreenshot(page, stepNumber);

    journeyResult.steps.push({
      step_number: stepNumber,
      action: 'Waves hand with high sensitivity',
      expected_response: 'Much larger startle response',
      actual_response: `Tension spike: +${tensionIncrease.toFixed(2)} (peak: ${spikedTension.toFixed(2)})`,
      learning_outcome: '"Parameters change behavior!"',
      result: 'pass',
      duration_ms: Date.now() - stepStartTime,
      screenshot_path: screenshotPath
    });
    journeyResult.artifacts.screenshots.push(screenshotPath);
  });

  test('@LEARN-006-008 Step 8: Save custom personality', async ({ page }) => {
    const stepStartTime = Date.now();
    const stepNumber = 8;

    // Setup: Open mixer and adjust parameters
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await page.getByTestId('start-exploring').click();
    await page.getByTestId('adjust-personality').click();
    await page.getByTestId('slider-startle-sensitivity').fill('0.8');
    await page.getByTestId('slider-curiosity-gain').fill('0.6');

    // When: Student clicks "Save Personality"
    await page.getByTestId('save-personality').click();

    // And: Enters name "My First AI"
    const nameInput = page.getByTestId('personality-name-input');
    await nameInput.fill('My First AI');

    // And: Clicks confirm
    await page.getByTestId('confirm-save').click();

    // Then: Personality should be saved
    await expect(page.getByTestId('save-success-message')).toBeVisible({ timeout: 3000 });
    await expect(page.getByTestId('save-success-message')).toContainText('saved');

    // Verify personality appears in saved list
    await page.getByTestId('personality-library').click();
    await expect(page.getByText('My First AI')).toBeVisible();

    const screenshotPath = await captureStepScreenshot(page, stepNumber);

    journeyResult.steps.push({
      step_number: stepNumber,
      action: 'Saves custom personality',
      expected_response: 'Personality saved successfully',
      actual_response: 'Personality "My First AI" saved and appears in library',
      learning_outcome: '"I designed an AI"',
      result: 'pass',
      duration_ms: Date.now() - stepStartTime,
      screenshot_path: screenshotPath
    });
    journeyResult.artifacts.screenshots.push(screenshotPath);
  });

  test('@LEARN-006-009 Complete first experiment journey', async ({ page }) => {
    const journeyStartTime = Date.now();

    // Execute all 8 steps in sequence
    const steps: Array<{ name: string; action: () => Promise<void> }> = [
      {
        name: 'Step 1: Open visualizer',
        action: async () => {
          await page.getByTestId('mode-selector').click();
          await page.getByTestId('mode-learning-lab').click();
          await page.getByTestId('start-exploring').click();
          await expect(page.getByTestId('nervous-system-visualizer')).toBeVisible();
        }
      },
      {
        name: 'Step 2: Observe baseline',
        action: async () => {
          await page.waitForTimeout(5000);
          const tension = await getTensionValue(page);
          expect(tension).toBeLessThan(0.3);
        }
      },
      {
        name: 'Step 3: Detect stimulus',
        action: async () => {
          await page.getByTestId('simulate-stimulus').click();
          await page.getByTestId('stimulus-type').selectOption('ultrasonic');
          await page.getByTestId('stimulus-distance').fill('20');
          await page.getByTestId('apply-stimulus').click();
          await page.waitForTimeout(100);
          const tension = await getTensionValue(page);
          expect(tension).toBeGreaterThan(0.4);
        }
      },
      {
        name: 'Step 4: Robot calms',
        action: async () => {
          await page.getByTestId('remove-stimulus').click();
          await page.waitForTimeout(5000);
          const mode = await page.getByTestId('current-reflex-mode').textContent();
          expect(mode?.toLowerCase()).toContain('calm');
        }
      },
      {
        name: 'Step 5: Open mixer',
        action: async () => {
          await page.getByTestId('adjust-personality').click();
          await expect(page.getByTestId('personality-mixer-panel')).toBeVisible();
        }
      },
      {
        name: 'Step 6: Adjust sensitivity',
        action: async () => {
          await page.getByTestId('slider-startle-sensitivity').fill('0.8');
          await page.getByTestId('apply-parameters').click();
          await expect(page.getByTestId('parameter-sync-status')).toHaveText('Synced');
        }
      },
      {
        name: 'Step 7: Test new sensitivity',
        action: async () => {
          await page.getByTestId('close-mixer').click();
          await page.getByTestId('simulate-stimulus').click();
          await page.getByTestId('stimulus-type').selectOption('ultrasonic');
          await page.getByTestId('stimulus-distance').fill('20');
          await page.getByTestId('apply-stimulus').click();
          await page.waitForTimeout(100);
          const tension = await getTensionValue(page);
          expect(tension).toBeGreaterThan(0.6);
        }
      },
      {
        name: 'Step 8: Save personality',
        action: async () => {
          await page.getByTestId('adjust-personality').click();
          await page.getByTestId('save-personality').click();
          await page.getByTestId('personality-name-input').fill('My First AI');
          await page.getByTestId('confirm-save').click();
          await expect(page.getByTestId('save-success-message')).toBeVisible();
        }
      }
    ];

    // Execute each step
    for (const step of steps) {
      console.log(`Executing: ${step.name}`);
      await step.action();
    }

    // Verify journey completed within time limit (I-LEARN-050)
    const totalTime = Date.now() - journeyStartTime;
    expect(totalTime).toBeLessThan(10 * 60 * 1000); // Under 10 minutes

    console.log(`Journey completed in ${(totalTime / 1000).toFixed(1)}s`);
  });

  test('@LEARN-006-010 Journey is accessible (WCAG 2.1 AA)', async ({ page }) => {
    // Navigate through key journey steps
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await page.getByTestId('start-exploring').click();
    await expect(page.getByTestId('nervous-system-visualizer')).toBeVisible();

    // Run accessibility audit on visualizer
    const visualizerResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();

    expect(visualizerResults.violations).toHaveLength(0);

    // Open personality mixer
    await page.getByTestId('adjust-personality').click();
    await expect(page.getByTestId('personality-mixer-panel')).toBeVisible();

    // Run accessibility audit on mixer
    const mixerResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();

    expect(mixerResults.violations).toHaveLength(0);

    // Calculate accessibility score
    const totalChecks = visualizerResults.passes.length + mixerResults.passes.length;
    const totalViolations = visualizerResults.violations.length + mixerResults.violations.length;
    const accessibilityScore = ((totalChecks - totalViolations) / totalChecks) * 100;

    journeyResult.accessibility_score = accessibilityScore;
    console.log(`Accessibility score: ${accessibilityScore.toFixed(1)}%`);

    // Verify keyboard navigation (I-LEARN-053)
    await page.keyboard.press('Tab'); // Should focus first interactive element
    const focusedElement = await page.locator(':focus').getAttribute('data-testid');
    expect(focusedElement).toBeTruthy();

    // Verify focus indicators are visible
    const focusedElementBox = await page.locator(':focus').boundingBox();
    expect(focusedElementBox).toBeTruthy();

    // Verify screen reader compatibility
    const sliderLabel = await page.getByTestId('slider-startle-sensitivity').getAttribute('aria-label');
    expect(sliderLabel).toBeTruthy();

    // Verify color contrast
    const contrastIssues = visualizerResults.violations.filter(v =>
      v.id === 'color-contrast' || v.id === 'color-contrast-enhanced'
    );
    expect(contrastIssues).toHaveLength(0);
  });

  test('@LEARN-006-011 Journey works on tablet (768px)', async ({ page, browserName }) => {
    // Set tablet viewport (I-LEARN-054)
    await page.setViewportSize({ width: 768, height: 1024 });

    // Navigate through journey
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await page.getByTestId('start-exploring').click();
    await expect(page.getByTestId('nervous-system-visualizer')).toBeVisible();

    // Verify no horizontal scrolling required
    const hasHorizontalScroll = await page.evaluate(() => document.body.scrollWidth > window.innerWidth);
    expect(hasHorizontalScroll).toBe(false);

    // Verify all UI elements are touchable (min 44x44 touch target)
    const startButton = page.getByTestId('start-exploring');
    const buttonBox = await startButton.boundingBox();
    expect(buttonBox?.width).toBeGreaterThanOrEqual(44);
    expect(buttonBox?.height).toBeGreaterThanOrEqual(44);

    // Open personality mixer
    await page.getByTestId('adjust-personality').click();
    await expect(page.getByTestId('personality-mixer-panel')).toBeVisible();

    // Verify sliders work with touch
    const slider = page.getByTestId('slider-startle-sensitivity');
    const sliderBox = await slider.boundingBox();

    if (sliderBox) {
      // Simulate touch drag
      await page.touchscreen.tap(sliderBox.x + sliderBox.width * 0.8, sliderBox.y + sliderBox.height / 2);

      const sliderValue = await slider.inputValue();
      expect(parseFloat(sliderValue)).toBeGreaterThan(0.5); // Moved to higher position
    }

    // Verify touch targets are properly spaced
    const buttons = await page.getByRole('button').all();
    for (let i = 0; i < buttons.length - 1; i++) {
      const box1 = await buttons[i].boundingBox();
      const box2 = await buttons[i + 1].boundingBox();

      if (box1 && box2) {
        const spacing = Math.abs(box1.y - box2.y) || Math.abs(box1.x - box2.x);
        expect(spacing).toBeGreaterThanOrEqual(8); // At least 8px spacing
      }
    }
  });
});

// Additional test suite for cross-browser compatibility
test.describe('J-LEARN-FIRST-EXPERIMENT: Cross-Browser Tests', () => {
  test('works on Chrome', async ({ page, browserName }) => {
    test.skip(browserName !== 'chromium', 'Chrome-specific test');

    await page.goto('/');
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await page.getByTestId('start-exploring').click();
    await expect(page.getByTestId('nervous-system-visualizer')).toBeVisible();
  });

  test('works on Firefox', async ({ page, browserName }) => {
    test.skip(browserName !== 'firefox', 'Firefox-specific test');

    await page.goto('/');
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await page.getByTestId('start-exploring').click();
    await expect(page.getByTestId('nervous-system-visualizer')).toBeVisible();
  });

  test('works on Safari', async ({ page, browserName }) => {
    test.skip(browserName !== 'webkit', 'Safari-specific test');

    await page.goto('/');
    await page.getByTestId('mode-selector').click();
    await page.getByTestId('mode-learning-lab').click();
    await page.getByTestId('start-exploring').click();
    await expect(page.getByTestId('nervous-system-visualizer')).toBeVisible();
  });
});
