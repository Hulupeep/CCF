/**
 * Marketplace React Hook
 * Issue #85 - Cloud Personality Marketplace
 */

import { useState, useCallback, useEffect } from 'react';
import { marketplaceService } from '../../services/marketplace';
import {
  MarketplaceListing,
  SearchQuery,
  SearchResult,
  PersonalityMetadata,
} from '../../types/marketplace';
import { PersonalityConfig } from '../../types/personality';

export interface UseMarketplaceReturn {
  // Search state
  searchResults: SearchResult | null;
  searchLoading: boolean;
  searchError: string | null;

  // Trending
  trending: MarketplaceListing[];
  trendingLoading: boolean;

  // User's published
  userPublished: MarketplaceListing[];
  userPublishedLoading: boolean;

  // Actions
  search: (query: SearchQuery) => Promise<void>;
  loadTrending: () => Promise<void>;
  loadUserPublished: () => Promise<void>;
  publish: (config: PersonalityConfig, metadata: PersonalityMetadata) => Promise<MarketplaceListing>;
  unpublish: (id: string) => Promise<void>;
  download: (id: string) => Promise<PersonalityConfig>;
  rate: (id: string, rating: number) => Promise<void>;
  report: (id: string, reason: string) => Promise<void>;
}

export function useMarketplace(): UseMarketplaceReturn {
  const [searchResults, setSearchResults] = useState<SearchResult | null>(null);
  const [searchLoading, setSearchLoading] = useState(false);
  const [searchError, setSearchError] = useState<string | null>(null);

  const [trending, setTrending] = useState<MarketplaceListing[]>([]);
  const [trendingLoading, setTrendingLoading] = useState(false);

  const [userPublished, setUserPublished] = useState<MarketplaceListing[]>([]);
  const [userPublishedLoading, setUserPublishedLoading] = useState(false);

  // Search personalities
  const search = useCallback(async (query: SearchQuery) => {
    setSearchLoading(true);
    setSearchError(null);
    try {
      const results = await marketplaceService.searchPersonalities(query);
      setSearchResults(results);
    } catch (error) {
      setSearchError(error instanceof Error ? error.message : 'Search failed');
    } finally {
      setSearchLoading(false);
    }
  }, []);

  // Load trending
  const loadTrending = useCallback(async () => {
    setTrendingLoading(true);
    try {
      const results = await marketplaceService.getTrendingPersonalities(10);
      setTrending(results);
    } catch (error) {
      console.error('Failed to load trending:', error);
    } finally {
      setTrendingLoading(false);
    }
  }, []);

  // Load user's published personalities
  const loadUserPublished = useCallback(async () => {
    setUserPublishedLoading(true);
    try {
      const results = await marketplaceService.getUserPublishedPersonalities();
      setUserPublished(results);
    } catch (error) {
      console.error('Failed to load user published:', error);
    } finally {
      setUserPublishedLoading(false);
    }
  }, []);

  // Publish personality
  const publish = useCallback(
    async (config: PersonalityConfig, metadata: PersonalityMetadata) => {
      const listing = await marketplaceService.publishPersonality(config, metadata);
      // Reload user published
      await loadUserPublished();
      return listing;
    },
    [loadUserPublished]
  );

  // Unpublish personality
  const unpublish = useCallback(
    async (id: string) => {
      await marketplaceService.unpublishPersonality(id);
      // Remove from local state
      setUserPublished((prev) => prev.filter((p) => p.id !== id));
    },
    []
  );

  // Download personality
  const download = useCallback(async (id: string) => {
    return marketplaceService.downloadPersonality(id);
  }, []);

  // Rate personality
  const rate = useCallback(async (id: string, rating: number) => {
    await marketplaceService.ratePersonality(id, rating);
    // Reload search results to reflect new rating
    if (searchResults) {
      const updated = { ...searchResults };
      const listing = updated.listings.find((l) => l.id === id);
      if (listing) {
        listing.ratingCount += 1;
        // Approximate new average (actual value comes from server)
        listing.rating =
          (listing.rating * (listing.ratingCount - 1) + rating) / listing.ratingCount;
      }
      setSearchResults(updated);
    }
  }, [searchResults]);

  // Report personality
  const report = useCallback(async (id: string, reason: string) => {
    return marketplaceService.reportPersonality(id, reason);
  }, []);

  // Load trending on mount
  useEffect(() => {
    loadTrending();
  }, [loadTrending]);

  return {
    searchResults,
    searchLoading,
    searchError,
    trending,
    trendingLoading,
    userPublished,
    userPublishedLoading,
    search,
    loadTrending,
    loadUserPublished,
    publish,
    unpublish,
    download,
    rate,
    report,
  };
}
