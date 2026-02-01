# Neural Visualizer Guide

**Feature:** Wave 6 Sprint 1
**Component:** `NeuralVisualizer.tsx`
**Difficulty:** Intermediate
**Time to Learn:** 15 minutes

## Overview

The Neural Visualizer provides real-time visualization of your robot's "brain state" through four animated meters showing tension, energy, coherence, and curiosity levels at 60fps.

### Key Features

- Real-time Canvas rendering at 60fps
- 4 animated meters with color-coding
- 60-second scrolling timeline
- 300-second data retention
- Interactive scrubber for time travel
- Zoom/pan controls (0.5x-3x)
- Export to CSV/JSON
- Stimulus response flash effects
- High-DPI retina display support

---

## Quick Start

### Basic Usage

```typescript
import { NeuralVisualizer } from '@/components/NeuralVisualizer';

function App() {
  return <NeuralVisualizer wsUrl="ws://localhost:4000" />;
}
```

### With Event Handlers

```typescript
import { NeuralVisualizer } from '@/components/NeuralVisualizer';

function App() {
  const handleStimulusDetected = (stimulus: string) => {
    console.log('Robot reacted to:', stimulus);
  };

  const handleDataExport = (data: NeuralDataPoint[]) => {
    console.log(`Exported ${data.length} data points`);
  };

  return (
    <NeuralVisualizer
      wsUrl="ws://localhost:4000"
      onStimulusDetected={handleStimulusDetected}
      onDataExport={handleDataExport}
    />
  );
}
```

---

## The 4 Meters

### 1. Tension (Red)

**What it measures:** Internal "stress" or conflict level

- **Low (0.0-0.3)**: Calm, relaxed state
- **Medium (0.3-0.7)**: Active processing
- **High (0.7-1.0)**: High cognitive load

**Visual:** Red bar with pulse animation

**Example:** Tension rises when robot faces difficult sorting decisions

### 2. Energy (Blue)

**What it measures:** Current activity level

- **Low (0.0-0.3)**: Idle or minimal activity
- **Medium (0.3-0.7)**: Normal operation
- **High (0.7-1.0)**: High activity, fast movements

**Visual:** Blue bar with smooth transitions

**Example:** Energy spikes during active gameplay

### 3. Coherence (Green)

**What it measures:** How well subsystems are coordinated

- **Low (0.0-0.3)**: Subsystems out of sync
- **Medium (0.3-0.7)**: Moderate coordination
- **High (0.7-1.0)**: Perfect synchronization

**Visual:** Green bar with stability indicator

**Example:** High coherence during smooth drawing motions

### 4. Curiosity (Purple)

**What it measures:** Exploration drive strength

- **Low (0.0-0.3)**: Routine behavior
- **Medium (0.3-0.7)**: Some exploration
- **High (0.7-1.0)**: Active exploration mode

**Visual:** Purple bar with search animation

**Example:** Curiosity high when encountering new objects

---

## Timeline Features

### Scrolling Window

- **Visible**: Last 60 seconds
- **Retention**: Last 300 seconds (5 minutes)
- **Update Rate**: 20Hz (50ms intervals)

### Time Travel Scrubber

```typescript
// Scrub to specific timestamp
const handleScrub = (timestamp: number) => {
  // View historical data at this moment
  const dataPoint = getNeuralDataAtTime(timestamp);
  console.log('State at', new Date(timestamp), ':', dataPoint);
};

<NeuralVisualizer onScrub={handleScrub} />
```

### Zoom Controls

```typescript
// Zoom levels: 0.5x, 1x, 2x, 3x
const [zoom, setZoom] = useState(1);

// 0.5x = Show 120 seconds (2 minutes)
// 1x = Show 60 seconds (default)
// 2x = Show 30 seconds
// 3x = Show 20 seconds

<NeuralVisualizer zoom={zoom} onZoomChange={setZoom} />
```

---

## Stimulus Response

### Flash Effects

When robot detects a stimulus (sound, touch, visual), meters flash:

- **Sound**: All meters flash blue (100ms)
- **Touch**: Meters flash green (100ms)
- **Visual**: Meters flash yellow (100ms)
- **Custom**: Configurable color/duration

### Detecting Stimuli

```typescript
import { useWebSocket } from '@/hooks/useWebSocket';

function StimulusMonitor() {
  const { messages } = useWebSocket('ws://localhost:4000');

  useEffect(() => {
    const lastMessage = messages[messages.length - 1];
    if (lastMessage?.type === 'stimulus') {
      console.log('Stimulus:', lastMessage.payload);
      // { type: 'sound', intensity: 0.8, timestamp: ... }
    }
  }, [messages]);

  return <NeuralVisualizer wsUrl="ws://localhost:4000" />;
}
```

---

## Data Export

### Export Formats

#### CSV Format

```csv
timestamp,tension,energy,coherence,curiosity
1738454321000,0.45,0.67,0.82,0.34
1738454321050,0.46,0.68,0.81,0.35
...
```

#### JSON Format

```json
{
  "exportedAt": 1738454400000,
  "duration": 300000,
  "dataPoints": [
    {
      "timestamp": 1738454321000,
      "tension": 0.45,
      "energy": 0.67,
      "coherence": 0.82,
      "curiosity": 0.34
    }
  ],
  "statistics": {
    "avgTension": 0.52,
    "avgEnergy": 0.61,
    "avgCoherence": 0.78,
    "avgCuriosity": 0.41
  }
}
```

### Exporting Data

```typescript
import { exportNeuralData } from '@/utils/neuralExport';

async function exportData() {
  // Get last 5 minutes of data
  const data = getNeuralHistory();

  // Export as CSV
  const csv = await exportNeuralData(data, 'csv');
  downloadFile(csv, 'neural-data.csv', 'text/csv');

  // Export as JSON
  const json = await exportNeuralData(data, 'json');
  downloadFile(json, 'neural-data.json', 'application/json');
}
```

---

## Configuration

### Component Props

```typescript
interface NeuralVisualizerProps {
  wsUrl?: string; // WebSocket URL
  updateRate?: number; // Hz (default: 20)
  retentionTime?: number; // Seconds (default: 300)
  visibleWindow?: number; // Seconds (default: 60)
  showTimeline?: boolean; // Show timeline (default: true)
  showScrubber?: boolean; // Show time scrubber (default: true)
  showZoom?: boolean; // Show zoom controls (default: true)
  showExport?: boolean; // Show export button (default: true)
  zoom?: number; // Initial zoom level (default: 1)
  onStimulusDetected?: (stimulus: Stimulus) => void;
  onDataExport?: (data: NeuralDataPoint[]) => void;
  onScrub?: (timestamp: number) => void;
  className?: string;
}
```

### Performance Tuning

```typescript
// For low-end devices
<NeuralVisualizer
  updateRate={10} // 10Hz instead of 20Hz
  retentionTime={180} // 3 minutes instead of 5
  visibleWindow={30} // 30 seconds instead of 60
/>

// For high-end devices
<NeuralVisualizer
  updateRate={30} // 30Hz for smoother animation
  retentionTime={600} // 10 minutes of history
  visibleWindow={120} // 2-minute view
/>
```

---

## Canvas Rendering

### High-DPI Support

Automatically detects and supports retina displays:

```typescript
// Automatic pixel ratio detection
const pixelRatio = window.devicePixelRatio || 1;
canvas.width = width * pixelRatio;
canvas.height = height * pixelRatio;
context.scale(pixelRatio, pixelRatio);
```

### 60fps Animation Loop

```typescript
// Internal rendering loop
function animate() {
  const now = performance.now();
  const delta = now - lastFrame;

  if (delta >= 16.67) { // ~60fps
    renderMeters();
    renderTimeline();
    renderScrubber();
    lastFrame = now;
  }

  requestAnimationFrame(animate);
}
```

---

## Troubleshooting

### Low frame rate

**Problem:** Animation stutters or drops below 60fps
**Solutions:**

```typescript
// 1. Reduce update rate
<NeuralVisualizer updateRate={10} />

// 2. Disable hardware acceleration
// In browser settings, disable "Use hardware acceleration"

// 3. Reduce retention time
<NeuralVisualizer retentionTime={120} />
```

### WebSocket data not showing

**Problem:** Meters show zero values
**Solutions:**

```typescript
// 1. Verify WebSocket connection
const { connected } = useWebSocket('ws://localhost:4000');
console.log('Connected:', connected);

// 2. Check message format
// Server MUST send messages matching:
{
  type: 'neural_state',
  payload: {
    tension: 0.5,
    energy: 0.7,
    coherence: 0.8,
    curiosity: 0.4
  },
  timestamp: Date.now()
}
```

### Export fails

**Problem:** Export button doesn't work
**Solutions:**

```typescript
// 1. Check browser permissions
// Some browsers block downloads from localhost

// 2. Use manual export
const data = getNeuralHistory();
console.log(JSON.stringify(data, null, 2));
// Copy/paste output

// 3. Save to localStorage first
localStorage.setItem('neural-export', JSON.stringify(data));
```

### Memory leak on long sessions

**Problem:** Browser slows down after 30+ minutes
**Solutions:**

```typescript
// 1. Reduce retention time
<NeuralVisualizer retentionTime={180} />

// 2. Manually clear old data
const clearOldData = () => {
  // Keeps only last 60 seconds
  neuralDataStore.pruneOlderThan(60);
};

// Call every 5 minutes
setInterval(clearOldData, 300000);
```

---

## Examples

### Example 1: Basic Monitor

```typescript
import { NeuralVisualizer } from '@/components/NeuralVisualizer';

export default function BrainMonitor() {
  return (
    <div className="container">
      <h1>Robot Brain Activity</h1>
      <NeuralVisualizer wsUrl="ws://localhost:4000" />
    </div>
  );
}
```

### Example 2: With Statistics

```typescript
import { NeuralVisualizer } from '@/components/NeuralVisualizer';
import { useState, useEffect } from 'react';

export default function DetailedMonitor() {
  const [stats, setStats] = useState({
    avgTension: 0,
    avgEnergy: 0,
    avgCoherence: 0,
    avgCuriosity: 0,
  });

  const handleDataUpdate = (data: NeuralDataPoint[]) => {
    // Calculate averages
    const sum = data.reduce((acc, point) => ({
      tension: acc.tension + point.tension,
      energy: acc.energy + point.energy,
      coherence: acc.coherence + point.coherence,
      curiosity: acc.curiosity + point.curiosity,
    }), { tension: 0, energy: 0, coherence: 0, curiosity: 0 });

    setStats({
      avgTension: sum.tension / data.length,
      avgEnergy: sum.energy / data.length,
      avgCoherence: sum.coherence / data.length,
      avgCuriosity: sum.curiosity / data.length,
    });
  };

  return (
    <div>
      <NeuralVisualizer
        wsUrl="ws://localhost:4000"
        onDataExport={handleDataUpdate}
      />
      <div className="stats">
        <p>Avg Tension: {stats.avgTension.toFixed(2)}</p>
        <p>Avg Energy: {stats.avgEnergy.toFixed(2)}</p>
        <p>Avg Coherence: {stats.avgCoherence.toFixed(2)}</p>
        <p>Avg Curiosity: {stats.avgCuriosity.toFixed(2)}</p>
      </div>
    </div>
  );
}
```

### Example 3: Experiment Logger

```typescript
import { NeuralVisualizer } from '@/components/NeuralVisualizer';
import { useState } from 'react';

export default function ExperimentLogger() {
  const [experiment, setExperiment] = useState({
    name: '',
    started: false,
    data: [],
  });

  const startExperiment = (name: string) => {
    setExperiment({
      name,
      started: true,
      data: [],
    });
  };

  const stopExperiment = async () => {
    // Export experiment data
    const json = JSON.stringify({
      experiment: experiment.name,
      data: experiment.data,
      timestamp: Date.now(),
    }, null, 2);

    // Download file
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `experiment-${experiment.name}.json`;
    a.click();

    setExperiment({ name: '', started: false, data: [] });
  };

  return (
    <div>
      <input
        placeholder="Experiment name"
        onChange={(e) => setExperiment({ ...experiment, name: e.target.value })}
      />
      <button onClick={() => startExperiment(experiment.name)}>
        Start Experiment
      </button>
      {experiment.started && (
        <button onClick={stopExperiment}>
          Stop & Export
        </button>
      )}
      <NeuralVisualizer
        wsUrl="ws://localhost:4000"
        onDataExport={(data) => {
          if (experiment.started) {
            setExperiment({ ...experiment, data });
          }
        }}
      />
    </div>
  );
}
```

---

## Related Features

- [Performance Dashboard](performance-benchmarking-guide.md) - System metrics
- [Learning from Play](learning-from-play-guide.md) - AI training
- [Predictive Behavior](predictive-behavior-guide.md) - Behavior prediction
- [WebSocket V2](websocket-v2-guide.md) - Real-time protocol

---

## API Reference

See: [Neural Visualizer API](../api/WAVE_6_APIs.md#neural-visualizer)

---

**Last Updated:** 2026-02-01
**Status:** Production Ready ✅
