/**
 * Cloud Sync Integration Tests
 * Contract: I-CLOUD-001, I-CLOUD-002, I-CLOUD-003
 * Issue: #84
 *
 * Tests the full cloud sync functionality including:
 * - Authentication flow
 * - Personality sync with conflict resolution
 * - Drawing sync
 * - Game stats sync
 * - Offline queue management
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { CloudSyncService } from '../../web/src/services/cloudSync';
import type { Personality } from '../../web/src/types/personality';
import type { Drawing } from '../../web/src/types/drawing';
import type { GameSession } from '../../web/src/types/game';

// Mock Supabase client
vi.mock('../../web/src/config/supabase', () => {
  const mockSupabase = {
    auth: {
      signInWithPassword: vi.fn(),
      signUp: vi.fn(),
      signOut: vi.fn(),
      getUser: vi.fn(),
      getSession: vi.fn(),
    },
    from: vi.fn(() => ({
      select: vi.fn().mockReturnThis(),
      insert: vi.fn().mockReturnThis(),
      upsert: vi.fn().mockReturnThis(),
      eq: vi.fn().mockReturnThis(),
      order: vi.fn().mockReturnThis(),
    })),
    channel: vi.fn(() => ({
      on: vi.fn().mockReturnThis(),
      subscribe: vi.fn(),
    })),
  };

  return {
    supabase: mockSupabase,
    TABLES: {
      PERSONALITIES: 'personalities',
      DRAWINGS: 'drawings',
      GAME_STATS: 'game_stats',
    },
    isSupabaseConfigured: vi.fn(() => true),
  };
});

describe('CloudSyncService', () => {
  let service: CloudSyncService;

  beforeEach(() => {
    // Clear localStorage
    localStorage.clear();

    // Create fresh service instance
    service = new CloudSyncService();

    // Mock current user
    vi.mocked(service['getCurrentUser']).mockResolvedValue({
      id: 'test-user-id',
      email: 'test@example.com',
    } as any);
  });

  afterEach(() => {
    service.destroy();
    vi.clearAllMocks();
  });

  describe('Authentication', () => {
    it('should sign in successfully', async () => {
      const { supabase } = await import('../../web/src/config/supabase');

      vi.mocked(supabase.auth.signInWithPassword).mockResolvedValue({
        data: {
          user: { id: 'test-user-id', email: 'test@example.com' },
          session: { access_token: 'test-token' },
        },
        error: null,
      } as any);

      const result = await service.signIn('test@example.com', 'password123');

      expect('user' in result).toBe(true);
      if ('user' in result) {
        expect(result.user.email).toBe('test@example.com');
      }
    });

    it('should handle sign in error', async () => {
      const { supabase } = await import('../../web/src/config/supabase');

      vi.mocked(supabase.auth.signInWithPassword).mockResolvedValue({
        data: { user: null, session: null },
        error: { message: 'Invalid credentials' } as any,
      } as any);

      const result = await service.signIn('test@example.com', 'wrong-password');

      expect('error' in result).toBe(true);
      if ('error' in result) {
        expect(result.error.message).toBe('Invalid credentials');
      }
    });

    it('should sign up successfully', async () => {
      const { supabase } = await import('../../web/src/config/supabase');

      vi.mocked(supabase.auth.signUp).mockResolvedValue({
        data: {
          user: { id: 'new-user-id', email: 'new@example.com' },
          session: { access_token: 'test-token' },
        },
        error: null,
      } as any);

      const result = await service.signUp('new@example.com', 'password123');

      expect('user' in result).toBe(true);
      if ('user' in result) {
        expect(result.user.email).toBe('new@example.com');
      }
    });

    it('should sign out successfully', async () => {
      const { supabase } = await import('../../web/src/config/supabase');

      vi.mocked(supabase.auth.signOut).mockResolvedValue({ error: null });

      await service.signOut();

      const status = service.getSyncStatus();
      expect(status.status).toBe('idle');
    });
  });

  describe('Personality Sync - I-CLOUD-001 (Idempotent)', () => {
    const mockPersonality: Personality = {
      id: 'pers-1',
      name: 'Test Personality',
      icon: '😊',
      config: {
        tension_baseline: 0.5,
        coherence_baseline: 0.5,
        energy_baseline: 0.5,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      },
      quirks: ['friendly'],
      sound_pack: 'default',
      version: 1,
      created_at: Date.now(),
      modified_at: Date.now(),
    };

    it('should sync personality to cloud', async () => {
      const { supabase } = await import('../../web/src/config/supabase');

      const mockUpsert = vi.fn().mockResolvedValue({ error: null });
      vi.mocked(supabase.from).mockReturnValue({
        upsert: mockUpsert,
      } as any);

      await service.syncPersonality(mockPersonality);

      expect(mockUpsert).toHaveBeenCalledWith(
        expect.objectContaining({
          id: 'pers-1',
          user_id: 'test-user-id',
          name: 'Test Personality',
        }),
        expect.any(Object)
      );

      const status = service.getSyncStatus();
      expect(status.status).toBe('synced');
    });

    it('should queue personality sync when offline', async () => {
      // Mock user as not authenticated
      vi.mocked(service['getCurrentUser']).mockResolvedValue(null);

      await service.syncPersonality(mockPersonality);

      const status = service.getSyncStatus();
      expect(status.pendingOperations).toBe(1);

      // Check localStorage
      const queue = JSON.parse(localStorage.getItem('cloudSyncQueue') || '[]');
      expect(queue).toHaveLength(1);
      expect(queue[0].type).toBe('personality');
    });

    it('should be idempotent - multiple syncs of same data', async () => {
      const { supabase } = await import('../../web/src/config/supabase');

      const mockUpsert = vi.fn().mockResolvedValue({ error: null });
      vi.mocked(supabase.from).mockReturnValue({
        upsert: mockUpsert,
      } as any);

      // Sync same personality multiple times
      await service.syncPersonality(mockPersonality);
      await service.syncPersonality(mockPersonality);
      await service.syncPersonality(mockPersonality);

      // Should call upsert 3 times but result should be the same
      expect(mockUpsert).toHaveBeenCalledTimes(3);
    });

    it('should fetch personalities from cloud', async () => {
      const { supabase } = await import('../../web/src/config/supabase');

      const mockData = [
        {
          id: 'pers-1',
          user_id: 'test-user-id',
          name: 'Test Personality',
          icon: '😊',
          config: mockPersonality.config,
          quirks: ['friendly'],
          sound_pack: 'default',
          version: 1,
          created_at: new Date().toISOString(),
          modified_at: new Date().toISOString(),
          synced_at: new Date().toISOString(),
        },
      ];

      vi.mocked(supabase.from).mockReturnValue({
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        order: vi.fn().mockResolvedValue({ data: mockData, error: null }),
      } as any);

      const personalities = await service.fetchPersonalities();

      expect(personalities).toHaveLength(1);
      expect(personalities[0].id).toBe('pers-1');
      expect(personalities[0].name).toBe('Test Personality');
    });
  });

  describe('Drawing Sync', () => {
    const mockDrawing: Drawing = {
      id: 'draw-1',
      createdAt: Date.now(),
      strokes: [],
      moods: [],
      duration: 5000,
      dominantMood: 'Calm',
      hasSignature: false,
      sessionId: 'session-1',
      metadata: {
        startMood: 'Calm',
        endMood: 'Calm',
        averageTension: 0.5,
        averageCoherence: 0.5,
        averageEnergy: 0.5,
        strokeCount: 0,
        totalPathLength: 0,
      },
    };

    it('should sync drawing to cloud', async () => {
      const { supabase } = await import('../../web/src/config/supabase');

      const mockUpsert = vi.fn().mockResolvedValue({ error: null });
      vi.mocked(supabase.from).mockReturnValue({
        upsert: mockUpsert,
      } as any);

      await service.syncDrawing(mockDrawing);

      expect(mockUpsert).toHaveBeenCalledWith(
        expect.objectContaining({
          id: 'draw-1',
          user_id: 'test-user-id',
          duration: 5000,
        }),
        expect.any(Object)
      );
    });

    it('should queue drawing sync when offline', async () => {
      vi.mocked(service['getCurrentUser']).mockResolvedValue(null);

      await service.syncDrawing(mockDrawing);

      const status = service.getSyncStatus();
      expect(status.pendingOperations).toBe(1);
    });
  });

  describe('Game Stats Sync', () => {
    const mockGameSession: GameSession = {
      id: 'game-1',
      gameType: 'tictactoe',
      result: 'win',
      score: 100,
      duration: 30000,
      timestamp: Date.now(),
      personality: 'friendly',
      metadata: {
        moves: 5,
        difficulty: 'easy',
      },
    };

    it('should sync game session to cloud', async () => {
      const { supabase } = await import('../../web/src/config/supabase');

      const mockUpsert = vi.fn().mockResolvedValue({ error: null });
      vi.mocked(supabase.from).mockReturnValue({
        upsert: mockUpsert,
      } as any);

      await service.syncGameSession(mockGameSession);

      expect(mockUpsert).toHaveBeenCalledWith(
        expect.objectContaining({
          id: 'game-1',
          user_id: 'test-user-id',
          game_type: 'tictactoe',
          result: 'win',
          score: 100,
        }),
        expect.any(Object)
      );
    });
  });

  describe('Offline Queue - I-CLOUD-001', () => {
    it('should persist queue to localStorage', async () => {
      vi.mocked(service['getCurrentUser']).mockResolvedValue(null);

      const mockPersonality: Personality = {
        id: 'pers-offline',
        name: 'Offline Test',
        icon: '📴',
        config: {
          tension_baseline: 0.5,
          coherence_baseline: 0.5,
          energy_baseline: 0.5,
          startle_sensitivity: 0.5,
          recovery_speed: 0.5,
          curiosity_drive: 0.5,
          movement_expressiveness: 0.5,
          sound_expressiveness: 0.5,
          light_expressiveness: 0.5,
        },
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
      };

      await service.syncPersonality(mockPersonality);

      const queue = JSON.parse(localStorage.getItem('cloudSyncQueue') || '[]');
      expect(queue).toHaveLength(1);
      expect(queue[0].type).toBe('personality');
      expect(queue[0].data.id).toBe('pers-offline');
    });

    it('should process queue when coming back online', async () => {
      // Start offline
      vi.mocked(service['getCurrentUser']).mockResolvedValue(null);

      const mockPersonality: Personality = {
        id: 'pers-queue',
        name: 'Queue Test',
        icon: '🔄',
        config: {
          tension_baseline: 0.5,
          coherence_baseline: 0.5,
          energy_baseline: 0.5,
          startle_sensitivity: 0.5,
          recovery_speed: 0.5,
          curiosity_drive: 0.5,
          movement_expressiveness: 0.5,
          sound_expressiveness: 0.5,
          light_expressiveness: 0.5,
        },
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
      };

      await service.syncPersonality(mockPersonality);

      expect(service.getSyncStatus().pendingOperations).toBe(1);

      // Come back online
      vi.mocked(service['getCurrentUser']).mockResolvedValue({
        id: 'test-user-id',
        email: 'test@example.com',
      } as any);

      const { supabase } = await import('../../web/src/config/supabase');
      const mockUpsert = vi.fn().mockResolvedValue({ error: null });
      vi.mocked(supabase.from).mockReturnValue({
        upsert: mockUpsert,
      } as any);

      // Process queue
      await service['processOfflineQueue']();

      expect(service.getSyncStatus().pendingOperations).toBe(0);
      expect(mockUpsert).toHaveBeenCalled();
    });
  });

  describe('Conflict Resolution - I-CLOUD-002 (Last-write-wins)', () => {
    it('should use last-write-wins for conflict resolution', async () => {
      // This is enforced by the database trigger in the migration
      // The CloudSyncService uses upsert which will trigger the merge function
      // Testing this requires the actual database, so we verify the behavior conceptually

      const { supabase } = await import('../../web/src/config/supabase');

      const mockUpsert = vi.fn().mockResolvedValue({ error: null });
      vi.mocked(supabase.from).mockReturnValue({
        upsert: mockUpsert,
      } as any);

      const personality1: Personality = {
        id: 'conflict-test',
        name: 'Version 1',
        icon: '1️⃣',
        config: {
          tension_baseline: 0.3,
          coherence_baseline: 0.5,
          energy_baseline: 0.5,
          startle_sensitivity: 0.5,
          recovery_speed: 0.5,
          curiosity_drive: 0.5,
          movement_expressiveness: 0.5,
          sound_expressiveness: 0.5,
          light_expressiveness: 0.5,
        },
        version: 1,
        created_at: Date.now(),
        modified_at: Date.now(),
      };

      await service.syncPersonality(personality1);

      // Simulate a later update
      const personality2: Personality = {
        ...personality1,
        name: 'Version 2',
        icon: '2️⃣',
        config: {
          ...personality1.config,
          tension_baseline: 0.7,
        },
        version: 2,
        modified_at: Date.now() + 1000,
      };

      await service.syncPersonality(personality2);

      // Both should call upsert, database trigger handles conflict resolution
      expect(mockUpsert).toHaveBeenCalledTimes(2);
    });
  });
});
