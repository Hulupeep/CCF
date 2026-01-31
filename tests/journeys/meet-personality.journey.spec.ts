/**
 * Journey Test: J-PERS-MEET-PERSONALITY
 * Issue: #29 - STORY-PERS-007: Meet Personality Journey Test
 * Contract: J-PERS-MEET-PERSONALITY
 *
 * Full 6-step journey test for experiencing personality differences:
 * 1. Opens personality menu
 * 2. Selects 'Nervous Nellie' (Timid preset)
 * 3. Hand approach triggers nervous response
 * 4. Gentle speech triggers recovery
 * 5. Switch to 'Bouncy Betty' (Energetic preset)
 * 6. Observe high-energy idle behavior
 *
 * Invariants: I-PERS-007, I-PERS-020, I-PERS-021, I-PERS-022
 */

import { test, expect } from '@playwright/test';

/**
 * Data contract for personality journey test
 */
interface JourneyTest {
  id: string;
  name: string;
  criticality: 'CRITICAL' | 'IMPORTANT' | 'NICE_TO_HAVE';
  preconditions: string[];
  steps: JourneyStep[];
  expected_outcome: string;
}

interface JourneyStep {
  step_number: number;
  user_action: string;
  robot_response: string;
  verification: string;
}

const MEET_PERSONALITY_JOURNEY: JourneyTest = {
  id: 'J-PERS-MEET-PERSONALITY',
  name: 'First time experiencing a robot\'s personality',
  criticality: 'CRITICAL',
  preconditions: [
    'mBot2 connected via Companion app',
    'Default personality loaded',
  ],
  steps: [
    {
      step_number: 1,
      user_action: 'Opens personality menu',
      robot_response: 'List of 5 presets shown',
      verification: 'UI shows all presets',
    },
    {
      step_number: 2,
      user_action: 'Selects "Nervous Nellie"',
      robot_response: 'Robot shudders, LEDs flicker yellow',
      verification: 'Transition animation',
    },
    {
      step_number: 3,
      user_action: 'Moves hand toward robot',
      robot_response: 'Robot backs away nervously',
      verification: 'Protect mode triggers',
    },
    {
      step_number: 4,
      user_action: 'Speaks gently',
      robot_response: 'Robot cautiously approaches',
      verification: 'Recovery behavior',
    },
    {
      step_number: 5,
      user_action: 'Selects "Bouncy Betty"',
      robot_response: 'Robot perks up, starts moving',
      verification: 'Energy increase visible',
    },
    {
      step_number: 6,
      user_action: 'Does nothing',
      robot_response: 'Robot circles, wiggles, seeks attention',
      verification: 'High baseline energy',
    },
  ],
  expected_outcome: 'User clearly sees the difference between personalities and feels the robot has a distinct character.',
};

test.describe('J-PERS-MEET-PERSONALITY: Meet Personality Journey', () => {
  test.beforeEach(async ({ page }) => {
    // Ensure mBot is connected and ready
    await page.goto('/');
    await expect(page.getByTestId('mbot-status')).toHaveText('Connected', { timeout: 10000 });
  });

  test('@PERS-007 @step-1 Step 1: Opens personality menu', async ({ page }) => {
    // When: User opens personality menu
    await page.getByTestId('personality-menu-button').click();

    // Then: List of 5 presets shown
    await expect(page.getByTestId('personality-menu')).toBeVisible();

    // Verify all 5 presets are present (mapping to actual preset names)
    const presetNames = [
      'Curious Cleo',  // Curious preset
      'Nervous Nellie', // Timid preset
      'Chill Charlie',  // Calm preset
      'Bouncy Betty',   // Energetic preset
      'Grumpy Gus',     // Grumpy preset
    ];

    for (const preset of presetNames) {
      await expect(page.getByTestId(`personality-preset-${preset.toLowerCase().replace(' ', '-')}`)).toBeVisible();
    }

    // Each preset should have name, icon, and description
    await expect(page.getByTestId('personality-preset-curious-cleo')).toContainText('Curious Cleo');
    await expect(page.getByTestId('personality-icon-curious-cleo')).toBeVisible();
    await expect(page.getByTestId('personality-description-curious-cleo')).toContainText(/curiosity|investigate/i);
  });

  test('@PERS-007 @step-2 Step 2: Selects Nervous Nellie', async ({ page }) => {
    // Given: Personality menu is open
    await page.getByTestId('personality-menu-button').click();
    await expect(page.getByTestId('personality-menu')).toBeVisible();

    // Record initial state
    const initialPersonality = await page.getByTestId('current-personality').textContent();

    // When: User selects 'Nervous Nellie' (Timid preset)
    await page.getByTestId('personality-preset-nervous-nellie').click();

    // Then: Robot shudders (transition animation)
    await expect(page.getByTestId('transition-animation')).toBeVisible({ timeout: 1000 });

    // LEDs flicker yellow (warning color during transition)
    await expect(page.getByTestId('led-color')).toContainText(/yellow|warning/i, { timeout: 2000 });

    // Transition completes within 3 seconds
    await expect(page.getByTestId('current-personality')).toHaveText('Nervous Nellie', { timeout: 3000 });

    // Verify personality changed
    expect(initialPersonality).not.toBe('Nervous Nellie');
  });

  test('@PERS-007 @step-3 Step 3: Hand approach triggers nervous response', async ({ page }) => {
    // Given: Active personality is 'Nervous Nellie'
    await page.getByTestId('personality-menu-button').click();
    await page.getByTestId('personality-preset-nervous-nellie').click();
    await expect(page.getByTestId('current-personality')).toHaveText('Nervous Nellie', { timeout: 3000 });

    // Record baseline tension
    const baselineTension = await page.getByTestId('tension-level').textContent();
    expect(baselineTension).toContain('High'); // Timid has high tension baseline

    // When: User moves hand toward robot (simulate close obstacle)
    await page.getByTestId('simulate-obstacle').click();
    await page.getByTestId('obstacle-distance').fill('5'); // 5cm - very close
    await page.getByTestId('apply-simulation').click();

    // Then: Robot backs away nervously
    await expect(page.getByTestId('motor-left')).toContainText('-', { timeout: 2000 }); // Negative = backing up
    await expect(page.getByTestId('motor-right')).toContainText('-', { timeout: 2000 });

    // Protect mode triggers
    await expect(page.getByTestId('behavior-mode')).toHaveText(/protect|defensive/i, { timeout: 3000 });

    // Tension visibly elevated
    await expect(page.getByTestId('tension-level')).toContainText(/high|elevated/i);
  });

  test('@PERS-007 @step-4 Step 4: Gentle speech triggers recovery', async ({ page }) => {
    // Given: Robot is in nervous state
    await page.getByTestId('personality-menu-button').click();
    await page.getByTestId('personality-preset-nervous-nellie').click();
    await expect(page.getByTestId('current-personality')).toHaveText('Nervous Nellie', { timeout: 3000 });

    // Trigger nervous response
    await page.getByTestId('simulate-obstacle').click();
    await page.getByTestId('obstacle-distance').fill('5');
    await page.getByTestId('apply-simulation').click();
    await expect(page.getByTestId('behavior-mode')).toHaveText(/protect|defensive/i, { timeout: 3000 });

    // Record high tension state
    const tensionBefore = await page.getByTestId('tension-value').textContent();
    const tensionBeforeNum = parseFloat(tensionBefore || '0');
    expect(tensionBeforeNum).toBeGreaterThan(0.6); // High tension

    // When: User speaks gently (simulate sound stimulus)
    await page.getByTestId('simulate-sound').click();
    await page.getByTestId('sound-level').fill('30'); // Gentle sound
    await page.getByTestId('sound-type').selectOption('voice'); // Friendly voice
    await page.getByTestId('apply-sound').click();

    // Then: Robot cautiously approaches
    await expect(page.getByTestId('behavior-mode')).toHaveText(/explore|calm/i, { timeout: 5000 });

    // Tension gradually decreases
    await page.waitForTimeout(2000); // Allow nervous system to respond
    const tensionAfter = await page.getByTestId('tension-value').textContent();
    const tensionAfterNum = parseFloat(tensionAfter || '0');
    expect(tensionAfterNum).toBeLessThan(tensionBeforeNum); // Tension decreased

    // Recovery behavior visible (forward movement)
    const motorLeft = await page.getByTestId('motor-left').textContent();
    expect(motorLeft).not.toContain('-'); // Not backing up anymore
  });

  test('@PERS-007 @step-5 Step 5: Switch to Bouncy Betty', async ({ page }) => {
    // Given: Active personality is 'Nervous Nellie'
    await page.getByTestId('personality-menu-button').click();
    await page.getByTestId('personality-preset-nervous-nellie').click();
    await expect(page.getByTestId('current-personality')).toHaveText('Nervous Nellie', { timeout: 3000 });

    // Record low energy state
    const energyBefore = await page.getByTestId('energy-level').textContent();

    // When: User selects 'Bouncy Betty' (Energetic preset)
    await page.getByTestId('personality-menu-button').click();
    await page.getByTestId('personality-preset-bouncy-betty').click();

    // Then: Robot perks up
    await expect(page.getByTestId('transition-animation')).toBeVisible({ timeout: 1000 });

    // Starts moving energetically
    await expect(page.getByTestId('current-personality')).toHaveText('Bouncy Betty', { timeout: 3000 });

    // Energy increase clearly visible
    await page.waitForTimeout(1000); // Allow transition to settle
    const energyAfter = await page.getByTestId('energy-level').textContent();
    expect(energyAfter).toContain('High'); // Energetic has high baseline
    expect(energyAfter).not.toBe(energyBefore);

    // Motor activity increased
    const motorSpeed = await page.getByTestId('motor-speed').textContent();
    const speedNum = parseFloat(motorSpeed || '0');
    expect(speedNum).toBeGreaterThan(50); // High movement expressiveness
  });

  test('@PERS-007 @step-6 Step 6: Bouncy Betty idle behavior', async ({ page }) => {
    // Given: Active personality is 'Bouncy Betty'
    await page.getByTestId('personality-menu-button').click();
    await page.getByTestId('personality-preset-bouncy-betty').click();
    await expect(page.getByTestId('current-personality')).toHaveText('Bouncy Betty', { timeout: 3000 });

    // Clear all stimuli
    await page.getByTestId('clear-all-stimuli').click();

    // When: User does nothing for 10 seconds
    await page.waitForTimeout(10000);

    // Then: Robot circles and wiggles (high baseline energy)
    const behaviorLog = await page.getByTestId('behavior-log').textContent();
    expect(behaviorLog).toMatch(/spin|circle|wiggle|move/i); // Active behaviors

    // Seeks attention (curiosity behaviors)
    expect(behaviorLog).toMatch(/explore|investigate|approach/i);

    // High baseline energy maintained
    await expect(page.getByTestId('energy-level')).toContainText('High');

    // Verify energy hasn't dropped below threshold
    const energyValue = await page.getByTestId('energy-value').textContent();
    const energyNum = parseFloat(energyValue || '0');
    expect(energyNum).toBeGreaterThan(0.6); // Maintains high energy
  });

  test('@full-journey Complete Meet Personality Journey (all 6 steps)', async ({ page }) => {
    // Execute complete journey as defined in MEET_PERSONALITY_JOURNEY

    // Step 1: Opens personality menu
    await page.getByTestId('personality-menu-button').click();
    await expect(page.getByTestId('personality-menu')).toBeVisible();
    await expect(page.getByTestId('personality-preset-nervous-nellie')).toBeVisible();

    // Step 2: Selects Nervous Nellie
    await page.getByTestId('personality-preset-nervous-nellie').click();
    await expect(page.getByTestId('current-personality')).toHaveText('Nervous Nellie', { timeout: 3000 });
    await expect(page.getByTestId('led-color')).toContainText(/yellow|warning/i);

    // Step 3: Hand approach triggers nervous response
    await page.getByTestId('simulate-obstacle').click();
    await page.getByTestId('obstacle-distance').fill('5');
    await page.getByTestId('apply-simulation').click();
    await expect(page.getByTestId('behavior-mode')).toHaveText(/protect|defensive/i, { timeout: 3000 });

    // Step 4: Gentle speech triggers recovery
    await page.getByTestId('simulate-sound').click();
    await page.getByTestId('sound-level').fill('30');
    await page.getByTestId('sound-type').selectOption('voice');
    await page.getByTestId('apply-sound').click();
    await expect(page.getByTestId('behavior-mode')).toHaveText(/explore|calm/i, { timeout: 5000 });

    // Step 5: Switch to Bouncy Betty
    await page.getByTestId('personality-menu-button').click();
    await page.getByTestId('personality-preset-bouncy-betty').click();
    await expect(page.getByTestId('current-personality')).toHaveText('Bouncy Betty', { timeout: 3000 });
    await expect(page.getByTestId('energy-level')).toContainText('High');

    // Step 6: Observe high-energy idle behavior
    await page.getByTestId('clear-all-stimuli').click();
    await page.waitForTimeout(10000);
    const behaviorLog = await page.getByTestId('behavior-log').textContent();
    expect(behaviorLog).toMatch(/spin|circle|wiggle|move/i);

    // Verify expected outcome: User sees distinct personality differences
    const journeyComplete = await page.getByTestId('journey-summary');
    await expect(journeyComplete).toContainText('distinct');
  });

  test('@all-presets Test all 5 preset personalities are distinguishable', async ({ page }) => {
    const presets = [
      { name: 'Curious Cleo', trait: 'High curiosity' },
      { name: 'Nervous Nellie', trait: 'High tension' },
      { name: 'Chill Charlie', trait: 'Low reactivity' },
      { name: 'Bouncy Betty', trait: 'High energy' },
      { name: 'Grumpy Gus', trait: 'Low coherence' },
    ];

    for (const preset of presets) {
      // Switch to personality
      await page.getByTestId('personality-menu-button').click();
      const presetId = preset.name.toLowerCase().replace(' ', '-');
      await page.getByTestId(`personality-preset-${presetId}`).click();

      // Wait for transition
      await expect(page.getByTestId('current-personality')).toHaveText(preset.name, { timeout: 3000 });

      // Clear stimuli and observe for 30 seconds (I-PERS-007 requirement)
      await page.getByTestId('clear-all-stimuli').click();
      await page.waitForTimeout(30000);

      // Record observable characteristics
      const behaviorLog = await page.getByTestId('behavior-log').textContent();
      const tensionLevel = await page.getByTestId('tension-level').textContent();
      const energyLevel = await page.getByTestId('energy-level').textContent();

      // Verify personality traits are observable
      if (preset.name === 'Curious Cleo') {
        expect(behaviorLog).toMatch(/explore|investigate/i);
      } else if (preset.name === 'Nervous Nellie') {
        expect(tensionLevel).toContain('High');
      } else if (preset.name === 'Chill Charlie') {
        expect(behaviorLog).toMatch(/calm|still|low/i);
      } else if (preset.name === 'Bouncy Betty') {
        expect(energyLevel).toContain('High');
        expect(behaviorLog).toMatch(/spin|move|active/i);
      } else if (preset.name === 'Grumpy Gus') {
        expect(behaviorLog).toMatch(/avoid|withdraw/i);
      }

      // Take screenshot for visual documentation
      await page.screenshot({
        path: `test-results/personality-${presetId}-30sec.png`,
        fullPage: true
      });
    }
  });

  test('@I-PERS-022 Transitions are smooth and visible', async ({ page }) => {
    // Test smooth transition between personalities (no jumps)
    await page.getByTestId('personality-menu-button').click();
    await page.getByTestId('personality-preset-nervous-nellie').click();
    await expect(page.getByTestId('current-personality')).toHaveText('Nervous Nellie', { timeout: 3000 });

    // Record initial tension
    const tensionBefore = await page.getByTestId('tension-value').textContent();
    const tensionBeforeNum = parseFloat(tensionBefore || '0');

    // Switch to opposite personality
    await page.getByTestId('personality-menu-button').click();
    await page.getByTestId('personality-preset-bouncy-betty').click();

    // Verify transition is visible
    await expect(page.getByTestId('transition-animation')).toBeVisible({ timeout: 1000 });

    // Sample tension during transition to verify smoothness
    const tensionSamples: number[] = [];
    for (let i = 0; i < 10; i++) {
      await page.waitForTimeout(300);
      const tensionText = await page.getByTestId('tension-value').textContent();
      tensionSamples.push(parseFloat(tensionText || '0'));
    }

    // Verify no sudden jumps (max delta < 0.15 per sample)
    for (let i = 1; i < tensionSamples.length; i++) {
      const delta = Math.abs(tensionSamples[i] - tensionSamples[i - 1]);
      expect(delta).toBeLessThan(0.15); // Smooth transition
    }

    // Verify transition completed
    await expect(page.getByTestId('current-personality')).toHaveText('Bouncy Betty', { timeout: 5000 });
  });

  test('personality persists across page reloads', async ({ page }) => {
    // Set personality to Energetic
    await page.getByTestId('personality-menu-button').click();
    await page.getByTestId('personality-preset-bouncy-betty').click();
    await expect(page.getByTestId('current-personality')).toHaveText('Bouncy Betty', { timeout: 3000 });

    // Reload page
    await page.reload();
    await expect(page.getByTestId('mbot-status')).toHaveText('Connected', { timeout: 10000 });

    // Should remember Bouncy Betty personality
    await expect(page.getByTestId('current-personality')).toHaveText('Bouncy Betty');
    await expect(page.getByTestId('energy-level')).toContainText('High');
  });
});
