/**
 * News Service Integration Tests
 * Tests the complete news fetching and personalization system
 *
 * Contract: I-NEWS-001 - News personalization and learning
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { NewsAPIClient } from '../../web/src/services/news/NewsAPIClient';
import { NewsPreferencesManager } from '../../web/src/services/news/NewsPreferencesManager';
import { NewsService } from '../../web/src/services/news/NewsService';
import { NewsBriefingGenerator } from '../../web/src/services/news/NewsBriefingGenerator';
import type { NewsArticle, UserFeedback } from '../../web/src/types/voice';

describe('NewsAPIClient', () => {
  let client: NewsAPIClient;

  beforeEach(() => {
    client = new NewsAPIClient('test-api-key');
  });

  afterEach(() => {
    client.clearCache();
  });

  it('should fetch top headlines', async () => {
    // Mock fetch
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        status: 'ok',
        totalResults: 10,
        articles: [
          {
            title: 'Test Headline',
            description: 'Test description',
            source: { name: 'BBC' },
            publishedAt: '2024-01-01T00:00:00Z',
            url: 'https://example.com',
            urlToImage: 'https://example.com/image.jpg'
          }
        ]
      })
    });

    const response = await client.getTopHeadlines({
      category: 'technology',
      pageSize: 10
    });

    expect(response.status).toBe('ok');
    expect(response.articles).toHaveLength(1);
    expect(response.articles[0].headline).toBe('Test Headline');
  });

  it('should cache responses', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        status: 'ok',
        totalResults: 1,
        articles: []
      })
    });

    // First call
    await client.getTopHeadlines({ category: 'technology' });

    // Second call should use cache
    await client.getTopHeadlines({ category: 'technology' });

    // Fetch should only be called once
    expect(global.fetch).toHaveBeenCalledTimes(1);
  });

  it('should enforce rate limits', async () => {
    // Create client with low rate limit for testing
    const limitedClient = new (class extends NewsAPIClient {
      constructor() {
        super('test-key');
      }
    })();

    // Mock to always succeed
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        status: 'ok',
        totalResults: 0,
        articles: []
      })
    });

    // Make many requests
    for (let i = 0; i < 101; i++) {
      try {
        await limitedClient.getTopHeadlines({ category: 'tech', pageSize: 1, page: i });
      } catch (error) {
        // Should hit rate limit
        expect((error as Error).message).toContain('Rate limit exceeded');
        return;
      }
    }
  });

  it('should handle API errors', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: false,
      statusText: 'Unauthorized',
      json: async () => ({ message: 'Invalid API key' })
    });

    await expect(client.getTopHeadlines({ category: 'tech' })).rejects.toThrow('Invalid API key');
  });
});

describe('NewsPreferencesManager', () => {
  let manager: NewsPreferencesManager;
  const userId = 'test-user-123';

  beforeEach(() => {
    manager = new NewsPreferencesManager();
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('should create default preferences', async () => {
    const prefs = await manager.getPreferences(userId);

    expect(prefs.userId).toBe(userId);
    expect(prefs.topics).toEqual(['technology', 'science']);
    expect(prefs.maxArticles).toBe(5);
  });

  it('should add topics', async () => {
    await manager.addTopic(userId, 'sports');

    const prefs = await manager.getPreferences(userId);
    expect(prefs.topics).toContain('sports');
    expect(prefs.topicWeights['sports']).toBe(1.0);
  });

  it('should remove topics', async () => {
    await manager.addTopic(userId, 'sports');
    await manager.removeTopic(userId, 'sports');

    const prefs = await manager.getPreferences(userId);
    expect(prefs.topics).not.toContain('sports');
    expect(prefs.topicWeights['sports']).toBeUndefined();
  });

  it('should exclude topics', async () => {
    await manager.excludeTopic(userId, 'politics');

    const prefs = await manager.getPreferences(userId);
    expect(prefs.excludeTopics).toContain('politics');
    expect(prefs.topicWeights['politics']).toBe(0.1); // MIN_WEIGHT
  });

  it('should adjust topic weights', async () => {
    await manager.addTopic(userId, 'technology');
    await manager.adjustWeights(userId, 'technology', 0.2);

    const weights = await manager.getTopicWeights(userId);
    expect(weights['technology']).toBeCloseTo(1.2, 1);
  });

  it('should learn from feedback', async () => {
    const article: NewsArticle = {
      id: 'test-1',
      headline: 'Test',
      summary: 'Test summary',
      source: 'BBC',
      category: 'technology',
      publishedAt: Date.now(),
      url: 'https://example.com',
      relevanceScore: 0.5
    };

    const feedback: UserFeedback = {
      articleId: article.id,
      action: 'like',
      timestamp: Date.now()
    };

    await manager.learnFromFeedback(userId, article, feedback);

    const weights = await manager.getTopicWeights(userId);
    expect(weights['technology']).toBeGreaterThan(1.0);
  });

  it('should clamp weights to min/max', async () => {
    await manager.addTopic(userId, 'test');

    // Try to increase beyond max
    for (let i = 0; i < 20; i++) {
      await manager.adjustWeights(userId, 'test', 0.2);
    }

    const weights = await manager.getTopicWeights(userId);
    expect(weights['test']).toBeLessThanOrEqual(2.0); // MAX_WEIGHT
  });

  it('should persist preferences', async () => {
    await manager.addTopic(userId, 'sports');
    await manager.updatePreferences(userId, { maxArticles: 10 });

    // Create new manager (simulates page reload)
    const newManager = new NewsPreferencesManager();
    const prefs = await newManager.getPreferences(userId);

    expect(prefs.topics).toContain('sports');
    expect(prefs.maxArticles).toBe(10);
  });
});

describe('NewsService', () => {
  let service: NewsService;
  let mockClient: NewsAPIClient;
  let mockManager: NewsPreferencesManager;
  const userId = 'test-user-123';

  beforeEach(() => {
    mockClient = new NewsAPIClient('test-key');
    mockManager = new NewsPreferencesManager();
    service = new NewsService(mockClient, mockManager);

    // Mock API responses
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        status: 'ok',
        totalResults: 5,
        articles: [
          {
            title: 'Tech News 1',
            description: 'Description 1',
            source: { name: 'TechCrunch' },
            publishedAt: new Date().toISOString(),
            url: 'https://example.com/1'
          },
          {
            title: 'Science News 1',
            description: 'Description 2',
            source: { name: 'Nature' },
            publishedAt: new Date().toISOString(),
            url: 'https://example.com/2'
          }
        ]
      })
    });
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('should fetch personalized news', async () => {
    const prefs = await mockManager.getPreferences(userId);
    const articles = await service.fetchNews(prefs);

    expect(articles).toBeInstanceOf(Array);
    expect(articles.length).toBeGreaterThan(0);
    expect(articles.length).toBeLessThanOrEqual(prefs.maxArticles);
  });

  it('should personalize articles with relevance scores', async () => {
    const prefs = await mockManager.getPreferences(userId);
    prefs.topics = ['technology'];

    const mockArticles: NewsArticle[] = [
      {
        id: '1',
        headline: 'Tech Article',
        summary: 'About technology',
        source: 'TechCrunch',
        category: 'technology',
        publishedAt: Date.now(),
        url: 'https://example.com/1',
        relevanceScore: 0.5
      },
      {
        id: '2',
        headline: 'Sports Article',
        summary: 'About sports',
        source: 'ESPN',
        category: 'sports',
        publishedAt: Date.now(),
        url: 'https://example.com/2',
        relevanceScore: 0.5
      }
    ];

    const personalized = await service.personalizeNews(userId, mockArticles);

    // Tech article should have higher relevance for user who likes tech
    expect(personalized[0].relevanceScore).toBeGreaterThan(personalized[1].relevanceScore);
  });

  it('should rank by relevance', async () => {
    const mockArticles: NewsArticle[] = [
      {
        id: '1',
        headline: 'Old Article',
        summary: 'Test',
        source: 'BBC',
        category: 'sports',
        publishedAt: Date.now() - 86400000, // 1 day old
        url: 'https://example.com/1',
        relevanceScore: 0.3
      },
      {
        id: '2',
        headline: 'New Article',
        summary: 'Test',
        source: 'BBC',
        category: 'technology',
        publishedAt: Date.now(), // Recent
        url: 'https://example.com/2',
        relevanceScore: 0.7
      }
    ];

    const ranked = await service.rankByRelevance(userId, mockArticles);

    // Higher relevance should come first
    expect(ranked[0].id).toBe('2');
    expect(ranked[1].id).toBe('1');
  });

  it('should update preferences based on interactions', async () => {
    const article: NewsArticle = {
      id: 'test-1',
      headline: 'Test Tech Article',
      summary: 'About AI',
      source: 'TechCrunch',
      category: 'technology',
      publishedAt: Date.now(),
      url: 'https://example.com',
      relevanceScore: 0.5
    };

    await service.updatePreferences(userId, article, 'read');

    const weights = await mockManager.getTopicWeights(userId);
    expect(weights['technology']).toBeGreaterThan(1.0);
  });

  it('should generate morning briefing', async () => {
    const articles = await service.getBriefing(userId, 3);

    expect(articles).toBeInstanceOf(Array);
    expect(articles.length).toBeLessThanOrEqual(3);
  });

  it('should search news', async () => {
    const articles = await service.searchNews('artificial intelligence');

    expect(articles).toBeInstanceOf(Array);
  });
});

describe('NewsBriefingGenerator', () => {
  let generator: NewsBriefingGenerator;
  const userId = 'test-user-123';

  beforeEach(() => {
    generator = new NewsBriefingGenerator();
  });

  it('should generate briefing section', async () => {
    const mockArticles: NewsArticle[] = [
      {
        id: '1',
        headline: 'Breaking Tech News',
        summary: 'AI advances rapidly',
        source: 'TechCrunch',
        category: 'technology',
        publishedAt: Date.now(),
        url: 'https://example.com/1',
        relevanceScore: 0.9
      }
    ];

    const briefing = await generator.generateBriefing(userId, mockArticles, 'Alice');

    expect(briefing.type).toBe('news');
    expect(briefing.content).toContain('Alice');
    expect(briefing.content).toContain('Breaking Tech News');
    expect(briefing.articles).toEqual(mockArticles);
  });

  it('should summarize articles', async () => {
    const article: NewsArticle = {
      id: '1',
      headline: 'Test Headline',
      summary: 'This is a test summary of moderate length that should be returned as-is',
      source: 'BBC',
      category: 'tech',
      publishedAt: Date.now(),
      url: 'https://example.com',
      relevanceScore: 0.5
    };

    const summary = await generator.summarizeArticle(article);

    expect(summary).toBe(article.summary);
  });

  it('should format for speech', async () => {
    const articles: NewsArticle[] = [
      {
        id: '1',
        headline: 'First Article',
        summary: 'Summary 1',
        source: 'Source 1',
        category: 'tech',
        publishedAt: Date.now(),
        url: 'https://example.com/1',
        relevanceScore: 0.8
      },
      {
        id: '2',
        headline: 'Second Article',
        summary: 'Summary 2',
        source: 'Source 2',
        category: 'science',
        publishedAt: Date.now(),
        url: 'https://example.com/2',
        relevanceScore: 0.7
      }
    ];

    const speech = await generator.formatForSpeech(articles);

    expect(speech).toContain('First:');
    expect(speech).toContain('First Article');
    expect(speech).toContain('Second:');
    expect(speech).toContain('Second Article');
  });

  it('should handle empty articles', async () => {
    const speech = await generator.formatForSpeech([]);

    expect(speech).toContain("don't have any news");
  });

  it('should estimate speaking duration', () => {
    const text = 'This is a test sentence. '.repeat(50); // ~150 words
    const duration = generator.estimateSpeakingDuration(text);

    expect(duration).toBeGreaterThan(0);
    expect(duration).toBeLessThan(120); // Should be around 60 seconds
  });

  it('should generate quick summary', () => {
    const articles: NewsArticle[] = [
      {
        id: '1',
        headline: 'Headline 1',
        summary: 'Summary 1',
        source: 'Source 1',
        category: 'tech',
        publishedAt: Date.now(),
        url: 'https://example.com/1',
        relevanceScore: 0.8
      },
      {
        id: '2',
        headline: 'Headline 2',
        summary: 'Summary 2',
        source: 'Source 2',
        category: 'science',
        publishedAt: Date.now(),
        url: 'https://example.com/2',
        relevanceScore: 0.7
      }
    ];

    const summary = generator.generateQuickSummary(articles, 2);

    expect(summary).toContain('Headline 1');
    expect(summary).toContain('Headline 2');
  });
});

describe('Integration: Complete News Flow', () => {
  it('should complete full news personalization workflow', async () => {
    const userId = 'integration-test-user';

    // 1. Setup
    const prefsManager = new NewsPreferencesManager();
    const apiClient = new NewsAPIClient('test-key');
    const service = new NewsService(apiClient, prefsManager);
    const generator = new NewsBriefingGenerator();

    // Mock API
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        status: 'ok',
        totalResults: 3,
        articles: [
          {
            title: 'AI Breakthrough',
            description: 'New AI model announced',
            source: { name: 'TechCrunch' },
            publishedAt: new Date().toISOString(),
            url: 'https://example.com/ai',
            urlToImage: 'https://example.com/ai.jpg'
          }
        ]
      })
    });

    // 2. Configure preferences
    await prefsManager.addTopic(userId, 'technology');
    await prefsManager.addTopic(userId, 'science');

    // 3. Fetch personalized news
    const prefs = await prefsManager.getPreferences(userId);
    const articles = await service.fetchNews(prefs);

    expect(articles.length).toBeGreaterThan(0);

    // 4. Generate briefing
    const briefing = await generator.generateBriefing(userId, articles, 'Test User');

    expect(briefing.type).toBe('news');
    expect(briefing.content).toContain('Test User');
    expect(briefing.articles).toEqual(articles);

    // 5. Learn from feedback
    await service.updatePreferences(userId, articles[0], 'read');

    const weights = await prefsManager.getTopicWeights(userId);
    expect(Object.keys(weights).length).toBeGreaterThan(0);

    // Cleanup
    await prefsManager.clearAllPreferences();
  });
});
