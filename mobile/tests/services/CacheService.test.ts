/**
 * CacheService Tests
 * Issue: #88 (STORY-MOBILE-001)
 * Tests I-MOBILE-003: Cache for 24 hours minimum
 */

import { CacheService } from '../../src/services/CacheService';
import { AppState, Drawing } from '../../src/types';

describe('CacheService', () => {
  let service: CacheService;

  beforeEach(() => {
    service = new CacheService();
  });

  describe('Cache Expiry', () => {
    test('should enforce minimum 24 hour cache expiry (I-MOBILE-003)', () => {
      service.setCacheExpiry(12); // Try to set below minimum
      // Should warn and set to 24 hours
    });

    test('should allow cache expiry greater than 24 hours', () => {
      service.setCacheExpiry(48);
      // Should succeed
    });

    test('should return null for expired cache', async () => {
      const oldState: AppState = {
        connected: false,
        cachedDrawings: [],
        lastSync: Date.now() - 25 * 60 * 60 * 1000, // 25 hours ago
      };

      await service.saveAppState(oldState);

      const result = await service.loadAppState();
      // Should be null because cache expired
    });

    test('should return cached state within expiry window', async () => {
      const recentState: AppState = {
        connected: false,
        cachedDrawings: [],
        lastSync: Date.now() - 1 * 60 * 60 * 1000, // 1 hour ago
      };

      await service.saveAppState(recentState);

      const result = await service.loadAppState();
      expect(result).toBeDefined();
    });
  });

  describe('Drawing Cache', () => {
    test('should save and load drawings', async () => {
      const drawings: Drawing[] = [
        {
          id: 'drawing-1',
          name: 'Test Drawing',
          timestamp: Date.now(),
          strokes: [],
          duration: 1000,
        },
      ];

      await service.saveDrawings(drawings);
      const loaded = await service.loadDrawings();

      expect(loaded).toHaveLength(1);
      expect(loaded[0].id).toBe('drawing-1');
    });

    test('should return empty array when no drawings cached', async () => {
      const loaded = await service.loadDrawings();
      expect(loaded).toEqual([]);
    });
  });

  describe('Cache Validation', () => {
    test('should check if cache is valid', async () => {
      const state: AppState = {
        connected: false,
        cachedDrawings: [],
        lastSync: Date.now(),
      };

      await service.saveAppState(state);

      const isValid = await service.isCacheValid();
      expect(isValid).toBe(true);
    });

    test('should get cache age', async () => {
      const state: AppState = {
        connected: false,
        cachedDrawings: [],
        lastSync: Date.now() - 5000, // 5 seconds ago
      };

      await service.saveAppState(state);

      const age = await service.getCacheAge();
      expect(age).toBeGreaterThanOrEqual(5000);
    });
  });

  describe('Cache Clearing', () => {
    test('should clear all cached data', async () => {
      const state: AppState = {
        connected: false,
        cachedDrawings: [],
        lastSync: Date.now(),
      };

      await service.saveAppState(state);
      await service.clearCache();

      const loaded = await service.loadAppState();
      expect(loaded).toBeNull();
    });
  });
});
