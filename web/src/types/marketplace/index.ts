/**
 * Marketplace Type Definitions
 * Issue #85 - Cloud Personality Marketplace
 *
 * Implements:
 * - I-CLOUD-004: Marketplace personalities must pass validation before publication
 * - I-CLOUD-005: Rating system prevents abuse (1 rating per user per personality)
 * - I-CLOUD-006: Search results must return within 500ms for up to 10,000 personalities
 */

import { PersonalityConfig } from '../personality';

export interface MarketplaceListing {
  id: string;
  name: string;
  description: string;
  tags: string[];
  authorId: string;
  authorName: string;
  config: PersonalityConfig;
  thumbnailUrl?: string;

  // Metrics
  rating: number;              // Average 1-5
  ratingCount: number;
  downloadCount: number;
  createdAt: number;
  updatedAt: number;

  // Validation
  validated: boolean;
  validationErrors?: string[];
}

export interface PersonalityMetadata {
  description: string;
  tags: string[];
  thumbnailUrl?: string;
}

export interface SearchQuery {
  query?: string;              // Text search
  tags?: string[];            // Filter by tags
  minRating?: number;         // Min average rating
  sortBy?: 'popular' | 'rating' | 'newest' | 'downloads';
  page?: number;
  limit?: number;             // Default: 20
}

export interface PersonalityRating {
  id: string;
  personalityId: string;
  userId: string;
  rating: number;             // 1-5
  createdAt: number;
}

export interface PersonalityReport {
  id: string;
  personalityId: string;
  userId: string;
  reason: string;
  createdAt: number;
}

// Supabase record types
export interface MarketplaceRecord {
  id: string;
  user_id: string;
  name: string;
  description: string;
  tags: string[];
  config: PersonalityConfig;  // JSONB
  thumbnail_url?: string;

  rating: number;
  rating_count: number;
  download_count: number;

  validated: boolean;
  validation_errors?: string[];

  created_at: string;
  updated_at: string;
}

export interface RatingRecord {
  id: string;
  personality_id: string;
  user_id: string;
  rating: number;
  created_at: string;
}

export interface ReportRecord {
  id: string;
  personality_id: string;
  user_id: string;
  reason: string;
  created_at: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: string[];
}

export interface SearchResult {
  listings: MarketplaceListing[];
  total: number;
  page: number;
  limit: number;
  hasMore: boolean;
}
