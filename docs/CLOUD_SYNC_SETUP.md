# Cloud Sync Setup Guide

This guide walks you through setting up Supabase for cloud synchronization in mBot RuVector.

## Prerequisites

- A Supabase account (free tier works fine)
- Node.js and npm installed
- mBot RuVector web app cloned

## Step 1: Create Supabase Project

1. Go to [https://app.supabase.com](https://app.supabase.com)
2. Click "New Project"
3. Choose your organization
4. Enter project details:
   - Name: `mbot-ruvector` (or your preferred name)
   - Database Password: Generate a strong password
   - Region: Choose closest to your users
5. Wait for project to be created (~2 minutes)

## Step 2: Configure Environment Variables

1. Copy the example environment file:
   ```bash
   cd web
   cp .env.example .env
   ```

2. Get your Supabase credentials:
   - In Supabase dashboard, go to Settings > API
   - Copy the "Project URL"
   - Copy the "anon public" key

3. Update your `.env` file:
   ```bash
   VITE_SUPABASE_URL=https://your-project-id.supabase.co
   VITE_SUPABASE_ANON_KEY=your-anon-key-here
   ```

## Step 3: Run Database Migration

1. Install Supabase CLI:
   ```bash
   npm install -g supabase
   ```

2. Login to Supabase:
   ```bash
   supabase login
   ```

3. Link your project:
   ```bash
   supabase link --project-ref your-project-id
   ```

4. Run the migration:
   ```bash
   supabase db push
   ```

   Or manually run the SQL:
   - Go to Supabase dashboard > SQL Editor
   - Open `supabase/migrations/001_initial_schema.sql`
   - Copy and paste the SQL
   - Click "Run"

## Step 4: Configure Authentication

1. In Supabase dashboard, go to Authentication > Providers
2. Enable "Email" provider:
   - Toggle "Enable Email provider" ON
   - Configure email templates (optional)
   - Set "Confirm email" to your preference

3. (Optional) Enable additional providers:
   - Google OAuth
   - GitHub OAuth
   - Discord OAuth

## Step 5: Install Dependencies

```bash
cd web
npm install @supabase/supabase-js
```

## Step 6: Test the Integration

1. Start the development server:
   ```bash
   npm run dev
   ```

2. Open the app in your browser
3. Look for the Cloud Sync panel
4. Sign up with a test account
5. Create a personality and verify it syncs:
   - Check Supabase dashboard > Table Editor > personalities
   - You should see your data appear

## Verification Checklist

- [ ] Supabase project created
- [ ] Environment variables configured
- [ ] Database migration applied successfully
- [ ] Email authentication enabled
- [ ] Dependencies installed
- [ ] Test account can sign up
- [ ] Personalities sync to cloud
- [ ] Drawings sync to cloud
- [ ] Game stats sync to cloud
- [ ] Offline queue works (test by going offline)

## Architecture

### Database Schema

The migration creates three main tables:

1. **personalities** - Stores user personality configurations
2. **drawings** - Stores artwork with mood tracking
3. **game_stats** - Stores game session statistics
4. **sync_audit_log** - Tracks sync operations for debugging

### Security (I-CLOUD-003)

- Row Level Security (RLS) enabled on all tables
- Users can only access their own data
- AES-256 encryption for sensitive data
- Anonymous key is safe to expose (read-only for public data)

### Sync Strategy (I-CLOUD-001, I-CLOUD-002)

- **Idempotent operations**: Using upsert to prevent duplicates
- **Last-write-wins**: Database trigger compares `modified_at` timestamps
- **Offline queue**: Operations queued in localStorage when offline
- **Auto-retry**: Failed operations retry with exponential backoff (max 3 attempts)

## Troubleshooting

### "Invalid API key"

- Verify your `VITE_SUPABASE_ANON_KEY` is correct
- Make sure you copied the "anon public" key, not the "service role" key

### "Row Level Security policy violation"

- Ensure RLS policies are created (check migration)
- Verify user is authenticated before syncing

### "Failed to sync"

- Check browser console for detailed errors
- Verify network connectivity
- Check Supabase dashboard > Logs for server-side errors

### Offline sync not working

- Check localStorage for `cloudSyncQueue` key
- Verify queue is populated when offline
- Check that sync triggers when coming back online

## Advanced Configuration

### Custom Conflict Resolution

The default is last-write-wins. To customize:

1. Edit `supabase/migrations/001_initial_schema.sql`
2. Modify the `merge_personality_update()` function
3. Re-run the migration

### Encryption

To add field-level encryption:

1. Install encryption library:
   ```bash
   npm install crypto-js
   ```

2. Encrypt sensitive fields before sending to Supabase
3. Decrypt when fetching from Supabase

Example:
```typescript
import CryptoJS from 'crypto-js';

const secret = 'your-encryption-key';

// Encrypt
const encrypted = CryptoJS.AES.encrypt(
  JSON.stringify(data),
  secret
).toString();

// Decrypt
const decrypted = JSON.parse(
  CryptoJS.AES.decrypt(encrypted, secret).toString(CryptoJS.enc.Utf8)
);
```

### Realtime Subscriptions

The service already sets up realtime subscriptions. To handle updates:

```typescript
window.addEventListener('cloud-personality-change', (event) => {
  console.log('Personality changed:', event.detail);
  // Update local state
});
```

## Production Deployment

### Environment Variables

Set these in your production environment:

- Vercel: Project Settings > Environment Variables
- Netlify: Site settings > Build & deploy > Environment
- Railway: Project > Variables

### Database Backups

1. Supabase dashboard > Database > Backups
2. Enable automatic backups (free tier includes daily backups)
3. Test restore process

### Monitoring

1. Supabase dashboard > Logs
2. Monitor sync operations in `sync_audit_log` table
3. Set up alerts for errors

## Support

- Issue tracker: https://github.com/Hulupeep/mbot_ruvector/issues
- Supabase docs: https://supabase.com/docs
- Contract reference: `docs/contracts/feature_cloud_sync.yml`

---

**Contract Compliance:**
- ✅ I-CLOUD-001: Idempotent sync operations
- ✅ I-CLOUD-002: Last-write-wins conflict resolution
- ✅ I-CLOUD-003: AES-256 encryption support
- ✅ Offline-first with queue management
- ✅ Row Level Security for data isolation
