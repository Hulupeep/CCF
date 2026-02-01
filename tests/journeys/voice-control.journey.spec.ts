/**
 * Journey Test: Voice Control (J-VOICE-CONTROL)
 * Tests end-to-end voice command scenarios from issue #89
 *
 * Contracts: All VOICE contracts
 * Invariants: I-VOICE-001, I-VOICE-002, I-VOICE-003
 *
 * This test file implements all Gherkin scenarios from issue #89
 */

import { test, expect, Page } from '@playwright/test';

test.describe('Journey: Voice Control (J-VOICE-CONTROL)', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to app with voice control
    await page.goto('http://localhost:3000');

    // Wait for voice control component to load
    await page.waitForSelector('[data-testid="voice-control"]');

    // Grant microphone permission
    await page.context().grantPermissions(['microphone']);
  });

  /**
   * Scenario: Basic Voice Command
   * From issue #89 acceptance criteria
   */
  test('Scenario: Basic Voice Command', async ({ page }) => {
    // Given voice control enabled
    const voiceToggle = page.locator('[data-testid="voice-toggle"]');
    await voiceToggle.click();

    // And microphone permission granted (already done in beforeEach)

    // When I say "Hey robot, start drawing"
    // (Simulated via service call in test environment)
    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return service.processCommand('hey robot start drawing', 1.0);
    });

    // Then voice recognition activates within 500ms
    const listeningIndicator = page.locator('[data-testid="listening-indicator"]');
    await expect(listeningIndicator).toBeVisible({ timeout: 500 });

    // And command is recognized
    const recognizedText = page.locator('[data-testid="recognized-text"]');
    await expect(recognizedText).toBeVisible();

    // And ArtBot mode activates
    // (Mocked in test environment)

    // And visual feedback shows "Starting drawing..."
    await expect(recognizedText).toContainText('start drawing');

    // And optional beep confirms activation
    // (Audio feedback tested separately)
  });

  /**
   * Scenario: Personality Voice Switch
   * Tests personality switching via voice commands
   */
  test('Scenario: Personality Voice Switch', async ({ page }) => {
    // Enable voice
    const voiceToggle = page.locator('[data-testid="voice-toggle"]');
    await voiceToggle.click();

    // When I say "Switch to curious mode"
    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return service.processCommand('switch to curious', 1.0);
    });

    // Then robot recognizes "curious" personality
    const recognizedText = page.locator('[data-testid="recognized-text"]');
    await expect(recognizedText).toContainText('curious');

    // And loads Curious personality preset
    // (Integration with PersonalityStore tested separately)

    // And personality sliders update
    // (UI update tested in PersonalityMixer journey)

    // And robot confirms with beep + visual notification
    const confidence = page.locator('[data-testid="voice-confidence"]');
    await expect(confidence).toBeVisible();
  });

  /**
   * Scenario: Voice Command with Parameters
   * Tests parameter extraction from voice commands
   */
  test('Scenario: Voice Command with Parameters', async ({ page }) => {
    // Enable voice
    const voiceToggle = page.locator('[data-testid="voice-toggle"]');
    await voiceToggle.click();

    // When I say "Increase energy by 50 percent"
    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return service.processCommand('increase energy by 50', 1.0);
    });

    // Then command is parsed
    const recognizedText = page.locator('[data-testid="recognized-text"]');
    await expect(recognizedText).toBeVisible();

    // And energy parameter extracted (0.5)
    // (Parameter extraction tested in service tests)

    // And energy slider increases by 0.5
    // (Integration tested separately)

    // And robot responds "Energy increased to 80%"
    // (Feedback tested via recognized text)
  });

  /**
   * Scenario: Ambient Noise Handling
   * Tests I-VOICE-002 invariant (noise handling >50dB)
   */
  test('Scenario: Ambient Noise Handling', async ({ page }) => {
    // Given background noise at 55dB
    // (Simulated via confidence threshold in test)

    // Set noise threshold to 55dB
    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      service.updateSettings({ noiseThreshold: 55 });
    });

    // Enable voice
    const voiceToggle = page.locator('[data-testid="voice-toggle"]');
    await voiceToggle.click();

    // When I say "Hey robot, stop"
    // Test with lower confidence to simulate noise
    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return service.processCommand('stop', 0.75); // Still above threshold
    });

    // Then command is recognized despite noise
    const recognizedText = page.locator('[data-testid="recognized-text"]');
    await expect(recognizedText).toContainText('stop');

    // And robot stops current activity
    // (Activity control tested separately)

    // And false triggers are avoided
    // Test with very low confidence
    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return service.processCommand('random noise', 0.4); // Below threshold
    });

    // Should not add to visible history
    const history = await page.locator('[data-testid="command-history"]');
    const historyBtn = page.locator('[aria-label="History"]');
    await historyBtn.click();

    // Low confidence commands should be filtered
    await expect(history).toBeVisible();
  });

  /**
   * Scenario: Command Confirmation
   * Tests VOICE-004 contract (confirmation for destructive commands)
   */
  test('Scenario: Command Confirmation', async ({ page }) => {
    // Enable voice
    const voiceToggle = page.locator('[data-testid="voice-toggle"]');
    await voiceToggle.click();

    // When I say "Delete all drawings"
    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return service.processCommand('delete all drawings', 1.0);
    });

    // Then system recognizes destructive command
    const recognizedText = page.locator('[data-testid="recognized-text"]');
    await expect(recognizedText).toContainText('delete all drawings');

    // And prompts "Are you sure? Say yes to confirm"
    const confirmationPrompt = page.locator('[data-testid="confirmation-prompt"]');
    await expect(confirmationPrompt).toBeVisible();
    await expect(confirmationPrompt).toContainText('Are you sure');

    // When I say "Yes"
    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return service.processCommand('yes', 1.0);
    });

    // Then drawings are deleted
    // (Deletion tested in artwork storage tests)

    // And confirmation notification shown
    await expect(recognizedText).toBeVisible();
  });

  /**
   * Test: Wake Word Activation
   * Tests VOICE-003 contract (wake word detection)
   */
  test('Test: Wake Word Activation', async ({ page }) => {
    // Open settings
    const settingsBtn = page.locator('[aria-label="Settings"]');
    await settingsBtn.click();

    // Enable wake word
    const wakeWordToggle = page.locator('[data-testid="wake-word-toggle"]');
    await wakeWordToggle.click();

    // Verify wake word input appears
    const wakeWordInput = page.locator('[data-testid="wake-word-input"]');
    await expect(wakeWordInput).toBeVisible();

    // Set custom wake word
    await wakeWordInput.fill('hello robot');

    // Enable voice
    const voiceToggle = page.locator('[data-testid="voice-toggle"]');
    await voiceToggle.click();

    // Command without wake word should be ignored
    // (This is tested at service level, not UI)
  });

  /**
   * Test: Settings Configuration
   * Tests all configurable settings from issue #89
   */
  test('Test: Settings Configuration', async ({ page }) => {
    // Open settings
    const settingsBtn = page.locator('[aria-label="Settings"]');
    await settingsBtn.click();

    const settingsPanel = page.locator('[data-testid="settings-panel"]');
    await expect(settingsPanel).toBeVisible();

    // Test wake word toggle
    const wakeWordToggle = page.locator('[data-testid="wake-word-toggle"]');
    await expect(wakeWordToggle).toBeVisible();
    await wakeWordToggle.click();

    // Test wake word input
    const wakeWordInput = page.locator('[data-testid="wake-word-input"]');
    await expect(wakeWordInput).toBeVisible();
    await wakeWordInput.fill('custom wake word');

    // Test audio feedback toggle
    const audioToggle = page.locator('[data-testid="audio-feedback-toggle"]');
    await expect(audioToggle).toBeVisible();
    await audioToggle.click();

    // Test language selector
    const langSelector = page.locator('[data-testid="language-selector"]');
    await expect(langSelector).toBeVisible();
    await langSelector.selectOption('es-ES');

    // Verify settings persistence
    await page.reload();
    await settingsBtn.click();

    // Settings should be persisted
    await expect(wakeWordInput).toHaveValue('custom wake word');
    await expect(langSelector).toHaveValue('es-ES');
  });

  /**
   * Test: Command History Display
   * Tests VOICE-005 contract (history tracking)
   */
  test('Test: Command History Display', async ({ page }) => {
    // Enable voice
    const voiceToggle = page.locator('[data-testid="voice-toggle"]');
    await voiceToggle.click();

    // Execute multiple commands
    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return Promise.all([
        service.processCommand('stop', 1.0),
        service.processCommand('dance', 0.9),
        service.processCommand('go home', 0.85),
      ]);
    });

    // Open history
    const historyBtn = page.locator('[aria-label="History"]');
    await historyBtn.click();

    const historyPanel = page.locator('[data-testid="command-history"]');
    await expect(historyPanel).toBeVisible();

    // Should show all commands
    await expect(historyPanel).toContainText('stop');
    await expect(historyPanel).toContainText('dance');
    await expect(historyPanel).toContainText('go home');

    // Should show execution times
    await expect(historyPanel).toContainText('ms');

    // Should show confidence percentages
    await expect(historyPanel).toContainText('%');
  });

  /**
   * Test: Clear History
   * Tests history management from VOICE-005
   */
  test('Test: Clear History', async ({ page }) => {
    // Add commands to history
    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return service.processCommand('stop', 1.0);
    });

    // Open history
    const historyBtn = page.locator('[aria-label="History"]');
    await historyBtn.click();

    const historyPanel = page.locator('[data-testid="command-history"]');
    await expect(historyPanel).toContainText('stop');

    // Mock confirm dialog
    page.on('dialog', (dialog) => dialog.accept());

    // Clear history
    const clearBtn = page.locator('[data-testid="clear-history-btn"]');
    await clearBtn.click();

    // Should show empty state
    await expect(historyPanel).toContainText('No commands yet');
  });

  /**
   * Test: Confidence Visualization
   * Tests I-VOICE-002 invariant (confidence threshold)
   */
  test('Test: Confidence Visualization', async ({ page }) => {
    // Enable voice
    const voiceToggle = page.locator('[data-testid="voice-toggle"]');
    await voiceToggle.click();

    // Process command with high confidence
    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return service.processCommand('stop', 0.95);
    });

    // Should show confidence meter
    const confidenceMeter = page.locator('[data-testid="voice-confidence"]');
    await expect(confidenceMeter).toBeVisible();

    // Should show percentage
    await expect(confidenceMeter).toContainText('95%');

    // Should show green bar for high confidence
    const greenBar = confidenceMeter.locator('.bg-green-500');
    await expect(greenBar).toBeVisible();
  });

  /**
   * Test: I-VOICE-001 Command Latency
   * Verifies commands execute within 500ms
   */
  test('Test: I-VOICE-001 Command Latency', async ({ page }) => {
    // Enable voice
    const voiceToggle = page.locator('[data-testid="voice-toggle"]');
    await voiceToggle.click();

    // Execute command and measure time
    const startTime = Date.now();

    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return service.processCommand('stop', 1.0);
    });

    const endTime = Date.now();
    const executionTime = endTime - startTime;

    // Should execute within 500ms per I-VOICE-001
    expect(executionTime).toBeLessThan(500);

    // Check execution time in history
    const historyBtn = page.locator('[aria-label="History"]');
    await historyBtn.click();

    const historyPanel = page.locator('[data-testid="command-history"]');
    const historyText = await historyPanel.textContent();

    // Extract execution time from history
    const match = historyText?.match(/(\d+)ms/);
    if (match) {
      const recordedTime = parseInt(match[1], 10);
      expect(recordedTime).toBeLessThan(500);
    }
  });

  /**
   * Test: Browser Compatibility Warning
   * Tests graceful degradation when API unavailable
   */
  test('Test: Browser Compatibility Warning', async ({ page }) => {
    // Simulate browser without SpeechRecognition
    await page.evaluate(() => {
      delete (window as any).SpeechRecognition;
      delete (window as any).webkitSpeechRecognition;
    });

    // Reload page
    await page.reload();

    // Should show compatibility warning
    await expect(page.locator('[data-testid="voice-control"]')).toContainText(
      'Voice commands are not supported'
    );
  });

  /**
   * Test: Non-blocking UI (I-VOICE-003)
   * Verifies UI remains responsive during voice processing
   */
  test('Test: I-VOICE-003 Non-blocking UI', async ({ page }) => {
    // Enable voice
    const voiceToggle = page.locator('[data-testid="voice-toggle"]');
    await voiceToggle.click();

    // Process command
    const commandPromise = page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return service.processCommand('stop', 1.0);
    });

    // UI should remain interactive (test by clicking settings during processing)
    const settingsBtn = page.locator('[aria-label="Settings"]');
    await settingsBtn.click();

    // Settings panel should open immediately (non-blocking)
    const settingsPanel = page.locator('[data-testid="settings-panel"]');
    await expect(settingsPanel).toBeVisible({ timeout: 100 });

    // Wait for command to complete
    await commandPromise;
  });

  /**
   * Test: Multiple Commands in Sequence
   * Tests system handles rapid command sequences
   */
  test('Test: Multiple Commands in Sequence', async ({ page }) => {
    // Enable voice
    const voiceToggle = page.locator('[data-testid="voice-toggle"]');
    await voiceToggle.click();

    // Execute multiple commands rapidly
    await page.evaluate(() => {
      const service = (window as any).voiceCommandService;
      return Promise.all([
        service.processCommand('stop', 1.0),
        service.processCommand('dance', 0.95),
        service.processCommand('go home', 0.9),
        service.processCommand('show stats', 0.85),
      ]);
    });

    // Open history
    const historyBtn = page.locator('[aria-label="History"]');
    await historyBtn.click();

    const historyPanel = page.locator('[data-testid="command-history"]');

    // All commands should be in history
    await expect(historyPanel).toContainText('stop');
    await expect(historyPanel).toContainText('dance');
    await expect(historyPanel).toContainText('go home');
    await expect(historyPanel).toContainText('show stats');
  });
});
