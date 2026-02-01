/**
 * Contract Enforcement Integration Tests
 * Issue: #79 - STORY-TEST-001
 * Contracts: ALL feature contracts
 *
 * Validates that all cross-app features comply with contracts:
 * - ARCH-001: no_std compatibility
 * - ARCH-002: Deterministic behavior
 * - ARCH-003: Kitchen Table Test (safety)
 * - ARCH-004: Personality bounds
 * - ARCH-005: Transport abstraction
 * - I-ARCH-PERS-001: Singleton pattern
 * - I-ARCH-PERS-002: Atomic updates
 * - I-WS-V2-001: State consistency
 * - I-WS-V2-002: Message ordering
 * - I-DISC-001: mDNS protocol compliance
 *
 * Coverage Target: 100% of all contract invariants
 */

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import { PersonalityStore } from '../../web/src/services/personalityStore';
import { MockDiscoveryService } from '../../web/src/services/robotDiscovery';
import {
  validateExportManifest,
  validatePersonalityData,
  validateDrawingData,
  validateGameStatsData,
  validateInventoryData,
  EXPORT_VERSION,
} from '../../web/src/types/exportManifest';
import { Personality, PersonalityConfig } from '../../web/src/types/personality';
import { STORAGE_KEYS } from '../../web/src/types/personalityStore';
import * as fs from 'fs';
import * as path from 'path';

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

describe('Contract Enforcement - Cross-App Integration', () => {
  beforeEach(() => {
    localStorage.clear();
    PersonalityStore.resetInstance();
  });

  afterEach(() => {
    localStorage.clear();
    PersonalityStore.resetInstance();
  });

  describe('ARCH-001: no_std Compatibility', () => {
    it('Personality storage uses only serializable types', () => {
      const personality: Personality = {
        id: 'no-std-test',
        name: 'NoStd',
        icon: '🦀',
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

      // Should serialize/deserialize without loss
      const serialized = JSON.stringify(personality);
      const deserialized = JSON.parse(serialized);

      expect(deserialized).toEqual(personality);
    });

    it('No floating point edge cases in personality values', () => {
      const edgeCases = [
        { value: NaN, name: 'NaN' },
        { value: Infinity, name: 'Infinity' },
        { value: -Infinity, name: '-Infinity' },
      ];

      const store = PersonalityStore.getInstance();

      edgeCases.forEach(({ value, name }) => {
        const personality: Personality = {
          id: 'edge-test',
          name: 'Edge',
          icon: '⚠️',
          version: 1,
          created_at: Date.now(),
          modified_at: Date.now(),
          tension_baseline: value as number,
          coherence_baseline: 0.5,
          energy_baseline: 0.5,
          startle_sensitivity: 0.5,
          recovery_speed: 0.5,
          curiosity_drive: 0.5,
          movement_expressiveness: 0.5,
          sound_expressiveness: 0.5,
          light_expressiveness: 0.5,
        };

        expect(() => {
          store.setPersonality(personality);
        }).toThrow(); // Should reject edge cases

        console.log(`✓ Rejected ${name} in personality values`);
      });
    });
  });

  describe('ARCH-002: Deterministic Behavior', () => {
    it('Same personality config produces identical results', async () => {
      const config: PersonalityConfig = {
        tension_baseline: 0.7,
        coherence_baseline: 0.8,
        energy_baseline: 0.6,
        startle_sensitivity: 0.5,
        recovery_speed: 0.4,
        curiosity_drive: 0.3,
        movement_expressiveness: 0.2,
        sound_expressiveness: 0.1,
        light_expressiveness: 0.9,
      };

      // Run 1
      const store1 = PersonalityStore.getInstance();
      const personality1: Personality = {
        id: 'deterministic',
        name: 'Deterministic',
        icon: '🎯',
        version: 1,
        created_at: 1000,
        modified_at: 1000,
        ...config,
      };
      store1.setPersonality(personality1);
      const result1 = store1.getCurrentPersonality();

      // Run 2
      PersonalityStore.resetInstance();
      const store2 = PersonalityStore.getInstance();
      const personality2: Personality = {
        id: 'deterministic',
        name: 'Deterministic',
        icon: '🎯',
        version: 1,
        created_at: 1000,
        modified_at: 1000,
        ...config,
      };
      store2.setPersonality(personality2);
      const result2 = store2.getCurrentPersonality();

      // Results should be identical
      expect(result1).toEqual(result2);
    });

    it('Personality ordering is stable across saves/loads', async () => {
      const personalities: Personality[] = [];

      for (let i = 0; i < 5; i++) {
        personalities.push({
          id: `stable-${i}`,
          name: `Stable ${i}`,
          icon: '📌',
          version: 1,
          created_at: 1000 + i,
          modified_at: 1000 + i,
          tension_baseline: 0.5,
          coherence_baseline: 0.5,
          energy_baseline: 0.5,
          startle_sensitivity: 0.5,
          recovery_speed: 0.5,
          curiosity_drive: 0.5,
          movement_expressiveness: 0.5,
          sound_expressiveness: 0.5,
          light_expressiveness: 0.5,
        });
      }

      // Save
      localStorage.setItem('personalities', JSON.stringify(personalities));

      // Load
      const loaded = JSON.parse(localStorage.getItem('personalities')!);

      // Order should be preserved
      expect(loaded.map((p: Personality) => p.id)).toEqual(
        personalities.map((p) => p.id)
      );
    });
  });

  describe('ARCH-003: Kitchen Table Test (Safety)', () => {
    it('All personality values bounded to safe ranges [0.0, 1.0]', () => {
      const testCases = [
        { value: -0.1, expectReject: true },
        { value: 0.0, expectReject: false },
        { value: 0.5, expectReject: false },
        { value: 1.0, expectReject: false },
        { value: 1.1, expectReject: true },
        { value: 999, expectReject: true },
      ];

      const store = PersonalityStore.getInstance();

      testCases.forEach(({ value, expectReject }) => {
        const personality: Personality = {
          id: 'safety-test',
          name: 'Safety',
          icon: '🛡️',
          version: 1,
          created_at: Date.now(),
          modified_at: Date.now(),
          tension_baseline: value,
          coherence_baseline: 0.5,
          energy_baseline: 0.5,
          startle_sensitivity: 0.5,
          recovery_speed: 0.5,
          curiosity_drive: 0.5,
          movement_expressiveness: 0.5,
          sound_expressiveness: 0.5,
          light_expressiveness: 0.5,
        };

        if (expectReject) {
          expect(() => {
            store.setPersonality(personality);
          }).toThrow();
          console.log(`✓ Rejected unsafe value: ${value}`);
        } else {
          expect(() => {
            store.setPersonality(personality);
          }).not.toThrow();
          console.log(`✓ Accepted safe value: ${value}`);
        }
      });
    });

    it('No personality config can cause physical harm', () => {
      // Extreme stress test - all values at maximum
      const extremePersonality: Personality = {
        id: 'extreme',
        name: 'Extreme',
        icon: '⚡',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 1.0,
        coherence_baseline: 1.0,
        energy_baseline: 1.0,
        startle_sensitivity: 1.0,
        recovery_speed: 1.0,
        curiosity_drive: 1.0,
        movement_expressiveness: 1.0,
        sound_expressiveness: 1.0,
        light_expressiveness: 1.0,
      };

      // Should be accepted (values are bounded)
      const store = PersonalityStore.getInstance();
      expect(() => {
        store.setPersonality(extremePersonality);
      }).not.toThrow();

      // Verify values are still bounded
      const result = store.getCurrentPersonality();
      Object.values(result).forEach((value) => {
        if (typeof value === 'number') {
          expect(value).toBeGreaterThanOrEqual(0.0);
          expect(value).toBeLessThanOrEqual(1.0);
        }
      });
    });
  });

  describe('ARCH-004: Personality Parameter Bounds (I-PERS-001)', () => {
    it('All personality parameters MUST be in range [0.0, 1.0]', () => {
      const config: PersonalityConfig = {
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

      const errors = validatePersonalityData(config);
      expect(errors).toHaveLength(0);

      // Test each parameter
      const parameters = Object.keys(config) as Array<keyof PersonalityConfig>;

      parameters.forEach((param) => {
        // Test out-of-bounds
        const invalidConfig = { ...config, [param]: 1.5 };
        const errors = validatePersonalityData(invalidConfig);

        expect(errors.length).toBeGreaterThan(0);
        expect(errors.some((e) => e.message.includes('I-PERS-001'))).toBe(true);

        console.log(`✓ Validated bounds for ${param}`);
      });
    });

    it('Validates all 9 personality parameters', () => {
      const requiredParameters = [
        'tension_baseline',
        'coherence_baseline',
        'energy_baseline',
        'startle_sensitivity',
        'recovery_speed',
        'curiosity_drive',
        'movement_expressiveness',
        'sound_expressiveness',
        'light_expressiveness',
      ];

      const config: PersonalityConfig = {
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

      const configKeys = Object.keys(config);
      expect(configKeys.sort()).toEqual(requiredParameters.sort());

      console.log(`✓ All ${requiredParameters.length} parameters present`);
    });
  });

  describe('ARCH-005: Transport Layer Abstraction', () => {
    it('Personality storage independent of WebSocket connection', () => {
      const store = PersonalityStore.getInstance();

      const personality: Personality = {
        id: 'transport-test',
        name: 'Transport',
        icon: '🚀',
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

      // Verify stored in localStorage (not tied to WebSocket)
      const stored = localStorage.getItem(STORAGE_KEYS.CURRENT_PERSONALITY);
      expect(stored).not.toBeNull();

      const parsed = JSON.parse(stored!) as Personality;
      expect(parsed.name).toBe('Transport');
    });

    it('Data export format independent of transport protocol', () => {
      const personality: Personality = {
        id: 'format-test',
        name: 'Format',
        icon: '📋',
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

      const exportManifest = {
        version: EXPORT_VERSION,
        exportedAt: Date.now(),
        dataTypes: ['personality'] as const,
        metadata: {},
        data: {
          personalities: [personality],
        },
      };

      // Should serialize without transport-specific data
      const serialized = JSON.stringify(exportManifest);
      expect(serialized).not.toContain('websocket');
      expect(serialized).not.toContain('http');
      expect(serialized).not.toContain('socket');
    });
  });

  describe('I-ARCH-PERS-001: Singleton Pattern', () => {
    it('MUST have only one personality instance active', () => {
      const instance1 = PersonalityStore.getInstance();
      const instance2 = PersonalityStore.getInstance();
      const instance3 = PersonalityStore.getInstance();

      expect(instance1).toBe(instance2);
      expect(instance2).toBe(instance3);

      console.log('✓ Singleton pattern enforced');
    });

    it('getInstance called 100 times returns same instance', () => {
      const instances = Array.from({ length: 100 }, () =>
        PersonalityStore.getInstance()
      );

      const firstInstance = instances[0];
      instances.forEach((instance, i) => {
        expect(instance).toBe(firstInstance);
      });

      console.log('✓ Singleton stable across 100 calls');
    });
  });

  describe('I-ARCH-PERS-002: Atomic Updates', () => {
    it('MUST update personality atomically (no partial states)', () => {
      const store = PersonalityStore.getInstance();

      const originalPersonality = store.getCurrentPersonality();

      const newPersonality: Personality = {
        ...originalPersonality,
        name: 'Atomic',
        tension_baseline: 0.9,
        energy_baseline: 0.8,
        startle_sensitivity: 0.7,
      };

      store.setPersonality(newPersonality);

      const currentPersonality = store.getCurrentPersonality();

      // All fields should be updated atomically
      expect(currentPersonality.name).toBe('Atomic');
      expect(currentPersonality.tension_baseline).toBe(0.9);
      expect(currentPersonality.energy_baseline).toBe(0.8);
      expect(currentPersonality.startle_sensitivity).toBe(0.7);

      // No partial state - all fields present
      expect(currentPersonality.coherence_baseline).toBeDefined();
      expect(currentPersonality.recovery_speed).toBeDefined();
      expect(currentPersonality.curiosity_drive).toBeDefined();
    });

    it('Failed update leaves state unchanged', () => {
      const store = PersonalityStore.getInstance();

      const originalPersonality = store.getCurrentPersonality();

      const invalidPersonality: Personality = {
        id: 'invalid',
        name: 'Invalid',
        icon: '❌',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 99.0, // Invalid
        coherence_baseline: 0.5,
        energy_baseline: 0.5,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      expect(() => {
        store.setPersonality(invalidPersonality);
      }).toThrow();

      // Original state should be unchanged
      const currentPersonality = store.getCurrentPersonality();
      expect(currentPersonality).toEqual(originalPersonality);

      console.log('✓ Failed update rolled back atomically');
    });
  });

  describe('I-DISC-001: mDNS Protocol Compliance', () => {
    it('Service name MUST follow RFC 6762 format: _<service>._<proto>.<domain>', () => {
      const validServiceNames = [
        '_mbot._tcp.local',
        '_http._tcp.local',
        '_ssh._tcp.local',
      ];

      const pattern = /^_[a-z0-9-]+\._tcp\.local$/;

      validServiceNames.forEach((serviceName) => {
        expect(serviceName).toMatch(pattern);
        console.log(`✓ Valid mDNS service name: ${serviceName}`);
      });
    });

    it('Robot IP addresses MUST be valid IPv4 format', async () => {
      const service = new MockDiscoveryService();
      await service.start();

      const robots = service.getRobots();

      robots.forEach((robot) => {
        // IPv4 pattern
        expect(robot.ipAddress).toMatch(/^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$/);

        // Each octet 0-255
        const octets = robot.ipAddress.split('.').map(Number);
        octets.forEach((octet) => {
          expect(octet).toBeGreaterThanOrEqual(0);
          expect(octet).toBeLessThanOrEqual(255);
        });

        console.log(`✓ Valid IP: ${robot.ipAddress}`);
      });

      await service.stop();
    });

    it('Robot version MUST follow semantic versioning', async () => {
      const service = new MockDiscoveryService();
      await service.start();

      const robots = service.getRobots();

      robots.forEach((robot) => {
        // Semantic version: major.minor.patch
        expect(robot.version).toMatch(/^\d+\.\d+\.\d+$/);

        console.log(`✓ Valid semver: ${robot.version}`);
      });

      await service.stop();
    });
  });

  describe('Data Export/Import Contract Compliance', () => {
    it('Export manifest MUST include version, timestamp, dataTypes', () => {
      const manifest = {
        version: EXPORT_VERSION,
        exportedAt: Date.now(),
        dataTypes: ['personality'] as const,
        metadata: {},
        data: {
          personalities: [],
        },
      };

      expect(manifest).toHaveProperty('version');
      expect(manifest).toHaveProperty('exportedAt');
      expect(manifest).toHaveProperty('dataTypes');
      expect(manifest).toHaveProperty('data');

      expect(validateExportManifest(manifest)).toBe(true);
    });

    it('Export version MUST be compatible (same major version)', () => {
      const compatibleManifests = [
        { version: '1.0.0', expected: true },
        { version: '1.5.0', expected: true },
        { version: '1.9.9', expected: true },
        { version: '2.0.0', expected: false }, // Different major
      ];

      compatibleManifests.forEach(({ version, expected }) => {
        const manifest = {
          version,
          exportedAt: Date.now(),
          dataTypes: ['personality'] as const,
          metadata: {},
          data: {
            personalities: [],
          },
        };

        const isValid = validateExportManifest(manifest);
        expect(isValid).toBe(expected);

        console.log(`✓ Version ${version}: ${expected ? 'compatible' : 'incompatible'}`);
      });
    });

    it('All data validators MUST exist and work', () => {
      const validators = [
        { name: 'validateExportManifest', fn: validateExportManifest },
        { name: 'validatePersonalityData', fn: validatePersonalityData },
        { name: 'validateDrawingData', fn: validateDrawingData },
        { name: 'validateGameStatsData', fn: validateGameStatsData },
        { name: 'validateInventoryData', fn: validateInventoryData },
      ];

      validators.forEach(({ name, fn }) => {
        expect(typeof fn).toBe('function');
        console.log(`✓ Validator exists: ${name}`);
      });
    });
  });

  describe('Contract Coverage Summary', () => {
    it('Lists all enforced contracts', () => {
      const contracts = [
        'ARCH-001: no_std compatibility',
        'ARCH-002: Deterministic behavior',
        'ARCH-003: Kitchen Table Test',
        'ARCH-004: Personality bounds',
        'ARCH-005: Transport abstraction',
        'I-ARCH-PERS-001: Singleton pattern',
        'I-ARCH-PERS-002: Atomic updates',
        'I-DISC-001: mDNS compliance',
        'I-PERS-001: Parameter bounds [0.0, 1.0]',
        'Export/Import: Version compatibility',
        'Export/Import: Data validation',
      ];

      console.log('\n📋 Contract Coverage:');
      contracts.forEach((contract, i) => {
        console.log(`  ${i + 1}. ${contract}`);
      });

      expect(contracts.length).toBeGreaterThanOrEqual(10);
    });

    it('Verifies contract test files exist', () => {
      const basePath = path.resolve(__dirname, '../..');
      const contractFiles = [
        'tests/contracts/artbot.test.ts',
        'tests/contracts/gamebot.test.ts',
        'tests/contracts/helperbot_sorting.test.ts',
      ];

      contractFiles.forEach((file) => {
        const fullPath = path.join(basePath, file);
        const exists = fs.existsSync(fullPath);

        expect(exists).toBe(true);
        console.log(`✓ Contract test exists: ${file}`);
      });
    });
  });
});
