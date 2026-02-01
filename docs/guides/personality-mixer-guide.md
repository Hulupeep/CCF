# Personality Mixer Guide

**Feature:** Wave 6 Sprint 1
**Component:** `PersonalityMixer.tsx`
**Difficulty:** Beginner
**Time to Learn:** 10 minutes

## Overview

The Personality Mixer is a visual editor that lets you customize your robot's behavior through 9 adjustable parameters. Create unique personalities, save presets, and see changes in real-time.

### Key Features

- 9 personality sliders with real-time validation
- 15 built-in presets (Mellow, Curious, Playful, etc.)
- Undo/redo stack (up to 50 states)
- Save/load custom personalities
- WebSocket integration (20Hz display, 2Hz send)
- Auto-disabled when disconnected

---

## Quick Start

### Basic Usage

```typescript
import { PersonalityMixer } from '@/components/PersonalityMixer';

function App() {
  return (
    <PersonalityMixer
      wsUrl="ws://localhost:4000"
      onPersonalityChange={(personality) => {
        console.log('New personality:', personality);
      }}
    />
  );
}
```

### With Persistence

```typescript
import { PersonalityMixer } from '@/components/PersonalityMixer';
import { personalityStore } from '@/services/personalityStore';

function App() {
  // Load saved personality on mount
  const [personality, setPersonality] = useState(
    personalityStore.getCurrentPersonality()
  );

  const handleChange = (newPersonality: PersonalityConfig) => {
    personalityStore.updatePersonality(newPersonality);
    setPersonality(newPersonality);
  };

  return <PersonalityMixer onPersonalityChange={handleChange} />;
}
```

---

## The 9 Parameters

### 1. Curiosity (0.0 - 1.0)

**What it does:** Controls exploration vs. routine behavior

- **0.0**: Sticks to familiar patterns, predictable
- **0.5**: Balanced between familiar and new
- **1.0**: Constantly explores, tries new things

**Example:** High curiosity = robot tries new drawing styles

### 2. Energy (0.0 - 1.0)

**What it does:** Controls activity level and speed

- **0.0**: Slow, deliberate movements
- **0.5**: Moderate pace
- **1.0**: Fast, energetic movements

**Example:** High energy = quick game responses

### 3. Playfulness (0.0 - 1.0)

**What it does:** Controls spontaneous vs. serious behavior

- **0.0**: Focused, task-oriented
- **0.5**: Mix of work and play
- **1.0**: Frequent surprises, jokes, playful actions

**Example:** High playfulness = adds flourishes to drawings

### 4. Patience (0.0 - 1.0)

**What it does:** Controls frustration tolerance

- **0.0**: Gives up quickly on difficult tasks
- **0.5**: Moderate persistence
- **1.0**: Never gives up, keeps trying

**Example:** High patience = keeps trying to sort small pieces

### 5. Focus (0.0 - 1.0)

**What it does:** Controls distraction vs. concentration

- **0.0**: Easily distracted by environment
- **0.5**: Can maintain some focus
- **1.0**: Laser-focused on current task

**Example:** High focus = ignores background noise during games

### 6. Sociability (0.0 - 1.0)

**What it does:** Controls interaction frequency

- **0.0**: Prefers independent work
- **0.5**: Balanced interaction
- **1.0**: Constantly seeks engagement

**Example:** High sociability = frequently asks for input

### 7. Boldness (0.0 - 1.0)

**What it does:** Controls risk-taking

- **0.0**: Cautious, avoids risks
- **0.5**: Balanced approach
- **1.0**: Takes bold risks, tries anything

**Example:** High boldness = attempts complex drawing techniques

### 8. Adaptability (0.0 - 1.0)

**What it does:** Controls learning rate

- **0.0**: Slow to change behavior
- **0.5**: Moderate learning speed
- **1.0**: Rapidly adapts to new situations

**Example:** High adaptability = quickly learns game strategies

### 9. Expressiveness (0.0 - 1.0)

**What it does:** Controls emotional display

- **0.0**: Minimal emotional feedback
- **0.5**: Moderate expressiveness
- **1.0**: Highly expressive, animated reactions

**Example:** High expressiveness = animated celebrations

---

## Presets

### Built-in Presets

| Preset | Personality Profile | Best For |
|--------|---------------------|----------|
| **Mellow** | Low energy, high patience | Calm activities |
| **Curious** | High curiosity, moderate energy | Exploration |
| **Zen** | Max patience, low energy | Meditative tasks |
| **Excitable** | Max energy, max expressiveness | Active play |
| **Timid** | Low boldness, low sociability | Quiet environments |
| **Adventurous** | High boldness, high curiosity | Experimental play |
| **Sleepy** | Min energy, max patience | Low-activity time |
| **Playful** | High playfulness, high sociability | Social games |
| **Grumpy** | Low playfulness, low sociability | Independent work |
| **Focused** | Max focus, low playfulness | Task completion |
| **Chaotic** | Low focus, high playfulness | Unpredictable fun |
| **Gentle** | High patience, low energy | Delicate tasks |
| **Scientist** | High curiosity, high focus | Experiments |
| **Artist** | High expressiveness, high curiosity | Creative work |
| **Guardian** | High patience, high boldness | Protective behavior |

### Loading a Preset

```typescript
import { PRESETS } from '@/types/presets';

// Apply a preset
const scientistPersonality = PRESETS.scientist;
personalityStore.updatePersonality(scientistPersonality);
```

### Creating Custom Presets

```typescript
const myPreset: PersonalityConfig = {
  curiosity: 0.9,
  energy: 0.4,
  playfulness: 0.7,
  patience: 0.8,
  focus: 0.6,
  sociability: 0.5,
  boldness: 0.3,
  adaptability: 0.7,
  expressiveness: 0.8,
};

// Save to localStorage
personalityStore.savePreset('My Custom Bot', myPreset);
```

---

## Undo/Redo

The Personality Mixer includes a 50-state history stack.

### Keyboard Shortcuts

- **Ctrl+Z** (Windows/Linux) or **Cmd+Z** (Mac): Undo
- **Ctrl+Shift+Z** or **Cmd+Shift+Z**: Redo

### Programmatic Control

```typescript
import { usePersonalityHistory } from '@/hooks/usePersonalityHistory';

function MyComponent() {
  const { undo, redo, canUndo, canRedo } = usePersonalityHistory();

  return (
    <div>
      <button onClick={undo} disabled={!canUndo}>Undo</button>
      <button onClick={redo} disabled={!canRedo}>Redo</button>
    </div>
  );
}
```

---

## WebSocket Integration

### Connection Setup

```typescript
import { usePersonalityWebSocket } from '@/hooks/usePersonalityWebSocket';

function MyComponent() {
  const { connected, personality, updatePersonality } = usePersonalityWebSocket(
    'ws://localhost:4000'
  );

  return (
    <div>
      <p>Status: {connected ? 'Connected' : 'Disconnected'}</p>
      {/* UI auto-disables when disconnected */}
    </div>
  );
}
```

### Update Rates

- **Display**: 20Hz (50ms) - Shows robot state changes
- **Send**: 2Hz (500ms) - Debounced to prevent spam

### Protocol

```typescript
// Outbound (to robot)
{
  type: 'personality_update',
  payload: {
    curiosity: 0.8,
    energy: 0.6,
    // ... other parameters
  },
  timestamp: 1738454321000
}

// Inbound (from robot)
{
  type: 'personality_state',
  payload: {
    current: { /* personality config */ },
    applied: true
  },
  timestamp: 1738454321100
}
```

---

## Save/Load

### Saving to LocalStorage

```typescript
import { personalityStore } from '@/services/personalityStore';

// Save current personality
personalityStore.savePreset('Evening Mode');

// List saved presets
const presets = personalityStore.listPresets();
console.log(presets); // ['Evening Mode', 'Morning Mode', ...]
```

### Loading from LocalStorage

```typescript
// Load a saved preset
const personality = personalityStore.loadPreset('Evening Mode');
if (personality) {
  personalityStore.updatePersonality(personality);
}
```

### Exporting/Importing

```typescript
import { dataExport } from '@/services/dataExport';
import { dataImport } from '@/services/dataImport';

// Export all personalities
const manifest = await dataExport.exportData(['personality']);
const json = JSON.stringify(manifest, null, 2);
// Save to file...

// Import personalities
const result = await dataImport.importFromManifest(manifest, {
  strategy: 'merge', // or 'overwrite'
  skipInvalid: true
});
console.log(`Imported ${result.importedCount} personalities`);
```

---

## Configuration

### Component Props

```typescript
interface PersonalityMixerProps {
  wsUrl?: string; // WebSocket URL
  initialPersonality?: PersonalityConfig; // Starting values
  onPersonalityChange?: (personality: PersonalityConfig) => void; // Callback
  showPresets?: boolean; // Show preset buttons (default: true)
  showHistory?: boolean; // Show undo/redo (default: true)
  showSave?: boolean; // Show save/load (default: true)
  disabled?: boolean; // Disable all controls
  className?: string; // Custom CSS class
}
```

### Contract Enforcement

All parameters are automatically bounded:

```typescript
// Contract: I-PERS-001, I-PERS-UI-001
// All parameters MUST be in range [0.0, 1.0]

// These are automatically clamped:
personalityStore.updateParameter('curiosity', 1.5); // Clamped to 1.0
personalityStore.updateParameter('energy', -0.1); // Clamped to 0.0
```

---

## Troubleshooting

### Slider not responding

**Problem:** Sliders are disabled
**Solution:** Check WebSocket connection status

```typescript
const { connected } = usePersonalityWebSocket('ws://localhost:4000');
if (!connected) {
  console.error('WebSocket disconnected!');
}
```

### Changes not persisting

**Problem:** localStorage quota exceeded
**Solution:** Clear old data

```typescript
// Check localStorage usage
const usage = personalityStore.getStorageUsage();
console.log(`Using ${usage.bytes} bytes`);

// Clear old presets
personalityStore.deletePreset('Old Preset Name');
```

### WebSocket sending too fast

**Problem:** Server rate-limiting
**Solution:** Debouncing is automatic (500ms), but verify:

```typescript
// Contract: I-PERS-UI-002
// Maximum 2 updates per second (500ms debounce)
```

### Undo/redo not working

**Problem:** History disabled
**Solution:** Enable in props

```typescript
<PersonalityMixer showHistory={true} />
```

---

## Examples

### Example 1: Simple Integration

```typescript
import { PersonalityMixer } from '@/components/PersonalityMixer';

export default function SimplePage() {
  return (
    <div className="container">
      <h1>Robot Personality</h1>
      <PersonalityMixer wsUrl="ws://localhost:4000" />
    </div>
  );
}
```

### Example 2: With Custom Styling

```typescript
import { PersonalityMixer } from '@/components/PersonalityMixer';
import './custom-styles.css';

export default function StyledPage() {
  return (
    <PersonalityMixer
      wsUrl="ws://localhost:4000"
      className="my-custom-mixer"
      showPresets={true}
      showHistory={true}
    />
  );
}
```

### Example 3: Controlled Component

```typescript
import { useState } from 'react';
import { PersonalityMixer } from '@/components/PersonalityMixer';
import { personalityStore } from '@/services/personalityStore';

export default function ControlledPage() {
  const [personality, setPersonality] = useState(
    personalityStore.getCurrentPersonality()
  );

  const handleChange = (newPersonality: PersonalityConfig) => {
    setPersonality(newPersonality);
    personalityStore.updatePersonality(newPersonality);
    console.log('Personality updated:', newPersonality);
  };

  return (
    <div>
      <PersonalityMixer
        initialPersonality={personality}
        onPersonalityChange={handleChange}
      />
      <pre>{JSON.stringify(personality, null, 2)}</pre>
    </div>
  );
}
```

---

## Related Features

- [Personality Persistence](personality-persistence-guide.md) - Cross-app sync
- [Cloud Sync](cloud-sync-guide.md) - Cloud backup
- [WebSocket V2](websocket-v2-guide.md) - Real-time protocol
- [Data Export/Import](data-export-import-guide.md) - Backup/restore

---

## API Reference

See: [Personality API](../api/WAVE_6_APIs.md#personality-mixer)

---

**Last Updated:** 2026-02-01
**Status:** Production Ready ✅
