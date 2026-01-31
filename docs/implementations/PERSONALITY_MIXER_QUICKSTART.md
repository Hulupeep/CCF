# Personality Mixer - Quick Start Guide

## 🎨 What Was Built

A complete React-based web UI for adjusting mBot2's personality in real-time.

## 📁 Files Created (20 total)

```
web/
├── src/
│   ├── components/
│   │   ├── PersonalityMixer.tsx          ⭐ Main component (467 lines)
│   │   ├── PersonalityMixer.css          🎨 Styles
│   │   └── __tests__/
│   │       └── PersonalityMixer.test.tsx ✅ Unit tests (350+ lines)
│   ├── hooks/
│   │   ├── usePersonalityWebSocket.ts    🔌 WebSocket + debouncing
│   │   ├── usePersonalityHistory.ts      ↶↷ Undo/redo stack
│   │   └── useLocalStorage.ts            💾 Persistence
│   ├── types/
│   │   ├── personality.ts                📐 Types + validation
│   │   └── presets.ts                    🤖 15 personalities
│   ├── App.tsx                           📱 Example app
│   ├── main.tsx                          🚀 Entry point
│   └── index.css                         🎨 Global styles
└── index.html                            📄 HTML entry
```

## 🎯 Key Features

### 1. Nine Parameter Sliders
- ⚖️ **Baselines**: Tension, Coherence, Energy
- ⚡ **Reactivity**: Startle, Recovery, Curiosity
- 🎭 **Expression**: Movement, Sound, Light

### 2. Fifteen Personality Presets
😌 Mellow | 🤔 Curious | 🧘 Zen | 🤩 Excitable | 😟 Timid
🚀 Adventurous | 😴 Sleepy | 🎉 Playful | 😠 Grumpy | 🎯 Focused
🌪️ Chaotic | 🌸 Gentle | 🔬 Scientist | 🎨 Artist | 🛡️ Guardian

### 3. Advanced Features
- ↶↷ **Undo/Redo** (50 state history)
- 💾 **Save Custom** personalities
- 🎲 **Randomize** button
- 👁️ **Hover Preview** on presets
- 🔌 **Live Connection** status

## 🚀 Quick Start

```bash
# Navigate to web source
cd web/src

# Install dependencies
npm install

# Start development server
npm run dev

# Access at http://localhost:5173
```

## 🧪 Run Tests

```bash
npm test
```

## 📊 Test Coverage

✅ **15 Test Suites** covering:
- Parameter validation (I-PERS-001)
- Bounds enforcement (ARCH-004)
- All 15 presets
- localStorage persistence
- Undo/redo logic
- WebSocket debouncing
- Default config safety

## 🔌 WebSocket Integration

Connects to: `ws://localhost:8081`

**Message format:**
```json
{
  "type": "personality_update",
  "params": {
    "tension_baseline": 0.7,
    "coherence_baseline": 0.5
  }
}
```

**Update frequency:**
- Display: 20Hz (50ms) - smooth UI
- Network: 2Hz (500ms) - debounced

## 📋 Contract Compliance

| Contract | Status | Implementation |
|----------|--------|----------------|
| I-PERS-001 | ✅ | Parameters bounded [0.0, 1.0] |
| I-PERS-UI-001 | ✅ | Sliders enforce range |
| I-PERS-UI-002 | ✅ | Debounced to 2/sec |
| I-PERS-UI-003 | ✅ | Disabled when disconnected |
| ARCH-004 | ✅ | Bounded parameters contract |

## 🎮 Usage Example

```tsx
import PersonalityMixer from './components/PersonalityMixer';

function App() {
  return (
    <PersonalityMixer
      wsUrl="ws://localhost:8081"
      onConfigChange={(config) => {
        console.log('New personality:', config);
      }}
    />
  );
}
```

## 📐 Data-testid Reference

**Sliders:**
- `slider-tension-baseline`
- `slider-coherence-baseline`
- `slider-energy-baseline`
- `slider-startle-sensitivity`
- `slider-recovery-speed`
- `slider-curiosity-drive`
- `slider-movement-expressiveness`
- `slider-sound-expressiveness`
- `slider-light-expressiveness`

**Buttons:**
- `preset-button-{id}` (15 presets)
- `save-custom-button`
- `reset-button`
- `randomize-button`
- `undo-button`
- `redo-button`

**Status:**
- `connection-status`

## 💾 Custom Personalities

Saved to: `localStorage['mbot-custom-personalities']`

```typescript
interface CustomPersonality {
  name: string;           // "My Robot"
  config: PersonalityConfig;
  created_at: number;     // Unix timestamp
}
```

## 🎨 UI Preview

```
┌─────────────────────────────────────────────┐
│ 🔌 Connected to robot                       │
└─────────────────────────────────────────────┘

┌───────────────────────────┬─────────────────┐
│ Personality Parameters    │ Presets         │
│                           │                 │
│ ↶ Undo  ↷ Redo           │ 😌 🤔 🧘 🤩 😟  │
│                           │ 🚀 😴 🎉 😠 🎯  │
│ ⚖️ Baselines             │ 🌪️ 🌸 🔬 🎨 🛡️ │
│ ├ Tension    [====|----] │                 │
│ ├ Coherence  [======|--] │ Custom:         │
│ └ Energy     [===|-----] │ • My Robot   ✕  │
│                           │                 │
│ ⚡ Reactivity            │                 │
│ ├ Startle    [====|----] │                 │
│ ├ Recovery   [======|--] │                 │
│ └ Curiosity  [===|-----] │                 │
│                           │                 │
│ 🎭 Expression            │                 │
│ ├ Movement   [====|----] │                 │
│ ├ Sound      [======|--] │                 │
│ └ Light      [===|-----] │                 │
│                           │                 │
│ [↺ Reset] [🎲 Random]    │                 │
│ [💾 Save Custom]         │                 │
└───────────────────────────┴─────────────────┘
```

## 🎯 Next Steps

1. **Integration**: Connect to mBot2 WebSocket server
2. **E2E Tests**: Run journey tests with live robot
3. **Deployment**: Build and deploy to production
4. **Mobile**: Responsive UI (Wave 7)

## 📚 Full Documentation

See: `web/src/README.md`
See: `docs/implementations/PERSONALITY_MIXER_IMPLEMENTATION.md`

## ✅ Definition of Done

- [x] TypeScript component implemented
- [x] All 9 sliders functional
- [x] 15 preset personalities
- [x] WebSocket messaging
- [x] localStorage persistence
- [x] Undo/redo stack
- [x] Connection handling
- [x] All data-testid attributes
- [x] Unit tests (15 suites)
- [x] Contract compliance
- [ ] E2E test passes (needs robot)
- [ ] Code review approved

## 🎉 Issue #58 Complete!

**Status**: ✅ Ready for review and integration
**Tests**: ✅ All unit tests passing
**Contracts**: ✅ All invariants enforced
**Documentation**: ✅ Complete
