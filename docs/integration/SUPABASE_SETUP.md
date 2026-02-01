# Supabase Setup Guide

**Feature:** Cloud Sync
**Time:** 20 minutes
**Difficulty:** Intermediate

## Overview

This guide walks you through setting up Supabase for cloud synchronization of mBot data.

## Prerequisites

- Supabase account (free tier works)
- Node.js 18+ installed
- mBot web companion app running

## Step 1: Create Supabase Project

1. Go to [supabase.com](https://supabase.com)
2. Click "New Project"
3. Enter project details:
   - Name: `mbot-ruvector`
   - Database Password: (generate strong password)
   - Region: Choose closest to your users
4. Click "Create Project"
5. Wait 2-3 minutes for provisioning

## Step 2: Create Database Schema

1. In Supabase dashboard, go to "SQL Editor"
2. Run this SQL script:

```sql
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Personalities table
CREATE TABLE personalities (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  name VARCHAR(255) NOT NULL,
  config JSONB NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  deleted BOOLEAN DEFAULT FALSE,

  CONSTRAINT valid_personality_config CHECK (
    jsonb_typeof(config) = 'object' AND
    (config->>'curiosity')::float BETWEEN 0 AND 1 AND
    (config->>'energy')::float BETWEEN 0 AND 1
  )
);

-- Drawings table
CREATE TABLE drawings (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  session_id VARCHAR(255) NOT NULL,
  strokes JSONB NOT NULL,
  mood VARCHAR(50) CHECK (mood IN ('happy', 'sad', 'angry', 'neutral')),
  thumbnail TEXT, -- Base64 encoded
  duration INT, -- milliseconds
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  deleted BOOLEAN DEFAULT FALSE
);

-- Game statistics table
CREATE TABLE game_stats (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  game_type VARCHAR(50) NOT NULL CHECK (game_type IN ('tictactoe', 'chase', 'simon')),
  result VARCHAR(20) NOT NULL CHECK (result IN ('win', 'loss', 'draw')),
  score INT NOT NULL,
  duration INT, -- milliseconds
  personality_config JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Inventory table
CREATE TABLE inventory (
  user_id UUID PRIMARY KEY REFERENCES auth.users(id) ON DELETE CASCADE,
  red INT DEFAULT 0 CHECK (red >= 0),
  green INT DEFAULT 0 CHECK (green >= 0),
  blue INT DEFAULT 0 CHECK (blue >= 0),
  yellow INT DEFAULT 0 CHECK (yellow >= 0),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Sync metadata table
CREATE TABLE sync_metadata (
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  device_id VARCHAR(255) NOT NULL,
  last_sync TIMESTAMPTZ DEFAULT NOW(),
  sync_version INT DEFAULT 1,
  PRIMARY KEY (user_id, device_id)
);

-- Indexes for performance
CREATE INDEX idx_personalities_user ON personalities(user_id);
CREATE INDEX idx_drawings_user ON drawings(user_id);
CREATE INDEX idx_drawings_session ON drawings(session_id);
CREATE INDEX idx_game_stats_user ON game_stats(user_id);
CREATE INDEX idx_game_stats_type ON game_stats(game_type);

-- Updated_at trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_personalities_updated_at
  BEFORE UPDATE ON personalities
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_drawings_updated_at
  BEFORE UPDATE ON drawings
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_inventory_updated_at
  BEFORE UPDATE ON inventory
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at_column();
```

## Step 3: Configure Row Level Security (RLS)

```sql
-- Enable RLS on all tables
ALTER TABLE personalities ENABLE ROW LEVEL SECURITY;
ALTER TABLE drawings ENABLE ROW LEVEL SECURITY;
ALTER TABLE game_stats ENABLE ROW LEVEL SECURITY;
ALTER TABLE inventory ENABLE ROW LEVEL SECURITY;
ALTER TABLE sync_metadata ENABLE ROW LEVEL SECURITY;

-- Personalities policies
CREATE POLICY "Users can view own personalities"
  ON personalities FOR SELECT
  USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own personalities"
  ON personalities FOR INSERT
  WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update own personalities"
  ON personalities FOR UPDATE
  USING (auth.uid() = user_id);

CREATE POLICY "Users can delete own personalities"
  ON personalities FOR DELETE
  USING (auth.uid() = user_id);

-- Drawings policies
CREATE POLICY "Users can view own drawings"
  ON drawings FOR SELECT
  USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own drawings"
  ON drawings FOR INSERT
  WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update own drawings"
  ON drawings FOR UPDATE
  USING (auth.uid() = user_id);

CREATE POLICY "Users can delete own drawings"
  ON drawings FOR DELETE
  USING (auth.uid() = user_id);

-- Game stats policies (similar pattern)
CREATE POLICY "Users can view own game stats"
  ON game_stats FOR SELECT
  USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own game stats"
  ON game_stats FOR INSERT
  WITH CHECK (auth.uid() = user_id);

-- Inventory policies
CREATE POLICY "Users can view own inventory"
  ON inventory FOR SELECT
  USING (auth.uid() = user_id);

CREATE POLICY "Users can manage own inventory"
  ON inventory FOR ALL
  USING (auth.uid() = user_id);

-- Sync metadata policies
CREATE POLICY "Users can manage own sync metadata"
  ON sync_metadata FOR ALL
  USING (auth.uid() = user_id);
```

## Step 4: Enable Realtime

1. Go to "Database" → "Replication"
2. Enable replication for these tables:
   - `personalities`
   - `drawings`
   - `game_stats`
   - `inventory`
3. Click "Save"

## Step 5: Get API Credentials

1. Go to "Settings" → "API"
2. Copy these values:
   - Project URL: `https://[project-ref].supabase.co`
   - Anon/Public Key: `eyJhbG...` (starts with eyJ)
   - Service Role Key: `eyJhbG...` (different from anon key)

3. Add to `.env.local`:

```bash
NEXT_PUBLIC_SUPABASE_URL=https://[project-ref].supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=eyJhbG...
SUPABASE_SERVICE_ROLE_KEY=eyJhbG... # Server-side only, never expose to client
```

## Step 6: Configure Authentication

### Option A: Email/Password (Simplest)

1. Go to "Authentication" → "Providers"
2. Enable "Email" provider
3. Configure email templates (optional)

### Option B: OAuth (Google, GitHub, etc.)

1. Go to "Authentication" → "Providers"
2. Enable desired provider (e.g., Google)
3. Enter OAuth credentials from provider
4. Configure redirect URLs

### Option C: Anonymous (For Testing)

```typescript
// Allow anonymous users
const { data, error } = await supabase.auth.signInAnonymously();
```

## Step 7: Test Connection

Create a test file:

```typescript
// test-supabase.ts
import { createClient } from '@supabase/supabase-js';

const supabase = createClient(
  process.env.NEXT_PUBLIC_SUPABASE_URL!,
  process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!
);

async function testConnection() {
  // Test authentication
  const { data: authData, error: authError } = await supabase.auth.signInWithPassword({
    email: 'test@example.com',
    password: 'test123'
  });

  if (authError) {
    console.error('Auth error:', authError);
    return;
  }

  console.log('✅ Authenticated:', authData.user.id);

  // Test database query
  const { data, error } = await supabase
    .from('personalities')
    .select('*')
    .limit(1);

  if (error) {
    console.error('Query error:', error);
    return;
  }

  console.log('✅ Database connected');
  console.log('Personalities:', data);

  // Test realtime
  const channel = supabase
    .channel('test-channel')
    .on('postgres_changes', {
      event: '*',
      schema: 'public',
      table: 'personalities'
    }, (payload) => {
      console.log('✅ Realtime working:', payload);
    })
    .subscribe();

  console.log('✅ Setup complete!');
}

testConnection();
```

Run: `npx tsx test-supabase.ts`

## Step 8: Initialize Cloud Sync

```typescript
// In your app
import { cloudSync } from '@/services/cloudSync';

await cloudSync.initialize({
  supabaseUrl: process.env.NEXT_PUBLIC_SUPABASE_URL!,
  supabaseAnonKey: process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
  userId: session.user.id,
  deviceId: getDeviceId()
});

// Test sync
await cloudSync.syncAll();
console.log('✅ Cloud sync working!');
```

## Troubleshooting

### Connection Fails

```bash
# Check environment variables
echo $NEXT_PUBLIC_SUPABASE_URL
echo $NEXT_PUBLIC_SUPABASE_ANON_KEY

# Verify project is active in Supabase dashboard
```

### RLS Blocks Queries

```sql
-- Temporarily disable RLS for debugging (NOT for production!)
ALTER TABLE personalities DISABLE ROW LEVEL SECURITY;

-- Check policies
SELECT * FROM pg_policies WHERE tablename = 'personalities';
```

### Realtime Not Working

1. Verify replication is enabled for the table
2. Check that you're subscribed to the correct schema and table
3. Test with `supabase.channel('test').subscribe()`

---

## Next Steps

- [Cloud Sync Guide](../guides/cloud-sync-guide.md)
- [Cloud Sync API](../api/CLOUD_SYNC_API.md)
- [Troubleshooting](../troubleshooting/CLOUD_SYNC_DEBUG.md)

---

**Last Updated:** 2026-02-01
**Status:** Production Ready ✅
