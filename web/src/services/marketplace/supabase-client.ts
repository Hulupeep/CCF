/**
 * Supabase Client for Marketplace
 * Issue #85 - Cloud Personality Marketplace
 *
 * Mock implementation until Supabase is configured.
 * Replace with actual @supabase/supabase-js when ready.
 */

import {
  MarketplaceListing,
  MarketplaceRecord,
  RatingRecord,
  ReportRecord,
  SearchQuery,
  SearchResult,
} from '../../types/marketplace';
import { PersonalityConfig } from '../../types/personality';

// Mock user ID (replace with auth.uid() from Supabase)
let mockUserId = 'mock-user-id';

export function setMockUserId(userId: string) {
  mockUserId = userId;
}

export function getCurrentUserId(): string {
  return mockUserId;
}

// Mock in-memory storage (replace with Supabase client)
export const mockMarketplace: MarketplaceRecord[] = [];
export const mockRatings: RatingRecord[] = [];
export const mockReports: ReportRecord[] = [];

// Clear mock storage (for testing)
export function clearMockStorage() {
  mockMarketplace.length = 0;
  mockRatings.length = 0;
  mockReports.length = 0;
}

// Convert database record to listing
function recordToListing(record: MarketplaceRecord): MarketplaceListing {
  return {
    id: record.id,
    name: record.name,
    description: record.description,
    tags: record.tags,
    authorId: record.user_id,
    authorName: `User ${record.user_id.slice(0, 8)}`, // Mock name
    config: record.config,
    thumbnailUrl: record.thumbnail_url,
    rating: record.rating,
    ratingCount: record.rating_count,
    downloadCount: record.download_count,
    createdAt: new Date(record.created_at).getTime(),
    updatedAt: new Date(record.updated_at).getTime(),
    validated: record.validated,
    validationErrors: record.validation_errors,
  };
}

export class SupabaseMarketplaceClient {
  /**
   * Publish personality to marketplace
   * I-CLOUD-004: Must be validated before appearing in search
   */
  async publishPersonality(
    name: string,
    description: string,
    tags: string[],
    config: PersonalityConfig,
    thumbnailUrl?: string,
    validated: boolean = true,
    validationErrors?: string[]
  ): Promise<MarketplaceListing> {
    const now = new Date().toISOString();
    const record: MarketplaceRecord = {
      id: `personality-${Date.now()}-${Math.random().toString(36).slice(2)}`,
      user_id: getCurrentUserId(),
      name,
      description,
      tags,
      config,
      thumbnail_url: thumbnailUrl,
      rating: 0,
      rating_count: 0,
      download_count: 0,
      validated,
      validation_errors: validationErrors,
      created_at: now,
      updated_at: now,
    };

    mockMarketplace.push(record);
    return recordToListing(record);
  }

  /**
   * Unpublish personality (only owner can do this)
   */
  async unpublishPersonality(personalityId: string): Promise<void> {
    const index = mockMarketplace.findIndex(
      (p) => p.id === personalityId && p.user_id === getCurrentUserId()
    );
    if (index === -1) {
      throw new Error('Personality not found or not owned by user');
    }
    mockMarketplace.splice(index, 1);
  }

  /**
   * Search personalities
   * I-CLOUD-006: Must return within 500ms for 10k personalities
   */
  async searchPersonalities(query: SearchQuery): Promise<SearchResult> {
    const startTime = Date.now();

    let results = mockMarketplace.filter((p) => p.validated);

    // Text search
    if (query.query) {
      const searchTerm = query.query.toLowerCase();
      results = results.filter(
        (p) =>
          p.name.toLowerCase().includes(searchTerm) ||
          p.description.toLowerCase().includes(searchTerm)
      );
    }

    // Tag filter
    if (query.tags && query.tags.length > 0) {
      results = results.filter((p) =>
        query.tags!.some((tag) => p.tags.includes(tag))
      );
    }

    // Rating filter
    if (query.minRating !== undefined) {
      results = results.filter((p) => p.rating >= query.minRating!);
    }

    // Sort
    const sortBy = query.sortBy || 'newest';
    results.sort((a, b) => {
      switch (sortBy) {
        case 'popular':
          return b.download_count - a.download_count;
        case 'rating':
          return b.rating - a.rating || b.rating_count - a.rating_count;
        case 'downloads':
          return b.download_count - a.download_count;
        case 'newest':
        default:
          return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      }
    });

    // Pagination
    const page = query.page || 1;
    const limit = query.limit || 20;
    const start = (page - 1) * limit;
    const end = start + limit;
    const paginatedResults = results.slice(start, end);

    const elapsedMs = Date.now() - startTime;
    console.log(`Search completed in ${elapsedMs}ms (I-CLOUD-006: must be <500ms)`);

    return {
      listings: paginatedResults.map(recordToListing),
      total: results.length,
      page,
      limit,
      hasMore: end < results.length,
    };
  }

  /**
   * Get single personality
   */
  async getPersonality(personalityId: string): Promise<MarketplaceListing> {
    const record = mockMarketplace.find((p) => p.id === personalityId);
    if (!record) {
      throw new Error('Personality not found');
    }
    return recordToListing(record);
  }

  /**
   * Get trending personalities
   */
  async getTrendingPersonalities(limit: number = 10): Promise<MarketplaceListing[]> {
    const validated = mockMarketplace.filter((p) => p.validated);
    const sorted = validated.sort((a, b) => {
      // Trending: combination of recent downloads and rating
      const aScore = a.download_count * 0.7 + a.rating * a.rating_count * 0.3;
      const bScore = b.download_count * 0.7 + b.rating * b.rating_count * 0.3;
      return bScore - aScore;
    });
    return sorted.slice(0, limit).map(recordToListing);
  }

  /**
   * Download personality
   * Increments download counter
   */
  async downloadPersonality(personalityId: string): Promise<PersonalityConfig> {
    const record = mockMarketplace.find((p) => p.id === personalityId);
    if (!record) {
      throw new Error('Personality not found');
    }

    // Increment download count
    record.download_count += 1;
    record.updated_at = new Date().toISOString();

    return record.config;
  }

  /**
   * Rate personality
   * I-CLOUD-005: One rating per user per personality
   */
  async ratePersonality(personalityId: string, rating: number): Promise<void> {
    // Check if user already rated
    const existing = mockRatings.find(
      (r) => r.personality_id === personalityId && r.user_id === getCurrentUserId()
    );

    if (existing) {
      throw new Error('You have already rated this personality');
    }

    // Add rating
    const ratingRecord: RatingRecord = {
      id: `rating-${Date.now()}`,
      personality_id: personalityId,
      user_id: getCurrentUserId(),
      rating,
      created_at: new Date().toISOString(),
    };
    mockRatings.push(ratingRecord);

    // Update average rating
    this.updatePersonalityRating(personalityId);
  }

  /**
   * Update personality rating average
   * Triggered automatically by database trigger in production
   */
  private updatePersonalityRating(personalityId: string): void {
    const ratings = mockRatings.filter((r) => r.personality_id === personalityId);
    const avgRating = ratings.reduce((sum, r) => sum + r.rating, 0) / ratings.length;

    const personality = mockMarketplace.find((p) => p.id === personalityId);
    if (personality) {
      personality.rating = parseFloat(avgRating.toFixed(2));
      personality.rating_count = ratings.length;
      personality.updated_at = new Date().toISOString();
    }
  }

  /**
   * Report personality
   */
  async reportPersonality(personalityId: string, reason: string): Promise<void> {
    const reportRecord: ReportRecord = {
      id: `report-${Date.now()}`,
      personality_id: personalityId,
      user_id: getCurrentUserId(),
      reason,
      created_at: new Date().toISOString(),
    };
    mockReports.push(reportRecord);
  }

  /**
   * Get user's published personalities
   */
  async getUserPublishedPersonalities(): Promise<MarketplaceListing[]> {
    const userPersonalities = mockMarketplace.filter(
      (p) => p.user_id === getCurrentUserId()
    );
    return userPersonalities.map(recordToListing);
  }

  /**
   * Check if user has rated a personality
   */
  async hasUserRated(personalityId: string): Promise<boolean> {
    return mockRatings.some(
      (r) => r.personality_id === personalityId && r.user_id === getCurrentUserId()
    );
  }
}

// Export singleton instance
export const marketplaceClient = new SupabaseMarketplaceClient();
