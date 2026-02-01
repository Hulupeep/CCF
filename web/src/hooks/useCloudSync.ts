/**
 * React Hook for Cloud Sync
 * Contract: I-CLOUD-001, I-CLOUD-002, I-CLOUD-003
 *
 * Provides React integration for CloudSyncService with automatic sync and state management
 */

import { useState, useEffect, useCallback } from 'react';
import { cloudSyncService, type SyncState, type SyncStatus } from '../services/cloudSync';
import type { Personality } from '../types/personality';
import type { Drawing } from '../types/drawing';
import type { GameSession } from '../types/game';
import type { User } from '@supabase/supabase-js';

export interface UseCloudSyncReturn {
  // Auth state
  user: User | null;
  isAuthenticated: boolean;

  // Sync state
  syncStatus: SyncStatus;
  lastSync: number | null;
  error: string | null;
  pendingOperations: number;

  // Auth methods
  signIn: (email: string, password: string) => Promise<void>;
  signUp: (email: string, password: string) => Promise<void>;
  signOut: () => Promise<void>;

  // Sync methods
  syncPersonality: (personality: Personality) => Promise<void>;
  fetchPersonalities: () => Promise<Personality[]>;
  syncDrawing: (drawing: Drawing) => Promise<void>;
  fetchDrawings: () => Promise<Drawing[]>;
  syncGameSession: (session: GameSession) => Promise<void>;
  fetchGameSessions: () => Promise<GameSession[]>;
  forceSyncAll: () => Promise<void>;
}

/**
 * Hook for cloud sync functionality
 *
 * Features:
 * - Automatic state management
 * - Auth state tracking
 * - Sync status monitoring
 * - Error handling
 * - Auto-sync on changes
 */
export function useCloudSync(): UseCloudSyncReturn {
  const [user, setUser] = useState<User | null>(null);
  const [syncState, setSyncState] = useState<SyncState>(cloudSyncService.getSyncStatus());

  // Subscribe to sync state changes
  useEffect(() => {
    const unsubscribe = cloudSyncService.subscribe((state) => {
      setSyncState(state);
    });

    return unsubscribe;
  }, []);

  // Load current user on mount
  useEffect(() => {
    loadCurrentUser();
  }, []);

  const loadCurrentUser = async () => {
    const currentUser = await cloudSyncService.getCurrentUser();
    setUser(currentUser);
  };

  // Auth methods
  const signIn = useCallback(async (email: string, password: string) => {
    const result = await cloudSyncService.signIn(email, password);

    if ('error' in result) {
      throw new Error(result.error.message);
    }

    setUser(result.user);
  }, []);

  const signUp = useCallback(async (email: string, password: string) => {
    const result = await cloudSyncService.signUp(email, password);

    if ('error' in result) {
      throw new Error(result.error.message);
    }

    setUser(result.user);
  }, []);

  const signOut = useCallback(async () => {
    await cloudSyncService.signOut();
    setUser(null);
  }, []);

  // Sync methods
  const syncPersonality = useCallback(async (personality: Personality) => {
    await cloudSyncService.syncPersonality(personality);
  }, []);

  const fetchPersonalities = useCallback(async () => {
    return await cloudSyncService.fetchPersonalities();
  }, []);

  const syncDrawing = useCallback(async (drawing: Drawing) => {
    await cloudSyncService.syncDrawing(drawing);
  }, []);

  const fetchDrawings = useCallback(async () => {
    return await cloudSyncService.fetchDrawings();
  }, []);

  const syncGameSession = useCallback(async (session: GameSession) => {
    await cloudSyncService.syncGameSession(session);
  }, []);

  const fetchGameSessions = useCallback(async () => {
    return await cloudSyncService.fetchGameSessions();
  }, []);

  const forceSyncAll = useCallback(async () => {
    await cloudSyncService.forceSyncAll();
  }, []);

  return {
    user,
    isAuthenticated: Boolean(user),
    syncStatus: syncState.status,
    lastSync: syncState.lastSync,
    error: syncState.error,
    pendingOperations: syncState.pendingOperations,
    signIn,
    signUp,
    signOut,
    syncPersonality,
    fetchPersonalities,
    syncDrawing,
    fetchDrawings,
    syncGameSession,
    fetchGameSessions,
    forceSyncAll,
  };
}

/**
 * Hook for automatic personality sync
 *
 * Automatically syncs personality changes to the cloud when enabled
 */
export function useAutoSyncPersonality(personality: Personality | null, enabled = true) {
  const { syncPersonality, isAuthenticated } = useCloudSync();

  useEffect(() => {
    if (!enabled || !isAuthenticated || !personality) return;

    // Debounce sync to avoid too many requests
    const timeoutId = setTimeout(() => {
      syncPersonality(personality).catch(console.error);
    }, 1000);

    return () => clearTimeout(timeoutId);
  }, [personality, enabled, isAuthenticated, syncPersonality]);
}

/**
 * Hook for automatic drawing sync
 *
 * Automatically syncs new drawings to the cloud when created
 */
export function useAutoSyncDrawing(drawing: Drawing | null, enabled = true) {
  const { syncDrawing, isAuthenticated } = useCloudSync();
  const [syncedIds, setSyncedIds] = useState<Set<string>>(new Set());

  useEffect(() => {
    if (!enabled || !isAuthenticated || !drawing || syncedIds.has(drawing.id)) return;

    syncDrawing(drawing)
      .then(() => {
        setSyncedIds(prev => new Set([...prev, drawing.id]));
      })
      .catch(console.error);
  }, [drawing, enabled, isAuthenticated, syncedIds, syncDrawing]);
}

/**
 * Hook for automatic game session sync
 *
 * Automatically syncs game sessions to the cloud after completion
 */
export function useAutoSyncGameSession(session: GameSession | null, enabled = true) {
  const { syncGameSession, isAuthenticated } = useCloudSync();
  const [syncedIds, setSyncedIds] = useState<Set<string>>(new Set());

  useEffect(() => {
    if (!enabled || !isAuthenticated || !session || syncedIds.has(session.id)) return;

    syncGameSession(session)
      .then(() => {
        setSyncedIds(prev => new Set([...prev, session.id]));
      })
      .catch(console.error);
  }, [session, enabled, isAuthenticated, syncedIds, syncGameSession]);
}
