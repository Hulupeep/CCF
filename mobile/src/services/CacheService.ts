/**
 * Cache Service for Offline Mode
 * Issue: #88 (STORY-MOBILE-001)
 * Invariant: I-MOBILE-003 - Cache last known state for 24 hours
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import { AppState, Drawing, NeuralState, PersonalityConfig } from '../types';

const CACHE_KEYS = {
  APP_STATE: '@mbot/app_state',
  DRAWINGS: '@mbot/drawings',
  NEURAL_STATE: '@mbot/neural_state',
  PERSONALITY: '@mbot/personality',
  LAST_SYNC: '@mbot/last_sync',
};

export class CacheService {
  private cacheExpiryHours: number = 24; // I-MOBILE-003

  /**
   * Set cache expiry time (must be at least 24 hours per I-MOBILE-003)
   */
  setCacheExpiry(hours: number): void {
    if (hours < 24) {
      console.warn('[Cache] Cache expiry cannot be less than 24 hours (I-MOBILE-003)');
      this.cacheExpiryHours = 24;
    } else {
      this.cacheExpiryHours = hours;
    }
  }

  /**
   * Save complete app state
   */
  async saveAppState(state: AppState): Promise<void> {
    try {
      await AsyncStorage.setItem(CACHE_KEYS.APP_STATE, JSON.stringify(state));
      await AsyncStorage.setItem(CACHE_KEYS.LAST_SYNC, Date.now().toString());
      console.log('[Cache] App state saved');
    } catch (error) {
      console.error('[Cache] Failed to save app state:', error);
      throw error;
    }
  }

  /**
   * Load app state from cache
   * Returns null if cache expired (> 24 hours old)
   */
  async loadAppState(): Promise<AppState | null> {
    try {
      const lastSyncStr = await AsyncStorage.getItem(CACHE_KEYS.LAST_SYNC);
      if (!lastSyncStr) {
        return null;
      }

      const lastSync = parseInt(lastSyncStr, 10);
      const expiryTime = this.cacheExpiryHours * 60 * 60 * 1000;

      // I-MOBILE-003: Check if cache expired
      if (Date.now() - lastSync > expiryTime) {
        console.log('[Cache] Cache expired');
        await this.clearCache();
        return null;
      }

      const stateStr = await AsyncStorage.getItem(CACHE_KEYS.APP_STATE);
      if (!stateStr) {
        return null;
      }

      const state = JSON.parse(stateStr) as AppState;
      console.log('[Cache] App state loaded from cache');
      return state;
    } catch (error) {
      console.error('[Cache] Failed to load app state:', error);
      return null;
    }
  }

  /**
   * Save drawings to cache
   */
  async saveDrawings(drawings: Drawing[]): Promise<void> {
    try {
      await AsyncStorage.setItem(CACHE_KEYS.DRAWINGS, JSON.stringify(drawings));
      console.log(`[Cache] Saved ${drawings.length} drawings`);
    } catch (error) {
      console.error('[Cache] Failed to save drawings:', error);
      throw error;
    }
  }

  /**
   * Load drawings from cache
   */
  async loadDrawings(): Promise<Drawing[]> {
    try {
      const drawingsStr = await AsyncStorage.getItem(CACHE_KEYS.DRAWINGS);
      if (!drawingsStr) {
        return [];
      }

      const drawings = JSON.parse(drawingsStr) as Drawing[];
      console.log(`[Cache] Loaded ${drawings.length} drawings from cache`);
      return drawings;
    } catch (error) {
      console.error('[Cache] Failed to load drawings:', error);
      return [];
    }
  }

  /**
   * Save neural state to cache
   */
  async saveNeuralState(state: NeuralState): Promise<void> {
    try {
      await AsyncStorage.setItem(CACHE_KEYS.NEURAL_STATE, JSON.stringify(state));
    } catch (error) {
      console.error('[Cache] Failed to save neural state:', error);
    }
  }

  /**
   * Load neural state from cache
   */
  async loadNeuralState(): Promise<NeuralState | null> {
    try {
      const stateStr = await AsyncStorage.getItem(CACHE_KEYS.NEURAL_STATE);
      if (!stateStr) {
        return null;
      }
      return JSON.parse(stateStr) as NeuralState;
    } catch (error) {
      console.error('[Cache] Failed to load neural state:', error);
      return null;
    }
  }

  /**
   * Save personality config to cache
   */
  async savePersonality(config: PersonalityConfig): Promise<void> {
    try {
      await AsyncStorage.setItem(CACHE_KEYS.PERSONALITY, JSON.stringify(config));
    } catch (error) {
      console.error('[Cache] Failed to save personality:', error);
    }
  }

  /**
   * Load personality config from cache
   */
  async loadPersonality(): Promise<PersonalityConfig | null> {
    try {
      const configStr = await AsyncStorage.getItem(CACHE_KEYS.PERSONALITY);
      if (!configStr) {
        return null;
      }
      return JSON.parse(configStr) as PersonalityConfig;
    } catch (error) {
      console.error('[Cache] Failed to load personality:', error);
      return null;
    }
  }

  /**
   * Clear all cached data
   */
  async clearCache(): Promise<void> {
    try {
      await AsyncStorage.multiRemove(Object.values(CACHE_KEYS));
      console.log('[Cache] All cache cleared');
    } catch (error) {
      console.error('[Cache] Failed to clear cache:', error);
      throw error;
    }
  }

  /**
   * Get cache age in milliseconds
   */
  async getCacheAge(): Promise<number | null> {
    try {
      const lastSyncStr = await AsyncStorage.getItem(CACHE_KEYS.LAST_SYNC);
      if (!lastSyncStr) {
        return null;
      }
      const lastSync = parseInt(lastSyncStr, 10);
      return Date.now() - lastSync;
    } catch (error) {
      console.error('[Cache] Failed to get cache age:', error);
      return null;
    }
  }

  /**
   * Check if cache is valid
   */
  async isCacheValid(): Promise<boolean> {
    const age = await this.getCacheAge();
    if (age === null) {
      return false;
    }
    const expiryTime = this.cacheExpiryHours * 60 * 60 * 1000;
    return age < expiryTime;
  }
}
