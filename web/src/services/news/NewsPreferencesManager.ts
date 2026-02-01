/**
 * News Preferences Manager
 * Manages user news preferences and learns from user interactions
 *
 * Contract: I-NEWS-001 - News personalization with learning
 */

import type {
  NewsPreferences,
  NewsArticle,
  UserFeedback,
  NewsTopic
} from '../../types/voice';
import { createDefaultNewsPreferences, VOICE_ASSISTANT_STORAGE_KEYS } from '../../types/voice';

export class NewsPreferencesManager {
  private preferences: Map<string, NewsPreferences> = new Map();
  private readonly LEARNING_RATE = 0.2; // How much to adjust weights based on feedback
  private readonly MAX_WEIGHT = 2.0;
  private readonly MIN_WEIGHT = 0.1;

  constructor() {
    this.loadPreferences();
  }

  /**
   * Get preferences for a user
   */
  async getPreferences(userId: string): Promise<NewsPreferences> {
    let prefs = this.preferences.get(userId);

    if (!prefs) {
      prefs = createDefaultNewsPreferences(userId);
      this.preferences.set(userId, prefs);
      await this.savePreferences();
    }

    return prefs;
  }

  /**
   * Update user preferences
   */
  async updatePreferences(
    userId: string,
    updates: Partial<NewsPreferences>
  ): Promise<void> {
    const current = await this.getPreferences(userId);

    const updated: NewsPreferences = {
      ...current,
      ...updates,
      userId, // Ensure userId doesn't change
      lastUpdated: Date.now()
    };

    this.preferences.set(userId, updated);
    await this.savePreferences();
  }

  /**
   * Add a topic to user preferences
   */
  async addTopic(userId: string, topic: string): Promise<void> {
    const prefs = await this.getPreferences(userId);

    if (!prefs.topics.includes(topic)) {
      prefs.topics.push(topic);

      // Initialize weight if not present
      if (!prefs.topicWeights[topic]) {
        prefs.topicWeights[topic] = 1.0;
      }

      prefs.lastUpdated = Date.now();
      await this.savePreferences();
    }
  }

  /**
   * Remove a topic from user preferences
   */
  async removeTopic(userId: string, topic: string): Promise<void> {
    const prefs = await this.getPreferences(userId);

    const index = prefs.topics.indexOf(topic);
    if (index !== -1) {
      prefs.topics.splice(index, 1);

      // Remove weight
      delete prefs.topicWeights[topic];

      prefs.lastUpdated = Date.now();
      await this.savePreferences();
    }
  }

  /**
   * Add a source to exclude list
   */
  async excludeTopic(userId: string, topic: string): Promise<void> {
    const prefs = await this.getPreferences(userId);

    if (!prefs.excludeTopics.includes(topic)) {
      prefs.excludeTopics.push(topic);

      // Remove from active topics if present
      const index = prefs.topics.indexOf(topic);
      if (index !== -1) {
        prefs.topics.splice(index, 1);
      }

      // Set very low weight
      prefs.topicWeights[topic] = this.MIN_WEIGHT;

      prefs.lastUpdated = Date.now();
      await this.savePreferences();
    }
  }

  /**
   * Remove a topic from exclude list
   */
  async unexcludeTopic(userId: string, topic: string): Promise<void> {
    const prefs = await this.getPreferences(userId);

    const index = prefs.excludeTopics.indexOf(topic);
    if (index !== -1) {
      prefs.excludeTopics.splice(index, 1);

      // Reset weight to default
      prefs.topicWeights[topic] = 1.0;

      prefs.lastUpdated = Date.now();
      await this.savePreferences();
    }
  }

  /**
   * Adjust topic weights based on user feedback
   * Implements preference learning from interactions
   */
  async adjustWeights(userId: string, topic: string, delta: number): Promise<void> {
    const prefs = await this.getPreferences(userId);

    const currentWeight = prefs.topicWeights[topic] || 1.0;
    let newWeight = currentWeight + delta;

    // Clamp to min/max
    newWeight = Math.max(this.MIN_WEIGHT, Math.min(this.MAX_WEIGHT, newWeight));

    prefs.topicWeights[topic] = newWeight;
    prefs.lastUpdated = Date.now();

    await this.savePreferences();
  }

  /**
   * Get topic weights for ranking
   */
  async getTopicWeights(userId: string): Promise<Record<string, number>> {
    const prefs = await this.getPreferences(userId);
    return prefs.topicWeights;
  }

  /**
   * Learn from user feedback on an article
   * Contract: I-NEWS-001 - Adaptive personalization
   */
  async learnFromFeedback(
    userId: string,
    article: NewsArticle,
    feedback: UserFeedback
  ): Promise<void> {
    const prefs = await this.getPreferences(userId);

    // Determine weight adjustment based on feedback
    let adjustment = 0;

    switch (feedback.action) {
      case 'read':
        // Positive signal - increase weight slightly
        adjustment = this.LEARNING_RATE;
        break;

      case 'skip':
        // Negative signal - decrease weight
        adjustment = -this.LEARNING_RATE;
        break;

      case 'like':
        // Strong positive signal
        adjustment = this.LEARNING_RATE * 2;
        break;

      case 'dislike':
        // Strong negative signal
        adjustment = -this.LEARNING_RATE * 2;
        break;
    }

    // Adjust weight for article category if available
    if (article.category) {
      await this.adjustWeights(userId, article.category, adjustment);
    }

    // If user spent significant time reading, boost the weight more
    if (feedback.duration && feedback.duration > 30000) { // 30 seconds
      const timeBonus = Math.min(0.2, feedback.duration / 300000); // Max 0.2 for 5 minutes
      if (article.category) {
        await this.adjustWeights(userId, article.category, timeBonus);
      }
    }

    // Learn from source preferences
    const sourceWeight = prefs.topicWeights[`source:${article.source}`] || 1.0;
    const sourceAdjustment = adjustment * 0.5; // Sources get half the weight of topics

    prefs.topicWeights[`source:${article.source}`] = Math.max(
      this.MIN_WEIGHT,
      Math.min(this.MAX_WEIGHT, sourceWeight + sourceAdjustment)
    );

    prefs.lastUpdated = Date.now();
    await this.savePreferences();
  }

  /**
   * Get recommended topics based on weights
   */
  async getRecommendedTopics(userId: string, count: number = 5): Promise<string[]> {
    const prefs = await this.getPreferences(userId);

    // Sort topics by weight
    const sortedTopics = Object.entries(prefs.topicWeights)
      .filter(([topic]) => !topic.startsWith('source:')) // Exclude source weights
      .sort(([, a], [, b]) => b - a)
      .slice(0, count)
      .map(([topic]) => topic);

    return sortedTopics;
  }

  /**
   * Reset preferences to defaults
   */
  async resetPreferences(userId: string): Promise<void> {
    const defaultPrefs = createDefaultNewsPreferences(userId);
    this.preferences.set(userId, defaultPrefs);
    await this.savePreferences();
  }

  /**
   * Load preferences from storage
   */
  private loadPreferences(): void {
    try {
      const stored = localStorage.getItem(VOICE_ASSISTANT_STORAGE_KEYS.NEWS_PREFERENCES);
      if (stored) {
        const data = JSON.parse(stored);
        this.preferences = new Map(Object.entries(data));
      }
    } catch (error) {
      console.error('Failed to load news preferences:', error);
    }
  }

  /**
   * Save preferences to storage
   */
  private async savePreferences(): Promise<void> {
    try {
      const data = Object.fromEntries(this.preferences);
      localStorage.setItem(
        VOICE_ASSISTANT_STORAGE_KEYS.NEWS_PREFERENCES,
        JSON.stringify(data)
      );
    } catch (error) {
      console.error('Failed to save news preferences:', error);
      throw error;
    }
  }

  /**
   * Export preferences for backup
   */
  exportPreferences(userId: string): string {
    const prefs = this.preferences.get(userId);
    if (!prefs) {
      throw new Error(`No preferences found for user ${userId}`);
    }
    return JSON.stringify(prefs, null, 2);
  }

  /**
   * Import preferences from backup
   */
  async importPreferences(userId: string, data: string): Promise<void> {
    try {
      const prefs = JSON.parse(data) as NewsPreferences;

      // Validate
      if (!prefs.userId || !prefs.topics || !prefs.topicWeights) {
        throw new Error('Invalid preferences format');
      }

      // Update userId to match
      prefs.userId = userId;
      prefs.lastUpdated = Date.now();

      this.preferences.set(userId, prefs);
      await this.savePreferences();
    } catch (error) {
      console.error('Failed to import preferences:', error);
      throw error;
    }
  }

  /**
   * Get all preferences (for admin/debugging)
   */
  getAllPreferences(): Map<string, NewsPreferences> {
    return new Map(this.preferences);
  }

  /**
   * Clear all preferences
   */
  async clearAllPreferences(): Promise<void> {
    this.preferences.clear();
    localStorage.removeItem(VOICE_ASSISTANT_STORAGE_KEYS.NEWS_PREFERENCES);
  }
}

// Singleton instance
let instance: NewsPreferencesManager | null = null;

/**
 * Get the singleton preferences manager instance
 */
export function getNewsPreferencesManager(): NewsPreferencesManager {
  if (!instance) {
    instance = new NewsPreferencesManager();
  }
  return instance;
}
