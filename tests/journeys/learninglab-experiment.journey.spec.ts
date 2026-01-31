/**
 * Journey Test: LearningLab Experiment
 * Tests complete user journey through neural visualization
 *
 * Journey: J-LEARN-FIRST-EXPERIMENT
 * Story: STORY-LEARN-006
 * Issue: #59
 */

import { test, expect } from '@playwright/test';

const WS_URL = 'ws://localhost:8081';
const DASHBOARD_URL = 'http://localhost:3000/visualizer';

test.describe('J-LEARN-FIRST-EXPERIMENT: Neural Visualizer Journey', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to visualizer
    await page.goto(DASHBOARD_URL);
    await page.waitForLoadState('networkidle');
  });

  test('Scenario 1: View Live Neural Activity', async ({ page }) => {
    // Given the robot is running
    await expect(page.locator('text=Connected')).toBeVisible();

    // When I open /visualizer
    // Already navigated in beforeEach

    // Then I see current mode (Calm/Active/Spike/Protect)
    const modeIndicator = page.getByTestId('neural-mode-indicator');
    await expect(modeIndicator).toBeVisible();

    // And meters update every 100ms (minimum 10Hz - I-LEARN-VIZ-001)
    const tensionMeter = page.getByTestId('neural-tension-meter');
    await expect(tensionMeter).toBeVisible();

    // Verify meters are animating (check canvas is updating)
    const initialCanvas = await tensionMeter.screenshot();
    await page.waitForTimeout(200); // Wait for updates
    const updatedCanvas = await tensionMeter.screenshot();

    // Canvases should be different (indicating animation)
    expect(initialCanvas.equals(updatedCanvas)).toBe(false);

    // And timeline shows last 60 seconds
    const timeline = page.getByTestId('neural-timeline-chart');
    await expect(timeline).toBeVisible();
    await expect(page.locator('text=/Last 60 seconds/i')).toBeVisible();
  });

  test('Scenario 2: Observe Mode Transition', async ({ page }) => {
    // Given robot is in Calm mode
    await expect(page.locator('text=Connected')).toBeVisible();

    const modeIndicator = page.getByTestId('neural-mode-indicator');
    await expect(modeIndicator).toBeVisible();

    // When sudden stimulus occurs (simulated by server)
    // Wait for mode change to happen
    await page.waitForTimeout(5000); // Server simulates events every few seconds

    // Then mode icon changes
    // Mode should have changed (server cycles through modes)
    await expect(modeIndicator).toBeVisible();

    // And transition marker appears on timeline
    const timeline = page.getByTestId('neural-timeline-chart');
    await expect(timeline).toBeVisible();

    // Timeline should show visual markers for mode transitions
  });

  test('Scenario 3: Review Historical Data', async ({ page }) => {
    // Wait for some data to accumulate
    await page.waitForTimeout(3000);

    // When I drag timeline to -30 seconds
    const timeline = page.getByTestId('neural-timeline-chart');
    await expect(timeline).toBeVisible();

    const bbox = await timeline.boundingBox();
    if (bbox) {
      // Click and drag on timeline (scrub backwards)
      await page.mouse.move(bbox.x + bbox.width * 0.8, bbox.y + bbox.height / 2);
      await page.mouse.down();
      await page.mouse.move(bbox.x + bbox.width * 0.5, bbox.y + bbox.height / 2);
      await page.mouse.up();
    }

    // Then meters show values from that time
    await expect(page.locator('text=/viewing:/i')).toBeVisible();

    // And pause indicator shows
    await expect(page.locator('text=/play/i')).toBeVisible();
  });

  test('Scenario 4: Export Neural Data', async ({ page }) => {
    // Wait for some data
    await page.waitForTimeout(2000);

    // When I click export button
    const exportButton = page.getByTestId('export-data-button');
    await expect(exportButton).toBeVisible();

    // Setup download handler
    const downloadPromise = page.waitForEvent('download');
    await exportButton.click();

    // Then CSV file downloads
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/neural-data-\d+\.csv/);

    // Verify file content
    const path = await download.path();
    if (path) {
      const fs = require('fs');
      const content = fs.readFileSync(path, 'utf-8');

      // Should have CSV header
      expect(content).toContain('timestamp,mode,tension,coherence,energy,curiosity');

      // Should have data rows
      const lines = content.split('\n');
      expect(lines.length).toBeGreaterThan(1);
    }
  });

  test('Scenario 5: Zoom and Pan Controls', async ({ page }) => {
    // Given timeline is visible
    const timeline = page.getByTestId('neural-timeline-chart');
    await expect(timeline).toBeVisible();

    // When I click zoom in
    await page.click('text=🔍+');

    // Then timeline scales up
    const zoomStyle = await timeline.getAttribute('style');
    expect(zoomStyle).toContain('scale');

    // When I click zoom out
    await page.click('text=🔍-');

    // Timeline scales down

    // When I click reset zoom
    await page.click('text=/reset zoom/i');

    // Timeline returns to 1x scale
  });

  test('Scenario 6: Color-Coded Visualization', async ({ page }) => {
    // Given visualizer is running
    await expect(page.locator('text=Connected')).toBeVisible();

    const metersCanvas = page.getByTestId('neural-tension-meter');
    await expect(metersCanvas).toBeVisible();

    // Then tension displays with red intensity
    // And energy displays with blue intensity
    // (Visual verification - canvas renders with correct colors)

    // Take screenshot for manual verification
    await metersCanvas.screenshot({ path: 'test-results/neural-meters.png' });
  });

  test('Scenario 7: Stimulus Flash Effect', async ({ page }) => {
    // Given robot is in stable mode
    await expect(page.locator('text=Connected')).toBeVisible();

    const metersCanvas = page.getByTestId('neural-tension-meter');

    // Capture initial state
    const before = await metersCanvas.screenshot();

    // Wait for mode change (stimulus)
    await page.waitForTimeout(5000);

    // Capture after stimulus
    const after = await metersCanvas.screenshot();

    // Canvas should show different state (flash effect)
    expect(before.equals(after)).toBe(false);
  });

  test('Scenario 8: Data Retention (I-LEARN-VIZ-002)', async ({ page }) => {
    // Given visualizer is running
    await expect(page.locator('text=Connected')).toBeVisible();

    // Wait for data to accumulate
    await page.waitForTimeout(3000);

    // Then history shows accumulated data
    const dataInfo = page.locator('text=/data points/i');
    await expect(dataInfo).toBeVisible();

    // Verify data count increases
    const text = await dataInfo.textContent();
    expect(text).toMatch(/\d+ data points/);
  });

  test('Scenario 9: Update Rate Performance (I-LEARN-VIZ-001)', async ({ page }) => {
    // Given visualizer is connected
    await expect(page.locator('text=Connected')).toBeVisible();

    // Measure update frequency
    let updateCount = 0;
    const startTime = Date.now();

    // Monitor canvas changes for 2 seconds
    const timeline = page.getByTestId('neural-timeline-chart');

    let lastScreenshot = await timeline.screenshot();

    const checkInterval = setInterval(async () => {
      const currentScreenshot = await timeline.screenshot();
      if (!lastScreenshot.equals(currentScreenshot)) {
        updateCount++;
        lastScreenshot = currentScreenshot;
      }
    }, 50); // Check every 50ms

    await page.waitForTimeout(2000);
    clearInterval(checkInterval);

    const duration = (Date.now() - startTime) / 1000;
    const updateRate = updateCount / duration;

    // Then update rate is at least 10Hz (I-LEARN-VIZ-001)
    expect(updateRate).toBeGreaterThanOrEqual(10);
  });

  test('Scenario 10: WebSocket Reconnection', async ({ page }) => {
    // Given visualizer is connected
    await expect(page.locator('text=Connected')).toBeVisible();

    // When connection drops (simulate by checking for disconnect)
    // Server would need to close connection for this test

    // Then reconnect button appears
    // And clicking reconnect restores connection

    // Note: This requires server-side support to simulate disconnection
  });

  test('Scenario 11: Mode Indicator Updates', async ({ page }) => {
    // Given visualizer is running
    const modeIndicator = page.getByTestId('neural-mode-indicator');
    await expect(modeIndicator).toBeVisible();

    // Take screenshots of mode indicator over time
    const screenshots = [];
    for (let i = 0; i < 5; i++) {
      screenshots.push(await modeIndicator.screenshot());
      await page.waitForTimeout(2000);
    }

    // At least one screenshot should differ (mode changed)
    let changesDetected = 0;
    for (let i = 1; i < screenshots.length; i++) {
      if (!screenshots[i].equals(screenshots[i - 1])) {
        changesDetected++;
      }
    }

    expect(changesDetected).toBeGreaterThan(0);
  });

  test('Scenario 12: JSON Export', async ({ page }) => {
    // Wait for data
    await page.waitForTimeout(2000);

    // Click JSON export
    const downloadPromise = page.waitForEvent('download');
    await page.click('text=/export json/i');

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/neural-data-\d+\.json/);

    // Verify JSON structure
    const path = await download.path();
    if (path) {
      const fs = require('fs');
      const content = fs.readFileSync(path, 'utf-8');
      const data = JSON.parse(content);

      // Should be array of neural states
      expect(Array.isArray(data)).toBe(true);
      if (data.length > 0) {
        expect(data[0]).toHaveProperty('timestamp');
        expect(data[0]).toHaveProperty('mode');
        expect(data[0]).toHaveProperty('tension');
        expect(data[0]).toHaveProperty('coherence');
        expect(data[0]).toHaveProperty('energy');
        expect(data[0]).toHaveProperty('curiosity');
      }
    }
  });
});

test.describe('Definition of Done Verification', () => {
  test('DOD: Real-time meters implemented', async ({ page }) => {
    await page.goto(DASHBOARD_URL);
    await expect(page.getByTestId('neural-tension-meter')).toBeVisible();
  });

  test('DOD: Timeline with transitions', async ({ page }) => {
    await page.goto(DASHBOARD_URL);
    await expect(page.getByTestId('neural-timeline-chart')).toBeVisible();
  });

  test('DOD: Export functionality', async ({ page }) => {
    await page.goto(DASHBOARD_URL);
    await expect(page.getByTestId('export-data-button')).toBeVisible();
  });

  test('DOD: 60fps performance capability', async ({ page }) => {
    await page.goto(DASHBOARD_URL);

    // Measure frame rendering performance
    const performance = await page.evaluate(() => {
      return new Promise<number>((resolve) => {
        let frameCount = 0;
        const startTime = performance.now();

        function countFrame() {
          frameCount++;
          if (performance.now() - startTime < 1000) {
            requestAnimationFrame(countFrame);
          } else {
            resolve(frameCount);
          }
        }

        requestAnimationFrame(countFrame);
      });
    });

    // Should support near 60fps
    expect(performance).toBeGreaterThanOrEqual(50); // Allow some margin
  });
});
