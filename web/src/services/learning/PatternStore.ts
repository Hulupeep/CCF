/**
 * PatternStore - IndexedDB persistence for crystallized patterns
 * Based on OpenClaw Foundry's persistent learning storage
 */

import {
  CrystallizedPattern,
  PatternStatistics,
  PatternStore as IPatternStore,
} from '../../types/learning';

const DB_NAME = 'mbot_learning';
const DB_VERSION = 1;
const PATTERN_STORE = 'patterns';

/**
 * IndexedDB-backed pattern storage
 */
export class PatternStore implements IPatternStore {
  private db: IDBDatabase | null = null;
  private initPromise: Promise<void> | null = null;

  constructor() {
    this.initPromise = this.initDB();
  }

  /**
   * Initialize IndexedDB
   */
  private async initDB(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => {
        reject(new Error('Failed to open IndexedDB'));
      };

      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        // Create pattern store
        if (!db.objectStoreNames.contains(PATTERN_STORE)) {
          const store = db.createObjectStore(PATTERN_STORE, { keyPath: 'id' });

          // Create indices
          store.createIndex('usageCount', 'usageCount', { unique: false });
          store.createIndex('successRate', 'successRate', { unique: false });
          store.createIndex('lastUsed', 'lastUsed', { unique: false });
          store.createIndex('createdAt', 'createdAt', { unique: false });
        }
      };
    });
  }

  /**
   * Ensure DB is initialized
   */
  private async ensureDB(): Promise<IDBDatabase> {
    if (!this.db) {
      await this.initPromise;
    }
    if (!this.db) {
      throw new Error('Database not initialized');
    }
    return this.db;
  }

  /**
   * Save pattern to storage
   */
  async savePattern(pattern: CrystallizedPattern): Promise<void> {
    const db = await this.ensureDB();

    return new Promise((resolve, reject) => {
      const transaction = db.transaction([PATTERN_STORE], 'readwrite');
      const store = transaction.objectStore(PATTERN_STORE);

      const request = store.put(pattern);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to save pattern'));
    });
  }

  /**
   * Load all patterns
   */
  async loadPatterns(): Promise<CrystallizedPattern[]> {
    const db = await this.ensureDB();

    return new Promise((resolve, reject) => {
      const transaction = db.transaction([PATTERN_STORE], 'readonly');
      const store = transaction.objectStore(PATTERN_STORE);

      const request = store.getAll();

      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(new Error('Failed to load patterns'));
    });
  }

  /**
   * Update pattern
   */
  async updatePattern(
    id: string,
    updates: Partial<CrystallizedPattern>
  ): Promise<void> {
    const db = await this.ensureDB();

    return new Promise((resolve, reject) => {
      const transaction = db.transaction([PATTERN_STORE], 'readwrite');
      const store = transaction.objectStore(PATTERN_STORE);

      // Get existing pattern
      const getRequest = store.get(id);

      getRequest.onsuccess = () => {
        const pattern = getRequest.result;
        if (!pattern) {
          reject(new Error(`Pattern ${id} not found`));
          return;
        }

        // Merge updates
        const updated = { ...pattern, ...updates };

        // Save updated pattern
        const putRequest = store.put(updated);
        putRequest.onsuccess = () => resolve();
        putRequest.onerror = () => reject(new Error('Failed to update pattern'));
      };

      getRequest.onerror = () => reject(new Error('Failed to get pattern'));
    });
  }

  /**
   * Delete pattern
   */
  async deletePattern(id: string): Promise<void> {
    const db = await this.ensureDB();

    return new Promise((resolve, reject) => {
      const transaction = db.transaction([PATTERN_STORE], 'readwrite');
      const store = transaction.objectStore(PATTERN_STORE);

      const request = store.delete(id);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to delete pattern'));
    });
  }

  /**
   * Get patterns by user (filter source observations)
   */
  async getPatternsByUser(userId: string): Promise<CrystallizedPattern[]> {
    const patterns = await this.loadPatterns();

    // Filter patterns that were created from observations by this user
    // (Would need to enhance data model to track userId in pattern)
    return patterns.filter((p) =>
      p.sourceObservations.some((id) => id.includes(userId))
    );
  }

  /**
   * Get top N patterns by usage count
   */
  async getTopPatterns(limit: number): Promise<CrystallizedPattern[]> {
    const db = await this.ensureDB();

    return new Promise((resolve, reject) => {
      const transaction = db.transaction([PATTERN_STORE], 'readonly');
      const store = transaction.objectStore(PATTERN_STORE);
      const index = store.index('usageCount');

      const request = index.openCursor(null, 'prev'); // Descending order
      const results: CrystallizedPattern[] = [];

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest).result;

        if (cursor && results.length < limit) {
          results.push(cursor.value);
          cursor.continue();
        } else {
          resolve(results);
        }
      };

      request.onerror = () => reject(new Error('Failed to get top patterns'));
    });
  }

  /**
   * Get pattern statistics
   */
  async getPatternStats(): Promise<PatternStatistics> {
    const patterns = await this.loadPatterns();

    const totalPatterns = patterns.length;
    const activePatterns = patterns.filter(
      (p) => p.lastUsed > Date.now() - 30 * 24 * 60 * 60 * 1000
    ).length;
    const totalUsages = patterns.reduce((sum, p) => sum + p.usageCount, 0);

    const avgSuccessRate =
      patterns.length > 0
        ? patterns.reduce((sum, p) => sum + p.successRate, 0) / patterns.length
        : 0;

    const avgConfidence =
      patterns.length > 0
        ? patterns.reduce((sum, p) => sum + p.confidence, 0) / patterns.length
        : 0;

    // Get top patterns
    const topPatterns = await this.getTopPatterns(5);

    // Recent activity (last 10 pattern uses)
    const recentActivity = patterns
      .sort((a, b) => b.lastUsed - a.lastUsed)
      .slice(0, 10)
      .map((p) => ({
        timestamp: p.lastUsed,
        patternId: p.id,
        success: p.successRate > 0.7,
      }));

    return {
      totalPatterns,
      activePatterns,
      totalUsages,
      avgSuccessRate,
      avgConfidence,
      topPatterns,
      recentActivity,
    };
  }

  /**
   * Clear all patterns (for testing)
   */
  async clearAll(): Promise<void> {
    const db = await this.ensureDB();

    return new Promise((resolve, reject) => {
      const transaction = db.transaction([PATTERN_STORE], 'readwrite');
      const store = transaction.objectStore(PATTERN_STORE);

      const request = store.clear();

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to clear patterns'));
    });
  }

  /**
   * Export patterns as JSON
   */
  async exportPatterns(): Promise<string> {
    const patterns = await this.loadPatterns();
    return JSON.stringify(patterns, null, 2);
  }

  /**
   * Import patterns from JSON
   */
  async importPatterns(json: string): Promise<number> {
    const patterns = JSON.parse(json) as CrystallizedPattern[];
    let imported = 0;

    for (const pattern of patterns) {
      await this.savePattern(pattern);
      imported++;
    }

    return imported;
  }
}
