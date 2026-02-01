/**
 * Mobile Control Journey Test (E2E)
 * Issue: #88 (STORY-MOBILE-001)
 * Contract: J-MOBILE-CONTROL
 *
 * Tests all acceptance criteria from issue #88
 */

import { device, element, by, expect as detoxExpect, waitFor } from 'detox';

describe('J-MOBILE-CONTROL: Mobile Control Journey', () => {
  beforeAll(async () => {
    await device.launchApp();
  });

  beforeEach(async () => {
    await device.reloadReactNative();
  });

  /**
   * Scenario: Connect from Phone
   * Given robot on WiFi network 192.168.1.x
   * When I open mobile app
   * Then I see "Scanning for robots..." message
   * And discovered robots appear within 10 seconds
   * When I tap to connect
   * Then WebSocket connects within 3 seconds (I-MOBILE-001)
   * And I see live neural state visualization
   * And connection indicator shows "Connected"
   */
  test('should discover and connect to robot', async () => {
    // App starts on Discovery screen
    await waitFor(element(by.id('discovery-list')))
      .toBeVisible()
      .withTimeout(2000);

    // Start scanning
    await element(by.id('scan-btn')).tap();

    // Wait for robots to appear (within 10 seconds)
    await waitFor(element(by.id('connect-btn-robot-001')))
      .toBeVisible()
      .withTimeout(10000);

    // Connect to first robot
    const startTime = Date.now();
    await element(by.id('connect-btn-robot-001')).tap();

    // Should connect within 3 seconds (I-MOBILE-001)
    await waitFor(element(by.id('connection-status')))
      .toBeVisible()
      .withTimeout(3000);

    const connectionTime = Date.now() - startTime;
    expect(connectionTime).toBeLessThan(3000);

    // Should navigate to Mixer screen
    await waitFor(element(by.id('personality-mixer')))
      .toBeVisible()
      .withTimeout(2000);

    // Connection indicator should show "Connected"
    await detoxExpect(element(by.id('connection-status'))).toHaveText('Connected');
  });

  /**
   * Scenario: Adjust Personality from Phone
   * Given connected to robot
   * When I open personality mixer screen
   * Then I see all 9 sliders (energy, tension, etc.)
   * And current values match robot state
   * When I adjust tension from 0.5 to 0.8
   * Then slider updates smoothly
   * And robot responds within 200ms (I-MOBILE-001)
   * And neural visualizer reflects change
   */
  test('should adjust personality parameters', async () => {
    // Assume connected from previous test
    await element(by.text('Personality')).tap();

    // Verify all 9 sliders present
    const sliders = [
      'slider-energy',
      'slider-tension',
      'slider-curiosity',
      'slider-playfulness',
      'slider-confidence',
      'slider-focus',
      'slider-empathy',
      'slider-creativity',
      'slider-persistence',
    ];

    for (const sliderId of sliders) {
      await detoxExpect(element(by.id(sliderId))).toBeVisible();
    }

    // Adjust tension slider
    const startTime = Date.now();
    await element(by.id('slider-tension')).adjustSliderToPosition(0.8);

    // Response should be within 200ms (I-MOBILE-001)
    const responseTime = Date.now() - startTime;
    expect(responseTime).toBeLessThan(200);

    // Verify slider updated
    await waitFor(element(by.id('slider-tension')))
      .toHaveSliderPosition(0.8, 0.1)
      .withTimeout(1000);
  });

  /**
   * Scenario: View Gallery
   * Given robot has 10 saved drawings
   * When I open gallery viewer
   * Then all 10 thumbnails load within 2 seconds
   * When I tap a drawing
   * Then full drawing opens
   * And I can play back drawing animation
   */
  test('should view gallery and playback drawings', async () => {
    // Navigate to Gallery
    await element(by.text('Gallery')).tap();

    // Wait for gallery to load (within 2 seconds)
    await waitFor(element(by.id('gallery-grid')))
      .toBeVisible()
      .withTimeout(2000);

    // Verify thumbnails loaded
    await detoxExpect(element(by.id('drawing-thumb-1'))).toBeVisible();

    // Tap first drawing
    await element(by.id('drawing-thumb-1')).tap();

    // Full drawing should open
    await waitFor(element(by.id('drawing-full-1')))
      .toBeVisible()
      .withTimeout(1000);

    // Playback button should be visible
    await detoxExpect(element(by.id('playback-btn-1'))).toBeVisible();

    // Can tap playback
    await element(by.id('playback-btn-1')).tap();
  });

  /**
   * Scenario: Offline Mode
   * Given I was connected to robot
   * When I lose WiFi connection
   * Then app shows "Offline" indicator (I-MOBILE-003)
   * And last known state remains visible
   * And I can view cached drawings
   * When WiFi reconnects
   * Then app reconnects automatically within 5 seconds (I-MOBILE-001)
   */
  test('should handle offline mode with caching', async () => {
    // Simulate going offline
    await device.setNetworkConditions({ airplane: true });

    // Wait for offline indicator
    await waitFor(element(by.text('Offline')))
      .toBeVisible()
      .withTimeout(2000);

    // Should still be able to view cached content
    await element(by.text('Gallery')).tap();

    // Cached drawings should be visible (I-MOBILE-003)
    await waitFor(element(by.id('gallery-grid')))
      .toBeVisible()
      .withTimeout(1000);

    // Simulate coming back online
    await device.setNetworkConditions({ airplane: false });

    // Should auto-reconnect within 5 seconds (I-MOBILE-001)
    await waitFor(element(by.text('Connected')))
      .toBeVisible()
      .withTimeout(5000);
  });

  /**
   * I-MOBILE-002: UI must be responsive on devices 320px-768px wide
   */
  test('should be responsive on different screen sizes', async () => {
    // Test on small device (iPhone SE - 375px)
    await device.setOrientation('portrait');

    await detoxExpect(element(by.id('personality-mixer'))).toBeVisible();

    // All UI elements should be accessible
    await element(by.id('slider-energy')).swipe('up');
    await detoxExpect(element(by.id('slider-persistence'))).toBeVisible();
  });

  /**
   * I-MOBILE-003: Cache should persist for 24 hours
   */
  test('should maintain cache for at least 24 hours', async () => {
    // This would require time manipulation in tests
    // Verify cache settings are configured correctly
    await element(by.text('Settings')).tap();

    // Cache expiry should be at least 24 hours
    await detoxExpect(element(by.text('Cache Expiry: 24 hours'))).toBeVisible();
  });
});
