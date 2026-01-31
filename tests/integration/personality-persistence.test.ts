/**
 * Integration tests for PersonalityStore
 * Contract: PERS-004, ARCH-005
 * Validates:
 * - I-ARCH-PERS-001: Singleton pattern
 * - I-ARCH-PERS-002: Atomic updates
 * - Cross-app persistence
 * - Restart survival
 *
 * Issue: #75 - Cross-App Personality Persistence
 */

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import { PersonalityStore } from '../../web/src/services/personalityStore';
import { Personality, PersonalityConfig } from '../../web/src/types/personality';
import { STORAGE_KEYS } from '../../web/src/types/personalityStore';

// Mock localStorage
const localStorageMock = (() => {
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
})();

global.localStorage = localStorageMock as any;

describe('PersonalityStore - Cross-App Persistence', () => {
  let store: PersonalityStore;

  beforeEach(() => {
    // Clear localStorage before each test
    localStorage.clear();
    // Reset singleton instance
    PersonalityStore.resetInstance();
    // Get fresh instance
    store = PersonalityStore.getInstance();
  });

  afterEach(() => {
    localStorage.clear();
    PersonalityStore.resetInstance();
  });

  describe('Scenario: Personality Persists Across Apps', () => {
    it('Given I set personality to "Curious" in mixer, When I switch to ArtBot, Then robot draws with Curious personality', async () => {
      // Given: Set personality in mixer
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

      store.setPersonality(curiousPersonality);

      // Simulate app switch - create new store instance (as if ArtBot loaded)
      PersonalityStore.resetInstance();
      const artBotStore = PersonalityStore.getInstance();
      await artBotStore.initialize();

      // Then: ArtBot should have Curious personality
      const artBotPersonality = artBotStore.getCurrentPersonality();
      expect(artBotPersonality.name).toBe('Curious');
      expect(artBotPersonality.curiosity_drive).toBe(0.9);
    });

    it('When I switch to GameBot, Then robot plays with same personality', async () => {
      // Setup: Create personality in mixer
      const zenPersonality: Personality = {
        id: 'zen',
        name: 'Zen',
        icon: '🧘',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.2, // Very calm
        coherence_baseline: 0.8,
        energy_baseline: 0.4,
        startle_sensitivity: 0.1,
        recovery_speed: 0.9,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.3,
        sound_expressiveness: 0.2,
        light_expressiveness: 0.4,
      };

      store.setPersonality(zenPersonality);

      // Simulate switch to GameBot
      PersonalityStore.resetInstance();
      const gameBotStore = PersonalityStore.getInstance();
      await gameBotStore.initialize();

      // Then: GameBot should have Zen personality
      const gameBotPersonality = gameBotStore.getCurrentPersonality();
      expect(gameBotPersonality.name).toBe('Zen');
      expect(gameBotPersonality.tension_baseline).toBe(0.2);
    });
  });

  describe('Scenario: Personality Survives Restart', () => {
    it('Given I set personality to "Zen", When I restart the companion app, Then robot resumes with Zen personality', async () => {
      // Given: Set personality to Zen
      const zenPersonality: Personality = {
        id: 'zen',
        name: 'Zen',
        icon: '🧘',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.2,
        coherence_baseline: 0.8,
        energy_baseline: 0.4,
        startle_sensitivity: 0.1,
        recovery_speed: 0.9,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.3,
        sound_expressiveness: 0.2,
        light_expressiveness: 0.4,
      };

      store.setPersonality(zenPersonality);

      // Wait for persistence (simulated)
      await new Promise((resolve) => setTimeout(resolve, 10));

      // When: Simulate app restart - clear instance but not localStorage
      PersonalityStore.resetInstance();
      const restartedStore = PersonalityStore.getInstance();
      await restartedStore.initialize();

      // Then: Personality should be restored
      const restoredPersonality = restartedStore.getCurrentPersonality();
      expect(restoredPersonality.name).toBe('Zen');
      expect(restoredPersonality.tension_baseline).toBe(0.2);
      expect(restoredPersonality.recovery_speed).toBe(0.9);
    });
  });

  describe('I-ARCH-PERS-001: Singleton Pattern', () => {
    it('MUST have only one personality instance active', () => {
      const instance1 = PersonalityStore.getInstance();
      const instance2 = PersonalityStore.getInstance();

      // Both should be the exact same instance
      expect(instance1).toBe(instance2);
    });

    it('Multiple getInstance calls should return same instance', () => {
      const instances = Array.from({ length: 10 }, () => PersonalityStore.getInstance());

      // All should be the same instance
      const firstInstance = instances[0];
      instances.forEach((instance) => {
        expect(instance).toBe(firstInstance);
      });
    });
  });

  describe('I-ARCH-PERS-002: Atomic Updates', () => {
    it('MUST update personality atomically (no partial states)', () => {
      const originalPersonality = store.getCurrentPersonality();

      const newPersonality: Personality = {
        ...originalPersonality,
        name: 'Hyper',
        energy_baseline: 0.9,
        startle_sensitivity: 0.8,
      };

      // Set new personality
      store.setPersonality(newPersonality);

      // Get personality immediately - should be complete
      const currentPersonality = store.getCurrentPersonality();
      expect(currentPersonality.name).toBe('Hyper');
      expect(currentPersonality.energy_baseline).toBe(0.9);
      expect(currentPersonality.startle_sensitivity).toBe(0.8);

      // All fields should be present (not partial)
      expect(currentPersonality.tension_baseline).toBeDefined();
      expect(currentPersonality.coherence_baseline).toBeDefined();
      expect(currentPersonality.curiosity_drive).toBeDefined();
    });

    it('Should reject invalid personality (out of bounds)', () => {
      const invalidPersonality: Personality = {
        id: 'invalid',
        name: 'Invalid',
        icon: '❌',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 1.5, // OUT OF BOUNDS
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
  });

  describe('Subscribe/Notify Pattern', () => {
    it('Should notify subscribers when personality changes', () => {
      let notificationCount = 0;
      let lastPersonality: Personality | null = null;

      const unsubscribe = store.subscribeToChanges((personality) => {
        notificationCount++;
        lastPersonality = personality;
      });

      const newPersonality: Personality = {
        id: 'test',
        name: 'TestBot',
        icon: '🧪',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.6,
        coherence_baseline: 0.7,
        energy_baseline: 0.8,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      store.setPersonality(newPersonality);

      expect(notificationCount).toBe(1);
      expect(lastPersonality?.name).toBe('TestBot');

      // Cleanup
      unsubscribe();
    });

    it('Should allow unsubscribing from changes', () => {
      let notificationCount = 0;

      const unsubscribe = store.subscribeToChanges(() => {
        notificationCount++;
      });

      const newPersonality: Personality = {
        id: 'test1',
        name: 'Test1',
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

      store.setPersonality(newPersonality);
      expect(notificationCount).toBe(1);

      // Unsubscribe
      unsubscribe();

      // Set another personality
      store.setPersonality({ ...newPersonality, name: 'Test2' });

      // Should still be 1 (not notified after unsubscribe)
      expect(notificationCount).toBe(1);
    });
  });

  describe('Persistence', () => {
    it('Should persist personality to localStorage', async () => {
      const testPersonality: Personality = {
        id: 'persistent',
        name: 'Persistent',
        icon: '💾',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.7,
        coherence_baseline: 0.6,
        energy_baseline: 0.5,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      store.setPersonality(testPersonality);

      // Check localStorage
      const storedData = localStorage.getItem(STORAGE_KEYS.CURRENT_PERSONALITY);
      expect(storedData).not.toBeNull();

      const parsed = JSON.parse(storedData!) as Personality;
      expect(parsed.name).toBe('Persistent');
      expect(parsed.tension_baseline).toBe(0.7);
    });

    it('Should load personality from localStorage on initialize', async () => {
      const testPersonality: Personality = {
        id: 'loadable',
        name: 'Loadable',
        icon: '📂',
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
        tension_baseline: 0.8,
        coherence_baseline: 0.7,
        energy_baseline: 0.6,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      // Store directly in localStorage
      localStorage.setItem(STORAGE_KEYS.CURRENT_PERSONALITY, JSON.stringify(testPersonality));

      // Create new store instance and initialize
      PersonalityStore.resetInstance();
      const newStore = PersonalityStore.getInstance();
      await newStore.initialize();

      const loadedPersonality = newStore.getCurrentPersonality();
      expect(loadedPersonality.name).toBe('Loadable');
      expect(loadedPersonality.tension_baseline).toBe(0.8);
    });

    it('Should handle missing localStorage gracefully', async () => {
      const result = await store.loadFromDisk();

      expect(result.success).toBe(true);
      expect(result.data?.name).toBe('Default');
    });
  });

  describe('updateConfig helper', () => {
    it('Should update config while preserving metadata', () => {
      const originalPersonality = store.getCurrentPersonality();
      const originalId = originalPersonality.id;
      const originalName = originalPersonality.name;
      const originalIcon = originalPersonality.icon;

      const newConfig: PersonalityConfig = {
        tension_baseline: 0.9,
        coherence_baseline: 0.8,
        energy_baseline: 0.7,
        startle_sensitivity: 0.6,
        recovery_speed: 0.5,
        curiosity_drive: 0.4,
        movement_expressiveness: 0.3,
        sound_expressiveness: 0.2,
        light_expressiveness: 0.1,
      };

      store.updateConfig(newConfig);

      const updatedPersonality = store.getCurrentPersonality();

      // Config should be updated
      expect(updatedPersonality.tension_baseline).toBe(0.9);
      expect(updatedPersonality.energy_baseline).toBe(0.7);

      // Metadata should be preserved
      expect(updatedPersonality.id).toBe(originalId);
      expect(updatedPersonality.name).toBe(originalName);
      expect(updatedPersonality.icon).toBe(originalIcon);
    });
  });
});
