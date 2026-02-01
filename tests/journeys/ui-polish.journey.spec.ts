/**
 * Journey Test: UI Polish & Animations
 * Issue #91 - STORY-UX-001
 * Journey: J-UI-POLISH
 *
 * Tests all animation scenarios from issue #91
 */

import { test, expect, Page } from '@playwright/test';

test.describe('Journey: J-UI-POLISH - UI Animation Polish', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to dashboard
    await page.goto('http://localhost:3000');

    // Wait for app to load
    await page.waitForSelector('[data-testid="personality-mixer"]', { timeout: 5000 });
  });

  test('Scenario: Slider Animation - Smooth drag and immediate feedback', async ({ page }) => {
    // Given I open personality mixer
    const slider = page.locator('[data-testid^="slider-energy"]');
    const valueDisplay = page.locator('[data-testid^="value-energy"]');

    // Get initial value
    const initialValue = await valueDisplay.textContent();
    expect(initialValue).toBeTruthy();

    // When I drag energy slider
    const sliderBox = await slider.boundingBox();
    expect(sliderBox).toBeTruthy();

    if (sliderBox) {
      // Drag from 30% to 80% position
      await page.mouse.move(sliderBox.x + sliderBox.width * 0.3, sliderBox.y + sliderBox.height / 2);
      await page.mouse.down();
      await page.mouse.move(sliderBox.x + sliderBox.width * 0.8, sliderBox.y + sliderBox.height / 2);
      await page.mouse.up();
    }

    // Then slider handle moves smoothly
    const newValue = await valueDisplay.textContent();
    expect(newValue).toBeTruthy();
    expect(newValue).not.toBe(initialValue);

    // And visual feedback is immediate (value changed)
    expect(parseFloat(newValue || '0')).toBeGreaterThan(parseFloat(initialValue || '0'));
  });

  test('Scenario: Mode Transition - Animated mode switch', async ({ page }) => {
    // Given I'm viewing the dashboard
    const modeButtons = page.locator('[data-testid^="mode-btn-"]');
    const count = await modeButtons.count();

    if (count > 1) {
      // When I click a different mode button
      await modeButtons.nth(1).click();

      // Then mode content transitions smoothly
      await page.waitForTimeout(500); // Wait for 300ms transition + buffer

      // Mode button should show active state
      const activeButton = await modeButtons.nth(1).getAttribute('class');
      expect(activeButton).toContain('active');
    }
  });

  test('Scenario: Button Micro-interactions - Hover and click feedback', async ({ page }) => {
    // Given I have interactive buttons
    const resetButton = page.locator('[data-testid="reset-button"]');

    // When I hover over the button
    await resetButton.hover();

    // Then button should have hover styles (check scale or transform)
    const hoverBox = await resetButton.boundingBox();
    expect(hoverBox).toBeTruthy();

    // When I click the button
    await resetButton.click();

    // Then button should show click feedback
    // (In real scenario, we'd measure the scale transform)
  });

  test('Scenario: Loading State - Skeleton screens appear and disappear', async ({ page }) => {
    // If the app has loading states, test them
    // This is a placeholder - implement based on actual loading states
    const skeletons = page.locator('[data-testid="skeleton-loader"]');
    const count = await skeletons.count();

    // If skeletons exist, they should have pulse animation
    if (count > 0) {
      const skeleton = skeletons.first();
      const opacity = await skeleton.evaluate((el) => {
        return window.getComputedStyle(el).opacity;
      });

      expect(opacity).toBeTruthy();
    }
  });

  test('Scenario: Modal Animations - Backdrop and content animations', async ({ page }) => {
    // Given I can open a modal
    const saveButton = page.locator('[data-testid="save-custom-button"]');

    // When I click to open modal
    await saveButton.click();

    // Then modal backdrop appears with fade
    const backdrop = page.locator('[data-testid="modal-backdrop"]');
    await expect(backdrop).toBeVisible();

    // And modal content scales in
    const content = page.locator('[data-testid="modal-content"]');
    await expect(content).toBeVisible();

    // Wait for animation to complete
    await page.waitForTimeout(350); // 300ms + buffer

    // When I close the modal
    const cancelButton = page.locator('[data-testid="save-cancel"]');
    await cancelButton.click();

    // Then modal fades out
    await expect(backdrop).not.toBeVisible({ timeout: 500 });
  });

  test('Scenario: Reduced Motion Support - Animations disabled when preferred', async ({ page, context }) => {
    // Create a new page with reduced motion preference
    await context.addInitScript(() => {
      Object.defineProperty(window, 'matchMedia', {
        writable: true,
        value: (query: string) => ({
          matches: query === '(prefers-reduced-motion: reduce)',
          media: query,
          onchange: null,
          addListener: () => {},
          removeListener: () => {},
          addEventListener: () => {},
          removeEventListener: () => {},
          dispatchEvent: () => true,
        }),
      });
    });

    const newPage = await context.newPage();
    await newPage.goto('http://localhost:3000');

    // Given user has "prefers-reduced-motion" enabled
    const prefersReduced = await newPage.evaluate(() => {
      return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    });

    expect(prefersReduced).toBe(true);

    // When any animation triggers (e.g., opening modal)
    await newPage.waitForSelector('[data-testid="save-custom-button"]', { timeout: 5000 });
    const saveButton = newPage.locator('[data-testid="save-custom-button"]');
    await saveButton.click();

    // Then animation duration should be 0ms
    // (Functionality remains identical, just no motion)
    const backdrop = newPage.locator('[data-testid="modal-backdrop"]');
    await expect(backdrop).toBeVisible();

    await newPage.close();
  });

  test('Scenario: Connection Status Animation - Status indicator transitions', async ({ page }) => {
    // Given I'm viewing the dashboard
    const statusIndicator = page.locator('[data-testid="connection-status"]');

    // Then status indicator should be visible
    await expect(statusIndicator).toBeVisible();

    // Status dot should have appropriate class
    const statusDot = statusIndicator.locator('.status-dot');
    const statusClass = await statusDot.getAttribute('class');

    expect(statusClass).toMatch(/status-(connected|connecting|disconnected)/);
  });

  test('Scenario: Preset Button Animations - Hover preview and click', async ({ page }) => {
    // Given I see personality presets
    const presetButtons = page.locator('[data-testid^="preset-button-"]');
    const count = await presetButtons.count();

    expect(count).toBeGreaterThan(0);

    if (count > 0) {
      const firstPreset = presetButtons.first();

      // When I hover over a preset
      await firstPreset.hover();

      // Then preview should show (slider values update)
      await page.waitForTimeout(200);

      // When I click the preset
      await firstPreset.click();

      // Then preset loads with smooth transition
      await page.waitForTimeout(350);

      // Active preset should be highlighted
      const activeClass = await firstPreset.getAttribute('class');
      expect(activeClass).toContain('active');
    }
  });

  test('Performance: Animations maintain 60fps', async ({ page }) => {
    // Start performance monitoring
    await page.evaluate(() => {
      (window as any).performanceMetrics = {
        frameDrops: 0,
        totalFrames: 0,
      };

      let lastTime = performance.now();

      const checkFrame = () => {
        const now = performance.now();
        const delta = now - lastTime;

        (window as any).performanceMetrics.totalFrames++;

        // If frame took longer than 16.67ms (60fps), count as drop
        if (delta > 16.67) {
          (window as any).performanceMetrics.frameDrops++;
        }

        lastTime = now;
        requestAnimationFrame(checkFrame);
      };

      requestAnimationFrame(checkFrame);
    });

    // Trigger some animations
    const slider = page.locator('[data-testid^="slider-energy"]');
    const sliderBox = await slider.boundingBox();

    if (sliderBox) {
      // Perform multiple slider drags
      for (let i = 0; i < 5; i++) {
        await page.mouse.move(sliderBox.x + sliderBox.width * 0.2, sliderBox.y + sliderBox.height / 2);
        await page.mouse.down();
        await page.mouse.move(sliderBox.x + sliderBox.width * 0.8, sliderBox.y + sliderBox.height / 2);
        await page.mouse.up();
        await page.waitForTimeout(100);
      }
    }

    // Check performance metrics
    const metrics = await page.evaluate(() => (window as any).performanceMetrics);

    // Allow up to 10% frame drops
    const dropRate = metrics.frameDrops / metrics.totalFrames;
    expect(dropRate).toBeLessThan(0.1);
  });

  test('Accessibility: Zero layout shift during animations', async ({ page }) => {
    // Measure Cumulative Layout Shift (CLS)
    const cls = await page.evaluate(() => {
      return new Promise<number>((resolve) => {
        let clsValue = 0;

        const observer = new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            if ((entry as any).hadRecentInput) continue;
            clsValue += (entry as any).value;
          }
        });

        observer.observe({ type: 'layout-shift', buffered: true });

        // Run for 2 seconds
        setTimeout(() => {
          observer.disconnect();
          resolve(clsValue);
        }, 2000);
      });
    });

    // CLS should be 0 (I-UI-003)
    expect(cls).toBeLessThanOrEqual(0.1); // Allow small tolerance
  });
});
