# Cloud Sync Debugging Guide

Detailed troubleshooting for cloud synchronization issues.

## Connection Issues

### Can't Connect to Supabase

**Symptoms:** "Failed to connect" or timeout
**Diagnosis:**
```typescript
// Test connection
const { data, error } = await supabase
  .from('personalities')
  .select('count');

if (error) {
  console.error('Connection error:', error);
}
```

**Solutions:**

1. **Check environment variables:**
```bash
echo $NEXT_PUBLIC_SUPABASE_URL
echo $NEXT_PUBLIC_SUPABASE_ANON_KEY
```

2. **Verify project status:**
- Go to Supabase dashboard
- Check if project is "Active"
- Check if project is paused (free tier auto-pauses after 7 days inactivity)

3. **Test with curl:**
```bash
curl -H "apikey: YOUR_ANON_KEY" \
  https://YOUR_PROJECT.supabase.co/rest/v1/personalities
```

### Authentication Fails

**Symptoms:** "JWT expired" or "Invalid token"
**Diagnosis:**
```typescript
const { data: { user }, error } = await supabase.auth.getUser();
console.log('User:', user);
console.log('Error:', error);
```

**Solutions:**

1. **Refresh session:**
```typescript
const { data, error } = await supabase.auth.refreshSession();
if (error) {
  // Re-authenticate
  await supabase.auth.signIn({
    email: email,
    password: password
  });
}
```

2. **Check token expiration:**
```typescript
const session = await supabase.auth.getSession();
const expiresAt = session.data.session?.expires_at;
const now = Math.floor(Date.now() / 1000);

if (expiresAt < now) {
  console.error('Token expired');
  await supabase.auth.refreshSession();
}
```

## Sync Issues

### Data Not Syncing

**Symptoms:** Changes don't appear on other devices
**Diagnosis:**
```typescript
// Check sync status
const result = await cloudSync.syncAll();
console.log('Sync result:', result);

// Check last sync time
const lastSync = await cloudSync.getLastSyncTime();
console.log('Last sync:', new Date(lastSync));
```

**Solutions:**

1. **Force sync:**
```typescript
await cloudSync.pushChanges(); // Upload local changes
await cloudSync.pullChanges(); // Download remote changes
```

2. **Check sync metadata:**
```typescript
const { data, error } = await supabase
  .from('sync_metadata')
  .select('*')
  .eq('device_id', getDeviceId())
  .single();

console.log('Sync metadata:', data);
```

3. **Clear sync cache:**
```typescript
await cloudSync.clearCache();
await cloudSync.syncAll();
```

### Conflicts Not Resolving

**Symptoms:** Conflict dialog keeps appearing
**Diagnosis:**
```typescript
// Check pending conflicts
const conflicts = await cloudSync.getPendingConflicts();
console.log('Pending conflicts:', conflicts.length);

conflicts.forEach(conflict => {
  console.log(`Conflict: ${conflict.type} - ${conflict.field}`);
  console.log('Local:', conflict.localData);
  console.log('Remote:', conflict.remoteData);
});
```

**Solutions:**

1. **Resolve manually:**
```typescript
for (const conflict of conflicts) {
  // Use most recent
  const useLocal = conflict.localTimestamp > conflict.remoteTimestamp;
  await cloudSync.resolveConflict(
    conflict.id,
    useLocal ? 'local' : 'remote'
  );
}
```

2. **Change strategy:**
```typescript
// Use last-write-wins (automatic)
cloudSync.setConflictStrategy('last-write-wins');
```

3. **Reset conflicted items:**
```typescript
// Delete local copy, re-download from cloud
await cloudSync.resetItem(conflict.type, conflict.id);
```

## Real-time Updates Issues

### Not Receiving Updates

**Symptoms:** Changes from other devices don't appear
**Diagnosis:**
```typescript
// Check subscription status
const channel = supabase
  .channel('test')
  .on('postgres_changes', {
    event: '*',
    schema: 'public',
    table: 'personalities'
  }, (payload) => {
    console.log('Received update:', payload);
  })
  .subscribe((status) => {
    console.log('Subscription status:', status);
  });

// Should log: "SUBSCRIBED" after a moment
```

**Solutions:**

1. **Verify replication enabled:**
```sql
-- In Supabase SQL Editor
SELECT * FROM pg_publication_tables
WHERE pubname = 'supabase_realtime';
-- Should show your tables
```

2. **Restart subscription:**
```typescript
channel.unsubscribe();
await new Promise(resolve => setTimeout(resolve, 1000));
channel.subscribe();
```

3. **Check network:**
```typescript
window.addEventListener('online', () => {
  cloudSync.reconnect();
});

window.addEventListener('offline', () => {
  console.warn('Offline - updates will queue');
});
```

## Performance Issues

### Slow Sync Times

**Symptoms:** Sync takes >5 seconds
**Diagnosis:**
```typescript
// Measure sync time
const start = Date.now();
await cloudSync.syncAll();
const duration = Date.now() - start;
console.log('Sync duration:', duration, 'ms');

// Break down by type
for (const type of ['personality', 'drawings', 'stats', 'inventory']) {
  const start = Date.now();
  await cloudSync.syncData([type]);
  const duration = Date.now() - start;
  console.log(`${type} sync:`, duration, 'ms');
}
```

**Solutions:**

1. **Selective sync:**
```typescript
// Only sync what changed
await cloudSync.syncData(['personality', 'inventory']);
// Don't sync 'drawings' (large data) every time
```

2. **Batch operations:**
```typescript
// Instead of syncing after every change:
// Queue changes and sync periodically
setInterval(() => {
  cloudSync.syncAll();
}, 5 * 60 * 1000); // Every 5 minutes
```

3. **Optimize queries:**
```sql
-- Add indexes for faster queries
CREATE INDEX idx_personalities_updated
  ON personalities(user_id, updated_at DESC);

CREATE INDEX idx_drawings_updated
  ON drawings(user_id, updated_at DESC);
```

### High Memory Usage

**Symptoms:** Browser/app becomes slow
**Diagnosis:**
```typescript
// Check data sizes
const estimate = await navigator.storage.estimate();
console.log('Storage used:', estimate.usage);

// Check specific data
const drawings = await supabase
  .from('drawings')
  .select('*')
  .eq('user_id', userId);

const size = JSON.stringify(drawings.data).length;
console.log('Drawings size:', size, 'bytes');
```

**Solutions:**
1. **Paginate large datasets:**
```typescript
// Instead of loading all at once:
const PAGE_SIZE = 20;
const { data, error } = await supabase
  .from('drawings')
  .select('*')
  .eq('user_id', userId)
  .order('created_at', { ascending: false })
  .range(0, PAGE_SIZE - 1);
```

2. **Clean old data:**
```typescript
// Delete data older than 90 days
await cloudSync.deleteOlderThan(90);
```

3. **Compress data:**
```typescript
// Use compression for large payloads
import { compress, decompress } from 'lz-string';

const compressed = compress(JSON.stringify(largeData));
await supabase.from('table').insert({ data: compressed });
```

## Quota & Limits Issues

### Rate Limit Exceeded

**Symptoms:** "Too many requests" error
**Solutions:**
```typescript
// Add retry with exponential backoff
async function syncWithRetry(maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      await cloudSync.syncAll();
      return;
    } catch (error) {
      if (error.code === 'RATE_LIMIT') {
        const delay = Math.pow(2, i) * 1000; // 1s, 2s, 4s
        await new Promise(resolve => setTimeout(resolve, delay));
      } else {
        throw error;
      }
    }
  }
}
```

### Storage Quota Exceeded

**Symptoms:** "Storage quota exceeded" error
**Solutions:**
```typescript
// Check quota
const { data, error } = await supabase.rpc('get_storage_usage');
console.log('Storage usage:', data);

// Clean up
await cloudSync.cleanupOldDrawings(30); // Keep only last 30 days
await cloudSync.compressDrawings(); // Compress old drawings
```

## Debugging Tools

### Enable Debug Logging

```typescript
// Enable verbose logging
cloudSync.setDebugMode(true);

// Log all Supabase operations
supabase.auth.onAuthStateChange((event, session) => {
  console.log('Auth event:', event, session);
});

supabase.channel('debug')
  .on('*', {}, (payload) => {
    console.log('Realtime event:', payload);
  })
  .subscribe();
```

### Test Sync Manually

```typescript
// Test each component
async function testSync() {
  console.log('Testing connection...');
  const connected = await cloudSync.testConnection();
  console.log('Connected:', connected);

  console.log('Testing auth...');
  const user = await cloudSync.getCurrentUser();
  console.log('User:', user?.id);

  console.log('Testing upload...');
  await cloudSync.pushChanges();
  console.log('Upload complete');

  console.log('Testing download...');
  await cloudSync.pullChanges();
  console.log('Download complete');

  console.log('Testing realtime...');
  const received = await new Promise((resolve) => {
    const timeout = setTimeout(() => resolve(false), 5000);
    cloudSync.subscribeToChanges((change) => {
      clearTimeout(timeout);
      resolve(true);
    });
    // Trigger a change from another device/tab
  });
  console.log('Realtime:', received ? 'Working' : 'Not working');
}

testSync();
```

### Check Database Directly

```sql
-- In Supabase SQL Editor

-- Check row counts
SELECT 'personalities' AS table_name, COUNT(*) AS count FROM personalities
UNION ALL
SELECT 'drawings', COUNT(*) FROM drawings
UNION ALL
SELECT 'game_stats', COUNT(*) FROM game_stats;

-- Check recent updates
SELECT * FROM personalities
WHERE updated_at > NOW() - INTERVAL '1 hour'
ORDER BY updated_at DESC;

-- Check for conflicts
SELECT
  p1.id,
  p1.user_id,
  p1.updated_at AS time1,
  p2.updated_at AS time2
FROM personalities p1
JOIN personalities p2 ON p1.id = p2.id AND p1.updated_at <> p2.updated_at
WHERE p1.user_id = 'YOUR_USER_ID';
```

---

**Last Updated:** 2026-02-01
