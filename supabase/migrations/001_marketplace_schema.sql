-- Marketplace Personalities Schema
-- Issue #85 - Cloud Personality Marketplace
--
-- Implements:
-- I-CLOUD-004: Marketplace personalities must pass validation before publication
-- I-CLOUD-005: Rating system prevents abuse (1 rating per user per personality)
-- I-CLOUD-006: Search results must return within 500ms for up to 10,000 personalities

-- Enable UUID extension if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================
-- Marketplace Personalities Table
-- ============================================

CREATE TABLE marketplace_personalities (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  name TEXT NOT NULL CHECK (char_length(name) >= 3 AND char_length(name) <= 50),
  description TEXT NOT NULL CHECK (char_length(description) >= 10 AND char_length(description) <= 500),
  tags TEXT[] NOT NULL DEFAULT '{}' CHECK (array_length(tags, 1) <= 10),
  config JSONB NOT NULL,
  thumbnail_url TEXT,

  -- Metrics
  rating NUMERIC(3,2) DEFAULT 0 CHECK (rating >= 0 AND rating <= 5),
  rating_count INTEGER DEFAULT 0 CHECK (rating_count >= 0),
  download_count INTEGER DEFAULT 0 CHECK (download_count >= 0),

  -- Validation (I-CLOUD-004)
  validated BOOLEAN DEFAULT FALSE,
  validation_errors TEXT[],

  -- Timestamps
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for fast search (I-CLOUD-006: <500ms for 10k personalities)
CREATE INDEX idx_marketplace_validated ON marketplace_personalities(validated) WHERE validated = TRUE;
CREATE INDEX idx_marketplace_name_tsvector ON marketplace_personalities USING GIN (to_tsvector('english', name));
CREATE INDEX idx_marketplace_description_tsvector ON marketplace_personalities USING GIN (to_tsvector('english', description));
CREATE INDEX idx_marketplace_tags ON marketplace_personalities USING GIN (tags);
CREATE INDEX idx_marketplace_rating ON marketplace_personalities(rating DESC);
CREATE INDEX idx_marketplace_downloads ON marketplace_personalities(download_count DESC);
CREATE INDEX idx_marketplace_created ON marketplace_personalities(created_at DESC);
CREATE INDEX idx_marketplace_user ON marketplace_personalities(user_id);

-- Composite index for popular sort (downloads + rating)
CREATE INDEX idx_marketplace_popularity ON marketplace_personalities(download_count DESC, rating DESC);

-- ============================================
-- Personality Ratings Table
-- ============================================

CREATE TABLE personality_ratings (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  personality_id UUID NOT NULL REFERENCES marketplace_personalities(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
  created_at TIMESTAMPTZ DEFAULT NOW(),

  -- I-CLOUD-005: One rating per user per personality
  UNIQUE(personality_id, user_id)
);

CREATE INDEX idx_ratings_personality ON personality_ratings(personality_id);
CREATE INDEX idx_ratings_user ON personality_ratings(user_id);

-- ============================================
-- Personality Reports Table
-- ============================================

CREATE TABLE personality_reports (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  personality_id UUID NOT NULL REFERENCES marketplace_personalities(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  reason TEXT NOT NULL CHECK (char_length(reason) >= 10 AND char_length(reason) <= 500),
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_reports_personality ON personality_reports(personality_id);
CREATE INDEX idx_reports_created ON personality_reports(created_at DESC);

-- ============================================
-- Triggers
-- ============================================

-- Update updated_at timestamp on personality update
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_marketplace_personalities_updated_at
BEFORE UPDATE ON marketplace_personalities
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Update personality rating average when new rating is added
CREATE OR REPLACE FUNCTION update_personality_rating()
RETURNS TRIGGER AS $$
BEGIN
  UPDATE marketplace_personalities
  SET
    rating = (
      SELECT AVG(rating)::NUMERIC(3,2)
      FROM personality_ratings
      WHERE personality_id = NEW.personality_id
    ),
    rating_count = (
      SELECT COUNT(*)
      FROM personality_ratings
      WHERE personality_id = NEW.personality_id
    )
  WHERE id = NEW.personality_id;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER rating_changed
AFTER INSERT OR UPDATE ON personality_ratings
FOR EACH ROW EXECUTE FUNCTION update_personality_rating();

-- ============================================
-- Row-Level Security (RLS) Policies
-- ============================================

ALTER TABLE marketplace_personalities ENABLE ROW LEVEL SECURITY;
ALTER TABLE personality_ratings ENABLE ROW LEVEL SECURITY;
ALTER TABLE personality_reports ENABLE ROW LEVEL SECURITY;

-- Marketplace Personalities Policies

-- Anyone can read validated personalities (public marketplace)
CREATE POLICY marketplace_select_public ON marketplace_personalities
  FOR SELECT
  USING (validated = TRUE);

-- Users can see their own personalities (even if not validated)
CREATE POLICY marketplace_select_own ON marketplace_personalities
  FOR SELECT
  USING (user_id = auth.uid());

-- Only authenticated users can insert
CREATE POLICY marketplace_insert ON marketplace_personalities
  FOR INSERT
  WITH CHECK (auth.uid() IS NOT NULL AND user_id = auth.uid());

-- Only owner can update their own personalities
CREATE POLICY marketplace_update ON marketplace_personalities
  FOR UPDATE
  USING (user_id = auth.uid())
  WITH CHECK (user_id = auth.uid());

-- Only owner can delete their own personalities
CREATE POLICY marketplace_delete ON marketplace_personalities
  FOR DELETE
  USING (user_id = auth.uid());

-- Ratings Policies

-- Anyone authenticated can insert ratings (trigger enforces uniqueness)
CREATE POLICY ratings_insert ON personality_ratings
  FOR INSERT
  WITH CHECK (auth.uid() IS NOT NULL AND user_id = auth.uid());

-- Anyone can read ratings
CREATE POLICY ratings_select ON personality_ratings
  FOR SELECT
  USING (TRUE);

-- Reports Policies

-- Anyone authenticated can insert reports
CREATE POLICY reports_insert ON personality_reports
  FOR INSERT
  WITH CHECK (auth.uid() IS NOT NULL AND user_id = auth.uid());

-- Only admins can read reports (modify auth.jwt() check as needed)
CREATE POLICY reports_select ON personality_reports
  FOR SELECT
  USING (auth.jwt() ->> 'role' = 'admin' OR user_id = auth.uid());

-- ============================================
-- Utility Functions
-- ============================================

-- Search personalities by text and filters
CREATE OR REPLACE FUNCTION search_marketplace_personalities(
  search_query TEXT DEFAULT NULL,
  filter_tags TEXT[] DEFAULT NULL,
  min_rating NUMERIC DEFAULT 0,
  sort_by TEXT DEFAULT 'newest',
  page_number INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 20
)
RETURNS TABLE (
  id UUID,
  user_id UUID,
  name TEXT,
  description TEXT,
  tags TEXT[],
  config JSONB,
  thumbnail_url TEXT,
  rating NUMERIC,
  rating_count INTEGER,
  download_count INTEGER,
  validated BOOLEAN,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  total_count BIGINT
) AS $$
BEGIN
  RETURN QUERY
  WITH filtered AS (
    SELECT
      mp.*,
      COUNT(*) OVER() AS total_count
    FROM marketplace_personalities mp
    WHERE
      mp.validated = TRUE
      AND (search_query IS NULL OR
           to_tsvector('english', mp.name || ' ' || mp.description) @@ plainto_tsquery('english', search_query))
      AND (filter_tags IS NULL OR mp.tags && filter_tags)
      AND mp.rating >= min_rating
    ORDER BY
      CASE
        WHEN sort_by = 'popular' THEN mp.download_count
        WHEN sort_by = 'rating' THEN mp.rating * mp.rating_count
        WHEN sort_by = 'downloads' THEN mp.download_count
        ELSE 0
      END DESC,
      CASE WHEN sort_by = 'newest' THEN mp.created_at END DESC
    LIMIT page_size
    OFFSET (page_number - 1) * page_size
  )
  SELECT
    f.id,
    f.user_id,
    f.name,
    f.description,
    f.tags,
    f.config,
    f.thumbnail_url,
    f.rating,
    f.rating_count,
    f.download_count,
    f.validated,
    f.created_at,
    f.updated_at,
    f.total_count
  FROM filtered f;
END;
$$ LANGUAGE plpgsql STABLE;

-- Increment download count
CREATE OR REPLACE FUNCTION increment_download_count(personality_id UUID)
RETURNS VOID AS $$
BEGIN
  UPDATE marketplace_personalities
  SET download_count = download_count + 1
  WHERE id = personality_id;
END;
$$ LANGUAGE plpgsql;

-- Get trending personalities (combination of downloads and rating)
CREATE OR REPLACE FUNCTION get_trending_personalities(limit_count INTEGER DEFAULT 10)
RETURNS TABLE (
  id UUID,
  user_id UUID,
  name TEXT,
  description TEXT,
  tags TEXT[],
  config JSONB,
  thumbnail_url TEXT,
  rating NUMERIC,
  rating_count INTEGER,
  download_count INTEGER,
  validated BOOLEAN,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ
) AS $$
BEGIN
  RETURN QUERY
  SELECT
    mp.id,
    mp.user_id,
    mp.name,
    mp.description,
    mp.tags,
    mp.config,
    mp.thumbnail_url,
    mp.rating,
    mp.rating_count,
    mp.download_count,
    mp.validated,
    mp.created_at,
    mp.updated_at
  FROM marketplace_personalities mp
  WHERE mp.validated = TRUE
  ORDER BY
    (mp.download_count * 0.7 + mp.rating * mp.rating_count * 0.3) DESC,
    mp.created_at DESC
  LIMIT limit_count;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================
-- Sample Data (for testing)
-- ============================================

-- Insert sample personalities (only if auth.users exists and has sample users)
-- Uncomment after setting up authentication

-- INSERT INTO marketplace_personalities (user_id, name, description, tags, config, validated)
-- VALUES
--   (
--     '00000000-0000-0000-0000-000000000001'::UUID,
--     'Energetic Explorer',
--     'A highly energetic personality perfect for playful interactions and exploration.',
--     ARRAY['energetic', 'playful', 'curious'],
--     '{"tension_baseline": 0.3, "coherence_baseline": 0.7, "energy_baseline": 0.9, "startle_sensitivity": 0.6, "recovery_speed": 0.8, "curiosity_drive": 0.9, "movement_expressiveness": 0.8, "sound_expressiveness": 0.7, "light_expressiveness": 0.6}'::JSONB,
--     TRUE
--   );

-- ============================================
-- Comments
-- ============================================

COMMENT ON TABLE marketplace_personalities IS 'Marketplace for sharing custom personality configurations';
COMMENT ON TABLE personality_ratings IS 'User ratings for marketplace personalities (I-CLOUD-005: 1 per user)';
COMMENT ON TABLE personality_reports IS 'User reports for inappropriate content';
COMMENT ON COLUMN marketplace_personalities.validated IS 'I-CLOUD-004: Must be TRUE to appear in public search';
COMMENT ON INDEX idx_marketplace_name_tsvector IS 'I-CLOUD-006: Full-text search on name for <500ms';
COMMENT ON INDEX idx_marketplace_description_tsvector IS 'I-CLOUD-006: Full-text search on description for <500ms';
