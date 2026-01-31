/**
 * PersonalityStore type definitions
 * Contract: PERS-004, ARCH-005
 * Implements cross-app personality persistence
 */

import { Personality, PersonalityConfig } from './personality';

/**
 * Personality change listener callback
 */
export type PersonalityChangeListener = (personality: Personality) => void;

/**
 * Storage result type
 */
export interface StorageResult<T> {
  success: boolean;
  data?: T;
  error?: string;
}

/**
 * PersonalityStore interface
 * Contract requirement from issue #75
 *
 * Invariants:
 * - I-ARCH-PERS-001: Only one instance active (singleton)
 * - I-ARCH-PERS-002: Atomic updates (no partial states)
 */
export interface IPersonalityStore {
  /**
   * Get current active personality
   */
  getCurrentPersonality(): Personality;

  /**
   * Set new personality atomically
   * Invariant: I-ARCH-PERS-002 - must be atomic, no partial states
   */
  setPersonality(personality: Personality): void;

  /**
   * Update personality config while preserving metadata
   */
  updateConfig(config: PersonalityConfig): void;

  /**
   * Subscribe to personality changes
   * Returns unsubscribe function
   */
  subscribeToChanges(callback: PersonalityChangeListener): () => void;

  /**
   * Persist current personality to disk (localStorage)
   * Contract: PERS-004
   */
  persistToDisk(): Promise<StorageResult<void>>;

  /**
   * Load personality from disk on startup
   * Contract: PERS-004
   */
  loadFromDisk(): Promise<StorageResult<Personality>>;

  /**
   * Reset to default personality
   */
  resetToDefault(): void;
}

/**
 * Storage keys for localStorage
 */
export const STORAGE_KEYS = {
  CURRENT_PERSONALITY: 'mbot_current_personality',
  PERSONALITY_VERSION: 'mbot_personality_version',
} as const;

/**
 * Storage version for forward compatibility
 */
export const STORAGE_VERSION = 1;
