/**
 * News API Client
 * Handles communication with NewsAPI.org or similar news aggregation service
 *
 * Contract: I-NEWS-001 - News personalization and filtering
 */

import type {
  NewsResponse,
  NewsArticle,
  TopHeadlinesParams,
  SearchParams
} from '../../types/voice';

interface CacheEntry {
  data: NewsResponse;
  timestamp: number;
  expiresAt: number;
}

export class NewsAPIClient {
  private baseURL = 'https://newsapi.org/v2';
  private apiKey: string;
  private cache: Map<string, CacheEntry> = new Map();
  private requestCount: Map<string, number[]> = new Map(); // Track requests per minute
  private readonly CACHE_TTL = 15 * 60 * 1000; // 15 minutes
  private readonly RATE_LIMIT = 100; // requests per day for free tier
  private readonly RATE_WINDOW = 24 * 60 * 60 * 1000; // 24 hours

  constructor(apiKey: string) {
    if (!apiKey) {
      throw new Error('News API key is required');
    }
    this.apiKey = apiKey;
  }

  /**
   * Get top headlines based on parameters
   * Implements rate limiting and caching
   */
  async getTopHeadlines(params: TopHeadlinesParams): Promise<NewsResponse> {
    const cacheKey = `headlines:${JSON.stringify(params)}`;

    // Check cache first
    const cached = this.getFromCache(cacheKey);
    if (cached) {
      return cached;
    }

    // Check rate limit
    await this.checkRateLimit();

    // Build query string
    const queryParams = new URLSearchParams();
    if (params.country) queryParams.append('country', params.country);
    if (params.category) queryParams.append('category', params.category);
    if (params.sources) queryParams.append('sources', params.sources);
    if (params.q) queryParams.append('q', params.q);
    if (params.pageSize) queryParams.append('pageSize', params.pageSize.toString());
    if (params.page) queryParams.append('page', params.page.toString());

    const url = `${this.baseURL}/top-headlines?${queryParams.toString()}`;

    try {
      const response = await fetch(url, {
        headers: {
          'X-Api-Key': this.apiKey
        }
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(`News API error: ${error.message || response.statusText}`);
      }

      const data = await response.json();
      const newsResponse = this.transformResponse(data);

      // Cache the response
      this.setCache(cacheKey, newsResponse);

      // Track request for rate limiting
      this.trackRequest();

      return newsResponse;
    } catch (error) {
      console.error('Failed to fetch top headlines:', error);
      throw error;
    }
  }

  /**
   * Search for news articles
   */
  async searchEverything(params: SearchParams): Promise<NewsResponse> {
    const cacheKey = `search:${JSON.stringify(params)}`;

    // Check cache first
    const cached = this.getFromCache(cacheKey);
    if (cached) {
      return cached;
    }

    // Check rate limit
    await this.checkRateLimit();

    // Build query string
    const queryParams = new URLSearchParams();
    queryParams.append('q', params.q);
    if (params.sources) queryParams.append('sources', params.sources);
    if (params.domains) queryParams.append('domains', params.domains);
    if (params.excludeDomains) queryParams.append('excludeDomains', params.excludeDomains);
    if (params.from) queryParams.append('from', params.from);
    if (params.to) queryParams.append('to', params.to);
    if (params.language) queryParams.append('language', params.language);
    if (params.sortBy) queryParams.append('sortBy', params.sortBy);
    if (params.pageSize) queryParams.append('pageSize', params.pageSize.toString());
    if (params.page) queryParams.append('page', params.page.toString());

    const url = `${this.baseURL}/everything?${queryParams.toString()}`;

    try {
      const response = await fetch(url, {
        headers: {
          'X-Api-Key': this.apiKey
        }
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(`News API error: ${error.message || response.statusText}`);
      }

      const data = await response.json();
      const newsResponse = this.transformResponse(data);

      // Cache the response
      this.setCache(cacheKey, newsResponse);

      // Track request for rate limiting
      this.trackRequest();

      return newsResponse;
    } catch (error) {
      console.error('Failed to search news:', error);
      throw error;
    }
  }

  /**
   * Transform API response to our format
   */
  private transformResponse(data: any): NewsResponse {
    return {
      status: data.status,
      totalResults: data.totalResults,
      articles: data.articles.map((article: any) => this.transformArticle(article))
    };
  }

  /**
   * Transform individual article
   */
  private transformArticle(article: any): NewsArticle {
    return {
      id: this.generateArticleId(article),
      headline: article.title || '',
      summary: article.description || '',
      content: article.content || '',
      source: article.source?.name || 'Unknown',
      category: '', // NewsAPI doesn't provide category in response
      author: article.author || undefined,
      publishedAt: new Date(article.publishedAt).getTime(),
      url: article.url,
      imageUrl: article.urlToImage || undefined,
      relevanceScore: 0.5 // Default, will be calculated by personalization
    };
  }

  /**
   * Generate a unique article ID
   */
  private generateArticleId(article: any): string {
    const source = article.source?.id || article.source?.name || 'unknown';
    const date = new Date(article.publishedAt).getTime();
    const titleHash = this.simpleHash(article.title || '');
    return `${source}-${date}-${titleHash}`;
  }

  /**
   * Simple hash function for generating article IDs
   */
  private simpleHash(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash).toString(36);
  }

  /**
   * Get from cache if valid
   */
  private getFromCache(key: string): NewsResponse | null {
    const entry = this.cache.get(key);
    if (!entry) return null;

    const now = Date.now();
    if (now > entry.expiresAt) {
      this.cache.delete(key);
      return null;
    }

    return entry.data;
  }

  /**
   * Set cache entry
   */
  private setCache(key: string, data: NewsResponse): void {
    const now = Date.now();
    this.cache.set(key, {
      data,
      timestamp: now,
      expiresAt: now + this.CACHE_TTL
    });
  }

  /**
   * Track request for rate limiting
   */
  private trackRequest(): void {
    const now = Date.now();
    const key = 'requests';
    const requests = this.requestCount.get(key) || [];

    // Add current request
    requests.push(now);

    // Remove requests older than rate window
    const filtered = requests.filter(time => now - time < this.RATE_WINDOW);

    this.requestCount.set(key, filtered);
  }

  /**
   * Check if rate limit is exceeded
   */
  private async checkRateLimit(): Promise<void> {
    const key = 'requests';
    const requests = this.requestCount.get(key) || [];
    const now = Date.now();

    // Filter to requests within window
    const recentRequests = requests.filter(time => now - time < this.RATE_WINDOW);

    if (recentRequests.length >= this.RATE_LIMIT) {
      throw new Error(
        `Rate limit exceeded: ${this.RATE_LIMIT} requests per day. ` +
        `Try again in ${Math.ceil((recentRequests[0] + this.RATE_WINDOW - now) / 1000 / 60)} minutes.`
      );
    }
  }

  /**
   * Clear cache (useful for testing)
   */
  clearCache(): void {
    this.cache.clear();
  }

  /**
   * Get current request count
   */
  getRequestCount(): number {
    const key = 'requests';
    const requests = this.requestCount.get(key) || [];
    const now = Date.now();
    return requests.filter(time => now - time < this.RATE_WINDOW).length;
  }

  /**
   * Check if cache is available for a given key pattern
   */
  hasCachedData(keyPattern: string): boolean {
    for (const key of this.cache.keys()) {
      if (key.includes(keyPattern)) {
        const entry = this.cache.get(key);
        if (entry && Date.now() <= entry.expiresAt) {
          return true;
        }
      }
    }
    return false;
  }
}

/**
 * Create a news API client instance
 */
export function createNewsAPIClient(): NewsAPIClient {
  const apiKey = import.meta.env.VITE_NEWS_API_KEY || process.env.NEWS_API_KEY || '';

  if (!apiKey) {
    console.warn('News API key not configured. Set VITE_NEWS_API_KEY environment variable.');
  }

  return new NewsAPIClient(apiKey);
}
