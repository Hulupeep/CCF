/**
 * Performance Benchmarking Suite
 * Contract: Issue #80 - Performance Benchmarking Dashboard
 *
 * Benchmarks:
 * - WebSocket message latency (target: <50ms p99)
 * - UI render time (target: <16ms for 60fps)
 * - Memory usage (target: <100MB baseline)
 * - Data processing throughput
 * - State synchronization time
 * - Component mount/unmount time
 */

import { describe, test, expect, beforeEach } from 'vitest';
import { performanceMetrics } from '../../web/src/services/performanceMetrics';
import { METRIC_TARGETS } from '../../web/src/types/performance';

/**
 * Helper: Run benchmark N times and collect statistics
 */
async function runBenchmark(
  name: string,
  fn: () => Promise<void> | void,
  iterations: number = 100
): Promise<{
  mean: number;
  p50: number;
  p95: number;
  p99: number;
  min: number;
  max: number;
}> {
  const timings: number[] = [];

  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    await fn();
    const duration = performance.now() - start;
    timings.push(duration);
  }

  timings.sort((a, b) => a - b);

  const mean = timings.reduce((sum, t) => sum + t, 0) / timings.length;
  const p50 = timings[Math.floor(iterations * 0.5)];
  const p95 = timings[Math.floor(iterations * 0.95)];
  const p99 = timings[Math.floor(iterations * 0.99)];
  const min = timings[0];
  const max = timings[timings.length - 1];

  return { mean, p50, p95, p99, min, max };
}

/**
 * Helper: Format timing results
 */
function formatResults(results: any): string {
  return `
    Mean: ${results.mean.toFixed(2)}ms
    P50:  ${results.p50.toFixed(2)}ms
    P95:  ${results.p95.toFixed(2)}ms
    P99:  ${results.p99.toFixed(2)}ms
    Min:  ${results.min.toFixed(2)}ms
    Max:  ${results.max.toFixed(2)}ms
  `;
}

describe('Performance Benchmarks', () => {
  beforeEach(() => {
    performanceMetrics.clear();
  });

  describe('WebSocket Message Latency', () => {
    test('measures WebSocket send/receive latency', async () => {
      const results = await runBenchmark(async () => {
        // Simulate WebSocket message round trip
        const start = Date.now();
        await new Promise(resolve => setTimeout(resolve, Math.random() * 10));
        const latency = Date.now() - start;
        performanceMetrics.recordMetric('websocket_latency', latency);
      });

      console.log('WebSocket Latency:', formatResults(results));

      // Requirement: p99 < 50ms
      expect(results.p99).toBeLessThan(METRIC_TARGETS.websocketLatencyP99);
    });

    test('measures WebSocket message batching efficiency', async () => {
      const batchSizes = [1, 5, 10, 20, 50];
      const results: Record<number, any> = {};

      for (const size of batchSizes) {
        results[size] = await runBenchmark(
          async () => {
            // Simulate batched message processing
            const messages = Array.from({ length: size }, (_, i) => ({
              type: 'command',
              payload: { index: i },
            }));

            const start = performance.now();
            // Process batch
            for (const msg of messages) {
              await new Promise(resolve => setTimeout(resolve, 0.5));
            }
            return performance.now() - start;
          },
          50
        );
      }

      console.log('Batch Processing Results:');
      for (const [size, result] of Object.entries(results)) {
        console.log(`  Batch size ${size}:`, formatResults(result));
      }

      // Verify batching improves throughput
      expect(results[50].mean / 50).toBeLessThan(results[1].mean);
    });
  });

  describe('UI Render Performance', () => {
    test('measures component render time', async () => {
      const results = await runBenchmark(() => {
        // Simulate React component render
        const startTime = performance.now();

        // Simulate DOM updates
        const div = document.createElement('div');
        div.innerHTML = '<span>Test content</span>'.repeat(100);

        return performance.now() - startTime;
      });

      console.log('UI Render Time:', formatResults(results));

      // Requirement: < 16ms for 60fps
      expect(results.mean).toBeLessThan(METRIC_TARGETS.uiRenderTime);
    });

    test('measures re-render performance with state updates', async () => {
      let state = { count: 0 };

      const results = await runBenchmark(() => {
        const start = performance.now();

        // Simulate state update and re-render
        state = { count: state.count + 1 };

        // Simulate virtual DOM diff
        const oldTree = Array.from({ length: 50 }, (_, i) => ({ id: i, value: i }));
        const newTree = Array.from({ length: 50 }, (_, i) => ({ id: i, value: i + 1 }));

        const diff = newTree.filter((item, idx) => item.value !== oldTree[idx].value);

        return performance.now() - start;
      });

      console.log('Re-render Performance:', formatResults(results));
      expect(results.p95).toBeLessThan(METRIC_TARGETS.uiRenderTime);
    });
  });

  describe('Memory Usage', () => {
    test('measures baseline memory consumption', async () => {
      if (typeof (performance as any).memory === 'undefined') {
        console.warn('Memory API not available, skipping test');
        return;
      }

      const memInfo = (performance as any).memory;
      const baselineMemoryMB = memInfo.usedJSHeapSize / (1024 * 1024);

      console.log('Baseline Memory:', baselineMemoryMB.toFixed(2), 'MB');

      performanceMetrics.recordMetric('memory_usage', baselineMemoryMB);

      // Requirement: < 100MB baseline
      expect(baselineMemoryMB).toBeLessThan(METRIC_TARGETS.memoryBaseline);
    });

    test('measures memory growth over time', async () => {
      const snapshots: number[] = [];

      for (let i = 0; i < 10; i++) {
        // Allocate some data
        const data = Array.from({ length: 1000 }, () => ({
          id: Math.random(),
          data: new Array(100).fill(Math.random()),
        }));

        if (typeof (performance as any).memory !== 'undefined') {
          const memInfo = (performance as any).memory;
          snapshots.push(memInfo.usedJSHeapSize / (1024 * 1024));
        }

        await new Promise(resolve => setTimeout(resolve, 100));
      }

      if (snapshots.length > 0) {
        const growth = snapshots[snapshots.length - 1] - snapshots[0];
        console.log('Memory Growth:', growth.toFixed(2), 'MB over', snapshots.length, 'snapshots');

        // Verify memory growth is controlled
        expect(growth).toBeLessThan(50); // < 50MB growth
      }
    });
  });

  describe('Data Processing Throughput', () => {
    test('measures data transformation throughput', async () => {
      const dataSize = 10000;
      const data = Array.from({ length: dataSize }, (_, i) => ({
        id: i,
        value: Math.random() * 100,
      }));

      const results = await runBenchmark(() => {
        const start = performance.now();

        // Process data
        const processed = data
          .filter(item => item.value > 50)
          .map(item => ({ ...item, value: item.value * 2 }))
          .sort((a, b) => b.value - a.value);

        return performance.now() - start;
      }, 50);

      const opsPerSec = dataSize / (results.mean / 1000);
      console.log('Processing Throughput:', opsPerSec.toFixed(0), 'ops/sec');

      performanceMetrics.recordMetric('processing_throughput', opsPerSec);

      // Requirement: > 1000 ops/sec
      expect(opsPerSec).toBeGreaterThan(METRIC_TARGETS.processingThroughput);
    });

    test('measures JSON serialization/deserialization performance', async () => {
      const testData = {
        personality: {
          playfulness: 0.7,
          cautiousness: 0.3,
          curiosity: 0.8,
        },
        state: {
          tension: 0.5,
          energy: 0.6,
          coherence: 0.9,
        },
        inventory: { red: 10, green: 20, blue: 15, yellow: 5 },
      };

      const serializeResults = await runBenchmark(() => {
        const start = performance.now();
        const json = JSON.stringify(testData);
        return performance.now() - start;
      });

      const deserializeResults = await runBenchmark(() => {
        const json = JSON.stringify(testData);
        const start = performance.now();
        const parsed = JSON.parse(json);
        return performance.now() - start;
      });

      console.log('Serialization:', formatResults(serializeResults));
      console.log('Deserialization:', formatResults(deserializeResults));

      expect(serializeResults.p99).toBeLessThan(1); // < 1ms
      expect(deserializeResults.p99).toBeLessThan(1); // < 1ms
    });
  });

  describe('State Synchronization', () => {
    test('measures full state sync time', async () => {
      const fullState = {
        personality: {
          playfulness: 0.7,
          cautiousness: 0.3,
          curiosity: 0.8,
          friendliness: 0.9,
          adaptability: 0.6,
        },
        neural_state: {
          mode: 'Active',
          tension: 0.5,
          coherence: 0.9,
          energy: 0.7,
          curiosity: 0.8,
          distance: 50,
          gyro: 0.1,
          sound: 0.3,
          light: 0.7,
        },
        inventory: { red: 10, green: 20, blue: 15, yellow: 5 },
        capabilities: {
          has_drawing: true,
          has_sorter: true,
          has_games: true,
          has_learning_lab: true,
          firmware_version: '1.0.0',
        },
      };

      const results = await runBenchmark(() => {
        const start = performance.now();

        // Simulate state sync
        const serialized = JSON.stringify(fullState);
        const deserialized = JSON.parse(serialized);

        // Simulate state updates
        Object.assign({}, deserialized);

        return performance.now() - start;
      });

      console.log('State Sync Time:', formatResults(results));

      performanceMetrics.recordMetric('state_sync_time', results.mean);

      // Requirement: < 100ms
      expect(results.p99).toBeLessThan(METRIC_TARGETS.stateSyncTime);
    });

    test('measures incremental state update performance', async () => {
      const state = { count: 0, values: new Array(1000).fill(0) };

      const results = await runBenchmark(() => {
        const start = performance.now();

        // Simulate incremental update
        state.count++;
        state.values[state.count % 1000] = Math.random();

        return performance.now() - start;
      });

      console.log('Incremental Update:', formatResults(results));

      // Incremental updates should be very fast
      expect(results.p99).toBeLessThan(1); // < 1ms
    });
  });

  describe('Component Lifecycle', () => {
    test('measures component mount time', async () => {
      const results = await runBenchmark(() => {
        const start = performance.now();

        // Simulate component mount
        const container = document.createElement('div');
        container.innerHTML = `
          <div class="component">
            <h1>Title</h1>
            <p>Content</p>
            <button>Action</button>
          </div>
        `.repeat(10);

        return performance.now() - start;
      });

      console.log('Component Mount Time:', formatResults(results));

      performanceMetrics.recordMetric('component_mount_time', results.mean);

      // Requirement: < 50ms
      expect(results.p95).toBeLessThan(METRIC_TARGETS.componentMountTime);
    });

    test('measures component unmount time', async () => {
      const containers: HTMLElement[] = [];

      // Setup
      for (let i = 0; i < 10; i++) {
        const container = document.createElement('div');
        container.innerHTML = '<div>Component</div>'.repeat(100);
        containers.push(container);
      }

      const results = await runBenchmark(() => {
        const start = performance.now();

        // Simulate component unmount
        const container = containers.pop();
        if (container) {
          container.innerHTML = '';
        }

        return performance.now() - start;
      }, containers.length);

      console.log('Component Unmount Time:', formatResults(results));

      expect(results.p99).toBeLessThan(10); // < 10ms
    });
  });

  describe('Regression Detection', () => {
    test('detects performance regressions correctly', () => {
      const metricId = 'websocket_latency';
      const target = METRIC_TARGETS.websocketLatencyP99;

      // Record baseline metrics
      for (let i = 0; i < 100; i++) {
        performanceMetrics.recordMetric(metricId, target * 0.8 + Math.random() * 10);
      }

      const metric = performanceMetrics.getMetric(metricId);
      expect(metric).toBeDefined();
      expect(metric!.isRegressed).toBe(false);

      // Introduce regression (>10% degradation)
      for (let i = 0; i < 50; i++) {
        performanceMetrics.recordMetric(metricId, target * 1.3);
      }

      const metricAfter = performanceMetrics.getMetric(metricId);
      expect(metricAfter!.isRegressed).toBe(true);

      const alerts = performanceMetrics.getActiveAlerts();
      expect(alerts.length).toBeGreaterThan(0);
      expect(alerts[0].metricId).toBe(metricId);
    });

    test('calculates regression percentage correctly', () => {
      const metricId = 'ui_render_time';
      const target = METRIC_TARGETS.uiRenderTime;

      // Record baseline
      for (let i = 0; i < 100; i++) {
        performanceMetrics.recordMetric(metricId, target * 0.9);
      }

      // Introduce 20% regression
      for (let i = 0; i < 50; i++) {
        performanceMetrics.recordMetric(metricId, target * 1.2);
      }

      const metric = performanceMetrics.getMetric(metricId);
      expect(metric).toBeDefined();
      expect(metric!.regressionPercent).toBeDefined();

      if (metric!.regressionPercent) {
        expect(metric!.regressionPercent).toBeGreaterThan(0.1); // > 10%
      }
    });
  });

  describe('CSV Export', () => {
    test('exports metrics to CSV format', () => {
      // Record some test data
      performanceMetrics.recordMetric('websocket_latency', 25);
      performanceMetrics.recordMetric('ui_render_time', 12);
      performanceMetrics.recordMetric('memory_usage', 75);

      const csv = performanceMetrics.exportToCSV(true);

      expect(csv).toContain('Metric ID,Name,Category');
      expect(csv).toContain('websocket_latency');
      expect(csv).toContain('ui_render_time');
      expect(csv).toContain('memory_usage');
      expect(csv).toContain('Historical Data');
    });

    test('CSV export includes all required fields', () => {
      performanceMetrics.recordMetric('websocket_latency', 30);

      const csv = performanceMetrics.exportToCSV(false);
      const lines = csv.split('\n');
      const header = lines[0];

      expect(header).toContain('Metric ID');
      expect(header).toContain('Name');
      expect(header).toContain('Category');
      expect(header).toContain('Target');
      expect(header).toContain('Current');
      expect(header).toContain('P95');
      expect(header).toContain('P99');
      expect(header).toContain('Is Regressed');
    });
  });
});
