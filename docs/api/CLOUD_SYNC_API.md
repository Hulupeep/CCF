# Cloud Sync API - Detailed Specification

**Version:** 1.0.0
**Backend:** Supabase (PostgreSQL + Realtime)
**Last Updated:** 2026-02-01

## Overview

Cloud Sync provides real-time data synchronization across devices using Supabase as the backend. Features include conflict resolution, offline queue, selective sync, and encryption at rest.

## Setup

### 1. Environment Variables

```bash
NEXT_PUBLIC_SUPABASE_URL=https://your-project.supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=your-anon-key
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key # Server-side only
```

### 2. Database Schema

```sql
-- Personalities table
CREATE TABLE personalities (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES auth.users(id),
  name VARCHAR(255) NOT NULL,
  config JSONB NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  deleted BOOLEAN DEFAULT FALSE
);

-- Drawings table
CREATE TABLE drawings (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES auth.users(id),
  session_id VARCHAR(255) NOT NULL,
  strokes JSONB NOT NULL,
  mood VARCHAR(50),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  deleted BOOLEAN DEFAULT FALSE
);

-- Game statistics table
CREATE TABLE game_stats (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES auth.users(id),
  game_type VARCHAR(50) NOT NULL,
  result VARCHAR(20) NOT NULL,
  score INT NOT NULL,
  personality_config JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Inventory table
CREATE TABLE inventory (
  user_id UUID PRIMARY KEY REFERENCES auth.users(id),
  red INT DEFAULT 0,
  green INT DEFAULT 0,
  blue INT DEFAULT 0,
  yellow INT DEFAULT 0,
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Sync metadata table
CREATE TABLE sync_metadata (
  user_id UUID NOT NULL REFERENCES auth.users(id),
  device_id VARCHAR(255) NOT NULL,
  last_sync TIMESTAMPTZ DEFAULT NOW(),
  sync_version INT DEFAULT 1,
  PRIMARY KEY (user_id, device_id)
);
```

## API Reference

### CloudSync Service

```typescript
import { cloudSync } from '@/services/cloudSync';

// Initialize
await cloudSync.initialize({
  supabaseUrl: process.env.NEXT_PUBLIC_SUPABASE_URL!,
  supabaseAnonKey: process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
  userId: session.user.id,
  deviceId: getDeviceId() // Unique per device
});
```

### Sync Operations

#### Full Sync

```typescript
const result = await cloudSync.syncAll();

interface SyncResult {
  success: boolean;
  synced: {
    personalities: number;
    drawings: number;
    stats: number;
    inventory: boolean;
  };
  conflicts: Conflict[];
  errors: Error[];
  timestamp: number;
}
```

#### Selective Sync

```typescript
// Sync only specific data types
await cloudSync.syncData(['personality', 'inventory']);

// Available types
type DataType = 'personality' | 'drawings' | 'stats' | 'inventory';
```

#### Push Changes

```typescript
// Push local changes to cloud
await cloudSync.pushChanges();
```

#### Pull Changes

```typescript
// Pull remote changes from cloud
await cloudSync.pullChanges();
```

### Conflict Resolution

#### Automatic Resolution

```typescript
// Set conflict strategy
cloudSync.setConflictStrategy('last-write-wins'); // Default
cloudSync.setConflictStrategy('manual'); // Require manual resolution
```

#### Manual Resolution

```typescript
// Listen for conflicts
cloudSync.onConflict((conflict: Conflict) => {
  console.log('Conflict detected:', conflict);

  // Show UI for user to choose
  showConflictDialog(conflict);
});

// Resolve conflict
await cloudSync.resolveConflict(conflict.id, 'local'); // Use local version
await cloudSync.resolveConflict(conflict.id, 'remote'); // Use remote version
```

#### Conflict Type

```typescript
interface Conflict {
  id: string;
  type: DataType;
  localData: any;
  remoteData: any;
  localTimestamp: number;
  remoteTimestamp: number;
  field: string; // Which field conflicted
}
```

### Offline Support

#### Offline Queue

```typescript
// Changes are automatically queued when offline
// and synced when connection is restored

// Check queue status
const queueSize = await cloudSync.getQueueSize();
console.log(`${queueSize} changes pending sync`);

// Force queue processing
await cloudSync.processQueue();
```

#### Connection Monitoring

```typescript
// Listen for connection changes
cloudSync.onConnectionChange((online: boolean) => {
  if (online) {
    console.log('Back online - syncing...');
    cloudSync.syncAll();
  } else {
    console.log('Offline - changes will be queued');
  }
});
```

### Real-time Updates

#### Subscribe to Changes

```typescript
// Subscribe to real-time updates from other devices
const unsubscribe = cloudSync.subscribeToChanges((change: Change) => {
  console.log('Remote change:', change);

  switch (change.type) {
    case 'personality':
      // Update local personality
      personalityStore.updatePersonality(change.data);
      break;
    case 'inventory':
      // Update local inventory
      inventoryStore.update(change.data);
      break;
  }
});

// Cleanup
unsubscribe();
```

#### Change Type

```typescript
interface Change {
  type: DataType;
  operation: 'insert' | 'update' | 'delete';
  data: any;
  deviceId: string; // Which device made the change
  timestamp: number;
}
```

### Encryption

#### Data Encryption

```typescript
// All data encrypted at rest in Supabase (I-CLOUD-001)
// Encryption handled by Supabase automatically

// Optional: Client-side encryption for sensitive data
await cloudSync.enableClientSideEncryption({
  key: generateEncryptionKey(),
  algorithm: 'AES-256-GCM'
});
```

### Events

#### Event Handlers

```typescript
// Sync complete
cloudSync.onSyncComplete(() => {
  console.log('Sync complete!');
  showNotification('Data synced successfully');
});

// Sync error
cloudSync.onSyncError((error: Error) => {
  console.error('Sync failed:', error);
  showNotification('Sync failed - will retry');
});

// Conflict detected
cloudSync.onConflict((conflict: Conflict) => {
  showConflictDialog(conflict);
});

// Connection status changed
cloudSync.onConnectionChange((online: boolean) => {
  updateConnectionIndicator(online);
});
```

## Best Practices

### 1. Sync Frequency

```typescript
// Sync on app start
useEffect(() => {
  cloudSync.syncAll();
}, []);

// Sync on window focus
window.addEventListener('focus', () => {
  cloudSync.syncAll();
});

// Periodic sync (every 5 minutes)
setInterval(() => {
  cloudSync.syncAll();
}, 5 * 60 * 1000);
```

### 2. Bandwidth Optimization

```typescript
// Only sync what changed
await cloudSync.pushChanges(); // Only sends modified data

// Use delta sync for large datasets
await cloudSync.syncData(['inventory']); // Small data
// Don't automatically sync 'drawings' (large data) - let user trigger
```

### 3. Error Handling

```typescript
try {
  await cloudSync.syncAll();
} catch (error) {
  if (error.code === 'NETWORK_ERROR') {
    // Queue for later
    await cloudSync.addToQueue(pendingChanges);
  } else if (error.code === 'AUTH_ERROR') {
    // Re-authenticate
    await refreshAuth();
  } else {
    // Log and notify user
    console.error('Sync error:', error);
    showError('Failed to sync data');
  }
}
```

## Testing

### Mock Cloud Sync

```typescript
// For testing without Supabase
import { MockCloudSync } from '@/services/__mocks__/cloudSync';

const mockSync = new MockCloudSync();
mockSync.simulateConflict({ /* ... */ });
mockSync.simulateOffline();
```

## Troubleshooting

See: [Cloud Sync Debugging Guide](../troubleshooting/CLOUD_SYNC_DEBUG.md)

---

**Last Updated:** 2026-02-01
**Status:** Production Ready ✅
