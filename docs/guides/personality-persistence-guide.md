# Personality Persistence Guide

**Feature:** Wave 6 Sprint 2
**Service:** `personalityStore.ts`
**Difficulty:** Intermediate
**Time to Learn:** 15 minutes

## Overview

Personality Persistence enables cross-app personality synchronization using a singleton pattern with atomic updates and localStorage backing.

### Key Features

- Singleton pattern (I-ARCH-PERS-001)
- Atomic updates (I-ARCH-PERS-002)
- LocalStorage persistence
- Subscribe/notify pattern
- Cross-app synchronization
- Survives app restarts

---

## Quick Start

```typescript
import { personalityStore } from '@/services/personalityStore';

// Update personality
personalityStore.updateParameter('curiosity', 0.8);

// Get current personality
const current = personalityStore.getCurrentPersonality();

// Subscribe to changes
const unsubscribe = personalityStore.subscribe((personality) => {
  console.log('Personality updated:', personality);
});
```

---

## Cross-App Sync

Personality persists across all apps:

```
Personality Mixer → ArtBot → GameBot → HelperBot
        ↓              ↓         ↓          ↓
    All use personalityStore (same instance)
```

### Example Flow

```typescript
// In Personality Mixer
personalityStore.updateParameter('playfulness', 0.9);

// Automatically synced to Drawing Gallery
// Robot now draws with more flourishes

// Automatically synced to Game Stats
// Robot plays more spontaneously
```

---

## API

### Update Methods

```typescript
// Update single parameter
personalityStore.updateParameter('energy', 0.7);

// Update entire personality
personalityStore.updatePersonality({
  curiosity: 0.8,
  energy: 0.6,
  // ... all 9 parameters
});
```

### Subscribe to Changes

```typescript
const unsubscribe = personalityStore.subscribe((personality) => {
  // Called whenever personality changes
  console.log('New personality:', personality);
});

// Later: cleanup
unsubscribe();
```

### Persistence

```typescript
// Save to localStorage (automatic on every update)
// Manual save:
personalityStore.save();

// Load from localStorage (automatic on app start)
// Manual load:
personalityStore.load();

// Check if personality exists
const hasSaved = personalityStore.hasSavedPersonality();
```

---

## Contract Compliance

### I-ARCH-PERS-001: Singleton Pattern

Only one personality instance active at a time:

```typescript
// Enforced by module pattern
export const personalityStore = new PersonalityStore();
// Only one export, only one instance
```

### I-ARCH-PERS-002: Atomic Updates

No partial states:

```typescript
// ✅ CORRECT: All parameters updated together
personalityStore.updatePersonality({
  curiosity: 0.8,
  energy: 0.7,
  // ... all 9 required
});

// ❌ INCORRECT: Partial update (blocked by TypeScript)
personalityStore.updatePersonality({
  curiosity: 0.8, // Missing other parameters!
});
```

---

## Troubleshooting

### Personality not persisting

```typescript
// Check localStorage quota
const usage = localStorage.length;
console.log('LocalStorage keys:', usage);

// Clear if needed
personalityStore.clear();
```

### Changes not syncing across apps

```typescript
// Verify subscription
const isSubscribed = personalityStore.getSubscriberCount() > 0;
console.log('Active subscribers:', isSubscribed);
```

---

## Related Features

- [Personality Mixer](personality-mixer-guide.md)
- [Cloud Sync](cloud-sync-guide.md)
- [Data Export/Import](data-export-import-guide.md)

---

**Last Updated:** 2026-02-01
**Status:** Production Ready ✅
