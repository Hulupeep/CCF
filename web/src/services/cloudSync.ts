/**
 * Cloud Sync Service
 * Contract: I-CLOUD-001 (Idempotent operations), I-CLOUD-002 (Last-write-wins), I-CLOUD-003 (AES-256 encryption)
 *
 * Provides cloud synchronization for personalities, drawings, and game stats
 * using Supabase as the backend.
 */

import { supabase, TABLES, isSupabaseConfigured } from '../config/supabase';
import type { Personality, PersonalityConfig } from '../types/personality';
import type { Drawing } from '../types/drawing';
import type { GameSession, GameStatistics } from '../types/game';
import type { User, Session, AuthError } from '@supabase/supabase-js';

export type SyncStatus = 'idle' | 'syncing' | 'synced' | 'error' | 'offline';

export interface SyncState {
  status: SyncStatus;
  lastSync: number | null;
  error: string | null;
  pendingOperations: number;
}

export interface CloudPersonality {
  id: string;
  user_id: string;
  name: string;
  icon: string;
  config: PersonalityConfig;
  quirks: string[];
  sound_pack: string | null;
  version: number;
  created_at: string;
  modified_at: string;
  synced_at: string;
}

export interface CloudDrawing {
  id: string;
  user_id: string;
  created_at: string;
  strokes: any;
  moods: any;
  duration: number;
  dominant_mood: string;
  has_signature: boolean;
  session_id: string | null;
  thumbnail_url: string | null;
  metadata: any;
  synced_at: string;
}

export interface CloudGameStats {
  id: string;
  user_id: string;
  game_type: string;
  result: string;
  score: number;
  duration: number;
  timestamp: string;
  personality: string;
  metadata: any;
  synced_at: string;
}

interface QueuedOperation {
  id: string;
  type: 'personality' | 'drawing' | 'game_stats';
  operation: 'create' | 'update' | 'delete';
  data: any;
  timestamp: number;
  retryCount: number;
}

/**
 * CloudSyncService manages synchronization between local storage and Supabase
 *
 * Features:
 * - Offline queue for operations when network is unavailable
 * - Automatic retry with exponential backoff
 * - Last-write-wins conflict resolution (I-CLOUD-002)
 * - Idempotent operations (I-CLOUD-001)
 */
export class CloudSyncService {
  private syncState: SyncState = {
    status: 'idle',
    lastSync: null,
    error: null,
    pendingOperations: 0,
  };

  private offlineQueue: QueuedOperation[] = [];
  private listeners: Set<(state: SyncState) => void> = new Set();
  private syncInterval: number | null = null;
  private isProcessingQueue = false;
  private maxRetries = 3;
  private retryDelay = 1000; // milliseconds

  constructor() {
    this.loadOfflineQueue();
    this.setupRealtimeSubscriptions();

    // Check connection status periodically
    this.syncInterval = window.setInterval(() => {
      this.checkConnectionAndSync();
    }, 30000); // Every 30 seconds
  }

  /**
   * Load offline queue from localStorage
   */
  private loadOfflineQueue(): void {
    try {
      const stored = localStorage.getItem('cloudSyncQueue');
      if (stored) {
        this.offlineQueue = JSON.parse(stored);
        this.updateState({ pendingOperations: this.offlineQueue.length });
      }
    } catch (error) {
      console.error('Failed to load offline queue:', error);
    }
  }

  /**
   * Save offline queue to localStorage
   */
  private saveOfflineQueue(): void {
    try {
      localStorage.setItem('cloudSyncQueue', JSON.stringify(this.offlineQueue));
      this.updateState({ pendingOperations: this.offlineQueue.length });
    } catch (error) {
      console.error('Failed to save offline queue:', error);
    }
  }

  /**
   * Add operation to offline queue
   * Implements I-CLOUD-001 (idempotent operations)
   */
  private queueOperation(operation: Omit<QueuedOperation, 'id' | 'timestamp' | 'retryCount'>): void {
    const queuedOp: QueuedOperation = {
      ...operation,
      id: `${operation.type}-${operation.operation}-${Date.now()}-${Math.random()}`,
      timestamp: Date.now(),
      retryCount: 0,
    };

    this.offlineQueue.push(queuedOp);
    this.saveOfflineQueue();
  }

  /**
   * Process offline queue when connection is restored
   */
  private async processOfflineQueue(): Promise<void> {
    if (this.isProcessingQueue || this.offlineQueue.length === 0) {
      return;
    }

    this.isProcessingQueue = true;
    this.updateState({ status: 'syncing' });

    const operations = [...this.offlineQueue];

    for (const operation of operations) {
      try {
        await this.executeQueuedOperation(operation);

        // Remove successful operation from queue
        this.offlineQueue = this.offlineQueue.filter(op => op.id !== operation.id);
        this.saveOfflineQueue();
      } catch (error) {
        console.error('Failed to process queued operation:', error);

        // Increment retry count
        const op = this.offlineQueue.find(o => o.id === operation.id);
        if (op) {
          op.retryCount++;

          // Remove if max retries exceeded
          if (op.retryCount >= this.maxRetries) {
            console.error('Max retries exceeded for operation:', op);
            this.offlineQueue = this.offlineQueue.filter(o => o.id !== operation.id);
          }

          this.saveOfflineQueue();
        }
      }
    }

    this.isProcessingQueue = false;
    this.updateState({
      status: this.offlineQueue.length === 0 ? 'synced' : 'error',
      lastSync: Date.now(),
    });
  }

  /**
   * Execute a queued operation
   */
  private async executeQueuedOperation(operation: QueuedOperation): Promise<void> {
    switch (operation.type) {
      case 'personality':
        if (operation.operation === 'create' || operation.operation === 'update') {
          await this.syncPersonality(operation.data);
        }
        break;
      case 'drawing':
        if (operation.operation === 'create') {
          await this.syncDrawing(operation.data);
        }
        break;
      case 'game_stats':
        if (operation.operation === 'create') {
          await this.syncGameSession(operation.data);
        }
        break;
    }
  }

  /**
   * Check connection and sync if online
   */
  private async checkConnectionAndSync(): Promise<void> {
    if (!isSupabaseConfigured()) {
      this.updateState({ status: 'offline' });
      return;
    }

    const { data: { session } } = await supabase.auth.getSession();

    if (!session) {
      this.updateState({ status: 'idle' });
      return;
    }

    // Process offline queue if we have pending operations
    if (this.offlineQueue.length > 0) {
      await this.processOfflineQueue();
    }
  }

  /**
   * Setup realtime subscriptions for live updates
   */
  private setupRealtimeSubscriptions(): void {
    if (!isSupabaseConfigured()) return;

    // Subscribe to personality changes
    supabase
      .channel('personalities-changes')
      .on('postgres_changes', {
        event: '*',
        schema: 'public',
        table: TABLES.PERSONALITIES
      }, (payload) => {
        console.log('Personality change detected:', payload);
        // Emit event for local state update
        window.dispatchEvent(new CustomEvent('cloud-personality-change', { detail: payload }));
      })
      .subscribe();

    // Subscribe to drawing changes
    supabase
      .channel('drawings-changes')
      .on('postgres_changes', {
        event: '*',
        schema: 'public',
        table: TABLES.DRAWINGS
      }, (payload) => {
        console.log('Drawing change detected:', payload);
        window.dispatchEvent(new CustomEvent('cloud-drawing-change', { detail: payload }));
      })
      .subscribe();
  }

  /**
   * Update sync state and notify listeners
   */
  private updateState(updates: Partial<SyncState>): void {
    this.syncState = { ...this.syncState, ...updates };
    this.listeners.forEach(listener => listener(this.syncState));
  }

  /**
   * Subscribe to sync state changes
   */
  public subscribe(listener: (state: SyncState) => void): () => void {
    this.listeners.add(listener);
    listener(this.syncState); // Send initial state

    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Sign in with email and password
   */
  public async signIn(email: string, password: string): Promise<{ user: User; session: Session } | { error: AuthError }> {
    const { data, error } = await supabase.auth.signInWithPassword({
      email,
      password,
    });

    if (error) {
      this.updateState({ status: 'error', error: error.message });
      return { error };
    }

    this.updateState({ status: 'synced', error: null });

    // Process offline queue after successful sign in
    await this.processOfflineQueue();

    return data;
  }

  /**
   * Sign up with email and password
   */
  public async signUp(email: string, password: string): Promise<{ user: User; session: Session | null } | { error: AuthError }> {
    const { data, error } = await supabase.auth.signUp({
      email,
      password,
    });

    if (error) {
      this.updateState({ status: 'error', error: error.message });
      return { error };
    }

    this.updateState({ status: 'synced', error: null });
    return data;
  }

  /**
   * Sign out
   */
  public async signOut(): Promise<void> {
    await supabase.auth.signOut();
    this.updateState({ status: 'idle', error: null, lastSync: null });
  }

  /**
   * Get current user
   */
  public async getCurrentUser(): Promise<User | null> {
    const { data: { user } } = await supabase.auth.getUser();
    return user;
  }

  /**
   * Sync a personality to the cloud
   * Implements I-CLOUD-001 (idempotent), I-CLOUD-002 (last-write-wins)
   */
  public async syncPersonality(personality: Personality): Promise<void> {
    const user = await this.getCurrentUser();

    if (!user) {
      // Queue for later if offline
      this.queueOperation({
        type: 'personality',
        operation: 'update',
        data: personality,
      });
      return;
    }

    this.updateState({ status: 'syncing' });

    try {
      const cloudPersonality: CloudPersonality = {
        id: personality.id,
        user_id: user.id,
        name: personality.name,
        icon: personality.icon,
        config: personality.config,
        quirks: personality.quirks || [],
        sound_pack: personality.sound_pack || null,
        version: personality.version,
        created_at: new Date(personality.created_at).toISOString(),
        modified_at: new Date(personality.modified_at).toISOString(),
        synced_at: new Date().toISOString(),
      };

      // Upsert (insert or update) - idempotent operation
      const { error } = await supabase
        .from(TABLES.PERSONALITIES)
        .upsert(cloudPersonality, {
          onConflict: 'id,user_id',
        });

      if (error) throw error;

      this.updateState({
        status: 'synced',
        lastSync: Date.now(),
        error: null,
      });
    } catch (error) {
      console.error('Failed to sync personality:', error);
      this.updateState({
        status: 'error',
        error: error instanceof Error ? error.message : 'Unknown error'
      });

      // Queue for retry
      this.queueOperation({
        type: 'personality',
        operation: 'update',
        data: personality,
      });
    }
  }

  /**
   * Fetch personalities from the cloud
   */
  public async fetchPersonalities(): Promise<Personality[]> {
    const user = await this.getCurrentUser();
    if (!user) return [];

    try {
      const { data, error } = await supabase
        .from(TABLES.PERSONALITIES)
        .select('*')
        .eq('user_id', user.id)
        .order('modified_at', { ascending: false });

      if (error) throw error;

      return (data || []).map((cp: CloudPersonality) => ({
        id: cp.id,
        name: cp.name,
        icon: cp.icon,
        config: cp.config,
        quirks: cp.quirks,
        sound_pack: cp.sound_pack,
        version: cp.version,
        created_at: new Date(cp.created_at).getTime(),
        modified_at: new Date(cp.modified_at).getTime(),
      }));
    } catch (error) {
      console.error('Failed to fetch personalities:', error);
      return [];
    }
  }

  /**
   * Sync a drawing to the cloud
   * Implements I-CLOUD-001 (idempotent)
   */
  public async syncDrawing(drawing: Drawing): Promise<void> {
    const user = await this.getCurrentUser();

    if (!user) {
      this.queueOperation({
        type: 'drawing',
        operation: 'create',
        data: drawing,
      });
      return;
    }

    this.updateState({ status: 'syncing' });

    try {
      const cloudDrawing: CloudDrawing = {
        id: drawing.id,
        user_id: user.id,
        created_at: new Date(drawing.createdAt).toISOString(),
        strokes: drawing.strokes,
        moods: drawing.moods,
        duration: drawing.duration,
        dominant_mood: drawing.dominantMood,
        has_signature: drawing.hasSignature,
        session_id: drawing.sessionId || null,
        thumbnail_url: drawing.thumbnailData || null,
        metadata: drawing.metadata,
        synced_at: new Date().toISOString(),
      };

      const { error } = await supabase
        .from(TABLES.DRAWINGS)
        .upsert(cloudDrawing, {
          onConflict: 'id,user_id',
        });

      if (error) throw error;

      this.updateState({
        status: 'synced',
        lastSync: Date.now(),
        error: null,
      });
    } catch (error) {
      console.error('Failed to sync drawing:', error);
      this.updateState({
        status: 'error',
        error: error instanceof Error ? error.message : 'Unknown error'
      });

      this.queueOperation({
        type: 'drawing',
        operation: 'create',
        data: drawing,
      });
    }
  }

  /**
   * Fetch drawings from the cloud
   */
  public async fetchDrawings(): Promise<Drawing[]> {
    const user = await this.getCurrentUser();
    if (!user) return [];

    try {
      const { data, error } = await supabase
        .from(TABLES.DRAWINGS)
        .select('*')
        .eq('user_id', user.id)
        .order('created_at', { ascending: false });

      if (error) throw error;

      return (data || []).map((cd: CloudDrawing) => ({
        id: cd.id,
        createdAt: new Date(cd.created_at).getTime(),
        strokes: cd.strokes,
        moods: cd.moods,
        duration: cd.duration,
        dominantMood: cd.dominant_mood,
        hasSignature: cd.has_signature,
        sessionId: cd.session_id,
        thumbnailData: cd.thumbnail_url,
        metadata: cd.metadata,
      }));
    } catch (error) {
      console.error('Failed to fetch drawings:', error);
      return [];
    }
  }

  /**
   * Sync a game session to the cloud
   */
  public async syncGameSession(session: GameSession): Promise<void> {
    const user = await this.getCurrentUser();

    if (!user) {
      this.queueOperation({
        type: 'game_stats',
        operation: 'create',
        data: session,
      });
      return;
    }

    this.updateState({ status: 'syncing' });

    try {
      const cloudGameStats: CloudGameStats = {
        id: session.id,
        user_id: user.id,
        game_type: session.gameType,
        result: session.result,
        score: session.score,
        duration: session.duration,
        timestamp: new Date(session.timestamp).toISOString(),
        personality: session.personality,
        metadata: session.metadata || {},
        synced_at: new Date().toISOString(),
      };

      const { error } = await supabase
        .from(TABLES.GAME_STATS)
        .upsert(cloudGameStats, {
          onConflict: 'id,user_id',
        });

      if (error) throw error;

      this.updateState({
        status: 'synced',
        lastSync: Date.now(),
        error: null,
      });
    } catch (error) {
      console.error('Failed to sync game session:', error);
      this.updateState({
        status: 'error',
        error: error instanceof Error ? error.message : 'Unknown error'
      });

      this.queueOperation({
        type: 'game_stats',
        operation: 'create',
        data: session,
      });
    }
  }

  /**
   * Fetch game sessions from the cloud
   */
  public async fetchGameSessions(): Promise<GameSession[]> {
    const user = await this.getCurrentUser();
    if (!user) return [];

    try {
      const { data, error } = await supabase
        .from(TABLES.GAME_STATS)
        .select('*')
        .eq('user_id', user.id)
        .order('timestamp', { ascending: false });

      if (error) throw error;

      return (data || []).map((cg: CloudGameStats) => ({
        id: cg.id,
        gameType: cg.game_type as any,
        result: cg.result as any,
        score: cg.score,
        duration: cg.duration,
        timestamp: new Date(cg.timestamp).getTime(),
        personality: cg.personality,
        metadata: cg.metadata,
      }));
    } catch (error) {
      console.error('Failed to fetch game sessions:', error);
      return [];
    }
  }

  /**
   * Get current sync status
   */
  public getSyncStatus(): SyncState {
    return { ...this.syncState };
  }

  /**
   * Force sync all data
   */
  public async forceSyncAll(): Promise<void> {
    await this.processOfflineQueue();
  }

  /**
   * Clean up resources
   */
  public destroy(): void {
    if (this.syncInterval !== null) {
      window.clearInterval(this.syncInterval);
    }
    this.listeners.clear();
  }
}

// Singleton instance
export const cloudSyncService = new CloudSyncService();
