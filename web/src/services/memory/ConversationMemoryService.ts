/**
 * Conversation Memory Service - Track conversations and daily activities
 *
 * Contract: I-VOICE-004, I-MEMORY-001
 * Maintains conversation context for at least 7 days and tracks daily activities
 */

import {
  Conversation,
  ConversationTurn,
  DailyActivity,
  ConversationMemory,
} from '../../types/voice';
import { memoryStore } from './MemoryStore';
import { keyPointsExtractor } from './KeyPointsExtractor';
import { v4 as uuidv4 } from 'uuid';

export class ConversationMemoryService {
  private readonly DEFAULT_RETENTION_DAYS = 7;

  /**
   * Store a conversation
   */
  async storeConversation(userId: string, turns: ConversationTurn[]): Promise<Conversation> {
    if (turns.length === 0) {
      throw new Error('Cannot store empty conversation');
    }

    const conversation: Conversation = {
      id: uuidv4(),
      timestamp: turns[0].timestamp,
      turns,
      topic: await this.inferTopic(turns),
      sentiment: this.analyzeSentiment(turns),
      keyPoints: await keyPointsExtractor.extractKeyPoints({
        id: '',
        timestamp: turns[0].timestamp,
        turns,
        topic: '',
        sentiment: 'neutral',
        keyPoints: [],
      }),
    };

    await memoryStore.saveConversation(conversation);
    return conversation;
  }

  /**
   * Get conversations for a user
   */
  async getConversations(userId: string, days?: number): Promise<Conversation[]> {
    const retentionDays = days || this.DEFAULT_RETENTION_DAYS;
    const cutoffTime = Date.now() - retentionDays * 24 * 60 * 60 * 1000;

    return memoryStore.getConversationsByDateRange(userId, cutoffTime, Date.now());
  }

  /**
   * Search conversations by query
   */
  async searchConversations(userId: string, query: string): Promise<Conversation[]> {
    const allConversations = await memoryStore.getConversationsByUser(userId);
    const lowerQuery = query.toLowerCase();

    return allConversations.filter((conv) => {
      // Search in turns
      const inTurns = conv.turns.some((turn) =>
        turn.text.toLowerCase().includes(lowerQuery)
      );

      // Search in key points
      const inKeyPoints = conv.keyPoints.some((point) =>
        point.toLowerCase().includes(lowerQuery)
      );

      // Search in topic
      const inTopic = conv.topic.toLowerCase().includes(lowerQuery);

      return inTurns || inKeyPoints || inTopic;
    });
  }

  /**
   * Extract key points from a conversation
   */
  async extractKeyPoints(conversation: Conversation): Promise<string[]> {
    return keyPointsExtractor.extractKeyPoints(conversation);
  }

  // ==================== Daily Activities ====================

  /**
   * Store daily activity
   */
  async storeDailyActivity(
    userId: string,
    date: string,
    activities: string[]
  ): Promise<void> {
    const existing = await memoryStore.getDailyActivity(userId, date);

    const dailyActivity: DailyActivity = {
      date,
      userId,
      plannedActivities: activities,
      completedActivities: existing?.completedActivities || [],
      notes: existing?.notes || '',
      mood: existing?.mood,
    };

    await memoryStore.saveDailyActivity(dailyActivity);
  }

  /**
   * Get daily activity
   */
  async getDailyActivity(userId: string, date: string): Promise<DailyActivity | null> {
    return memoryStore.getDailyActivity(userId, date);
  }

  /**
   * Get yesterday's activity
   */
  async getYesterdayActivity(userId: string): Promise<DailyActivity | null> {
    const yesterday = this.getDateString(new Date(Date.now() - 24 * 60 * 60 * 1000));
    return memoryStore.getDailyActivity(userId, yesterday);
  }

  /**
   * Mark activity as completed
   */
  async markActivityCompleted(
    userId: string,
    date: string,
    activity: string
  ): Promise<void> {
    await memoryStore.markActivityCompleted(userId, date, activity);
  }

  /**
   * Get activities in date range
   */
  async getActivitiesInRange(
    userId: string,
    startDate: string,
    endDate: string
  ): Promise<DailyActivity[]> {
    return memoryStore.getActivitiesInRange(userId, startDate, endDate);
  }

  /**
   * Update activity notes
   */
  async updateActivityNotes(userId: string, date: string, notes: string): Promise<void> {
    const activity = await memoryStore.getDailyActivity(userId, date);
    if (!activity) {
      throw new Error('Activity not found');
    }

    activity.notes = notes;
    await memoryStore.saveDailyActivity(activity);
  }

  /**
   * Update activity mood
   */
  async updateActivityMood(userId: string, date: string, mood: string): Promise<void> {
    const activity = await memoryStore.getDailyActivity(userId, date);
    if (!activity) {
      throw new Error('Activity not found');
    }

    activity.mood = mood;
    await memoryStore.saveDailyActivity(activity);
  }

  // ==================== Memory Recall ====================

  /**
   * Recall recent topics
   */
  async recallRecentTopics(userId: string, limit: number = 5): Promise<string[]> {
    const conversations = await this.getConversations(userId, 7);
    const topics = new Set<string>();

    conversations.forEach((conv) => {
      topics.add(conv.topic);
      conv.keyPoints.forEach((point) => topics.add(point));
    });

    return Array.from(topics).slice(0, limit);
  }

  /**
   * Find related conversations by topic
   */
  async findRelatedConversations(topic: string, userId: string): Promise<Conversation[]> {
    return this.searchConversations(userId, topic);
  }

  /**
   * Get conversation summary for date range
   */
  async getConversationSummary(
    userId: string,
    startDate: string,
    endDate: string
  ): Promise<{
    totalConversations: number;
    topTopics: string[];
    keyActivities: string[];
    averageSentiment: string;
  }> {
    const startTime = new Date(startDate).getTime();
    const endTime = new Date(endDate).getTime();

    const conversations = await memoryStore.getConversationsByDateRange(
      userId,
      startTime,
      endTime
    );

    const activities = await memoryStore.getActivitiesInRange(userId, startDate, endDate);

    // Count topics
    const topicCounts = new Map<string, number>();
    conversations.forEach((conv) => {
      topicCounts.set(conv.topic, (topicCounts.get(conv.topic) || 0) + 1);
    });

    const topTopics = Array.from(topicCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5)
      .map(([topic]) => topic);

    // Get key activities
    const allActivities = activities.flatMap((a) => a.plannedActivities);
    const keyActivities = Array.from(new Set(allActivities)).slice(0, 10);

    // Average sentiment
    const sentimentCounts = { positive: 0, neutral: 0, negative: 0 };
    conversations.forEach((conv) => {
      sentimentCounts[conv.sentiment]++;
    });

    const averageSentiment =
      sentimentCounts.positive > sentimentCounts.negative
        ? 'positive'
        : sentimentCounts.negative > sentimentCounts.positive
        ? 'negative'
        : 'neutral';

    return {
      totalConversations: conversations.length,
      topTopics,
      keyActivities,
      averageSentiment,
    };
  }

  // ==================== Cleanup ====================

  /**
   * Prune old conversations based on retention policy
   */
  async pruneOldConversations(userId: string, retentionDays: number): Promise<number> {
    return memoryStore.deleteOldConversations(userId, retentionDays);
  }

  /**
   * Prune old conversations for all users with default retention
   */
  async pruneAllOldConversations(): Promise<void> {
    // This would need to iterate through all users
    // For now, it's a placeholder that should be called per-user
    console.log('Pruning old conversations...');
  }

  // ==================== Private Helpers ====================

  /**
   * Infer conversation topic from turns
   */
  private async inferTopic(turns: ConversationTurn[]): Promise<string> {
    const userTurns = turns.filter((turn) => turn.speaker === 'user');
    if (userTurns.length === 0) return 'General conversation';

    const allText = userTurns.map((turn) => turn.text).join(' ');

    // Use KeyPointsExtractor to identify topics
    const topics = await keyPointsExtractor.identifyTopics({
      id: '',
      timestamp: turns[0].timestamp,
      turns,
      topic: '',
      sentiment: 'neutral',
      keyPoints: [],
    });

    if (topics.length > 0) {
      return topics[0];
    }

    // Fallback: use first few words of first user turn
    const firstTurn = userTurns[0].text;
    const words = firstTurn.split(/\s+/).slice(0, 5);
    return words.join(' ');
  }

  /**
   * Analyze sentiment from conversation turns
   */
  private analyzeSentiment(turns: ConversationTurn[]): 'positive' | 'neutral' | 'negative' {
    const userTurns = turns.filter((turn) => turn.speaker === 'user');
    if (userTurns.length === 0) return 'neutral';

    let positiveScore = 0;
    let negativeScore = 0;

    const positiveWords = new Set([
      'happy',
      'great',
      'awesome',
      'excellent',
      'fantastic',
      'wonderful',
      'love',
      'like',
      'enjoy',
      'fun',
      'exciting',
      'amazing',
      'good',
      'nice',
      'pleased',
      'glad',
      'excited',
      'cool',
      'yay',
      'yes',
      'thanks',
      'thank',
    ]);

    const negativeWords = new Set([
      'sad',
      'bad',
      'terrible',
      'awful',
      'horrible',
      'hate',
      'dislike',
      'boring',
      'frustrating',
      'annoying',
      'difficult',
      'hard',
      'problem',
      'issue',
      'worry',
      'scared',
      'angry',
      'upset',
      'no',
      'not',
      'never',
    ]);

    userTurns.forEach((turn) => {
      const words = turn.text.toLowerCase().split(/\s+/);
      words.forEach((word) => {
        if (positiveWords.has(word)) positiveScore++;
        if (negativeWords.has(word)) negativeScore++;
      });
    });

    if (positiveScore > negativeScore + 1) return 'positive';
    if (negativeScore > positiveScore + 1) return 'negative';
    return 'neutral';
  }

  /**
   * Get date string in YYYY-MM-DD format
   */
  private getDateString(date: Date): string {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  }
}

// Export singleton instance
export const conversationMemoryService = new ConversationMemoryService();
