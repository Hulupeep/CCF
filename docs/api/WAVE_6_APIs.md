# Wave 6 API Reference

**Version:** 1.0.0
**Last Updated:** 2026-02-01

## Table of Contents

1. [Personality Mixer API](#personality-mixer-api)
2. [Neural Visualizer API](#neural-visualizer-api)
3. [Drawing Gallery API](#drawing-gallery-api)
4. [Game Statistics API](#game-statistics-api)
5. [Inventory Dashboard API](#inventory-dashboard-api)
6. [Personality Persistence API](#personality-persistence-api)
7. [WebSocket V2 API](#websocket-v2-api)
8. [Multi-Robot Discovery API](#multi-robot-discovery-api)
9. [Data Export/Import API](#data-export-import-api)

---

## Personality Mixer API

### PersonalityStore

```typescript
interface PersonalityStore {
  // Get current personality
  getCurrentPersonality(): PersonalityConfig;

  // Update single parameter
  updateParameter(key: PersonalityKey, value: number): void;

  // Update entire personality
  updatePersonality(personality: PersonalityConfig): void;

  // Subscribe to changes
  subscribe(callback: (personality: PersonalityConfig) => void): () => void;

  // Preset management
  savePreset(name: string, personality?: PersonalityConfig): void;
  loadPreset(name: string): PersonalityConfig | null;
  listPresets(): string[];
  deletePreset(name: string): void;

  // Persistence
  save(): void;
  load(): void;
  clear(): void;
}
```

### PersonalityConfig

```typescript
interface PersonalityConfig {
  curiosity: number; // 0.0-1.0
  energy: number; // 0.0-1.0
  playfulness: number; // 0.0-1.0
  patience: number; // 0.0-1.0
  focus: number; // 0.0-1.0
  sociability: number; // 0.0-1.0
  boldness: number; // 0.0-1.0
  adaptability: number; // 0.0-1.0
  expressiveness: number; // 0.0-1.0
}
```

---

## Neural Visualizer API

### NeuralData Types

```typescript
interface NeuralDataPoint {
  timestamp: number;
  tension: number; // 0.0-1.0
  energy: number; // 0.0-1.0
  coherence: number; // 0.0-1.0
  curiosity: number; // 0.0-1.0
}

interface Stimulus {
  type: 'sound' | 'touch' | 'visual' | 'custom';
  intensity: number; // 0.0-1.0
  timestamp: number;
  metadata?: Record<string, any>;
}
```

### Export Functions

```typescript
function exportNeuralData(
  data: NeuralDataPoint[],
  format: 'json' | 'csv'
): Promise<string>;

function getNeuralHistory(
  duration?: number // milliseconds, default 300000 (5 min)
): NeuralDataPoint[];
```

---

## Drawing Gallery API

### ArtworkStorage

```typescript
interface ArtworkStorage {
  // CRUD operations
  saveDrawing(drawing: Drawing): Promise<string>; // Returns ID
  getDrawing(id: string): Promise<Drawing | null>;
  getAllDrawings(): Promise<Drawing[]>;
  deleteDrawing(id: string): Promise<void>;

  // Filtering
  getDrawingsByMood(mood: Mood): Promise<Drawing[]>;
  getDrawingsByDateRange(start: number, end: number): Promise<Drawing[]>;
  searchDrawings(query: string): Promise<Drawing[]>;

  // Maintenance
  getDrawingCount(): Promise<number>;
  deleteOlderThan(days: number): Promise<number>; // Returns count deleted
  clearAll(): Promise<void>;
}
```

### Drawing Types

```typescript
interface Drawing {
  id: string;
  sessionId: string;
  createdAt: number;
  duration: number; // milliseconds
  mood: 'happy' | 'sad' | 'angry' | 'neutral';
  strokes: Stroke[];
  thumbnail: string; // Base64 data URL
  metadata: DrawingMetadata;
}

interface Stroke {
  path: string; // SVG path data
  timestamp: number;
  color: string; // hex color
  width: number; // pixels
}
```

---

## Game Statistics API

### GameStorage

```typescript
interface GameStorage {
  // Record game
  recordGame(game: GameResult): Promise<void>;

  // Get statistics
  getStats(filter?: StatsFilter): Promise<GameStatistics>;
  getLeaderboard(limit?: number): Promise<LeaderboardEntry[]>;
  getAchievements(): Promise<Achievement[]>;

  // Export
  exportStats(format: 'json' | 'csv'): Promise<string>;
}
```

### Game Types

```typescript
interface GameResult {
  game: 'tictactoe' | 'chase' | 'simon';
  result: 'win' | 'loss' | 'draw';
  score: number;
  duration: number; // milliseconds
  personality: PersonalityConfig;
  timestamp: number;
}

interface Achievement {
  id: string;
  name: string;
  description: string;
  category: 'games' | 'scores' | 'streaks' | 'special';
  unlocked: boolean;
  unlockedAt?: number;
  progress: number; // 0.0-1.0
}
```

---

## Inventory Dashboard API

### InventoryStorage

```typescript
interface InventoryStorage {
  // Get state
  getInventory(): Promise<InventoryState>;
  getStation(color: StationColor): Promise<Station>;

  // Update
  updateStation(color: StationColor, count: number): Promise<void>;
  adjustStation(color: StationColor, delta: number, reason: string): Promise<void>;

  // History
  getHistoricalData(
    color: StationColor,
    period: 'daily' | 'weekly'
  ): Promise<HistoricalSnapshot[]>;

  // Export/Import
  exportInventory(): Promise<InventoryExport>;
  importInventory(data: InventoryExport): Promise<void>;
}
```

### Inventory Types

```typescript
interface Station {
  color: 'red' | 'green' | 'blue' | 'yellow';
  count: number;
  lowStockThreshold: number;
  criticalThreshold: number;
  lastSync: number;
  lastAdjustment?: Adjustment;
}

interface Adjustment {
  timestamp: number;
  delta: number;
  reason: string;
  previousCount: number;
  newCount: number;
}
```

---

## Personality Persistence API

### Singleton Pattern

```typescript
// Only one export, only one instance (I-ARCH-PERS-001)
export const personalityStore: PersonalityStore;
```

### Atomic Updates

```typescript
// Contract I-ARCH-PERS-002: No partial states
// All parameters MUST be provided together
personalityStore.updatePersonality({
  curiosity: 0.8,
  energy: 0.7,
  playfulness: 0.6,
  patience: 0.5,
  focus: 0.9,
  sociability: 0.4,
  boldness: 0.3,
  adaptability: 0.7,
  expressiveness: 0.6,
}); // ✅ Valid

personalityStore.updatePersonality({
  curiosity: 0.8,
  // Missing other parameters
}); // ❌ TypeScript error
```

---

## WebSocket V2 API

### Message Protocol

```typescript
interface WebSocketMessage {
  type: 'state' | 'command' | 'event' | 'batch' | 'ping' | 'pong';
  version: 2;
  payload: any;
  timestamp: number;
  sequence?: number; // For ordering (I-WS-V2-002)
}
```

### State Snapshot

```typescript
interface StateSnapshot {
  personality: PersonalityConfig;
  neuralState: NeuralDataPoint;
  inventory: InventoryState;
  gameState?: GameState;
  timestamp: number;
}
```

### React Hook

```typescript
function useWebSocketV2(url: string, options?: WebSocketOptions): {
  connected: boolean;
  reconnecting: boolean;
  reconnectAttempts: number;
  state: StateSnapshot | null;
  sendCommand: (command: string, payload?: any) => void;
  subscribe: (event: string, callback: (data: any) => void) => () => void;
  reconnect: () => void;
};
```

---

## Multi-Robot Discovery API

### RobotDiscovery Service

```typescript
interface RobotDiscovery {
  // Discovery
  startDiscovery(): void;
  stopDiscovery(): void;
  getRobots(): Robot[];

  // Connection
  connect(robotId: string): Promise<WebSocket>;
  disconnect(robotId: string): void;

  // Events
  on(event: 'robot_found' | 'robot_lost' | 'robot_updated', callback: (robot: Robot) => void): void;
}
```

### Robot Type

```typescript
interface Robot {
  id: string;
  name: string;
  ip: string;
  port: number;
  version: string;
  status: 'connected' | 'disconnected' | 'error' | 'discovering';
  lastSeen: number;
  metadata?: Record<string, any>;
}
```

---

## Data Export/Import API

### Export Service

```typescript
interface DataExport {
  // Export functions
  exportAllData(): Promise<ExportManifest>;
  exportData(types: DataType[]): Promise<ExportManifest>;
  exportToCSV(type: DataType): Promise<string>;

  // Backup management
  createBackup(name: string): Promise<string>; // Returns backup ID
  listBackups(): Promise<BackupInfo[]>;
  deleteBackup(id: string): Promise<void>;
}
```

### Import Service

```typescript
interface DataImport {
  // Import functions
  importFromManifest(manifest: ExportManifest, options: ImportOptions): Promise<ImportResult>;
  importFromCSV(csv: string, type: DataType): Promise<ImportResult>;

  // Validation
  validateManifest(manifest: ExportManifest): ValidationResult;

  // Restore
  restoreBackup(id: string): Promise<void>;
}
```

### Types

```typescript
type DataType = 'personality' | 'drawings' | 'stats' | 'inventory';

interface ExportManifest {
  version: string;
  exportedAt: number;
  dataTypes: DataType[];
  data: {
    personalities?: PersonalityConfig[];
    drawings?: Drawing[];
    stats?: GameStatistics;
    inventory?: InventoryState;
  };
}

interface ImportOptions {
  strategy: 'merge' | 'overwrite';
  skipInvalid: boolean;
  validateSchema: boolean;
}

interface ImportResult {
  success: boolean;
  importedCount: number;
  skippedCount: number;
  errors: string[];
}
```

---

## Contract References

All APIs enforce these contracts:

| Contract | Description | APIs Affected |
|----------|-------------|---------------|
| **I-PERS-001** | Parameters bounded [0.0, 1.0] | Personality Mixer, Persistence |
| **I-PERS-UI-001** | UI sliders bounded | Personality Mixer |
| **I-PERS-UI-002** | 500ms debouncing | Personality Mixer |
| **I-ARCH-PERS-001** | Singleton pattern | Personality Persistence |
| **I-ARCH-PERS-002** | Atomic updates | Personality Persistence |
| **I-WS-V2-001** | State consistency | WebSocket V2 |
| **I-WS-V2-002** | Message ordering | WebSocket V2 |
| **I-DISC-001** | mDNS RFC 6762 | Multi-Robot Discovery |
| **SORT-004** | Inventory persistence | Inventory Dashboard |
| **I-SORT-INV-001** | NFC sync ≤5s | Inventory Dashboard |

---

**Last Updated:** 2026-02-01
**Status:** Production Ready ✅
