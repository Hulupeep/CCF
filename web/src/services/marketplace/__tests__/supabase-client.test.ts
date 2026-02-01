/**
 * Supabase Marketplace Client Tests
 * Issue #85 - Cloud Personality Marketplace
 */

import { SupabaseMarketplaceClient, setMockUserId } from '../supabase-client';
import { createDefaultConfig } from '../../../types/personality';

// Import mock storage arrays for cleanup
import { mockMarketplace, mockRatings, mockReports, clearMockStorage } from '../supabase-client';

describe('SupabaseMarketplaceClient', () => {
  let client: SupabaseMarketplaceClient;

  beforeEach(() => {
    // Clear mock storage before each test
    clearMockStorage();
    client = new SupabaseMarketplaceClient();
    setMockUserId('test-user-123');
  });

  describe('publishPersonality', () => {
    test('publishes personality with valid data', async () => {
      const config = createDefaultConfig();
      const listing = await client.publishPersonality(
        'Test Personality',
        'A great test personality',
        ['test', 'energetic'],
        config
      );

      expect(listing.id).toBeDefined();
      expect(listing.name).toBe('Test Personality');
      expect(listing.description).toBe('A great test personality');
      expect(listing.tags).toEqual(['test', 'energetic']);
      expect(listing.validated).toBe(true);
      expect(listing.rating).toBe(0);
      expect(listing.downloadCount).toBe(0);
    });

    test('stores validation errors if invalid', async () => {
      const config = createDefaultConfig();
      const listing = await client.publishPersonality(
        'Test',
        'Short',
        [],
        config,
        undefined,
        false,
        ['Description too short', 'No tags']
      );

      expect(listing.validated).toBe(false);
      expect(listing.validationErrors).toContain('Description too short');
    });
  });

  describe('searchPersonalities', () => {
    beforeEach(async () => {
      const config = createDefaultConfig();
      await client.publishPersonality('Energetic Bot', 'Very energetic', ['energetic'], config);
      await client.publishPersonality('Calm Bot', 'Very calm', ['calm'], config);
      await client.publishPersonality('Creative Bot', 'Very creative', ['creative'], config);
    });

    test('returns all validated personalities with no filters', async () => {
      const result = await client.searchPersonalities({});

      expect(result.listings.length).toBe(3);
      expect(result.total).toBe(3);
    });

    test('filters by text search', async () => {
      const result = await client.searchPersonalities({ query: 'energetic' });

      expect(result.listings.length).toBe(1);
      expect(result.listings[0].name).toBe('Energetic Bot');
    });

    test('filters by tags', async () => {
      const result = await client.searchPersonalities({ tags: ['calm'] });

      expect(result.listings.length).toBe(1);
      expect(result.listings[0].name).toBe('Calm Bot');
    });

    test('filters by minimum rating', async () => {
      // Rate one personality
      const allResults = await client.searchPersonalities({});
      const personalityId = allResults.listings[0].id;
      await client.ratePersonality(personalityId, 5);

      const result = await client.searchPersonalities({ minRating: 4 });

      expect(result.listings.length).toBe(1);
      expect(result.listings[0].rating).toBeGreaterThanOrEqual(4);
    });

    test('sorts by downloads', async () => {
      const allResults = await client.searchPersonalities({});
      const personality1 = allResults.listings[0].id;
      const personality2 = allResults.listings[1].id;

      // Download personality2 twice
      await client.downloadPersonality(personality2);
      await client.downloadPersonality(personality2);

      const result = await client.searchPersonalities({ sortBy: 'downloads' });

      expect(result.listings[0].id).toBe(personality2);
      expect(result.listings[0].downloadCount).toBe(2);
    });

    test('paginates results', async () => {
      const page1 = await client.searchPersonalities({ page: 1, limit: 2 });
      const page2 = await client.searchPersonalities({ page: 2, limit: 2 });

      expect(page1.listings.length).toBe(2);
      expect(page2.listings.length).toBe(1);
      expect(page1.hasMore).toBe(true);
      expect(page2.hasMore).toBe(false);
    });

    test('completes search in reasonable time (I-CLOUD-006)', async () => {
      const start = Date.now();
      await client.searchPersonalities({});
      const elapsed = Date.now() - start;

      // Should be very fast for small dataset (< 100ms)
      expect(elapsed).toBeLessThan(100);
    });
  });

  describe('downloadPersonality', () => {
    test('increments download count', async () => {
      const config = createDefaultConfig();
      const listing = await client.publishPersonality('Test', 'Description', ['tag'], config);

      await client.downloadPersonality(listing.id);
      await client.downloadPersonality(listing.id);

      const updated = await client.getPersonality(listing.id);
      expect(updated.downloadCount).toBe(2);
    });

    test('returns personality config', async () => {
      const config = createDefaultConfig();
      const listing = await client.publishPersonality('Test', 'Description', ['tag'], config);

      const downloaded = await client.downloadPersonality(listing.id);

      expect(downloaded).toEqual(config);
    });
  });

  describe('ratePersonality - I-CLOUD-005', () => {
    test('allows user to rate once', async () => {
      const config = createDefaultConfig();
      const listing = await client.publishPersonality('Test', 'Description', ['tag'], config);

      await client.ratePersonality(listing.id, 5);

      const updated = await client.getPersonality(listing.id);
      expect(updated.rating).toBe(5);
      expect(updated.ratingCount).toBe(1);
    });

    test('prevents duplicate ratings from same user', async () => {
      const config = createDefaultConfig();
      const listing = await client.publishPersonality('Test', 'Description', ['tag'], config);

      await client.ratePersonality(listing.id, 5);

      await expect(client.ratePersonality(listing.id, 4)).rejects.toThrow(
        'You have already rated this personality'
      );
    });

    test('calculates average rating correctly', async () => {
      const config = createDefaultConfig();
      const listing = await client.publishPersonality('Test', 'Description', ['tag'], config);

      setMockUserId('user-1');
      await client.ratePersonality(listing.id, 5);

      setMockUserId('user-2');
      await client.ratePersonality(listing.id, 3);

      const updated = await client.getPersonality(listing.id);
      expect(updated.rating).toBe(4); // (5 + 3) / 2
      expect(updated.ratingCount).toBe(2);
    });
  });

  describe('reportPersonality', () => {
    test('stores report', async () => {
      const config = createDefaultConfig();
      const listing = await client.publishPersonality('Test', 'Description', ['tag'], config);

      await expect(
        client.reportPersonality(listing.id, 'This is inappropriate')
      ).resolves.not.toThrow();
    });
  });

  describe('unpublishPersonality', () => {
    test('removes personality from marketplace', async () => {
      const config = createDefaultConfig();
      const listing = await client.publishPersonality('Test', 'Description', ['tag'], config);

      await client.unpublishPersonality(listing.id);

      await expect(client.getPersonality(listing.id)).rejects.toThrow('Personality not found');
    });

    test('only owner can unpublish', async () => {
      const config = createDefaultConfig();
      setMockUserId('user-1');
      const listing = await client.publishPersonality('Test', 'Description', ['tag'], config);

      setMockUserId('user-2');
      await expect(client.unpublishPersonality(listing.id)).rejects.toThrow(
        'Personality not found or not owned by user'
      );
    });
  });

  describe('getUserPublishedPersonalities', () => {
    test('returns only current user personalities', async () => {
      const config = createDefaultConfig();

      setMockUserId('user-1');
      await client.publishPersonality('User1 Bot', 'Description', ['tag'], config);

      setMockUserId('user-2');
      await client.publishPersonality('User2 Bot', 'Description', ['tag'], config);

      const user2Personalities = await client.getUserPublishedPersonalities();

      expect(user2Personalities.length).toBe(1);
      expect(user2Personalities[0].name).toBe('User2 Bot');
    });
  });

  describe('hasUserRated', () => {
    test('returns true if user has rated', async () => {
      const config = createDefaultConfig();
      const listing = await client.publishPersonality('Test', 'Description', ['tag'], config);

      await client.ratePersonality(listing.id, 5);

      const hasRated = await client.hasUserRated(listing.id);
      expect(hasRated).toBe(true);
    });

    test('returns false if user has not rated', async () => {
      const config = createDefaultConfig();
      const listing = await client.publishPersonality('Test', 'Description', ['tag'], config);

      const hasRated = await client.hasUserRated(listing.id);
      expect(hasRated).toBe(false);
    });
  });
});
