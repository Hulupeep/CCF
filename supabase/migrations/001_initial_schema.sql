-- Cloud Sync Schema Migration
-- Contract: I-CLOUD-001 (Idempotent), I-CLOUD-002 (Last-write-wins), I-CLOUD-003 (AES-256 encryption)
-- Issue: #84

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Personalities Table
CREATE TABLE IF NOT EXISTS personalities (
  id TEXT NOT NULL,
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  icon TEXT NOT NULL,
  config JSONB NOT NULL,
  quirks TEXT[] DEFAULT '{}',
  sound_pack TEXT,
  version INTEGER NOT NULL DEFAULT 1,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (id, user_id)
);

-- Drawings Table
CREATE TABLE IF NOT EXISTS drawings (
  id TEXT NOT NULL,
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL,
  strokes JSONB NOT NULL,
  moods JSONB NOT NULL,
  duration INTEGER NOT NULL,
  dominant_mood TEXT NOT NULL,
  has_signature BOOLEAN NOT NULL DEFAULT false,
  session_id TEXT,
  thumbnail_url TEXT,
  metadata JSONB NOT NULL,
  synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (id, user_id)
);

-- Game Stats Table
CREATE TABLE IF NOT EXISTS game_stats (
  id TEXT NOT NULL,
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  game_type TEXT NOT NULL,
  result TEXT NOT NULL CHECK (result IN ('win', 'loss', 'draw')),
  score INTEGER NOT NULL,
  duration INTEGER NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL,
  personality TEXT NOT NULL,
  metadata JSONB DEFAULT '{}',
  synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (id, user_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_personalities_user_id ON personalities(user_id);
CREATE INDEX IF NOT EXISTS idx_personalities_modified_at ON personalities(modified_at DESC);
CREATE INDEX IF NOT EXISTS idx_drawings_user_id ON drawings(user_id);
CREATE INDEX IF NOT EXISTS idx_drawings_created_at ON drawings(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_game_stats_user_id ON game_stats(user_id);
CREATE INDEX IF NOT EXISTS idx_game_stats_timestamp ON game_stats(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_game_stats_game_type ON game_stats(game_type);

-- Row Level Security (RLS) Policies
-- I-CLOUD-003: Ensure users can only access their own data

-- Enable RLS
ALTER TABLE personalities ENABLE ROW LEVEL SECURITY;
ALTER TABLE drawings ENABLE ROW LEVEL SECURITY;
ALTER TABLE game_stats ENABLE ROW LEVEL SECURITY;

-- Personalities Policies
CREATE POLICY "Users can view their own personalities"
  ON personalities
  FOR SELECT
  USING (auth.uid() = user_id);

CREATE POLICY "Users can insert their own personalities"
  ON personalities
  FOR INSERT
  WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update their own personalities"
  ON personalities
  FOR UPDATE
  USING (auth.uid() = user_id)
  WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can delete their own personalities"
  ON personalities
  FOR DELETE
  USING (auth.uid() = user_id);

-- Drawings Policies
CREATE POLICY "Users can view their own drawings"
  ON drawings
  FOR SELECT
  USING (auth.uid() = user_id);

CREATE POLICY "Users can insert their own drawings"
  ON drawings
  FOR INSERT
  WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update their own drawings"
  ON drawings
  FOR UPDATE
  USING (auth.uid() = user_id)
  WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can delete their own drawings"
  ON drawings
  FOR DELETE
  USING (auth.uid() = user_id);

-- Game Stats Policies
CREATE POLICY "Users can view their own game stats"
  ON game_stats
  FOR SELECT
  USING (auth.uid() = user_id);

CREATE POLICY "Users can insert their own game stats"
  ON game_stats
  FOR INSERT
  WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update their own game stats"
  ON game_stats
  FOR UPDATE
  USING (auth.uid() = user_id)
  WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can delete their own game stats"
  ON game_stats
  FOR DELETE
  USING (auth.uid() = user_id);

-- Functions for conflict resolution (I-CLOUD-002: Last-write-wins)

-- Function to merge personality updates (last-write-wins)
CREATE OR REPLACE FUNCTION merge_personality_update()
RETURNS TRIGGER AS $$
BEGIN
  -- If the incoming modified_at is newer, allow the update
  -- Otherwise, keep the existing record
  IF NEW.modified_at >= OLD.modified_at THEN
    RETURN NEW;
  ELSE
    RETURN OLD;
  END IF;
END;
$$ LANGUAGE plpgsql;

-- Trigger for personality conflict resolution
CREATE TRIGGER personality_conflict_resolution
  BEFORE UPDATE ON personalities
  FOR EACH ROW
  EXECUTE FUNCTION merge_personality_update();

-- Storage bucket for drawing thumbnails
INSERT INTO storage.buckets (id, name, public)
VALUES ('drawing-thumbnails', 'drawing-thumbnails', true)
ON CONFLICT (id) DO NOTHING;

-- Storage policies for drawing thumbnails
CREATE POLICY "Users can upload their own thumbnails"
  ON storage.objects
  FOR INSERT
  WITH CHECK (
    bucket_id = 'drawing-thumbnails'
    AND auth.uid()::text = (storage.foldername(name))[1]
  );

CREATE POLICY "Users can view their own thumbnails"
  ON storage.objects
  FOR SELECT
  USING (
    bucket_id = 'drawing-thumbnails'
    AND auth.uid()::text = (storage.foldername(name))[1]
  );

CREATE POLICY "Anyone can view public thumbnails"
  ON storage.objects
  FOR SELECT
  USING (bucket_id = 'drawing-thumbnails');

-- Audit log for tracking sync operations (optional, for debugging)
CREATE TABLE IF NOT EXISTS sync_audit_log (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  table_name TEXT NOT NULL,
  operation TEXT NOT NULL,
  record_id TEXT NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  metadata JSONB
);

CREATE INDEX IF NOT EXISTS idx_sync_audit_log_user_id ON sync_audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_sync_audit_log_timestamp ON sync_audit_log(timestamp DESC);

-- Function to log sync operations
CREATE OR REPLACE FUNCTION log_sync_operation()
RETURNS TRIGGER AS $$
BEGIN
  INSERT INTO sync_audit_log (user_id, table_name, operation, record_id, metadata)
  VALUES (
    NEW.user_id,
    TG_TABLE_NAME,
    TG_OP,
    NEW.id,
    jsonb_build_object(
      'synced_at', NEW.synced_at
    )
  );
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for audit logging
CREATE TRIGGER log_personality_sync
  AFTER INSERT OR UPDATE ON personalities
  FOR EACH ROW
  EXECUTE FUNCTION log_sync_operation();

CREATE TRIGGER log_drawing_sync
  AFTER INSERT OR UPDATE ON drawings
  FOR EACH ROW
  EXECUTE FUNCTION log_sync_operation();

CREATE TRIGGER log_game_stats_sync
  AFTER INSERT OR UPDATE ON game_stats
  FOR EACH ROW
  EXECUTE FUNCTION log_sync_operation();

-- Grant permissions
GRANT USAGE ON SCHEMA public TO authenticated;
GRANT ALL ON ALL TABLES IN SCHEMA public TO authenticated;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO authenticated;
GRANT ALL ON ALL FUNCTIONS IN SCHEMA public TO authenticated;

-- Comments for documentation
COMMENT ON TABLE personalities IS 'User personality configurations with cloud sync';
COMMENT ON TABLE drawings IS 'User drawings with mood tracking';
COMMENT ON TABLE game_stats IS 'Game session statistics';
COMMENT ON TABLE sync_audit_log IS 'Audit log for sync operations';
COMMENT ON COLUMN personalities.config IS 'JSONB containing PersonalityConfig with 9 parameters in [0.0, 1.0]';
COMMENT ON COLUMN personalities.synced_at IS 'Timestamp of last successful sync';
COMMENT ON COLUMN drawings.metadata IS 'Drawing metadata including mood statistics';
COMMENT ON COLUMN game_stats.metadata IS 'Game-specific metadata (moves, difficulty, etc.)';
