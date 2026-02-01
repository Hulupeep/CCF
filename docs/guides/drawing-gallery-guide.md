# Drawing Gallery Guide

**Feature:** Wave 6 Sprint 1
**Component:** `DrawingGallery.tsx`
**Difficulty:** Beginner
**Time to Learn:** 10 minutes

## Overview

The Drawing Gallery displays all artworks created by your robot with full stroke-by-stroke playback animation, filtering, search, and export capabilities.

### Key Features

- Gallery grid with 200x200px thumbnails
- IndexedDB storage preserves all stroke data
- Stroke-by-stroke playback matching original speed
- Multi-filter system (mood, date, session, signature)
- Search by session ID or date text
- Export to PNG (1200x900), SVG (vector), JSON
- Delete with confirmation
- Pagination (20 items per page)
- Modal viewer with metadata

---

## Quick Start

### Basic Usage

```typescript
import { DrawingGallery } from '@/components/DrawingGallery';

function App() {
  return <DrawingGallery />;
}
```

### With Event Handlers

```typescript
import { DrawingGallery } from '@/components/DrawingGallery';

function App() {
  const handleDrawingSelect = (drawing: Drawing) => {
    console.log('Selected:', drawing.sessionId);
  };

  const handleDelete = (drawingId: string) => {
    console.log('Deleted:', drawingId);
  };

  return (
    <DrawingGallery
      onDrawingSelect={handleDrawingSelect}
      onDelete={handleDelete}
    />
  );
}
```

---

## Gallery Features

### Thumbnail View

Each thumbnail shows:
- **Preview**: 200x200px raster image
- **Mood**: Emoji indicator (😊 happy, 😔 sad, 😠 angry, 😐 neutral)
- **Date**: Human-readable timestamp
- **Duration**: How long the drawing took

```typescript
// Thumbnail data structure
interface DrawingThumbnail {
  id: string;
  sessionId: string;
  thumbnail: string; // Base64 data URL
  mood: 'happy' | 'sad' | 'angry' | 'neutral';
  createdAt: number;
  duration: number; // milliseconds
}
```

### Grid Layout

- **Default**: 4 columns on desktop, 2 on tablet, 1 on mobile
- **Responsive**: Automatically adjusts to screen size
- **Lazy Loading**: Only renders visible thumbnails
- **Infinite Scroll**: Loads more as you scroll down

---

## Playback Animation

### Stroke-by-Stroke Replay

Click any drawing to watch it being drawn:

```typescript
// Playback matches original timing exactly
const replayDrawing = (drawing: Drawing) => {
  drawing.strokes.forEach((stroke, index) => {
    setTimeout(() => {
      renderStroke(stroke);
    }, stroke.timestamp - drawing.strokes[0].timestamp);
  });
};
```

### Playback Controls

- **Play/Pause**: Start or stop playback
- **Speed**: 0.5x, 1x (original), 2x, 4x
- **Scrub**: Jump to any point in timeline
- **Loop**: Repeat continuously

```typescript
import { useDrawingPlayback } from '@/hooks/useDrawingPlayback';

function PlaybackControls() {
  const {
    playing,
    speed,
    progress,
    play,
    pause,
    setSpeed,
    scrub
  } = useDrawingPlayback(drawing);

  return (
    <div>
      <button onClick={playing ? pause : play}>
        {playing ? 'Pause' : 'Play'}
      </button>
      <select value={speed} onChange={(e) => setSpeed(Number(e.target.value))}>
        <option value={0.5}>0.5x</option>
        <option value={1}>1x</option>
        <option value={2}>2x</option>
        <option value={4}>4x</option>
      </select>
      <input
        type="range"
        min={0}
        max={100}
        value={progress}
        onChange={(e) => scrub(Number(e.target.value))}
      />
    </div>
  );
}
```

---

## Filtering & Search

### Filter by Mood

```typescript
// Filter options
type MoodFilter = 'all' | 'happy' | 'sad' | 'angry' | 'neutral';

const [filter, setFilter] = useState<MoodFilter>('all');

<DrawingGallery moodFilter={filter} />
```

### Filter by Date Range

```typescript
const [dateRange, setDateRange] = useState({
  start: Date.now() - 7 * 24 * 60 * 60 * 1000, // Last week
  end: Date.now()
});

<DrawingGallery
  startDate={dateRange.start}
  endDate={dateRange.end}
/>
```

### Search by Session ID

```typescript
const [searchTerm, setSearchTerm] = useState('');

<DrawingGallery searchQuery={searchTerm} />

// Searches:
// - Session ID
// - Drawing title (if set)
// - Date text (e.g., "january", "2026")
```

### Filter by Signature

Some drawings have a "signature" stroke or pattern:

```typescript
const [showOnlySigned, setShowOnlySigned] = useState(false);

<DrawingGallery filterSigned={showOnlySigned} />
```

---

## Export Features

### PNG Export (Raster)

Export as high-quality PNG image:

```typescript
import { exportDrawingToPNG } from '@/services/drawingExport';

async function exportPNG(drawing: Drawing) {
  const png = await exportDrawingToPNG(drawing, {
    width: 1200,
    height: 900,
    backgroundColor: '#FFFFFF',
    includeSignature: true
  });

  // Download file
  const link = document.createElement('a');
  link.href = png; // Data URL
  link.download = `drawing-${drawing.sessionId}.png`;
  link.click();
}
```

**Default Settings:**
- Size: 1200x900px
- Format: PNG
- Background: White
- DPI: 72

### SVG Export (Vector)

Export as scalable vector graphics:

```typescript
import { exportDrawingToSVG } from '@/services/drawingExport';

async function exportSVG(drawing: Drawing) {
  const svg = await exportDrawingToSVG(drawing, {
    preserveStrokes: true, // Keep stroke order
    includeMetadata: true, // Embed mood, timestamp, etc.
  });

  const blob = new Blob([svg], { type: 'image/svg+xml' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `drawing-${drawing.sessionId}.svg`;
  link.click();
}
```

**Benefits:**
- Infinite scalability
- Editable in vector programs
- Smaller file size
- Preserves stroke order

### JSON Export (Data)

Export raw stroke data for analysis:

```typescript
import { exportDrawingToJSON } from '@/services/drawingExport';

async function exportJSON(drawing: Drawing) {
  const json = exportDrawingToJSON(drawing);

  // JSON structure:
  {
    "sessionId": "abc123",
    "createdAt": 1738454321000,
    "duration": 45000,
    "mood": "happy",
    "strokes": [
      {
        "path": "M10,10 L20,20 L30,15",
        "timestamp": 1738454321100,
        "color": "#000000",
        "width": 2
      }
    ],
    "metadata": {
      "personality": { /* ... */ },
      "environment": "kitchen_table"
    }
  }
}
```

---

## Storage Management

### IndexedDB Structure

```typescript
// Database: 'mbot-drawings'
// Store: 'artworks'
// Index: 'by-date', 'by-mood', 'by-session'

interface DrawingRecord {
  id: string; // Primary key
  sessionId: string;
  createdAt: number; // Indexed
  mood: string; // Indexed
  strokes: Stroke[];
  thumbnail: string;
  duration: number;
  metadata: Record<string, any>;
}
```

### Storage Quotas

IndexedDB has browser-specific quotas:

```typescript
// Check available storage
const checkStorageQuota = async () => {
  if ('storage' in navigator && 'estimate' in navigator.storage) {
    const estimate = await navigator.storage.estimate();
    const percent = (estimate.usage! / estimate.quota!) * 100;
    console.log(`Using ${estimate.usage} of ${estimate.quota} (${percent.toFixed(1)}%)`);
  }
};
```

**Typical Limits:**
- Chrome: 60% of free disk space
- Firefox: 50% of free disk space
- Safari: 1GB

### Cleanup Strategies

```typescript
import { artworkStorage } from '@/services/artworkStorage';

// Delete old drawings
await artworkStorage.deleteOlderThan(90); // 90 days

// Delete by mood
await artworkStorage.deleteByMood('sad');

// Manual cleanup
const drawings = await artworkStorage.getAllDrawings();
const toDelete = drawings.filter(d => d.duration < 5000); // Less than 5 seconds
await Promise.all(toDelete.map(d => artworkStorage.deleteDrawing(d.id)));
```

---

## Configuration

### Component Props

```typescript
interface DrawingGalleryProps {
  itemsPerPage?: number; // Default: 20
  columns?: number; // Grid columns (default: auto)
  moodFilter?: MoodFilter; // Filter by mood
  startDate?: number; // Filter start date
  endDate?: number; // Filter end date
  searchQuery?: string; // Search term
  filterSigned?: boolean; // Show only signed drawings
  showMetadata?: boolean; // Show metadata in modal (default: true)
  allowDelete?: boolean; // Enable delete (default: true)
  allowExport?: boolean; // Enable export (default: true)
  onDrawingSelect?: (drawing: Drawing) => void;
  onDelete?: (drawingId: string) => void;
  onExport?: (drawing: Drawing, format: 'png' | 'svg' | 'json') => void;
  className?: string;
}
```

### Performance Options

```typescript
// For large galleries (1000+ drawings)
<DrawingGallery
  itemsPerPage={10} // Fewer items per page
  lazyLoad={true} // Only render visible thumbnails
  thumbnailQuality="low" // Use lower quality thumbnails
/>
```

---

## Troubleshooting

### Gallery shows "No drawings"

**Problem:** No drawings appear
**Solutions:**

```typescript
// 1. Verify drawings exist in IndexedDB
import { artworkStorage } from '@/services/artworkStorage';
const count = await artworkStorage.getDrawingCount();
console.log(`${count} drawings in storage`);

// 2. Check filters
// Remove all filters to see all drawings
<DrawingGallery moodFilter="all" searchQuery="" />
```

### Playback doesn't match original speed

**Problem:** Animation is too fast or slow
**Solution:**

```typescript
// Contract: I-ART-GAL-002
// Playback MUST match original timing exactly

// Verify stroke timestamps are preserved:
const drawing = await artworkStorage.getDrawing(id);
console.log('First stroke:', drawing.strokes[0].timestamp);
console.log('Last stroke:', drawing.strokes[drawing.strokes.length - 1].timestamp);
console.log('Duration:', drawing.duration);

// If timestamps are wrong, strokes weren't captured correctly
```

### Export fails or produces blank images

**Problem:** Exported PNG/SVG is empty
**Solutions:**

```typescript
// 1. Check canvas rendering
const canvas = document.createElement('canvas');
canvas.width = 1200;
canvas.height = 900;
const ctx = canvas.getContext('2d');
// Verify ctx is not null

// 2. Verify stroke data exists
if (drawing.strokes.length === 0) {
  console.error('No strokes to export!');
}

// 3. Try different export format
// If PNG fails, try SVG or JSON
```

### IndexedDB quota exceeded

**Problem:** Can't save new drawings
**Solutions:**

```typescript
// 1. Check storage usage
const estimate = await navigator.storage.estimate();
console.log('Storage used:', estimate.usage);

// 2. Delete old drawings
await artworkStorage.deleteOlderThan(30); // Keep only last 30 days

// 3. Export and clear
const allDrawings = await artworkStorage.getAllDrawings();
// Export to JSON file for backup
await artworkStorage.clearAll();
```

---

## Examples

### Example 1: Simple Gallery

```typescript
import { DrawingGallery } from '@/components/DrawingGallery';

export default function ArtGalleryPage() {
  return (
    <div className="container">
      <h1>Robot Artwork Gallery</h1>
      <DrawingGallery />
    </div>
  );
}
```

### Example 2: Filtered Gallery

```typescript
import { DrawingGallery } from '@/components/DrawingGallery';
import { useState } from 'react';

export default function FilteredGalleryPage() {
  const [mood, setMood] = useState<MoodFilter>('all');
  const [searchTerm, setSearchTerm] = useState('');

  return (
    <div>
      <div className="filters">
        <select value={mood} onChange={(e) => setMood(e.target.value as MoodFilter)}>
          <option value="all">All Moods</option>
          <option value="happy">Happy</option>
          <option value="sad">Sad</option>
          <option value="angry">Angry</option>
          <option value="neutral">Neutral</option>
        </select>
        <input
          placeholder="Search..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
      </div>
      <DrawingGallery
        moodFilter={mood}
        searchQuery={searchTerm}
      />
    </div>
  );
}
```

### Example 3: Gallery with Analytics

```typescript
import { DrawingGallery } from '@/components/DrawingGallery';
import { useEffect, useState } from 'react';
import { artworkStorage } from '@/services/artworkStorage';

export default function AnalyticsGalleryPage() {
  const [stats, setStats] = useState({
    total: 0,
    byMood: { happy: 0, sad: 0, angry: 0, neutral: 0 },
    avgDuration: 0,
  });

  useEffect(() => {
    const loadStats = async () => {
      const drawings = await artworkStorage.getAllDrawings();
      const moodCounts = drawings.reduce((acc, d) => {
        acc[d.mood] = (acc[d.mood] || 0) + 1;
        return acc;
      }, {} as Record<string, number>);

      const totalDuration = drawings.reduce((sum, d) => sum + d.duration, 0);

      setStats({
        total: drawings.length,
        byMood: moodCounts as any,
        avgDuration: totalDuration / drawings.length,
      });
    };

    loadStats();
  }, []);

  return (
    <div>
      <div className="stats">
        <h2>Gallery Statistics</h2>
        <p>Total Drawings: {stats.total}</p>
        <p>Happy: {stats.byMood.happy}</p>
        <p>Sad: {stats.byMood.sad}</p>
        <p>Angry: {stats.byMood.angry}</p>
        <p>Neutral: {stats.byMood.neutral}</p>
        <p>Avg Duration: {(stats.avgDuration / 1000).toFixed(1)}s</p>
      </div>
      <DrawingGallery />
    </div>
  );
}
```

---

## Related Features

- [Personality Mixer](personality-mixer-guide.md) - Affects drawing style
- [Neural Visualizer](neural-visualizer-guide.md) - Brain state during drawing
- [Data Export/Import](data-export-import-guide.md) - Backup gallery
- [Cloud Sync](cloud-sync-guide.md) - Sync gallery across devices

---

## API Reference

See: [Drawing Gallery API](../api/WAVE_6_APIs.md#drawing-gallery)

---

**Last Updated:** 2026-02-01
**Status:** Production Ready ✅
