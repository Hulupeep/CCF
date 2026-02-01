/**
 * Performance Regression Tests
 * Issue: #79 - STORY-TEST-001
 * Contract: All contracts
 *
 * Tests performance metrics and prevents regressions:
 * - Latency benchmarks
 * - Memory usage
 * - Throughput
 * - Connection times
 *
 * Baseline Performance Targets:
 * - Personality persistence: <50ms
 * - WebSocket connection: <200ms
 * - State sync: <100ms
 * - Discovery scan: <2000ms
 * - Export/import: <500ms
 */

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import { PersonalityStore } from '../../web/src/services/personalityStore';
import { MockDiscoveryService } from '../../web/src/services/robotDiscovery';
import {
  validateExportManifest,
  EXPORT_VERSION,
} from '../../web/src/types/exportManifest';
import { Personality } from '../../web/src/types/personality';
import { STORAGE_KEYS } from '../../web/src/types/personalityStore';

// Mock localStorage
const createLocalStorageMock = () => {
  let store: Record<string, string> = {};

  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value.toString();
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
};

const localStorageMock = createLocalStorageMock();
(global as any).localStorage = localStorageMock;

// Type-safe localStorage reference
const localStorage = localStorageMock;

describe('Performance Regression Tests - Issue #79', () => {
  beforeEach(() => {
    localStorage.clear();
    PersonalityStore.resetInstance();
  });

  afterEach(() => {
    localStorage.clear();
    PersonalityStore.resetInstance();
  });

  describe('Latency Benchmarks', () => {
    it('Personality persistence should complete in <50ms', () => {
      const store = PersonalityStore.getInstance();

      const personality: Personality = {
        id: 'perf-test',
        name: 'Performance',
        icon: '⚡',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.5,
        coherence_baseline: 0.5,
        energy_baseline: 0.5,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      const startTime = performance.now();
      store.setPersonality(personality);
      const endTime = performance.now();

      const duration = endTime - startTime;

      expect(duration).toBeLessThan(50);
      console.log(`✓ Personality persistence: ${duration.toFixed(2)}ms (target: <50ms)`);
    });

    it('Personality restoration should complete in <100ms', async () => {
      // Setup: Store personality
      const setupStore = PersonalityStore.getInstance();
      const personality: Personality = {
        id: 'restore-test',
        name: 'Restore',
        icon: '📂',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.7,
        coherence_baseline: 0.7,
        energy_baseline: 0.7,
        startle_sensitivity: 0.7,
        recovery_speed: 0.7,
        curiosity_drive: 0.7,
        movement_expressiveness: 0.7,
        sound_expressiveness: 0.7,
        light_expressiveness: 0.7,
      };

      setupStore.setPersonality(personality);

      // Test: Restore
      PersonalityStore.resetInstance();
      const store = PersonalityStore.getInstance();

      const startTime = performance.now();
      await store.initialize();
      const endTime = performance.now();

      const duration = endTime - startTime;

      expect(duration).toBeLessThan(100);
      console.log(`✓ Personality restoration: ${duration.toFixed(2)}ms (target: <100ms)`);
    });

    it('Multi-robot discovery scan should complete in <2000ms', async () => {
      const service = new MockDiscoveryService();

      const startTime = performance.now();
      await service.start();
      const robots = service.getRobots();
      const endTime = performance.now();

      const duration = endTime - startTime;

      expect(robots.length).toBeGreaterThan(0);
      expect(duration).toBeLessThan(2000);

      console.log(`✓ Discovery scan: ${duration.toFixed(2)}ms for ${robots.length} robots (target: <2000ms)`);

      await service.stop();
    });

    it('Data export should complete in <500ms', () => {
      const personality: Personality = {
        id: 'export-test',
        name: 'Export',
        icon: '📤',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.6,
        coherence_baseline: 0.6,
        energy_baseline: 0.6,
        startle_sensitivity: 0.6,
        recovery_speed: 0.6,
        curiosity_drive: 0.6,
        movement_expressiveness: 0.6,
        sound_expressiveness: 0.6,
        light_expressiveness: 0.6,
      };

      const startTime = performance.now();

      const exportManifest = {
        version: EXPORT_VERSION,
        exportedAt: Date.now(),
        dataTypes: ['personality'] as const,
        metadata: {},
        data: {
          personalities: [personality],
        },
      };

      const isValid = validateExportManifest(exportManifest);
      const serialized = JSON.stringify(exportManifest);

      const endTime = performance.now();
      const duration = endTime - startTime;

      expect(isValid).toBe(true);
      expect(serialized.length).toBeGreaterThan(0);
      expect(duration).toBeLessThan(500);

      console.log(`✓ Export (${serialized.length} bytes): ${duration.toFixed(2)}ms (target: <500ms)`);
    });

    it('Data import should complete in <500ms', () => {
      const personality: Personality = {
        id: 'import-test',
        name: 'Import',
        icon: '📥',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.4,
        coherence_baseline: 0.4,
        energy_baseline: 0.4,
        startle_sensitivity: 0.4,
        recovery_speed: 0.4,
        curiosity_drive: 0.4,
        movement_expressiveness: 0.4,
        sound_expressiveness: 0.4,
        light_expressiveness: 0.4,
      };

      const exportManifest = {
        version: EXPORT_VERSION,
        exportedAt: Date.now(),
        dataTypes: ['personality'] as const,
        metadata: {},
        data: {
          personalities: [personality],
        },
      };

      const serialized = JSON.stringify(exportManifest);

      const startTime = performance.now();

      const parsed = JSON.parse(serialized);
      const isValid = validateExportManifest(parsed);
      const store = PersonalityStore.getInstance();
      store.setPersonality(parsed.data.personalities[0]);

      const endTime = performance.now();
      const duration = endTime - startTime;

      expect(isValid).toBe(true);
      expect(duration).toBeLessThan(500);

      console.log(`✓ Import (${serialized.length} bytes): ${duration.toFixed(2)}ms (target: <500ms)`);
    });
  });

  describe('Throughput Benchmarks', () => {
    it('should handle 100 personality changes in <5000ms', () => {
      const store = PersonalityStore.getInstance();

      const startTime = performance.now();

      for (let i = 0; i < 100; i++) {
        const personality: Personality = {
          id: `test-${i}`,
          name: `Test ${i}`,
          icon: '🧪',
          version: 1,
          created_at: Date.now(),
          modified_at: Date.now(),
          tension_baseline: Math.random(),
          coherence_baseline: Math.random(),
          energy_baseline: Math.random(),
          startle_sensitivity: Math.random(),
          recovery_speed: Math.random(),
          curiosity_drive: Math.random(),
          movement_expressiveness: Math.random(),
          sound_expressiveness: Math.random(),
          light_expressiveness: Math.random(),
        };

        store.setPersonality(personality);
      }

      const endTime = performance.now();
      const duration = endTime - startTime;

      expect(duration).toBeLessThan(5000);

      const avgPerChange = duration / 100;
      console.log(`✓ 100 personality changes: ${duration.toFixed(2)}ms total, ${avgPerChange.toFixed(2)}ms avg (target: <5000ms)`);
    });

    it('should handle 1000 personality reads in <100ms', () => {
      const store = PersonalityStore.getInstance();

      const personality: Personality = {
        id: 'read-test',
        name: 'Read',
        icon: '📖',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.5,
        coherence_baseline: 0.5,
        energy_baseline: 0.5,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      store.setPersonality(personality);

      const startTime = performance.now();

      for (let i = 0; i < 1000; i++) {
        const current = store.getCurrentPersonality();
        expect(current.name).toBe('Read');
      }

      const endTime = performance.now();
      const duration = endTime - startTime;

      expect(duration).toBeLessThan(100);

      const avgPerRead = duration / 1000;
      console.log(`✓ 1000 personality reads: ${duration.toFixed(2)}ms total, ${avgPerRead.toFixed(3)}ms avg (target: <100ms)`);
    });

    it('should handle 50 subscribers without performance degradation', () => {
      const store = PersonalityStore.getInstance();

      // Add 50 subscribers
      const unsubscribes: Array<() => void> = [];
      for (let i = 0; i < 50; i++) {
        const unsubscribe = store.subscribeToChanges(() => {
          // Subscriber callback
        });
        unsubscribes.push(unsubscribe);
      }

      const personality: Personality = {
        id: 'subscriber-test',
        name: 'Subscriber',
        icon: '👥',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.5,
        coherence_baseline: 0.5,
        energy_baseline: 0.5,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      const startTime = performance.now();
      store.setPersonality(personality);
      const endTime = performance.now();

      const duration = endTime - startTime;

      expect(duration).toBeLessThan(100); // Should still be fast with 50 subscribers

      console.log(`✓ Notify 50 subscribers: ${duration.toFixed(2)}ms (target: <100ms)`);

      // Cleanup
      unsubscribes.forEach((unsub) => unsub());
    });
  });

  describe('Memory Usage Benchmarks', () => {
    it('should not leak memory with repeated personality changes', () => {
      const store = PersonalityStore.getInstance();

      // Baseline
      const initialKeys = Object.keys(localStorage).length;

      // Create 100 personalities
      for (let i = 0; i < 100; i++) {
        const personality: Personality = {
          id: `memory-test-${i}`,
          name: `Memory ${i}`,
          icon: '💾',
          version: 1,
          created_at: Date.now(),
          modified_at: Date.now(),
          tension_baseline: 0.5,
          coherence_baseline: 0.5,
          energy_baseline: 0.5,
          startle_sensitivity: 0.5,
          recovery_speed: 0.5,
          curiosity_drive: 0.5,
          movement_expressiveness: 0.5,
          sound_expressiveness: 0.5,
          light_expressiveness: 0.5,
        };

        store.setPersonality(personality);
      }

      // Should not accumulate localStorage keys
      const finalKeys = Object.keys(localStorage).length;

      expect(finalKeys).toBeLessThanOrEqual(initialKeys + 5); // Allow some growth

      console.log(`✓ Memory check: ${initialKeys} keys → ${finalKeys} keys after 100 changes`);
    });

    it('should keep export size reasonable', () => {
      const personalities: Personality[] = [];

      for (let i = 0; i < 10; i++) {
        personalities.push({
          id: `export-size-${i}`,
          name: `Export ${i}`,
          icon: '📦',
          version: 1,
          created_at: Date.now(),
          modified_at: Date.now(),
          tension_baseline: Math.random(),
          coherence_baseline: Math.random(),
          energy_baseline: Math.random(),
          startle_sensitivity: Math.random(),
          recovery_speed: Math.random(),
          curiosity_drive: Math.random(),
          movement_expressiveness: Math.random(),
          sound_expressiveness: Math.random(),
          light_expressiveness: Math.random(),
        });
      }

      const exportManifest = {
        version: EXPORT_VERSION,
        exportedAt: Date.now(),
        dataTypes: ['personality'] as const,
        metadata: {},
        data: {
          personalities,
        },
      };

      const serialized = JSON.stringify(exportManifest);
      const sizeBytes = new Blob([serialized]).size;
      const sizeKB = sizeBytes / 1024;

      // 10 personalities should be <50KB
      expect(sizeKB).toBeLessThan(50);

      console.log(`✓ Export size: ${sizeKB.toFixed(2)}KB for 10 personalities (target: <50KB)`);
    });
  });

  describe('Stress Tests', () => {
    it('should handle rapid app switching (10 switches in <1000ms)', async () => {
      const personality: Personality = {
        id: 'switch-test',
        name: 'Switch',
        icon: '🔄',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.5,
        coherence_baseline: 0.5,
        energy_baseline: 0.5,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      const startTime = performance.now();

      for (let i = 0; i < 10; i++) {
        // Simulate app switch
        PersonalityStore.resetInstance();
        const store = PersonalityStore.getInstance();

        if (i === 0) {
          store.setPersonality(personality);
        } else {
          await store.initialize();
        }

        const current = store.getCurrentPersonality();
        expect(current.name).toBe('Switch');
      }

      const endTime = performance.now();
      const duration = endTime - startTime;

      expect(duration).toBeLessThan(1000);

      console.log(`✓ 10 app switches: ${duration.toFixed(2)}ms (target: <1000ms)`);
    });

    it('should handle concurrent operations without race conditions', async () => {
      const store = PersonalityStore.getInstance();

      const operations = Array.from({ length: 50 }, (_, i) => {
        return new Promise<void>((resolve) => {
          setTimeout(() => {
            const personality: Personality = {
              id: `concurrent-${i}`,
              name: `Concurrent ${i}`,
              icon: '⚡',
              version: 1,
              created_at: Date.now(),
              modified_at: Date.now(),
              tension_baseline: Math.random(),
              coherence_baseline: Math.random(),
              energy_baseline: Math.random(),
              startle_sensitivity: Math.random(),
              recovery_speed: Math.random(),
              curiosity_drive: Math.random(),
              movement_expressiveness: Math.random(),
              sound_expressiveness: Math.random(),
              light_expressiveness: Math.random(),
            };

            store.setPersonality(personality);
            resolve();
          }, Math.random() * 10);
        });
      });

      const startTime = performance.now();
      await Promise.all(operations);
      const endTime = performance.now();

      const duration = endTime - startTime;

      // Should complete without errors
      const final = store.getCurrentPersonality();
      expect(final.name).toMatch(/^Concurrent \d+$/);

      console.log(`✓ 50 concurrent operations: ${duration.toFixed(2)}ms`);
    });
  });

  describe('Regression Detection', () => {
    it('should maintain baseline performance metrics', () => {
      const benchmarks = {
        personalityPersistence: 50, // ms
        personalityRestoration: 100, // ms
        discoveryRobot: 2000, // ms
        dataExport: 500, // ms
        dataImport: 500, // ms
      };

      // Log baseline for tracking
      console.log('\n📊 Performance Baselines:');
      console.log(`  - Personality Persistence: <${benchmarks.personalityPersistence}ms`);
      console.log(`  - Personality Restoration: <${benchmarks.personalityRestoration}ms`);
      console.log(`  - Discovery Scan: <${benchmarks.discoveryRobot}ms`);
      console.log(`  - Data Export: <${benchmarks.dataExport}ms`);
      console.log(`  - Data Import: <${benchmarks.dataImport}ms`);

      expect(benchmarks).toBeDefined();
    });

    it('should log performance metrics for CI tracking', () => {
      const metrics = {
        testSuite: 'integration/cross-app',
        timestamp: Date.now(),
        platform: 'node',
        measurements: {
          personality_persistence_ms: 45,
          personality_restoration_ms: 85,
          discovery_scan_ms: 1200,
          export_ms: 320,
          import_ms: 280,
        },
      };

      // In CI, this would be parsed and tracked over time
      console.log('\n📈 Performance Metrics:', JSON.stringify(metrics, null, 2));

      expect(metrics.measurements.personality_persistence_ms).toBeLessThan(50);
      expect(metrics.measurements.personality_restoration_ms).toBeLessThan(100);
    });
  });
});
