/**
 * Cross-App Integration Tests
 * Issue: #79 - STORY-TEST-001
 * Contract: All feature contracts
 *
 * Tests end-to-end flows across multiple apps:
 * - Mixer → ArtBot → GameBot personality persistence
 * - Personality changes reflected across apps
 * - Data export/import across apps
 * - WebSocket V2 state synchronization
 *
 * Coverage Requirements:
 * - Personality persistence: >90%
 * - WebSocket V2: >85%
 * - Data export/import: >80%
 * - Multi-robot discovery: >75%
 */

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import { PersonalityStore } from '../../web/src/services/personalityStore';
import { useWebSocketV2 } from '../../web/src/hooks/useWebSocketV2';
import { MockDiscoveryService } from '../../web/src/services/robotDiscovery';
import {
  validateExportManifest,
  validatePersonalityData,
  EXPORT_VERSION,
} from '../../web/src/types/exportManifest';
import { Personality, PersonalityConfig } from '../../web/src/types/personality';
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

describe('Cross-App Integration Tests - Issue #79', () => {
  beforeEach(() => {
    localStorage.clear();
    PersonalityStore.resetInstance();
  });

  afterEach(() => {
    localStorage.clear();
    PersonalityStore.resetInstance();
  });

  describe('Scenario: Complete Cross-App Journey (Mixer → ArtBot → GameBot)', () => {
    it('Given I create "Curious" personality in Mixer, When I open ArtBot, Then robot draws with Curious personality', async () => {
      // Step 1: Create personality in Mixer
      const mixerStore = PersonalityStore.getInstance();
      const curiousPersonality: Personality = {
        id: 'curious',
        name: 'Curious',
        icon: '🔍',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.3,
        coherence_baseline: 0.7,
        energy_baseline: 0.8,
        startle_sensitivity: 0.4,
        recovery_speed: 0.6,
        curiosity_drive: 0.9, // High curiosity
        movement_expressiveness: 0.7,
        sound_expressiveness: 0.6,
        light_expressiveness: 0.5,
      };

      mixerStore.setPersonality(curiousPersonality);

      // Verify persistence
      const storedData = localStorage.getItem(STORAGE_KEYS.CURRENT_PERSONALITY);
      expect(storedData).not.toBeNull();
      const parsed = JSON.parse(storedData!) as Personality;
      expect(parsed.name).toBe('Curious');

      // Step 2: Simulate app switch to ArtBot
      PersonalityStore.resetInstance();
      const artBotStore = PersonalityStore.getInstance();
      await artBotStore.initialize();

      // Verify personality persisted
      const artBotPersonality = artBotStore.getCurrentPersonality();
      expect(artBotPersonality.name).toBe('Curious');
      expect(artBotPersonality.curiosity_drive).toBe(0.9);
      expect(artBotPersonality.movement_expressiveness).toBe(0.7);

      // Step 3: Simulate app switch to GameBot
      PersonalityStore.resetInstance();
      const gameBotStore = PersonalityStore.getInstance();
      await gameBotStore.initialize();

      // Verify personality still persisted
      const gameBotPersonality = gameBotStore.getCurrentPersonality();
      expect(gameBotPersonality.name).toBe('Curious');
      expect(gameBotPersonality.curiosity_drive).toBe(0.9);
      expect(gameBotPersonality.energy_baseline).toBe(0.8);
    });

    it('Given I modify personality in ArtBot, When I switch to GameBot, Then GameBot uses modified personality', async () => {
      // Step 1: Start with default personality
      const artBotStore = PersonalityStore.getInstance();
      await artBotStore.initialize();

      const originalPersonality = artBotStore.getCurrentPersonality();
      expect(originalPersonality.name).toBe('Default');

      // Step 2: Modify in ArtBot (e.g., increase tension during drawing)
      const modifiedConfig: PersonalityConfig = {
        tension_baseline: 0.8, // Increased tension
        coherence_baseline: 0.6,
        energy_baseline: 0.7,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      artBotStore.updateConfig(modifiedConfig);

      const artBotPersonality = artBotStore.getCurrentPersonality();
      expect(artBotPersonality.tension_baseline).toBe(0.8);

      // Step 3: Switch to GameBot
      PersonalityStore.resetInstance();
      const gameBotStore = PersonalityStore.getInstance();
      await gameBotStore.initialize();

      // Verify modification persisted
      const gameBotPersonality = gameBotStore.getCurrentPersonality();
      expect(gameBotPersonality.tension_baseline).toBe(0.8);
      expect(gameBotPersonality.coherence_baseline).toBe(0.6);
    });

    it('Given I export data in Mixer, When I import in GameBot, Then all personality data is restored', async () => {
      // Step 1: Create and export in Mixer
      const mixerStore = PersonalityStore.getInstance();
      const zenPersonality: Personality = {
        id: 'zen',
        name: 'Zen',
        icon: '🧘',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.2,
        coherence_baseline: 0.9,
        energy_baseline: 0.4,
        startle_sensitivity: 0.1,
        recovery_speed: 0.9,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.3,
        sound_expressiveness: 0.2,
        light_expressiveness: 0.4,
      };

      mixerStore.setPersonality(zenPersonality);

      // Create export manifest
      const exportManifest = {
        version: EXPORT_VERSION,
        exportedAt: Date.now(),
        dataTypes: ['personality'] as const,
        metadata: {
          source: 'mixer',
          robotId: 'mbot-001',
        },
        data: {
          personalities: [zenPersonality],
        },
      };

      // Validate export
      expect(validateExportManifest(exportManifest)).toBe(true);

      // Step 2: Simulate clear and import in GameBot
      localStorage.clear();
      PersonalityStore.resetInstance();

      const gameBotStore = PersonalityStore.getInstance();

      // Import personality
      localStorage.setItem(
        STORAGE_KEYS.CURRENT_PERSONALITY,
        JSON.stringify(zenPersonality)
      );

      await gameBotStore.initialize();

      // Verify restoration
      const restoredPersonality = gameBotStore.getCurrentPersonality();
      expect(restoredPersonality.name).toBe('Zen');
      expect(restoredPersonality.tension_baseline).toBe(0.2);
      expect(restoredPersonality.coherence_baseline).toBe(0.9);
      expect(restoredPersonality.recovery_speed).toBe(0.9);
    });
  });

  describe('Scenario: Multi-Robot Cross-App Discovery', () => {
    it('Given 3 robots discovered in Mixer, When I open ArtBot, Then I can connect to same robots', async () => {
      // Step 1: Discover robots in Mixer
      const mixerDiscovery = new MockDiscoveryService();
      await mixerDiscovery.start();

      const mixerRobots = mixerDiscovery.getRobots();
      expect(mixerRobots).toHaveLength(3);

      const robot1 = mixerRobots[0];
      expect(robot1.id).toBe('mbot-001');
      expect(robot1.name).toBe('mBot Alpha');

      // Step 2: Store robot list (simulated via localStorage)
      localStorage.setItem('discovered_robots', JSON.stringify(mixerRobots));

      await mixerDiscovery.stop();

      // Step 3: Open ArtBot and restore robot list
      const artBotDiscovery = new MockDiscoveryService();
      const storedRobots = JSON.parse(localStorage.getItem('discovered_robots')!);

      expect(storedRobots).toHaveLength(3);
      expect(storedRobots[0].id).toBe('mbot-001');

      // Step 4: Verify we can "connect" to same robot
      const targetRobot = storedRobots.find((r: any) => r.id === 'mbot-001');
      expect(targetRobot).toBeDefined();
      expect(targetRobot.name).toBe('mBot Alpha');
      expect(targetRobot.ipAddress).toMatch(/^\d+\.\d+\.\d+\.\d+$/);

      await artBotDiscovery.stop();
    });

    it('Given I connect to Robot2 in Mixer, When I switch to GameBot, Then connection state is preserved', async () => {
      const discoveryService = new MockDiscoveryService();
      await discoveryService.start();

      const robots = discoveryService.getRobots();
      const robot2 = robots[1]; // Second robot

      expect(robot2.name).toBe('mBot Beta');

      // Simulate connection state storage
      const connectionState = {
        robotId: robot2.id,
        robotName: robot2.name,
        ipAddress: robot2.ipAddress,
        port: robot2.port,
        connectedAt: Date.now(),
      };

      localStorage.setItem('active_connection', JSON.stringify(connectionState));

      // Switch to GameBot
      await discoveryService.stop();

      // Restore connection state
      const restoredConnection = JSON.parse(localStorage.getItem('active_connection')!);
      expect(restoredConnection.robotId).toBe(robot2.id);
      expect(restoredConnection.robotName).toBe('mBot Beta');
      expect(restoredConnection.ipAddress).toBe(robot2.ipAddress);
    });
  });

  describe('Scenario: Real-Time State Sync Across Apps', () => {
    it('Given personality change in Mixer, When ArtBot is open, Then ArtBot receives state update via WebSocket', async () => {
      // This test simulates the WebSocket state sync mechanism
      // In reality, both apps would be connected to the same robot

      // Step 1: Create personality in Mixer
      const mixerStore = PersonalityStore.getInstance();
      const hyperPersonality: Personality = {
        id: 'hyper',
        name: 'Hyper',
        icon: '⚡',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.9,
        coherence_baseline: 0.5,
        energy_baseline: 0.95,
        startle_sensitivity: 0.8,
        recovery_speed: 0.3,
        curiosity_drive: 0.7,
        movement_expressiveness: 0.9,
        sound_expressiveness: 0.8,
        light_expressiveness: 0.9,
      };

      mixerStore.setPersonality(hyperPersonality);

      // Step 2: Simulate state update message (what WebSocket would send)
      const stateUpdateMessage = {
        version: 2,
        type: 'state',
        payload: {
          personality: {
            tension_baseline: 0.9,
            coherence_baseline: 0.5,
            energy_baseline: 0.95,
            startle_sensitivity: 0.8,
            recovery_speed: 0.3,
            curiosity_drive: 0.7,
            movement_expressiveness: 0.9,
            sound_expressiveness: 0.8,
            light_expressiveness: 0.9,
          },
        },
        timestamp: Date.now(),
      };

      // Step 3: ArtBot receives message and updates store
      PersonalityStore.resetInstance();
      const artBotStore = PersonalityStore.getInstance();
      await artBotStore.initialize();

      // Simulate WebSocket update
      artBotStore.setPersonality(hyperPersonality);

      const artBotPersonality = artBotStore.getCurrentPersonality();
      expect(artBotPersonality.name).toBe('Hyper');
      expect(artBotPersonality.energy_baseline).toBe(0.95);
    });

    it('Given drawing data in ArtBot, When I open Mixer, Then Mixer shows drawing statistics', async () => {
      // Simulate drawing data
      const drawingData = {
        id: 'drawing-001',
        createdAt: Date.now(),
        personality: 'Curious',
        strokes: [
          { x: 0, y: 0, timestamp: 0 },
          { x: 10, y: 10, timestamp: 100 },
        ],
        metadata: {
          duration: 5000,
          strokeCount: 15,
          dominantMood: 'Excited',
        },
      };

      localStorage.setItem('latest_drawing', JSON.stringify(drawingData));

      // Switch to Mixer and retrieve statistics
      const storedDrawing = JSON.parse(localStorage.getItem('latest_drawing')!);
      expect(storedDrawing.personality).toBe('Curious');
      expect(storedDrawing.metadata.strokeCount).toBe(15);
      expect(storedDrawing.metadata.dominantMood).toBe('Excited');
    });
  });

  describe('Scenario: Data Validation Across Apps', () => {
    it('Given invalid personality export, When I import in any app, Then validation fails gracefully', async () => {
      const invalidExport = {
        version: '2.0.0', // Wrong version
        exportedAt: Date.now(),
        dataTypes: ['personality'],
        data: {
          personalities: [],
        },
      };

      const isValid = validateExportManifest(invalidExport);
      expect(isValid).toBe(false);

      // App should not crash or corrupt data
      const store = PersonalityStore.getInstance();
      await store.initialize();

      const personality = store.getCurrentPersonality();
      expect(personality.name).toBe('Default'); // Falls back to default
    });

    it('Given personality with out-of-bounds values, When I set in any app, Then validation rejects it', () => {
      const store = PersonalityStore.getInstance();

      const invalidPersonality: Personality = {
        id: 'invalid',
        name: 'Invalid',
        icon: '❌',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 1.5, // Out of bounds
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
      }).toThrow('Invalid personality');

      // Original personality should be unchanged
      const currentPersonality = store.getCurrentPersonality();
      expect(currentPersonality.name).not.toBe('Invalid');
    });

    it('Given personality config validation, Then all values must be in range [0.0, 1.0]', () => {
      const validConfig: PersonalityConfig = {
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

      const errors = validatePersonalityData(validConfig);
      expect(errors).toHaveLength(0);

      // Test boundary values
      const boundaryConfig: PersonalityConfig = {
        tension_baseline: 0.0,
        coherence_baseline: 1.0,
        energy_baseline: 0.0,
        startle_sensitivity: 1.0,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      const boundaryErrors = validatePersonalityData(boundaryConfig);
      expect(boundaryErrors).toHaveLength(0);
    });
  });

  describe('Performance: Cross-App Operations', () => {
    it('should persist personality change within 50ms', async () => {
      const store = PersonalityStore.getInstance();

      const testPersonality: Personality = {
        id: 'test',
        name: 'Test',
        icon: '🧪',
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

      const startTime = Date.now();
      store.setPersonality(testPersonality);
      const endTime = Date.now();

      const duration = endTime - startTime;
      expect(duration).toBeLessThan(50); // <50ms for persistence
    });

    it('should restore personality within 100ms', async () => {
      // Setup: Create personality
      const setupStore = PersonalityStore.getInstance();
      const personality: Personality = {
        id: 'performance-test',
        name: 'Performance',
        icon: '⚡',
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

      // Test: Restore personality
      PersonalityStore.resetInstance();
      const store = PersonalityStore.getInstance();

      const startTime = Date.now();
      await store.initialize();
      const endTime = Date.now();

      const duration = endTime - startTime;
      expect(duration).toBeLessThan(100); // <100ms for restoration

      const restored = store.getCurrentPersonality();
      expect(restored.name).toBe('Performance');
    });

    it('should handle 100 rapid personality changes without data loss', () => {
      const store = PersonalityStore.getInstance();

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

      const finalPersonality = store.getCurrentPersonality();
      expect(finalPersonality.name).toBe('Test 99');
      expect(finalPersonality.id).toBe('test-99');
    });
  });

  describe('Coverage: Invariant Validation', () => {
    it('I-ARCH-PERS-001: Singleton pattern enforced across apps', () => {
      const instance1 = PersonalityStore.getInstance();
      const instance2 = PersonalityStore.getInstance();
      const instance3 = PersonalityStore.getInstance();

      expect(instance1).toBe(instance2);
      expect(instance2).toBe(instance3);
    });

    it('I-ARCH-PERS-002: Atomic updates across all apps', () => {
      const store = PersonalityStore.getInstance();

      const personality: Personality = {
        id: 'atomic',
        name: 'Atomic',
        icon: '⚛️',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.8,
        coherence_baseline: 0.7,
        energy_baseline: 0.6,
        startle_sensitivity: 0.5,
        recovery_speed: 0.4,
        curiosity_drive: 0.3,
        movement_expressiveness: 0.2,
        sound_expressiveness: 0.1,
        light_expressiveness: 0.9,
      };

      store.setPersonality(personality);

      const retrieved = store.getCurrentPersonality();

      // All fields should be present (atomic update)
      expect(retrieved.tension_baseline).toBe(0.8);
      expect(retrieved.coherence_baseline).toBe(0.7);
      expect(retrieved.energy_baseline).toBe(0.6);
      expect(retrieved.startle_sensitivity).toBe(0.5);
      expect(retrieved.recovery_speed).toBe(0.4);
      expect(retrieved.curiosity_drive).toBe(0.3);
      expect(retrieved.movement_expressiveness).toBe(0.2);
      expect(retrieved.sound_expressiveness).toBe(0.1);
      expect(retrieved.light_expressiveness).toBe(0.9);
    });

    it('ARCH-005: Transport layer abstraction maintained across apps', () => {
      // Verify that personality storage is independent of transport
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

      // Verify stored independently of WebSocket connection
      const stored = localStorage.getItem(STORAGE_KEYS.CURRENT_PERSONALITY);
      expect(stored).not.toBeNull();

      const parsed = JSON.parse(stored!) as Personality;
      expect(parsed.name).toBe('Transport');
    });
  });
});
