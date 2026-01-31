/**
 * IndexedDB Artwork Storage Service
 * Contract: I-ART-GAL-001 (Stroke Data Storage)
 * Implements persistent storage for all drawing data
 */

import { Drawing, DrawingFilter } from '../types/drawing';

const DB_NAME = 'mbot_artwork_db';
const DB_VERSION = 1;
const STORE_NAME = 'drawings';

export class ArtworkStorage {
  private db: IDBDatabase | null = null;

  /**
   * Initialize IndexedDB connection
   */
  async init(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => {
        reject(new Error('Failed to open IndexedDB'));
      };

      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event: IDBVersionChangeEvent) => {
        const db = (event.target as IDBOpenDBRequest).result;

        if (!db.objectStoreNames.contains(STORE_NAME)) {
          const objectStore = db.createObjectStore(STORE_NAME, { keyPath: 'id' });

          // Create indexes for efficient querying
          objectStore.createIndex('createdAt', 'createdAt', { unique: false });
          objectStore.createIndex('dominantMood', 'dominantMood', { unique: false });
          objectStore.createIndex('sessionId', 'sessionId', { unique: false });
          objectStore.createIndex('hasSignature', 'hasSignature', { unique: false });
        }
      };
    });
  }

  /**
   * Save a drawing to IndexedDB
   * Invariant: I-ART-GAL-001 - Must save all stroke data
   */
  async saveDrawing(drawing: Drawing): Promise<void> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.put(drawing);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to save drawing'));
    });
  }

  /**
   * Get a single drawing by ID
   */
  async getDrawing(id: string): Promise<Drawing | null> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.get(id);

      request.onsuccess = () => {
        resolve(request.result || null);
      };
      request.onerror = () => reject(new Error('Failed to get drawing'));
    });
  }

  /**
   * Get all drawings with optional filtering and pagination
   */
  async getDrawings(
    filter?: DrawingFilter,
    page: number = 1,
    itemsPerPage: number = 20
  ): Promise<{ drawings: Drawing[]; total: number }> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const store = transaction.objectStore(STORE_NAME);

      // Use index if filtering by specific field
      let request: IDBRequest;
      if (filter?.mood) {
        const index = store.index('dominantMood');
        request = index.getAll(filter.mood);
      } else {
        request = store.getAll();
      }

      request.onsuccess = () => {
        let drawings = request.result as Drawing[];

        // Apply additional filters
        if (filter) {
          drawings = this.applyFilters(drawings, filter);
        }

        // Sort by creation date (newest first)
        drawings.sort((a, b) => b.createdAt - a.createdAt);

        const total = drawings.length;

        // Apply pagination
        const start = (page - 1) * itemsPerPage;
        const end = start + itemsPerPage;
        const paginatedDrawings = drawings.slice(start, end);

        resolve({ drawings: paginatedDrawings, total });
      };

      request.onerror = () => reject(new Error('Failed to get drawings'));
    });
  }

  /**
   * Apply filters to drawing list
   */
  private applyFilters(drawings: Drawing[], filter: DrawingFilter): Drawing[] {
    return drawings.filter(drawing => {
      // Date range filter
      if (filter.dateFrom && drawing.createdAt < filter.dateFrom) {
        return false;
      }
      if (filter.dateTo && drawing.createdAt > filter.dateTo) {
        return false;
      }

      // Session ID filter
      if (filter.sessionId && drawing.sessionId !== filter.sessionId) {
        return false;
      }

      // Signature filter
      if (filter.hasSignature !== undefined && drawing.hasSignature !== filter.hasSignature) {
        return false;
      }

      // Search query (searches in sessionId and dominant mood)
      if (filter.searchQuery) {
        const query = filter.searchQuery.toLowerCase();
        const sessionMatch = drawing.sessionId?.toLowerCase().includes(query);
        const moodMatch = drawing.dominantMood.toLowerCase().includes(query);
        const dateMatch = new Date(drawing.createdAt).toLocaleDateString().toLowerCase().includes(query);

        if (!sessionMatch && !moodMatch && !dateMatch) {
          return false;
        }
      }

      return true;
    });
  }

  /**
   * Delete a drawing by ID
   */
  async deleteDrawing(id: string): Promise<void> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.delete(id);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to delete drawing'));
    });
  }

  /**
   * Delete multiple drawings by IDs
   */
  async deleteMultipleDrawings(ids: string[]): Promise<void> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    const transaction = this.db.transaction([STORE_NAME], 'readwrite');
    const store = transaction.objectStore(STORE_NAME);

    return new Promise((resolve, reject) => {
      const deletePromises = ids.map(id => {
        return new Promise<void>((res, rej) => {
          const request = store.delete(id);
          request.onsuccess = () => res();
          request.onerror = () => rej(new Error(`Failed to delete drawing ${id}`));
        });
      });

      Promise.all(deletePromises)
        .then(() => resolve())
        .catch(reject);
    });
  }

  /**
   * Get total count of drawings
   */
  async getDrawingCount(): Promise<number> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.count();

      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(new Error('Failed to count drawings'));
    });
  }

  /**
   * Get unique moods from all drawings
   */
  async getUniqueMoods(): Promise<string[]> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.getAll();

      request.onsuccess = () => {
        const drawings = request.result as Drawing[];
        const moods = new Set<string>();
        drawings.forEach(drawing => moods.add(drawing.dominantMood));
        resolve(Array.from(moods).sort());
      };

      request.onerror = () => reject(new Error('Failed to get unique moods'));
    });
  }

  /**
   * Clear all drawings (for testing/reset)
   */
  async clearAll(): Promise<void> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.clear();

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to clear drawings'));
    });
  }

  /**
   * Close the database connection
   */
  close(): void {
    if (this.db) {
      this.db.close();
      this.db = null;
    }
  }
}

// Export singleton instance
export const artworkStorage = new ArtworkStorage();
