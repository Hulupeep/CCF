/**
 * Marketplace Service
 * Issue #85 - Cloud Personality Marketplace
 *
 * Main service interface for marketplace operations.
 */

import { PersonalityConfig } from '../../types/personality';
import {
  MarketplaceListing,
  PersonalityMetadata,
  SearchQuery,
  SearchResult,
} from '../../types/marketplace';
import { marketplaceClient } from './supabase-client';
import { validatePersonalityForMarketplace, validateRating, validateReport } from './validation';

export class MarketplaceService {
  /**
   * Publish personality to marketplace
   * I-CLOUD-004: Validates before publication
   */
  async publishPersonality(
    config: PersonalityConfig,
    metadata: PersonalityMetadata
  ): Promise<MarketplaceListing> {
    // Validate
    const validation = validatePersonalityForMarketplace(
      metadata.description.split('\n')[0] || 'Untitled', // Use first line as name
      metadata.description,
      metadata.tags,
      config
    );

    // Publish (validated or with errors)
    return marketplaceClient.publishPersonality(
      metadata.description.split('\n')[0] || 'Untitled',
      metadata.description,
      metadata.tags,
      config,
      metadata.thumbnailUrl,
      validation.valid,
      validation.errors
    );
  }

  /**
   * Unpublish personality
   */
  async unpublishPersonality(id: string): Promise<void> {
    return marketplaceClient.unpublishPersonality(id);
  }

  /**
   * Search personalities
   * I-CLOUD-006: Results within 500ms for 10k personalities
   */
  async searchPersonalities(query: SearchQuery): Promise<SearchResult> {
    return marketplaceClient.searchPersonalities(query);
  }

  /**
   * Get single personality
   */
  async getPersonality(id: string): Promise<MarketplaceListing> {
    return marketplaceClient.getPersonality(id);
  }

  /**
   * Get trending personalities
   */
  async getTrendingPersonalities(limit: number): Promise<MarketplaceListing[]> {
    return marketplaceClient.getTrendingPersonalities(limit);
  }

  /**
   * Download personality
   */
  async downloadPersonality(id: string): Promise<PersonalityConfig> {
    return marketplaceClient.downloadPersonality(id);
  }

  /**
   * Rate personality
   * I-CLOUD-005: One rating per user per personality
   */
  async ratePersonality(id: string, rating: number): Promise<void> {
    if (!validateRating(rating)) {
      throw new Error('Rating must be an integer between 1 and 5');
    }

    return marketplaceClient.ratePersonality(id, rating);
  }

  /**
   * Report personality
   */
  async reportPersonality(id: string, reason: string): Promise<void> {
    const validation = validateReport(reason);
    if (!validation.valid) {
      throw new Error(validation.errors.join(', '));
    }

    return marketplaceClient.reportPersonality(id, reason);
  }

  /**
   * Get user's published personalities
   */
  async getUserPublishedPersonalities(): Promise<MarketplaceListing[]> {
    return marketplaceClient.getUserPublishedPersonalities();
  }

  /**
   * Check if user has rated a personality
   */
  async hasUserRated(id: string): Promise<boolean> {
    return marketplaceClient.hasUserRated(id);
  }
}

export const marketplaceService = new MarketplaceService();
