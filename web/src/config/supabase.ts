/**
 * Supabase Configuration
 * Contract: I-CLOUD-001, I-CLOUD-002, I-CLOUD-003
 *
 * Environment variables required:
 * - VITE_SUPABASE_URL: Your Supabase project URL
 * - VITE_SUPABASE_ANON_KEY: Your Supabase anonymous key
 */

import { createClient } from '@supabase/supabase-js';

const supabaseUrl = import.meta.env.VITE_SUPABASE_URL || '';
const supabaseAnonKey = import.meta.env.VITE_SUPABASE_ANON_KEY || '';

if (!supabaseUrl || !supabaseAnonKey) {
  console.warn('Supabase credentials not configured. Cloud sync will be disabled.');
}

/**
 * Supabase client instance
 * Used for authentication and database operations
 */
export const supabase = createClient(supabaseUrl, supabaseAnonKey, {
  auth: {
    persistSession: true,
    autoRefreshToken: true,
    detectSessionInUrl: true,
  },
  db: {
    schema: 'public',
  },
  global: {
    headers: {
      'X-Client-Info': 'ccf-web-dashboard',
    },
  },
});

/**
 * Checks if Supabase is properly configured
 */
export function isSupabaseConfigured(): boolean {
  return Boolean(supabaseUrl && supabaseAnonKey);
}

/**
 * Database table names
 */
export const TABLES = {
  PERSONALITIES: 'personalities',
  DRAWINGS: 'drawings',
  GAME_STATS: 'game_stats',
} as const;

/**
 * Storage bucket names
 */
export const BUCKETS = {
  DRAWING_THUMBNAILS: 'drawing-thumbnails',
} as const;
