# Supabase Setup for CCF on RuVector

## Overview

This directory contains database migrations for the CCF on RuVector cloud features, including the Cloud Personality Marketplace (Issue #85).

## Prerequisites

1. Install Supabase CLI:
```bash
npm install -g supabase
```

2. Create a Supabase account at https://supabase.com

## Initial Setup

### 1. Create Supabase Project

```bash
# Initialize Supabase in your project
supabase init

# Link to your Supabase project
supabase link --project-ref <your-project-ref>
```

### 2. Run Migrations

```bash
# Apply all migrations
supabase db push

# Or run specific migration
psql <your-database-url> -f migrations/001_marketplace_schema.sql
```

### 3. Environment Variables

Create `.env.local` in your project root:

```bash
# Supabase Configuration
VITE_SUPABASE_URL=https://<your-project-ref>.supabase.co
VITE_SUPABASE_ANON_KEY=<your-anon-key>
VITE_SUPABASE_SERVICE_KEY=<your-service-key> # Server-side only, never expose!
```

## Migrations

### 001_marketplace_schema.sql

Creates the Cloud Personality Marketplace database schema:

**Tables:**
- `marketplace_personalities` - Published personalities with validation
- `personality_ratings` - User ratings (1-5 stars, 1 per user)
- `personality_reports` - User reports for inappropriate content

**Indexes:**
- Full-text search on name and description (I-CLOUD-006: <500ms for 10k)
- GIN indexes on tags array
- Performance indexes for sorting

**Triggers:**
- Auto-update `updated_at` timestamp
- Auto-recalculate average rating

**RLS Policies:**
- Public read for validated personalities
- Authenticated users can publish
- Owner-only update/delete
- Admin-only access to reports

**Functions:**
- `search_marketplace_personalities()` - Optimized search
- `get_trending_personalities()` - Popularity algorithm
- `increment_download_count()` - Track downloads

## Contracts

All database design follows contracts defined in:
- `docs/contracts/feature_cloud_marketplace.yml`

**Key Invariants:**
- I-CLOUD-004: Validation before publication
- I-CLOUD-005: One rating per user (UNIQUE constraint)
- I-CLOUD-006: Search results <500ms for 10k personalities
- ARCH-CLOUD-001: Offline-first design
- SAFE-CLOUD-001: Content moderation

## Replace Mock Client with Real Supabase

### 1. Install Supabase Client

```bash
cd web
npm install @supabase/supabase-js
```

### 2. Update `web/src/services/marketplace/supabase-client.ts`

Replace mock implementation with real Supabase:

```typescript
import { createClient, SupabaseClient } from '@supabase/supabase-js';

const supabase = createClient(
  import.meta.env.VITE_SUPABASE_URL,
  import.meta.env.VITE_SUPABASE_ANON_KEY
);

export class SupabaseMarketplaceClient {
  async publishPersonality(...) {
    const { data, error } = await supabase
      .from('marketplace_personalities')
      .insert({ ... })
      .select()
      .single();

    if (error) throw error;
    return recordToListing(data);
  }

  async searchPersonalities(query: SearchQuery) {
    const { data, error } = await supabase
      .rpc('search_marketplace_personalities', {
        search_query: query.query,
        filter_tags: query.tags,
        min_rating: query.minRating || 0,
        sort_by: query.sortBy || 'newest',
        page_number: query.page || 1,
        page_size: query.limit || 20,
      });

    if (error) throw error;
    return { ... };
  }

  // ... implement other methods
}
```

### 3. Update Authentication

Replace `getCurrentUserId()` with real auth:

```typescript
export function getCurrentUserId(): string {
  const { data: { user } } = await supabase.auth.getUser();
  if (!user) throw new Error('Not authenticated');
  return user.id;
}
```

## Testing

### Verify Schema

```bash
# Check tables exist
supabase db list

# Verify RLS policies
psql <your-database-url> -c "\d+ marketplace_personalities"

# Test full-text search
psql <your-database-url> -c "SELECT * FROM search_marketplace_personalities('energetic', NULL, 0, 'newest', 1, 10);"
```

### Performance Testing (I-CLOUD-006)

Load 10,000 test personalities and verify search speed:

```sql
-- Insert 10k test records
INSERT INTO marketplace_personalities (user_id, name, description, tags, config, validated)
SELECT
  gen_random_uuid(),
  'Test Bot ' || generate_series,
  'Description for test bot ' || generate_series,
  ARRAY['test', 'automated'],
  '{"tension_baseline": 0.5, "coherence_baseline": 0.5, "energy_baseline": 0.5, "startle_sensitivity": 0.5, "recovery_speed": 0.5, "curiosity_drive": 0.5, "movement_expressiveness": 0.5, "sound_expressiveness": 0.5, "light_expressiveness": 0.5}'::JSONB,
  TRUE
FROM generate_series(1, 10000);

-- Test search performance (MUST be <500ms)
EXPLAIN ANALYZE
SELECT * FROM search_marketplace_personalities('test', NULL, 0, 'newest', 1, 20);
```

Expected: `Execution Time: < 500ms`

## Troubleshooting

### Migration Failed

```bash
# Rollback
supabase db reset

# Re-apply
supabase db push
```

### RLS Policy Errors

Check user authentication:
```sql
SELECT auth.uid(); -- Should return your user ID
```

### Slow Search

Verify indexes:
```sql
SELECT * FROM pg_indexes WHERE tablename = 'marketplace_personalities';
```

All GIN indexes should be present for full-text search.

## Production Checklist

Before deploying to production:

- [ ] All migrations applied successfully
- [ ] RLS policies tested (authenticated and anonymous users)
- [ ] Full-text search verified (<500ms with 10k records)
- [ ] Rate limiting configured (Supabase dashboard)
- [ ] Backup strategy in place
- [ ] Monitoring alerts configured
- [ ] Environment variables secured
- [ ] Service key never exposed to client
- [ ] CORS configured for your domain

## Support

- Supabase Docs: https://supabase.com/docs
- Issue Tracker: https://github.com/Hulupeep/CCF/issues
- Contract Reference: `docs/contracts/feature_cloud_marketplace.yml`
