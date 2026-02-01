/**
 * Memory Store - IndexedDB storage for conversations and activities
 *
 * Contract: I-MEMORY-001
 * Provides persistent storage for conversation history and daily activities
 */

import {
  Conversation,
  ConversationTurn,
  DailyActivity,
  FollowUpQuestion,
  ConversationMemory,
} from '../../types/voice';

const DB_NAME = 'mbot_memory';
const DB_VERSION = 1;
const CONVERSATIONS_STORE = 'conversations';
const ACTIVITIES_STORE = 'activities';
const QUESTIONS_STORE = 'questions';

export class MemoryStore {
  private db: IDBDatabase | null = null;
  private initPromise: Promise<void>;

  constructor() {
    this.initPromise = this.initialize();
  }

  /**
   * Initialize IndexedDB database
   */
  private async initialize(): Promise<void> {
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

        // Conversations store
        if (!db.objectStoreNames.contains(CONVERSATIONS_STORE)) {
          const conversationsStore = db.createObjectStore(CONVERSATIONS_STORE, {
            keyPath: 'id',
          });
          conversationsStore.createIndex('userId', 'userId', { unique: false });
          conversationsStore.createIndex('timestamp', 'timestamp', { unique: false });
          conversationsStore.createIndex('userId_timestamp', ['userId', 'timestamp'], {
            unique: false,
          });
        }

        // Activities store
        if (!db.objectStoreNames.contains(ACTIVITIES_STORE)) {
          const activitiesStore = db.createObjectStore(ACTIVITIES_STORE, {
            keyPath: ['userId', 'date'],
          });
          activitiesStore.createIndex('userId', 'userId', { unique: false });
          activitiesStore.createIndex('date', 'date', { unique: false });
        }

        // Questions store
        if (!db.objectStoreNames.contains(QUESTIONS_STORE)) {
          const questionsStore = db.createObjectStore(QUESTIONS_STORE, {
            keyPath: 'id',
          });
          questionsStore.createIndex('userId', 'userId', { unique: false });
          questionsStore.createIndex('answered', 'answered', { unique: false });
          questionsStore.createIndex('validUntil', 'validUntil', { unique: false });
        }
      };
    });
  }

  /**
   * Ensure database is initialized
   */
  private async ensureInitialized(): Promise<void> {
    await this.initPromise;
    if (!this.db) {
      throw new Error('Database not initialized');
    }
  }

  // ==================== Conversations ====================

  /**
   * Save a conversation
   */
  async saveConversation(conversation: Conversation): Promise<void> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([CONVERSATIONS_STORE], 'readwrite');
      const store = transaction.objectStore(CONVERSATIONS_STORE);
      const request = store.put(conversation);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to save conversation'));
    });
  }

  /**
   * Get conversations by user
   */
  async getConversationsByUser(userId: string): Promise<Conversation[]> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([CONVERSATIONS_STORE], 'readonly');
      const store = transaction.objectStore(CONVERSATIONS_STORE);
      const index = store.index('userId');
      const request = index.getAll(userId);

      request.onsuccess = () => {
        const conversations = request.result as Conversation[];
        // Sort by timestamp descending
        conversations.sort((a, b) => b.timestamp - a.timestamp);
        resolve(conversations);
      };
      request.onerror = () => reject(new Error('Failed to get conversations'));
    });
  }

  /**
   * Get conversations by date range
   */
  async getConversationsByDateRange(
    userId: string,
    startDate: number,
    endDate: number
  ): Promise<Conversation[]> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([CONVERSATIONS_STORE], 'readonly');
      const store = transaction.objectStore(CONVERSATIONS_STORE);
      const index = store.index('userId_timestamp');
      const range = IDBKeyRange.bound([userId, startDate], [userId, endDate]);
      const request = index.getAll(range);

      request.onsuccess = () => {
        const conversations = request.result as Conversation[];
        conversations.sort((a, b) => b.timestamp - a.timestamp);
        resolve(conversations);
      };
      request.onerror = () => reject(new Error('Failed to get conversations by date range'));
    });
  }

  /**
   * Get a single conversation by ID
   */
  async getConversationById(id: string): Promise<Conversation | null> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([CONVERSATIONS_STORE], 'readonly');
      const store = transaction.objectStore(CONVERSATIONS_STORE);
      const request = store.get(id);

      request.onsuccess = () => resolve(request.result || null);
      request.onerror = () => reject(new Error('Failed to get conversation'));
    });
  }

  /**
   * Delete conversations older than retention period
   */
  async deleteOldConversations(userId: string, retentionDays: number): Promise<number> {
    await this.ensureInitialized();

    const cutoffTime = Date.now() - retentionDays * 24 * 60 * 60 * 1000;

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([CONVERSATIONS_STORE], 'readwrite');
      const store = transaction.objectStore(CONVERSATIONS_STORE);
      const index = store.index('userId_timestamp');
      const range = IDBKeyRange.bound([userId, 0], [userId, cutoffTime]);
      const request = index.openCursor(range);
      let deletedCount = 0;

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest).result;
        if (cursor) {
          cursor.delete();
          deletedCount++;
          cursor.continue();
        } else {
          resolve(deletedCount);
        }
      };

      request.onerror = () => reject(new Error('Failed to delete old conversations'));
    });
  }

  // ==================== Daily Activities ====================

  /**
   * Save daily activity
   */
  async saveDailyActivity(activity: DailyActivity): Promise<void> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([ACTIVITIES_STORE], 'readwrite');
      const store = transaction.objectStore(ACTIVITIES_STORE);
      const request = store.put(activity);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to save daily activity'));
    });
  }

  /**
   * Get daily activity
   */
  async getDailyActivity(userId: string, date: string): Promise<DailyActivity | null> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([ACTIVITIES_STORE], 'readonly');
      const store = transaction.objectStore(ACTIVITIES_STORE);
      const request = store.get([userId, date]);

      request.onsuccess = () => resolve(request.result || null);
      request.onerror = () => reject(new Error('Failed to get daily activity'));
    });
  }

  /**
   * Get activities in date range
   */
  async getActivitiesInRange(
    userId: string,
    startDate: string,
    endDate: string
  ): Promise<DailyActivity[]> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([ACTIVITIES_STORE], 'readonly');
      const store = transaction.objectStore(ACTIVITIES_STORE);
      const request = store.openCursor();
      const activities: DailyActivity[] = [];

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest).result;
        if (cursor) {
          const activity = cursor.value as DailyActivity;
          if (
            activity.userId === userId &&
            activity.date >= startDate &&
            activity.date <= endDate
          ) {
            activities.push(activity);
          }
          cursor.continue();
        } else {
          activities.sort((a, b) => b.date.localeCompare(a.date));
          resolve(activities);
        }
      };

      request.onerror = () => reject(new Error('Failed to get activities in range'));
    });
  }

  /**
   * Update activity completion status
   */
  async markActivityCompleted(
    userId: string,
    date: string,
    activity: string
  ): Promise<void> {
    await this.ensureInitialized();

    const dailyActivity = await this.getDailyActivity(userId, date);
    if (!dailyActivity) {
      throw new Error('Activity not found');
    }

    if (!dailyActivity.completedActivities.includes(activity)) {
      dailyActivity.completedActivities.push(activity);
      await this.saveDailyActivity(dailyActivity);
    }
  }

  // ==================== Follow-up Questions ====================

  /**
   * Save follow-up question
   */
  async saveFollowUpQuestion(question: FollowUpQuestion): Promise<void> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([QUESTIONS_STORE], 'readwrite');
      const store = transaction.objectStore(QUESTIONS_STORE);
      const request = store.put(question);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to save follow-up question'));
    });
  }

  /**
   * Get active questions for user
   */
  async getActiveQuestions(userId: string): Promise<FollowUpQuestion[]> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([QUESTIONS_STORE], 'readonly');
      const store = transaction.objectStore(QUESTIONS_STORE);
      const index = store.index('userId');
      const request = index.getAll(userId);

      request.onsuccess = () => {
        const questions = request.result as FollowUpQuestion[];
        const now = Date.now();

        // Filter for active questions (not answered, not expired)
        const activeQuestions = questions
          .filter((q) => !q.answered && q.validUntil > now)
          .sort((a, b) => b.priority - a.priority);

        resolve(activeQuestions);
      };
      request.onerror = () => reject(new Error('Failed to get active questions'));
    });
  }

  /**
   * Mark question as answered
   */
  async markQuestionAnswered(questionId: string, answer: string): Promise<void> {
    await this.ensureInitialized();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([QUESTIONS_STORE], 'readwrite');
      const store = transaction.objectStore(QUESTIONS_STORE);
      const getRequest = store.get(questionId);

      getRequest.onsuccess = () => {
        const question = getRequest.result as FollowUpQuestion | undefined;
        if (!question) {
          reject(new Error('Question not found'));
          return;
        }

        question.answered = true;
        question.answer = answer;

        const putRequest = store.put(question);
        putRequest.onsuccess = () => resolve();
        putRequest.onerror = () => reject(new Error('Failed to update question'));
      };

      getRequest.onerror = () => reject(new Error('Failed to get question'));
    });
  }

  /**
   * Delete expired questions
   */
  async deleteExpiredQuestions(): Promise<number> {
    await this.ensureInitialized();

    const now = Date.now();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([QUESTIONS_STORE], 'readwrite');
      const store = transaction.objectStore(QUESTIONS_STORE);
      const index = store.index('validUntil');
      const range = IDBKeyRange.upperBound(now);
      const request = index.openCursor(range);
      let deletedCount = 0;

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest).result;
        if (cursor) {
          cursor.delete();
          deletedCount++;
          cursor.continue();
        } else {
          resolve(deletedCount);
        }
      };

      request.onerror = () => reject(new Error('Failed to delete expired questions'));
    });
  }

  /**
   * Close database connection
   */
  close(): void {
    if (this.db) {
      this.db.close();
      this.db = null;
    }
  }
}

// Export singleton instance
export const memoryStore = new MemoryStore();
