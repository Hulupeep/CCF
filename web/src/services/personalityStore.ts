/**
 * PersonalityStore Service - Singleton Pattern
 * Contract: PERS-004 (localStorage persistence), ARCH-005 (transport abstraction)
 * Invariants:
 * - I-ARCH-PERS-001: Only one instance active
 * - I-ARCH-PERS-002: Atomic updates (no partial states)
 *
 * Issue: #75 - Cross-App Personality Persistence
 */

import { Personality, PersonalityConfig, createDefaultConfig } from '../types/personality';
import {
  IPersonalityStore,
  PersonalityChangeListener,
  StorageResult,
  STORAGE_KEYS,
  STORAGE_VERSION,
} from '../types/personalityStore';

/**
 * Creates a default personality with metadata
 */
function createDefaultPersonality(): Personality {
  const now = Date.now();
  return {
    ...createDefaultConfig(),
    id: 'default',
    name: 'Default',
    icon: '🤖',
    version: 1,
    created_at: now,
    modified_at: now,
  };
}

/**
 * PersonalityStore - Singleton implementation
 * Manages personality state across all apps (ArtBot, GameBot, HelperBot)
 */
class PersonalityStore implements IPersonalityStore {
  private static instance: PersonalityStore | null = null;
  private currentPersonality: Personality;
  private listeners: Set<PersonalityChangeListener> = new Set();

  /**
   * Private constructor enforces singleton pattern
   * Invariant: I-ARCH-PERS-001
   */
  private constructor() {
    this.currentPersonality = createDefaultPersonality();
  }

  /**
   * Get singleton instance
   * Invariant: I-ARCH-PERS-001 - only one instance active
   */
  public static getInstance(): PersonalityStore {
    if (!PersonalityStore.instance) {
      PersonalityStore.instance = new PersonalityStore();
    }
    return PersonalityStore.instance;
  }

  /**
   * Initialize store by loading from disk
   * Should be called on app startup
   */
  public async initialize(): Promise<void> {
    const result = await this.loadFromDisk();
    if (result.success && result.data) {
      // Atomic update - all or nothing
      this.currentPersonality = result.data;
      this.notifyListeners();
    }
  }

  /**
   * Get current personality
   * Returns a deep copy to prevent external mutation
   */
  public getCurrentPersonality(): Personality {
    return { ...this.currentPersonality };
  }

  /**
   * Set personality atomically
   * Invariant: I-ARCH-PERS-002 - atomic updates only
   *
   * @param personality - New personality to set
   */
  public setPersonality(personality: Personality): void {
    // Validate personality before setting
    if (!this.isValidPersonality(personality)) {
      throw new Error('Invalid personality: all parameters must be in range [0.0, 1.0]');
    }

    // Update modified_at timestamp
    const updatedPersonality: Personality = {
      ...personality,
      modified_at: Date.now(),
    };

    // Atomic update - replace entire personality
    this.currentPersonality = updatedPersonality;

    // Persist to disk and notify listeners
    this.persistToDisk().catch((error) => {
      console.error('Failed to persist personality:', error);
    });

    this.notifyListeners();
  }

  /**
   * Update only the config portion while preserving metadata
   *
   * @param config - New personality configuration
   */
  public updateConfig(config: PersonalityConfig): void {
    // Create updated personality with new config
    const updatedPersonality: Personality = {
      ...this.currentPersonality,
      ...config,
      modified_at: Date.now(),
    };

    this.setPersonality(updatedPersonality);
  }

  /**
   * Subscribe to personality changes
   *
   * @param callback - Function to call when personality changes
   * @returns Unsubscribe function
   */
  public subscribeToChanges(callback: PersonalityChangeListener): () => void {
    this.listeners.add(callback);

    // Return unsubscribe function
    return () => {
      this.listeners.delete(callback);
    };
  }

  /**
   * Persist personality to localStorage
   * Contract: PERS-004
   *
   * @returns Promise with storage result
   */
  public async persistToDisk(): Promise<StorageResult<void>> {
    try {
      const personalityJson = JSON.stringify(this.currentPersonality);
      localStorage.setItem(STORAGE_KEYS.CURRENT_PERSONALITY, personalityJson);
      localStorage.setItem(STORAGE_KEYS.PERSONALITY_VERSION, String(STORAGE_VERSION));

      return { success: true };
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error('Failed to persist personality:', errorMessage);
      return {
        success: false,
        error: errorMessage,
      };
    }
  }

  /**
   * Load personality from localStorage
   * Contract: PERS-004
   *
   * @returns Promise with storage result containing personality or error
   */
  public async loadFromDisk(): Promise<StorageResult<Personality>> {
    try {
      const personalityJson = localStorage.getItem(STORAGE_KEYS.CURRENT_PERSONALITY);
      const versionStr = localStorage.getItem(STORAGE_KEYS.PERSONALITY_VERSION);

      // No saved personality
      if (!personalityJson) {
        return {
          success: true,
          data: createDefaultPersonality(),
        };
      }

      // Check version compatibility
      const savedVersion = versionStr ? parseInt(versionStr, 10) : 0;
      if (savedVersion > STORAGE_VERSION) {
        return {
          success: false,
          error: `Incompatible version: saved=${savedVersion}, current=${STORAGE_VERSION}`,
        };
      }

      const personality = JSON.parse(personalityJson) as Personality;

      // Validate loaded personality
      if (!this.isValidPersonality(personality)) {
        return {
          success: false,
          error: 'Invalid personality data in storage',
        };
      }

      return {
        success: true,
        data: personality,
      };
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error('Failed to load personality:', errorMessage);
      return {
        success: false,
        error: errorMessage,
      };
    }
  }

  /**
   * Reset to default personality
   */
  public resetToDefault(): void {
    this.setPersonality(createDefaultPersonality());
  }

  /**
   * Notify all listeners of personality change
   */
  private notifyListeners(): void {
    const personalityCopy = this.getCurrentPersonality();
    this.listeners.forEach((listener) => {
      try {
        listener(personalityCopy);
      } catch (error) {
        console.error('Error in personality change listener:', error);
      }
    });
  }

  /**
   * Validate personality parameters are within bounds [0.0, 1.0]
   * Enforces I-PERS-001 invariant
   *
   * @param personality - Personality to validate
   * @returns true if valid, false otherwise
   */
  private isValidPersonality(personality: Personality): boolean {
    const params: (keyof PersonalityConfig)[] = [
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

    for (const param of params) {
      const value = personality[param];
      if (typeof value !== 'number' || value < 0.0 || value > 1.0) {
        return false;
      }
    }

    // Validate required metadata
    if (!personality.id || !personality.name || typeof personality.version !== 'number') {
      return false;
    }

    return true;
  }

  /**
   * FOR TESTING ONLY: Reset singleton instance
   * Invariant: I-ARCH-PERS-001 - ensure clean state between tests
   */
  public static resetInstance(): void {
    PersonalityStore.instance = null;
  }
}

// Export singleton accessor
export const personalityStore = PersonalityStore.getInstance();

// Export class for testing
export { PersonalityStore };
