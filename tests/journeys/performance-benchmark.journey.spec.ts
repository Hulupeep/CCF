/**
 * Performance Benchmark Journey Test
 *
 * Tests the complete performance optimization system end-to-end, validating
 * all invariants (I-PERF-001 through I-PERF-004) under realistic conditions.
 *
 * Contract: feature_performance.yml
 * Issue: #90 (STORY-PERF-001)
 */

import { test, expect } from '@playwright/test';

test.describe('Performance Benchmark Journey', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to dashboard with performance monitoring
    await page.goto('/dashboard?enable_profiling=true');

    // Wait for app to load
    await expect(page.locator('[data-testid="app-loaded"]')).toBeVisible({ timeout: 5000 });
  });

  test('Scenario: WebSocket Latency Optimization', async ({ page }) => {
    // Given baseline WebSocket latency is 80ms p99
    await page.locator('[data-testid="perf-dashboard"]').click();

    // Record baseline
    await page.locator('[data-testid="record-baseline-btn"]').click();
    await page.waitForTimeout(2000); // Collect samples

    const baselineLatency = await page.locator('[data-testid="latency-p99"]').textContent();
    expect(parseFloat(baselineLatency!)).toBeGreaterThan(0);

    // When I implement message batching and compression
    await page.locator('[data-testid="optimization-toggle"]').check();
    await page.locator('[data-testid="enable-batching"]').check();
    await page.locator('[data-testid="enable-compression"]').check();
    await page.locator('[data-testid="apply-optimizations-btn"]').click();

    // Wait for optimizations to take effect
    await page.waitForTimeout(2000);

    // Record optimized metrics
    await page.locator('[data-testid="record-optimized-btn"]').click();
    await page.waitForTimeout(2000);

    // Then p99 latency reduces to <50ms
    const optimizedLatency = await page.locator('[data-testid="latency-p99"]').textContent();
    const p99 = parseFloat(optimizedLatency!);

    expect(p99).toBeLessThan(50);

    // And message throughput increases by 30%
    const throughputImprovement = await page.locator('[data-testid="throughput-improvement"]').textContent();
    expect(parseFloat(throughputImprovement!)).toBeGreaterThan(30);

    // I-PERF-001: WebSocket latency must be <50ms at p99
    const invariantCheck = await page.locator('[data-testid="i-perf-001-status"]');
    await expect(invariantCheck).toHaveClass(/passed/);
  });

  test('Scenario: UI Render Performance', async ({ page }) => {
    // Given baseline render rate is 45fps
    await page.locator('[data-testid="perf-dashboard"]').click();

    // When I profile React components with DevTools
    await page.locator('[data-testid="profiler-control"]').click();
    await page.locator('[data-testid="start-profiling"]').click();

    // And I memoize expensive components
    await page.locator('[data-testid="enable-memoization"]').check();

    // And I optimize canvas rendering
    await page.locator('[data-testid="enable-offscreen-canvas"]').check();

    await page.locator('[data-testid="apply-optimizations-btn"]').click();

    // Perform actions that stress rendering
    await page.locator('[data-testid="personality-slider"]').fill('0.8');
    await page.locator('[data-testid="personality-slider"]').fill('0.2');
    await page.locator('[data-testid="personality-slider"]').fill('0.5');

    // Wait for frame rate to stabilize
    await page.waitForTimeout(3000);

    // Then UI maintains 60fps during personality adjustments
    const fps = await page.locator('[data-testid="frame-rate-meter"]').textContent();
    expect(parseFloat(fps!)).toBeGreaterThanOrEqual(60);

    // And no frame drops during neural visualization
    const droppedFrames = await page.locator('[data-testid="dropped-frames"]').textContent();
    expect(parseInt(droppedFrames!)).toBeLessThan(5);

    // I-PERF-002: UI must maintain 60fps
    const invariantCheck = await page.locator('[data-testid="i-perf-002-status"]');
    await expect(invariantCheck).toHaveClass(/passed/);
  });

  test('Scenario: Memory Leak Prevention', async ({ page }) => {
    // Given baseline memory usage is 120MB
    await page.locator('[data-testid="perf-dashboard"]').click();
    await page.locator('[data-testid="memory-tab"]').click();

    const baselineMemory = await page.locator('[data-testid="memory-usage-mb"]').textContent();
    const baseline = parseFloat(baselineMemory!);

    // When I identify memory leaks with heap snapshots
    await page.locator('[data-testid="take-heap-snapshot"]').click();
    await page.waitForTimeout(1000);

    // And I fix event listener leaks
    await page.locator('[data-testid="enable-listener-cleanup"]').check();

    // And I implement proper cleanup in useEffect
    await page.locator('[data-testid="enable-effect-cleanup"]').check();

    await page.locator('[data-testid="apply-optimizations-btn"]').click();

    // Perform actions for 10 minutes (simulated with rapid actions)
    for (let i = 0; i < 100; i++) {
      await page.locator('[data-testid="personality-slider"]').fill(String(Math.random()));
      await page.waitForTimeout(50);
    }

    // Then memory usage stabilizes at <100MB
    const currentMemory = await page.locator('[data-testid="memory-usage-mb"]').textContent();
    const current = parseFloat(currentMemory!);

    expect(current).toBeLessThan(100);

    // And no memory growth after 10 minutes of use
    const memoryGrowth = current - baseline;
    expect(memoryGrowth).toBeLessThan(10); // Less than 10MB growth

    // I-PERF-003: Memory usage must not exceed 100MB
    const invariantCheck = await page.locator('[data-testid="i-perf-003-status"]');
    await expect(invariantCheck).toHaveClass(/passed/);
  });

  test('Scenario: Drawing Playback Optimization', async ({ page }) => {
    // Given drawing with 1000 strokes
    await page.goto('/artbot?strokes=1200');

    // Enable playback optimization
    await page.locator('[data-testid="enable-playback-optimization"]').check();
    await page.locator('[data-testid="enable-webgl"]').check();

    // When I play back drawing animation
    await page.locator('[data-testid="playback-btn"]').click();

    // Wait for playback to start
    await page.waitForTimeout(500);

    // Monitor frame rate during playback
    await page.waitForTimeout(5000); // 5 seconds of playback

    // Then playback maintains 60fps
    const fps = await page.locator('[data-testid="playback-fps"]').textContent();
    expect(parseFloat(fps!)).toBeGreaterThanOrEqual(60);

    // And no stuttering or frame drops
    const droppedFrames = await page.locator('[data-testid="dropped-frames"]').textContent();
    expect(parseInt(droppedFrames!)).toBeLessThan(10);

    // And CPU usage <50%
    const cpuUsage = await page.locator('[data-testid="cpu-usage"]').textContent();
    expect(parseFloat(cpuUsage!)).toBeLessThan(50);

    // I-PERF-004: Drawing playback smooth at 60fps for 1000+ strokes
    const invariantCheck = await page.locator('[data-testid="i-perf-004-status"]');
    await expect(invariantCheck).toHaveClass(/passed/);
  });

  test('Scenario: Performance Dashboard Integration', async ({ page }) => {
    // Given I'm on the performance dashboard
    await page.locator('[data-testid="perf-dashboard"]').click();

    // When I view real-time metrics
    await expect(page.locator('[data-testid="latency-chart"]')).toBeVisible();
    await expect(page.locator('[data-testid="frame-rate-meter"]')).toBeVisible();
    await expect(page.locator('[data-testid="memory-graph"]')).toBeVisible();

    // Then I see all performance indicators
    const websocketLatency = await page.locator('[data-testid="ws-latency-indicator"]');
    const uiFps = await page.locator('[data-testid="ui-fps-indicator"]');
    const memoryUsage = await page.locator('[data-testid="memory-indicator"]');

    await expect(websocketLatency).toBeVisible();
    await expect(uiFps).toBeVisible();
    await expect(memoryUsage).toBeVisible();

    // And all invariants show passing status
    const invariants = ['i-perf-001', 'i-perf-002', 'i-perf-003', 'i-perf-004'];
    for (const invariant of invariants) {
      const status = await page.locator(`[data-testid="${invariant}-status"]`);
      await expect(status).toHaveClass(/passed/);
    }
  });

  test('Scenario: Benchmark Suite Execution', async ({ page }) => {
    // Given I want to run comprehensive benchmarks
    await page.locator('[data-testid="perf-dashboard"]').click();
    await page.locator('[data-testid="benchmarks-tab"]').click();

    // When I click "Run Benchmarks"
    await page.locator('[data-testid="run-benchmark-btn"]').click();

    // Then I see progress indicator
    await expect(page.locator('[data-testid="benchmark-progress"]')).toBeVisible();

    // Wait for benchmarks to complete
    await page.waitForSelector('[data-testid="benchmark-results"]', { timeout: 30000 });

    // And I see detailed results
    const results = await page.locator('[data-testid="benchmark-results"]');
    await expect(results).toBeVisible();

    // And all benchmarks show pass/fail status
    const wsLatencyBenchmark = await page.locator('[data-testid="ws-latency-benchmark"]');
    const memoryBenchmark = await page.locator('[data-testid="memory-benchmark"]');
    const renderBenchmark = await page.locator('[data-testid="render-benchmark"]');

    await expect(wsLatencyBenchmark).toContainText(/PASS|FAIL/);
    await expect(memoryBenchmark).toContainText(/PASS|FAIL/);
    await expect(renderBenchmark).toContainText(/PASS|FAIL/);

    // And overall status is displayed
    const overallStatus = await page.locator('[data-testid="overall-benchmark-status"]');
    await expect(overallStatus).toBeVisible();
  });

  test('Scenario: Performance Regression Detection', async ({ page }) => {
    // Given baseline performance metrics are recorded
    await page.locator('[data-testid="perf-dashboard"]').click();
    await page.locator('[data-testid="record-baseline-btn"]').click();
    await page.waitForTimeout(2000);

    // When I make changes and re-measure
    await page.locator('[data-testid="record-optimized-btn"]').click();
    await page.waitForTimeout(2000);

    // Then I see improvement percentages
    const latencyImprovement = await page.locator('[data-testid="latency-improvement"]');
    const memoryReduction = await page.locator('[data-testid="memory-reduction"]');
    const fpsImprovement = await page.locator('[data-testid="fps-improvement"]');

    await expect(latencyImprovement).toBeVisible();
    await expect(memoryReduction).toBeVisible();
    await expect(fpsImprovement).toBeVisible();

    // And regression warnings if performance degrades
    const regressionWarnings = await page.locator('[data-testid="regression-warnings"]');

    // Should show no regressions with optimizations enabled
    await expect(regressionWarnings).toHaveCount(0);
  });

  test('Scenario: Load Testing Under Stress', async ({ page }) => {
    // Given I want to test under high load
    await page.locator('[data-testid="perf-dashboard"]').click();
    await page.locator('[data-testid="stress-test-tab"]').click();

    // When I enable stress testing
    await page.locator('[data-testid="stress-test-mode"]').check();
    await page.locator('[data-testid="concurrent-users"]').fill('100');
    await page.locator('[data-testid="start-stress-test"]').click();

    // Then performance metrics are tracked under load
    await page.waitForTimeout(10000); // 10 seconds of stress

    // And invariants still hold
    const websocketLatency = await page.locator('[data-testid="stress-ws-latency"]').textContent();
    const uiFps = await page.locator('[data-testid="stress-ui-fps"]').textContent();
    const memoryUsage = await page.locator('[data-testid="stress-memory"]').textContent();

    expect(parseFloat(websocketLatency!)).toBeLessThan(50);
    expect(parseFloat(uiFps!)).toBeGreaterThanOrEqual(55); // Allow slight drop under stress
    expect(parseFloat(memoryUsage!)).toBeLessThan(100);
  });
});
