# Cloud Sync Guide

**Feature:** Wave 7
**Service:** `cloudSync.ts`
**Difficulty:** Intermediate
**Time:** 15 minutes

## Overview
Supabase-powered cloud synchronization for personalities, drawings, and game stats across devices.

## Quick Start
\`\`\`typescript
import { cloudSync } from '@/services/cloudSync';

// Initialize
await cloudSync.initialize({
  supabaseUrl: process.env.NEXT_PUBLIC_SUPABASE_URL!,
  supabaseKey: process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!
});

// Sync data
await cloudSync.syncAll();

// Listen for changes
cloudSync.onSyncComplete(() => {
  console.log('Sync complete!');
});
\`\`\`

## Features
- Real-time sync across devices
- Conflict resolution (last-write-wins)
- Offline queue with retry
- Selective sync (choose what to sync)
- Encryption at rest

## Setup
See: [Supabase Setup Guide](../integration/SUPABASE_SETUP.md)

## API
See: [Cloud Sync API](../api/CLOUD_SYNC_API.md)

**Last Updated:** 2026-02-01
