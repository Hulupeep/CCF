/**
 * News Service
 * Main service for fetching and personalizing news content
 *
 * Contract: I-NEWS-001 - News personalization with relevance ranking
 */

import type {
  NewsPreferences,
  NewsArticle,
  UserFeedback
} from '../../types/voice';
import { NewsAPIClient, createNewsAPIClient } from './NewsAPIClient';
import { NewsPreferencesManager, getNewsPreferencesManager } from './NewsPreferencesManager';

export class NewsService {
  private apiClient: NewsAPIClient;
  private preferencesManager: NewsPreferencesManager;

  constructor(
    apiClient?: NewsAPIClient,
    preferencesManager?: NewsPreferencesManager
  ) {
    this.apiClient = apiClient || createNewsAPIClient();
    this.preferencesManager = preferencesManager || getNewsPreferencesManager();
  }

  /**
   * Fetch personalized news for a user
   * Applies preferences and ranks by relevance
   */
  async fetchNews(preferences: NewsPreferences): Promise<NewsArticle[]> {
    try {
      const articles: NewsArticle[] = [];

      // Fetch news for each preferred topic
      for (const topic of preferences.topics) {
        // Skip excluded topics
        if (preferences.excludeTopics.includes(topic)) {
          continue;
        }

        try {
          const response = await this.apiClient.getTopHeadlines({
            category: topic,
            pageSize: Math.ceil(preferences.maxArticles / preferences.topics.length),
            country: 'us' // TODO: Make configurable
          });

          articles.push(...response.articles);
        } catch (error) {
          console.warn(`Failed to fetch news for topic ${topic}:`, error);
          // Continue with other topics
        }
      }

      // Personalize articles
      const personalized = await this.personalizeNews(preferences.userId, articles);

      // Limit to max articles
      return personalized.slice(0, preferences.maxArticles);
    } catch (error) {
      console.error('Failed to fetch news:', error);
      throw error;
    }
  }

  /**
   * Search for news articles
   */
  async searchNews(query: string, category?: string): Promise<NewsArticle[]> {
    try {
      const response = await this.apiClient.searchEverything({
        q: query,
        language: 'en',
        sortBy: 'relevancy',
        pageSize: 20
      });

      // Filter by category if specified
      if (category) {
        return response.articles.filter(article =>
          article.category === category ||
          article.headline.toLowerCase().includes(category.toLowerCase()) ||
          article.summary.toLowerCase().includes(category.toLowerCase())
        );
      }

      return response.articles;
    } catch (error) {
      console.error('Failed to search news:', error);
      throw error;
    }
  }

  /**
   * Get top headlines for a country and category
   */
  async getTopHeadlines(country: string, category: string): Promise<NewsArticle[]> {
    try {
      const response = await this.apiClient.getTopHeadlines({
        country,
        category,
        pageSize: 20
      });

      return response.articles;
    } catch (error) {
      console.error('Failed to get top headlines:', error);
      throw error;
    }
  }

  /**
   * Personalize news articles based on user preferences
   * Calculates relevance scores based on topic weights and reading history
   */
  async personalizeNews(userId: string, articles: NewsArticle[]): Promise<NewsArticle[]> {
    const preferences = await this.preferencesManager.getPreferences(userId);
    const topicWeights = await this.preferencesManager.getTopicWeights(userId);

    // Calculate relevance scores
    const scoredArticles = articles.map(article => {
      const score = this.calculateRelevanceScore(article, preferences, topicWeights);
      return {
        ...article,
        relevanceScore: score
      };
    });

    // Sort by relevance score (highest first)
    scoredArticles.sort((a, b) => b.relevanceScore - a.relevanceScore);

    return scoredArticles;
  }

  /**
   * Rank articles by relevance to user
   */
  async rankByRelevance(userId: string, articles: NewsArticle[]): Promise<NewsArticle[]> {
    return this.personalizeNews(userId, articles);
  }

  /**
   * Calculate relevance score for an article
   * Score components:
   * - Topic match weight (40%)
   * - Source preference (20%)
   * - Recency (20%)
   * - Content quality (20%)
   */
  private calculateRelevanceScore(
    article: NewsArticle,
    preferences: NewsPreferences,
    topicWeights: Record<string, number>
  ): number {
    let score = 0;

    // 1. Topic match weight (40%)
    if (article.category) {
      const topicWeight = topicWeights[article.category] || 1.0;
      const topicMatch = preferences.topics.includes(article.category) ? 1.0 : 0.5;
      score += (topicWeight * topicMatch) * 0.4;
    }

    // Check headline and summary for topic keywords
    const text = `${article.headline} ${article.summary}`.toLowerCase();
    for (const topic of preferences.topics) {
      if (text.includes(topic.toLowerCase())) {
        const weight = topicWeights[topic] || 1.0;
        score += weight * 0.1; // Bonus for keyword match
      }
    }

    // 2. Source preference (20%)
    const sourceWeight = topicWeights[`source:${article.source}`] || 1.0;
    const sourceMatch = preferences.sources.length === 0 ||
      preferences.sources.includes(article.source.toLowerCase());
    score += (sourceWeight * (sourceMatch ? 1.0 : 0.7)) * 0.2;

    // 3. Recency (20%)
    const ageHours = (Date.now() - article.publishedAt) / (1000 * 60 * 60);
    const recencyScore = Math.max(0, 1 - (ageHours / 24)); // Decay over 24 hours
    score += recencyScore * 0.2;

    // 4. Content quality indicators (20%)
    let qualityScore = 0.5; // Base score

    // Has image
    if (article.imageUrl) qualityScore += 0.1;

    // Has author
    if (article.author) qualityScore += 0.1;

    // Has substantial summary
    if (article.summary && article.summary.length > 100) qualityScore += 0.1;

    // Has content
    if (article.content && article.content.length > 200) qualityScore += 0.2;

    score += Math.min(1.0, qualityScore) * 0.2;

    // Normalize to 0-1 range
    return Math.max(0, Math.min(1, score));
  }

  /**
   * Update preferences based on user interaction with an article
   * Contract: I-NEWS-001 - Learning from user behavior
   */
  async updatePreferences(
    userId: string,
    article: NewsArticle,
    action: 'read' | 'skip'
  ): Promise<void> {
    const feedback: UserFeedback = {
      articleId: article.id,
      action,
      timestamp: Date.now()
    };

    await this.preferencesManager.learnFromFeedback(userId, article, feedback);
  }

  /**
   * Learn from explicit user feedback
   */
  async learnFromFeedback(userId: string, feedback: UserFeedback): Promise<void> {
    // We need the article to learn from it, but we don't have it in the feedback
    // This would typically come from a cache or database
    // For now, this is a placeholder that delegates to the preferences manager

    console.warn('learnFromFeedback requires article context - implement caching');

    // TODO: Implement article caching to support this method
  }

  /**
   * Get personalized news briefing for user
   * Returns top articles formatted for morning briefing
   */
  async getBriefing(userId: string, maxArticles: number = 5): Promise<NewsArticle[]> {
    const preferences = await this.preferencesManager.getPreferences(userId);

    // Override max articles for briefing
    const briefingPrefs = {
      ...preferences,
      maxArticles
    };

    const articles = await this.fetchNews(briefingPrefs);

    return articles;
  }

  /**
   * Get news for a specific topic with personalization
   */
  async getTopicNews(userId: string, topic: string, limit: number = 10): Promise<NewsArticle[]> {
    try {
      const response = await this.apiClient.getTopHeadlines({
        category: topic,
        pageSize: limit,
        country: 'us'
      });

      // Apply personalization
      const personalized = await this.personalizeNews(userId, response.articles);

      return personalized.slice(0, limit);
    } catch (error) {
      console.error(`Failed to get ${topic} news:`, error);
      throw error;
    }
  }

  /**
   * Get trending topics based on available news
   */
  async getTrendingTopics(): Promise<string[]> {
    try {
      // Fetch general headlines
      const response = await this.apiClient.getTopHeadlines({
        country: 'us',
        pageSize: 50
      });

      // Extract keywords from headlines
      const keywords: Map<string, number> = new Map();

      response.articles.forEach(article => {
        const words = article.headline
          .toLowerCase()
          .split(/\s+/)
          .filter(word => word.length > 4); // Filter short words

        words.forEach(word => {
          keywords.set(word, (keywords.get(word) || 0) + 1);
        });
      });

      // Sort by frequency and return top 10
      return Array.from(keywords.entries())
        .sort((a, b) => b[1] - a[1])
        .slice(0, 10)
        .map(([word]) => word);
    } catch (error) {
      console.error('Failed to get trending topics:', error);
      return [];
    }
  }

  /**
   * Clear all caches (for testing)
   */
  clearCache(): void {
    this.apiClient.clearCache();
  }
}

// Singleton instance
let instance: NewsService | null = null;

/**
 * Get the singleton news service instance
 */
export function getNewsService(): NewsService {
  if (!instance) {
    instance = new NewsService();
  }
  return instance;
}
