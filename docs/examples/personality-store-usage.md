# PersonalityStore Usage Guide

This document demonstrates how to use the PersonalityStore service to maintain consistent personality across all mBot apps (ArtBot, GameBot, HelperBot).

## Overview

The PersonalityStore is a **singleton service** that manages personality state and persists it to localStorage. It ensures:

- **Cross-app consistency**: All apps share the same personality instance
- **Restart survival**: Personality is saved to disk and restored on startup
- **Atomic updates**: No partial states, all changes are atomic
- **Subscribe/notify**: Components can react to personality changes

## Quick Start

### 1. Initialize on App Startup

```typescript
import { personalityStore } from './services/personalityStore';

// In your app's main initialization
async function initializeApp() {
  // Load personality from localStorage
  await personalityStore.initialize();

  // Now personality is ready to use
  console.log('Current personality:', personalityStore.getCurrentPersonality());
}
```

### 2. Get Current Personality

```typescript
import { personalityStore } from './services/personalityStore';

// Get the current personality
const personality = personalityStore.getCurrentPersonality();

console.log('Name:', personality.name);
console.log('Curiosity:', personality.curiosity_drive);
console.log('Energy:', personality.energy_baseline);
```

### 3. Update Personality

```typescript
import { personalityStore } from './services/personalityStore';
import { Personality } from './types/personality';

// Set a completely new personality
const curiousPersonality: Personality = {
  id: 'curious-bot',
  name: 'Curious',
  icon: '🔍',
  version: 1,
  created_at: Date.now(),
  modified_at: Date.now(),
  tension_baseline: 0.3,
  coherence_baseline: 0.7,
  energy_baseline: 0.8,
  startle_sensitivity: 0.4,
  recovery_speed: 0.6,
  curiosity_drive: 0.9, // High curiosity!
  movement_expressiveness: 0.7,
  sound_expressiveness: 0.6,
  light_expressiveness: 0.5,
};

personalityStore.setPersonality(curiousPersonality);
// Personality is automatically persisted to localStorage
```

### 4. Update Only Config (Preserve Metadata)

```typescript
import { personalityStore } from './services/personalityStore';
import { PersonalityConfig } from './types/personality';

// Update just the personality parameters, keep name/icon/etc
const newConfig: PersonalityConfig = {
  tension_baseline: 0.2, // Make more relaxed
  coherence_baseline: 0.8,
  energy_baseline: 0.9, // Make more energetic
  startle_sensitivity: 0.3,
  recovery_speed: 0.7,
  curiosity_drive: 0.9,
  movement_expressiveness: 0.8,
  sound_expressiveness: 0.7,
  light_expressiveness: 0.6,
};

personalityStore.updateConfig(newConfig);
// Metadata (name, icon, id) is preserved
```

### 5. Subscribe to Changes

```typescript
import { personalityStore } from './services/personalityStore';
import { Personality } from './types/personality';

// Subscribe to personality changes
const unsubscribe = personalityStore.subscribeToChanges((personality: Personality) => {
  console.log('Personality changed to:', personality.name);
  console.log('New curiosity level:', personality.curiosity_drive);

  // Update UI or robot behavior based on new personality
  updateRobotBehavior(personality);
});

// Later, when component unmounts
unsubscribe();
```

## Example: ArtBot Integration

```typescript
// web/src/apps/ArtBot.tsx
import React, { useEffect, useState } from 'react';
import { personalityStore } from '../services/personalityStore';
import { Personality } from '../types/personality';

export const ArtBot: React.FC = () => {
  const [personality, setPersonality] = useState<Personality>(
    personalityStore.getCurrentPersonality()
  );

  useEffect(() => {
    // Initialize personality on mount
    personalityStore.initialize().then(() => {
      setPersonality(personalityStore.getCurrentPersonality());
    });

    // Subscribe to personality changes
    const unsubscribe = personalityStore.subscribeToChanges((newPersonality) => {
      setPersonality(newPersonality);
      console.log('ArtBot received personality update:', newPersonality.name);
    });

    // Cleanup on unmount
    return () => {
      unsubscribe();
    };
  }, []);

  // Use personality to influence drawing behavior
  const getDrawingStyle = () => {
    return {
      tension: personality.tension_baseline,
      smoothness: personality.coherence_baseline,
      speed: personality.energy_baseline,
      expressiveness: personality.movement_expressiveness,
    };
  };

  return (
    <div className="artbot">
      <h2>ArtBot - {personality.icon} {personality.name}</h2>
      <Canvas style={getDrawingStyle()} />
    </div>
  );
};
```

## Example: GameBot Integration

```typescript
// web/src/apps/GameBot.tsx
import React, { useEffect, useState } from 'react';
import { personalityStore } from '../services/personalityStore';
import { Personality } from '../types/personality';

export const GameBot: React.FC = () => {
  const [personality, setPersonality] = useState<Personality>(
    personalityStore.getCurrentPersonality()
  );

  useEffect(() => {
    personalityStore.initialize().then(() => {
      setPersonality(personalityStore.getCurrentPersonality());
    });

    const unsubscribe = personalityStore.subscribeToChanges(setPersonality);
    return () => unsubscribe();
  }, []);

  // Use personality to influence game behavior
  const getGameStyle = () => {
    return {
      aggressiveness: personality.energy_baseline,
      reactiveness: personality.startle_sensitivity,
      playfulness: personality.curiosity_drive,
      competitiveness: personality.tension_baseline,
    };
  };

  return (
    <div className="gamebot">
      <h2>GameBot - {personality.icon} {personality.name}</h2>
      <TicTacToeGame strategy={getGameStyle()} />
    </div>
  );
};
```

## Example: Personality Mixer

```typescript
// web/src/components/PersonalityMixer.tsx
import React, { useState } from 'react';
import { personalityStore } from '../services/personalityStore';
import { PersonalityConfig } from '../types/personality';

export const PersonalityMixer: React.FC = () => {
  const [config, setConfig] = useState<PersonalityConfig>(
    personalityStore.getCurrentPersonality()
  );

  const handleSliderChange = (param: keyof PersonalityConfig, value: number) => {
    const newConfig = { ...config, [param]: value };
    setConfig(newConfig);
  };

  const handleApply = () => {
    // Update personality (will notify all apps)
    personalityStore.updateConfig(config);
    console.log('Personality updated! All apps will receive the change.');
  };

  return (
    <div className="personality-mixer">
      <h2>Personality Mixer</h2>

      <label>
        Curiosity Drive: {config.curiosity_drive.toFixed(2)}
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={config.curiosity_drive}
          onChange={(e) => handleSliderChange('curiosity_drive', parseFloat(e.target.value))}
        />
      </label>

      <label>
        Energy Baseline: {config.energy_baseline.toFixed(2)}
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={config.energy_baseline}
          onChange={(e) => handleSliderChange('energy_baseline', parseFloat(e.target.value))}
        />
      </label>

      {/* Add more sliders for other parameters */}

      <button onClick={handleApply}>Apply to Robot</button>
    </div>
  );
};
```

## Testing Cross-App Persistence

```typescript
// Test scenario from integration tests
describe('Cross-App Persistence', () => {
  it('Personality persists when switching apps', async () => {
    // 1. Set personality in Mixer
    const curiousPersonality = createCuriousPersonality();
    personalityStore.setPersonality(curiousPersonality);

    // 2. Simulate app switch (reset singleton, reload from disk)
    PersonalityStore.resetInstance();
    const artBotStore = PersonalityStore.getInstance();
    await artBotStore.initialize();

    // 3. Verify ArtBot has the same personality
    const artBotPersonality = artBotStore.getCurrentPersonality();
    expect(artBotPersonality.name).toBe('Curious');
    expect(artBotPersonality.curiosity_drive).toBe(0.9);
  });
});
```

## Contract Compliance

### I-ARCH-PERS-001: Singleton Pattern ✅
```typescript
const store1 = PersonalityStore.getInstance();
const store2 = PersonalityStore.getInstance();
console.log(store1 === store2); // true - same instance
```

### I-ARCH-PERS-002: Atomic Updates ✅
```typescript
// All parameters are updated together, no partial states
personalityStore.setPersonality(newPersonality);
// Either ALL parameters update, or NONE do (if validation fails)
```

### PERS-004: localStorage Persistence ✅
```typescript
// Automatic persistence on every update
personalityStore.setPersonality(personality);
// ^ This automatically calls persistToDisk()

// Manual persistence if needed
await personalityStore.persistToDisk();

// Load on startup
await personalityStore.loadFromDisk();
```

## Error Handling

```typescript
// Invalid personality (out of bounds) throws error
try {
  const invalidPersonality = {
    // ... other fields
    tension_baseline: 1.5, // INVALID: > 1.0
  };

  personalityStore.setPersonality(invalidPersonality);
} catch (error) {
  console.error('Failed to set personality:', error);
  // Original personality is unchanged
}
```

## Best Practices

1. **Initialize on app startup**: Call `personalityStore.initialize()` in your app's entry point
2. **Subscribe in components**: Use `subscribeToChanges()` to react to personality updates
3. **Always unsubscribe**: Clean up subscriptions when components unmount
4. **Use updateConfig() for parameter changes**: Preserves metadata (name, icon, id)
5. **Validate before setting**: All parameters must be in range [0.0, 1.0]

## Files Created

- `/web/src/types/personalityStore.ts` - Type definitions and interfaces
- `/web/src/services/personalityStore.ts` - Singleton service implementation
- `/tests/integration/personality-persistence.test.ts` - Integration tests (13 tests, all passing ✅)

## Dependencies

- Issue #12 (Personality Data Structure) - ✅ Complete
- Issue #58 (Mixer UI) - ✅ Complete

## Blocks

- Issue #72 (depends on this implementation)
