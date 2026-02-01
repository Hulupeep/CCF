# mBot Mobile App - Architecture

Complete architecture documentation for the React Native mobile application.

**Issue:** #88 (STORY-MOBILE-001)

## Overview

The mBot mobile app is built with React Native and Expo, providing cross-platform mobile control for mBot robots.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      React Native App                        │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                    UI Layer (Screens)                   │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │ │
│  │  │Discovery │ │Personality│ │  Neural  │ │ Gallery  │  │ │
│  │  │  Screen  │ │  Mixer    │ │Visualizer│ │  Screen  │  │ │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │ │
│  └───────┼────────────┼────────────┼────────────┼────────┘ │
│          │            │            │            │           │
│  ┌───────▼────────────▼────────────▼────────────▼────────┐ │
│  │                    React Hooks Layer                   │ │
│  │  useAppService | useRobotDiscovery | useNeuralState   │ │
│  │  usePersonality | useGallery | useSettings            │ │
│  └────────────────────────┬───────────────────────────────┘ │
│                           │                                  │
│  ┌────────────────────────▼───────────────────────────────┐ │
│  │              MobileAppService (Orchestrator)           │ │
│  └────┬─────────────┬─────────────┬─────────────┬─────────┘ │
│       │             │             │             │            │
│  ┌────▼────┐  ┌────▼────┐  ┌────▼────┐  ┌────▼─────┐     │
│  │WebSocket│  │Discovery│  │  Cache  │  │Components│     │
│  │ Client  │  │ Service │  │ Service │  │          │     │
│  └────┬────┘  └────┬────┘  └────┬────┘  └──────────┘     │
│       │            │             │                          │
└───────┼────────────┼─────────────┼──────────────────────────┘
        │            │             │
        ▼            ▼             ▼
   WebSocket      mDNS        AsyncStorage
  192.168.1.x   Network        (Cache)
    :8080

        │
        ▼
┌───────────────────┐
│   mBot Robot      │
│  (Data Plane)     │
│                   │
│  ┌─────────────┐  │
│  │  WebSocket  │  │
│  │   Server    │  │
│  └─────────────┘  │
│                   │
│  ┌─────────────┐  │
│  │   Neural    │  │
│  │   System    │  │
│  └─────────────┘  │
│                   │
│  ┌─────────────┐  │
│  │Personality  │  │
│  │   Engine    │  │
│  └─────────────┘  │
└───────────────────┘
```

## Layers

### 1. UI Layer (Screens)

React Native components that render the user interface.

#### DiscoveryScreen
- **Purpose:** Find and connect to robots
- **data-testid:** `discovery-list`, `scan-btn`, `connect-btn-{id}`
- **State:** robots, scanning, error
- **Actions:** startScan(), connect()

#### PersonalityMixerScreen
- **Purpose:** Adjust 9 personality parameters
- **data-testid:** `personality-mixer`, `slider-{param}`, `connection-status`
- **State:** config, connected
- **Actions:** updateSlider(), reset(), savePreset()
- **Invariant:** I-MOBILE-001 (200ms response time)

#### NeuralVisualizerScreen
- **Purpose:** Real-time neural state visualization
- **data-testid:** `neural-visualizer`, `viz-play-pause`
- **State:** neuralState, animationActive, zoom
- **Actions:** toggleAnimation(), zoom()

#### GalleryScreen
- **Purpose:** Browse and playback drawings
- **data-testid:** `gallery-grid`, `drawing-thumb-{id}`, `playback-btn-{id}`
- **State:** drawings, loading
- **Actions:** fetchDrawings(), openDrawing(), deleteDrawing()

### 2. Hooks Layer

React hooks that connect UI to services.

#### useAppService()
```typescript
{
  service: MobileAppService,
  connected: boolean,
  currentRobot?: Robot
}
```

#### useRobotDiscovery()
```typescript
{
  robots: Robot[],
  scanning: boolean,
  error?: string,
  startScan: () => Promise<void>,
  connect: (robotId: string) => Promise<void>
}
```

#### useNeuralState()
```typescript
NeuralState | null
```

#### usePersonality()
```typescript
{
  config: PersonalityConfig,
  updateSlider: (param, value) => Promise<void>,
  reset: () => Promise<void>
}
```

#### useGallery()
```typescript
{
  drawings: Drawing[],
  loading: boolean,
  error?: string,
  fetchDrawings: () => Promise<void>
}
```

### 3. Service Layer

Business logic services.

#### MobileAppService

**Main orchestrator** that coordinates all other services.

```typescript
class MobileAppService {
  private wsClient: WebSocketClient;
  private discoveryService: RobotDiscoveryService;
  private cacheService: CacheService;

  async initialize(): Promise<void>;
  async connectToRobot(robotId: string): Promise<void>;
  subscribeToNeuralState(callback): Subscription;
  async updatePersonality(config): Promise<void>;
  async fetchDrawings(): Promise<Drawing[]>;
}
```

#### WebSocketClient

**WebSocket connection management** with auto-reconnect.

**I-MOBILE-001:** Auto-reconnect within 5 seconds.

```typescript
class WebSocketClient {
  async connect(ip: string, port: number): Promise<void>;
  disconnect(): void;
  send(message: any): void;
  on(event: string, callback: Function): UnsubscribeFn;
  isConnected(): boolean;
}
```

**Message Types:**
- `neural_state` - Neural state updates
- `personality_update` - Personality config changes
- `drawing_complete` - New drawing available
- `status_change` - Robot status changes
- `error` - Error messages

#### RobotDiscoveryService

**Network discovery** via mDNS (Bonjour/Zeroconf).

```typescript
class RobotDiscoveryService {
  async discoverRobots(): Promise<Robot[]>;
  startScanning(interval?: number): void;
  stopScanning(): void;
  getRobot(robotId: string): Robot | undefined;
}
```

**Discovery Protocol:**
1. Scan network for `_mbot._tcp` service
2. Resolve IP address and port
3. Update robot list
4. Monitor robot availability

#### CacheService

**Offline cache** with AsyncStorage.

**I-MOBILE-003:** Cache for at least 24 hours.

```typescript
class CacheService {
  async saveAppState(state: AppState): Promise<void>;
  async loadAppState(): Promise<AppState | null>;
  async saveDrawings(drawings: Drawing[]): Promise<void>;
  async loadDrawings(): Promise<Drawing[]>;
  async clearCache(): Promise<void>;
  async isCacheValid(): Promise<boolean>;
}
```

**Cache Keys:**
- `@mbot/app_state` - Complete app state
- `@mbot/drawings` - Drawing gallery
- `@mbot/neural_state` - Last neural state
- `@mbot/personality` - Personality config
- `@mbot/last_sync` - Last sync timestamp

### 4. Component Layer

Reusable UI components.

#### PersonalitySlider
- 9 parameters (energy, tension, etc.)
- Value display (0-100%)
- Smooth slider interaction
- I-MOBILE-002: Responsive 320px-768px

#### ConnectionIndicator
- Online/offline status
- Visual indicator (green/red dot)
- I-MOBILE-003: Shows offline mode

#### NeuralGraph
- 2D/3D neural visualization
- SVG-based rendering
- Node types: sensory, motor, cognitive
- Connection visualization

## Data Flow

### Connection Flow

```
User taps "Scan"
  → DiscoveryScreen.startScan()
  → useRobotDiscovery().startScan()
  → RobotDiscoveryService.discoverRobots()
  → mDNS network scan
  → Robot list updated

User taps "Connect"
  → DiscoveryScreen.connect(robotId)
  → useRobotDiscovery().connect()
  → MobileAppService.connectToRobot()
  → WebSocketClient.connect()
  → WebSocket established (within 3s - I-MOBILE-001)
  → Navigate to PersonalityMixerScreen
```

### Personality Update Flow

```
User adjusts slider
  → PersonalitySlider.onValueChange()
  → usePersonality().updateSlider()
  → MobileAppService.updatePersonality()
  → WebSocketClient.send({ type: 'update_personality', payload })
  → Robot responds (within 200ms - I-MOBILE-001)
  → Neural visualizer updates
```

### Offline Mode Flow

```
WiFi disconnects
  → WebSocketClient detects close event
  → Emit 'connection' event { status: 'disconnected' }
  → UI shows "Offline" indicator
  → CacheService loads cached data (I-MOBILE-003)
  → User can view cached drawings

WiFi reconnects
  → WebSocketClient auto-reconnect (within 5s - I-MOBILE-001)
  → Emit 'connection' event { status: 'connected' }
  → UI shows "Connected" indicator
  → Sync with robot
```

## State Management

### Local State (useState)

Each screen manages its own UI state:
- Loading indicators
- Modal visibility
- Form inputs
- Animation state

### Shared State (Service Layer)

Services maintain application state:
- Connection status
- Current robot
- Neural state stream
- Personality configuration
- Drawing gallery cache

**No global state library** (Redux/MobX) - services handle coordination.

## Networking

### WebSocket Protocol

**Endpoint:** `ws://{robotIp}:{port}/ws`

**Client → Robot Messages:**
```typescript
{ type: 'update_personality', payload: PersonalityConfig }
{ type: 'get_drawings' }
{ type: 'get_drawing', payload: { id: string } }
```

**Robot → Client Messages:**
```typescript
{ type: 'neural_state', payload: NeuralState }
{ type: 'personality_update', payload: PersonalityConfig }
{ type: 'drawings_list', payload: Drawing[] }
{ type: 'drawing_data', payload: Drawing }
{ type: 'status_change', payload: { status: string } }
{ type: 'error', payload: { message: string } }
```

### Auto-Reconnect Strategy

**I-MOBILE-001:** Reconnect within 5 seconds.

```
Attempt 1: 2000ms delay (2s)
Attempt 2: 4000ms delay (4s)
Attempt 3: 5000ms delay (5s) - capped
Attempt 4: 5000ms delay (5s)
Attempt 5: 5000ms delay (5s)
Max attempts: 5
```

Exponential backoff capped at 5000ms per invariant I-MOBILE-001.

## Offline Support

**I-MOBILE-003:** Cache last known state for 24 hours.

### Cached Data

1. **App State** - Connection info, current robot
2. **Neural State** - Last neural activity snapshot
3. **Personality Config** - Last configuration
4. **Drawings** - Full drawing gallery with thumbnails

### Cache Expiry

```typescript
const CACHE_EXPIRY = 24 * 60 * 60 * 1000; // 24 hours

if (Date.now() - lastSync > CACHE_EXPIRY) {
  // Cache expired - clear and prompt reconnect
  await cacheService.clearCache();
}
```

### Offline-First Features

✅ View cached drawings
✅ View last neural state
✅ View personality configuration
❌ Cannot adjust personality (requires connection)
❌ Cannot create new drawings (requires connection)

## Responsive Design

**I-MOBILE-002:** UI must be responsive 320px-768px wide.

### Breakpoints

```typescript
const BREAKPOINTS = {
  small: 320,   // iPhone SE
  medium: 375,  // iPhone 12/13
  large: 414,   // iPhone 12/13 Pro Max
  tablet: 768,  // iPad Mini
};
```

### Adaptive Layout

- **320-375px:** Single column, compact spacing
- **375-414px:** Standard phone layout
- **414-768px:** Larger phones, more spacing
- **768px+:** Tablet layout, two-column where applicable

### Platform-Specific Styles

```typescript
Platform.select({
  ios: { fontFamily: 'System' },
  android: { fontFamily: 'Roboto' },
})
```

## Performance Optimization

### Image Optimization
- Use WebP format where supported
- Lazy load thumbnails
- Cache images locally

### List Performance
- FlatList with `getItemLayout`
- Windowing for long lists
- Image recycling

### Memory Management
- Unsubscribe from events on unmount
- Clear timers and intervals
- Release WebSocket connections

## Security

### Network Security
- WebSocket over TLS in production (wss://)
- Validate server certificates
- Timeout connections after inactivity

### Data Privacy
- No sensitive data stored unencrypted
- Clear cache on logout
- No analytics without consent

## Testing Strategy

### Unit Tests (Jest)

Test services and hooks:
- WebSocketClient connection logic
- CacheService expiry enforcement
- RobotDiscoveryService network handling

### Integration Tests

Test screen interactions:
- Discovery → Connection flow
- Personality adjustment
- Gallery browsing

### E2E Tests (Detox)

Test user journeys (J-MOBILE-CONTROL):
- Connect from phone
- Adjust personality
- View gallery
- Offline mode

## Deployment

### iOS

1. Build with EAS
2. Submit to TestFlight
3. Release to App Store

### Android

1. Build APK/AAB with EAS
2. Submit to Play Console (internal testing)
3. Release to Production

## Invariants Summary

| ID | Description | Implementation |
|----|-------------|----------------|
| I-MOBILE-001 | Auto-reconnect within 5s | `WebSocketClient.scheduleReconnect()` |
| I-MOBILE-002 | Responsive 320-768px | StyleSheet with responsive values |
| I-MOBILE-003 | Cache for 24 hours | `CacheService.setCacheExpiry()` |

## Dependencies

### Core
- `react-native` - Mobile framework
- `expo` - Development platform
- `@react-navigation` - Navigation

### Networking
- WebSocket (built-in)
- react-native-zeroconf (for mDNS)

### Storage
- `@react-native-async-storage/async-storage`

### UI
- `react-native-svg` - Graphics
- `@react-native-community/slider` - Sliders
- `react-native-vector-icons` - Icons

## Future Enhancements

- **Voice Commands** (#89) - Voice control integration
- **Multi-Robot** - Control multiple robots
- **Presets** - Save personality presets
- **Recording** - Record and replay sessions
- **Social** - Share drawings
