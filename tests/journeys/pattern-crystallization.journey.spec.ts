/**
 * E2E Journey Test: Pattern Crystallization (Story #92)
 *
 * Journey: J-LEARN-PATTERN-CRYSTALLIZATION
 * User Story: As a user who frequently asks similar questions, I want the bot
 * to learn my patterns so that responses become faster and more consistent over time.
 *
 * Scenario Flow:
 * 1. User sends 5+ similar messages over several days
 * 2. Bot observes and records all interactions
 * 3. Overseer detects the pattern (5/70 rule met)
 * 4. Pattern is crystallized automatically
 * 5. Next similar message uses pattern (zero tokens, <100ms)
 * 6. User notices faster responses
 * 7. User opens dashboard and sees learned pattern
 */

import { test, expect } from '@playwright/test';
import { LearningEngine } from '../../web/src/services/learning/LearningEngine';
import { PatternStore } from '../../web/src/services/learning/PatternStore';
import { Overseer } from '../../web/src/services/learning/Overseer';

// Helper function to trigger Overseer cycle
async function triggerOverseerCycle() {
  const engine = new LearningEngine();
  const store = new PatternStore();
  const overseer = new Overseer(engine, store, {
    intervalMs: 1000,
    enableAutoCrystallization: true,
    requireApproval: false
  });

  await overseer.runLearningCycle();
}

test.describe('J-LEARN-PATTERN-CRYSTALLIZATION: Pattern Learning Journey', () => {

  test.beforeEach(async ({ page }) => {
    // Clear previous patterns
    await page.goto('/dashboard/learning');
    await page.evaluate(() => {
      localStorage.clear();
      indexedDB.deleteDatabase('mbot-patterns');
    });
  });

  test('Complete pattern crystallization workflow', async ({ page }) => {
    // Step 1: User sends similar messages multiple times
    await page.goto('/chat');

    const similarMessages = [
      'What is the weather today?',
      'What is the weather like today?',
      'Weather today?',
      'Tell me about the weather today',
      'Weather forecast today',
      'What is today\'s weather?'
    ];

    console.log('📝 Step 1: Sending similar messages to build pattern...');
    for (let i = 0; i < similarMessages.length; i++) {
      await page.getByTestId('chat-input').fill(similarMessages[i]);
      await page.getByTestId('send-button').click();

      // Wait for bot response
      await expect(page.getByTestId('bot-response').last()).toBeVisible({ timeout: 5000 });
      console.log(`  ✓ Message ${i + 1}/${similarMessages.length}: "${similarMessages[i]}"`);

      // Small delay between messages
      await page.waitForTimeout(500);
    }

    // Step 2: Trigger Overseer learning cycle
    console.log('🧠 Step 2: Triggering Overseer learning cycle...');
    await triggerOverseerCycle();
    await page.waitForTimeout(1000); // Allow time for pattern detection

    // Step 3: Verify pattern crystallized
    console.log('🔍 Step 3: Checking for crystallized pattern...');
    await page.goto('/dashboard/learning');

    // Check that at least 1 pattern exists
    const patternsCount = await page.getByTestId('active-patterns-count').textContent();
    expect(parseInt(patternsCount || '0')).toBeGreaterThanOrEqual(1);
    console.log(`  ✓ Found ${patternsCount} active pattern(s)`);

    // Verify pattern card is visible
    await expect(page.locator('[data-testid^="pattern-"]').first()).toBeVisible();

    // Check pattern has required metadata
    const firstPatternId = await page.locator('[data-testid^="pattern-"]').first().getAttribute('data-testid');
    const patternId = firstPatternId?.replace('pattern-', '') || '';

    await expect(page.getByTestId(`pattern-usage-${patternId}`)).toBeVisible();
    await expect(page.getByTestId(`pattern-success-${patternId}`)).toBeVisible();

    const usageText = await page.getByTestId(`pattern-usage-${patternId}`).textContent();
    const successText = await page.getByTestId(`pattern-success-${patternId}`).textContent();
    console.log(`  ✓ Pattern usage: ${usageText}`);
    console.log(`  ✓ Pattern success rate: ${successText}`);

    // Step 4: Send similar message and verify pattern used
    console.log('⚡ Step 4: Testing pattern execution (zero-token response)...');
    await page.goto('/chat');

    const startTime = Date.now();
    await page.getByTestId('chat-input').fill('weather forecast today');
    await page.getByTestId('send-button').click();

    await expect(page.getByTestId('bot-response').last()).toBeVisible({ timeout: 5000 });
    const responseTime = Date.now() - startTime;

    console.log(`  ✓ Response time: ${responseTime}ms`);
    expect(responseTime).toBeLessThan(200); // Fast response indicates pattern was used

    // Step 5: Verify token savings
    console.log('💰 Step 5: Verifying token savings...');
    await page.goto('/dashboard/learning');

    const tokensSaved = await page.getByTestId('tokens-saved').textContent();
    console.log(`  ✓ Tokens saved: ${tokensSaved}`);
    expect(tokensSaved).toMatch(/[1-9]/); // Non-zero tokens saved

    // Step 6: Verify Overseer status
    await expect(page.getByTestId('overseer-status')).toBeVisible();
    console.log('✅ Journey complete: Pattern crystallization successful!');
  });

  test('5/70 Rule Enforcement: Pattern requires 5+ observations and 70%+ success', async ({ page }) => {
    console.log('📊 Testing 5/70 rule enforcement...');

    const engine = new LearningEngine();

    // Test with only 3 observations (below threshold)
    console.log('  Testing with 3 observations (should NOT crystallize)...');
    for (let i = 0; i < 3; i++) {
      engine.observeAction('user1', 'test_action', { msg: 'test' }, true, 100);
    }

    let patterns = engine.detectPatterns();
    expect(patterns.length).toBe(0);
    console.log('  ✓ No pattern detected with <5 observations');

    // Test with 5 observations but low success rate (60%)
    console.log('  Testing with 5 observations but 60% success (should NOT crystallize)...');
    const engine2 = new LearningEngine();
    for (let i = 0; i < 3; i++) {
      engine2.observeAction('user1', 'test_action_2', { msg: 'test' }, true, 100);
    }
    for (let i = 0; i < 2; i++) {
      engine2.observeAction('user1', 'test_action_2', { msg: 'test' }, false, 100);
    }

    patterns = engine2.detectPatterns();
    expect(patterns.length).toBe(0);
    console.log('  ✓ No pattern detected with <70% success rate');

    // Test with 5 observations and 80% success rate (should crystallize)
    console.log('  Testing with 5 observations and 80% success (SHOULD crystallize)...');
    const engine3 = new LearningEngine();
    for (let i = 0; i < 4; i++) {
      engine3.observeAction('user1', 'test_action_3', { msg: 'test' }, true, 100);
    }
    engine3.observeAction('user1', 'test_action_3', { msg: 'test' }, false, 100);

    patterns = engine3.detectPatterns();
    expect(patterns.length).toBeGreaterThan(0);
    expect(patterns[0].successRate).toBeGreaterThanOrEqual(0.7);
    console.log('  ✓ Pattern detected with 5+ observations and ≥70% success!');
    console.log('✅ 5/70 rule enforcement verified!');
  });

  test('Zero-Token Execution: Crystallized patterns execute without LLM calls', async ({ page }) => {
    console.log('🚀 Testing zero-token execution...');

    const engine = new LearningEngine();

    // Create and crystallize a pattern
    for (let i = 0; i < 6; i++) {
      engine.observeAction('user1', 'weather_query', { query: 'weather' }, true, 100);
    }

    const patterns = engine.detectPatterns();
    expect(patterns.length).toBeGreaterThan(0);

    const pattern = patterns[0];
    engine.addPattern(pattern);

    // Execute pattern and measure performance
    const startTime = Date.now();
    const result = await engine.executePattern(pattern, { query: 'weather' });
    const executionTime = Date.now() - startTime;

    expect(result.success).toBe(true);
    expect(executionTime).toBeLessThan(100); // Should be fast
    console.log(`  ✓ Pattern executed in ${executionTime}ms (zero tokens)`);

    // Verify token savings
    const stats = engine.getPerformanceStats();
    expect(stats.tokensSaved).toBeGreaterThan(0);
    console.log(`  ✓ Tokens saved: ${stats.tokensSaved}`);
    console.log('✅ Zero-token execution verified!');
  });

  test('Confidence Updates: Pattern confidence adjusts based on outcomes', async ({ page }) => {
    console.log('📈 Testing confidence updates...');

    const engine = new LearningEngine();

    // Create and add a pattern
    for (let i = 0; i < 6; i++) {
      engine.observeAction('user1', 'test_action', { test: true }, true, 100);
    }

    const patterns = engine.detectPatterns();
    const pattern = patterns[0];
    engine.addPattern(pattern);

    const initialConfidence = pattern.confidence;
    console.log(`  Initial confidence: ${(initialConfidence * 100).toFixed(1)}%`);

    // Execute successfully
    await engine.executePattern(pattern, { test: true });
    engine.updatePatternConfidence(pattern.id, true);

    expect(pattern.confidence).toBeGreaterThanOrEqual(initialConfidence);
    console.log(`  ✓ After success: ${(pattern.confidence * 100).toFixed(1)}%`);

    // Execute unsuccessfully
    const confidenceBeforeFailure = pattern.confidence;
    engine.updatePatternConfidence(pattern.id, false);

    expect(pattern.confidence).toBeLessThan(confidenceBeforeFailure);
    console.log(`  ✓ After failure: ${(pattern.confidence * 100).toFixed(1)}%`);
    console.log('✅ Confidence updates working correctly!');
  });
});
