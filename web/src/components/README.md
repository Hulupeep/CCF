# Neural Visualizer Component

Real-time visualization of mBot2 RuVector's nervous system state.

## Features

### ✅ Implemented (Issue #59)

1. **Real-time Meters** (I-LEARN-VIZ-001: 10Hz+ update rate)
   - Tension meter (red intensity color-coding)
   - Energy meter (blue intensity color-coding)
   - Coherence meter
   - Curiosity meter
   - Smooth transitions with easing functions

2. **Animated Timeline**
   - 60-second visible window
   - 300-second data retention (I-LEARN-VIZ-002)
   - Mode transition markers
   - Red line for tension, blue line for energy
   - Interactive scrubbing

3. **Mode Indicator**
   - Visual emoji icons (😌🤔⚡🛡️)
   - Color-coded by mode
   - Calm, Active, Spike, Protect states

4. **Timeline Scrubber**
   - Drag to review past 60 seconds
   - Pause/play controls
   - Shows scrub position indicator

5. **Zoom/Pan Controls**
   - Zoom in/out buttons
   - Reset zoom button
   - Smooth scaling transitions

6. **Export Functionality**
   - CSV export with all data points
   - JSON export with full state history
   - Timestamped filenames

7. **Stimulus Response**
   - Flash effects on mode changes
   - Visual feedback for state transitions

8. **WebSocket Integration**
   - 20Hz update rate (50ms interval)
   - Auto-reconnection on disconnect
   - Connection status indicator

9. **Performance**
   - 60fps rendering capability
   - High-DPI display support
   - Canvas-based rendering for efficiency

## Usage

### Development

```bash
# Start the server
npm start

# Open browser
open http://localhost:3000/visualizer.html
```

### Production Build

```bash
# Build TypeScript component
npm run build

# TypeScript type checking
npm run typecheck
```

### Testing

```bash
# Run unit tests
npm test

# Run tests in watch mode
npm test:watch

# Run E2E journey tests (from project root)
npm run test:journeys -- tests/journeys/learninglab-experiment.journey.spec.ts
```

## File Structure

```
web/
├── src/
│   ├── components/
│   │   ├── NeuralVisualizer.tsx          # Main component
│   │   └── __tests__/
│   │       └── NeuralVisualizer.test.tsx # Unit tests
│   ├── hooks/
│   │   └── useWebSocket.ts               # WebSocket hook
│   ├── types/
│   │   └── neural.ts                     # Type definitions
│   └── utils/
│       └── canvasRenderer.ts             # Canvas rendering utilities
├── public/
│   └── visualizer.html                   # HTML page
├── server.js                             # WebSocket server
├── package.json
├── tsconfig.json
├── webpack.config.js
└── jest.config.js
```

## Data Contract

```typescript
interface NeuralState {
  timestamp: number;
  mode: 'Calm' | 'Active' | 'Spike' | 'Protect';
  tension: number;      // 0-1, red intensity
  coherence: number;    // 0-1
  energy: number;       // 0-1, blue intensity
  curiosity: number;    // 0-1
}
```

## WebSocket Protocol

### Server → Client (20Hz)

```json
{
  "mode": "Active",
  "tension": 0.45,
  "coherence": 0.82,
  "energy": 0.91,
  "curiosity": 0.63,
  "distance": 45.2,
  "gyro": 5.3,
  "sound": 0.18,
  "light": 0.62
}
```

## Invariants

### I-LEARN-VIZ-001: Update Rate
**MUST** Update at minimum 10Hz (100ms).
- Implementation: WebSocket receives updates at 20Hz (50ms)
- Component handles updates without frame drops

### I-LEARN-VIZ-002: Data Retention
**MUST** Store last 300 seconds (5 min).
- Implementation: `maxHistorySeconds` prop defaults to 300
- Old data automatically pruned from history array

## Testing

### Unit Tests
- All data-testid elements present
- WebSocket connection handling
- Play/pause functionality
- Timeline scrubbing
- Zoom controls
- Export CSV/JSON
- Color coding
- Performance (60fps capability)

### Journey Tests
Located in: `tests/journeys/learninglab-experiment.journey.spec.ts`

Tests complete user journey:
1. View live neural activity
2. Observe mode transitions
3. Review historical data
4. Export data
5. Use zoom/pan controls
6. Verify color coding
7. Check stimulus flash effects
8. Validate data retention
9. Measure update rate performance
10. Test reconnection
11. Mode indicator updates
12. JSON export

## Contract References

- **Feature Contracts:** [LEARN-001, LEARN-002, LEARN-004]
- **Journey Contract:** [J-LEARN-FIRST-EXPERIMENT]
- **Epics:** [EPIC-LEARN]

## Definition of Done

- [x] Real-time meters implemented
- [x] Timeline with transitions
- [x] Export functionality
- [x] 60fps performance
- [x] E2E test passes
- [x] All data-testid attributes present
- [x] WebSocket 20Hz updates
- [x] Zoom/pan controls
- [x] Timeline scrubber
- [x] Color-coding (red=tension, blue=energy)
- [x] Stimulus flash effects
- [x] I-LEARN-VIZ-001 compliance (10Hz+ updates)
- [x] I-LEARN-VIZ-002 compliance (300s retention)

## Browser Compatibility

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

Requires:
- Canvas API
- WebSocket API
- ES2020 features

## Performance Characteristics

- **Update Rate:** 20Hz (50ms intervals)
- **Render Rate:** 60fps (16.67ms frame time)
- **Memory Usage:** ~2MB for 300s history
- **Canvas Rendering:** Hardware accelerated
- **High-DPI:** Automatic pixel ratio detection

## Future Enhancements

Not in scope for #59:
- Long-term database storage
- Statistical analysis tools
- Multi-robot monitoring
- 3D visualization with Three.js
- Custom color themes
- Configurable time windows
- Alert thresholds
- Video recording of sessions
