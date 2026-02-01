# mBot Mobile App

React Native mobile application for controlling mBot robots.

**Issue:** [#88 - STORY-MOBILE-001: Mobile App Foundation](https://github.com/Hulupeep/mbot_ruvector/issues/88)

## Features

- **Robot Discovery** - Find robots on local network via mDNS
- **Personality Mixer** - Adjust 9 personality parameters in real-time
- **Neural Visualizer** - View live neural state activity
- **Gallery** - Browse and playback saved drawings
- **Offline Mode** - Cache last known state for 24 hours

## Invariants

| ID | Description | Status |
|----|-------------|--------|
| I-MOBILE-001 | WebSocket auto-reconnect within 5 seconds | Implemented |
| I-MOBILE-002 | Responsive UI (320px-768px) | Implemented |
| I-MOBILE-003 | Offline cache for 24 hours | Implemented |

## Installation

```bash
cd mobile
npm install
```

## Development

### iOS

```bash
npm run ios
```

### Android

```bash
npm run android
```

### Web (for testing)

```bash
npm run web
```

## Architecture

```
mobile/
├── src/
│   ├── components/          # Reusable UI components
│   │   ├── ConnectionIndicator.tsx
│   │   ├── NeuralGraph.tsx
│   │   └── PersonalitySlider.tsx
│   ├── screens/             # Screen components
│   │   ├── DiscoveryScreen.tsx
│   │   ├── GalleryScreen.tsx
│   │   ├── NeuralVisualizerScreen.tsx
│   │   └── PersonalityMixerScreen.tsx
│   ├── services/            # Business logic
│   │   ├── CacheService.ts
│   │   ├── MobileAppService.ts
│   │   ├── RobotDiscoveryService.ts
│   │   └── WebSocketClient.ts
│   ├── hooks/               # React hooks
│   │   └── useAppService.ts
│   ├── types/               # TypeScript types
│   │   └── index.ts
│   └── App.tsx              # Main app component
├── tests/
│   ├── services/            # Unit tests
│   └── journeys/            # E2E tests
├── package.json
├── app.json
└── tsconfig.json
```

## Services

### WebSocketClient

Handles WebSocket connections with auto-reconnect.

**I-MOBILE-001:** Reconnects within 5 seconds on connection loss.

```typescript
const client = new WebSocketClient(settings);
await client.connect('192.168.1.100', 8080);
```

### RobotDiscoveryService

Discovers robots on local network via mDNS.

```typescript
const service = new RobotDiscoveryService();
const robots = await service.discoverRobots();
```

### CacheService

Manages offline cache with AsyncStorage.

**I-MOBILE-003:** Caches data for at least 24 hours.

```typescript
const cache = new CacheService();
await cache.saveAppState(state);
const cached = await cache.loadAppState();
```

### MobileAppService

Main service orchestrator that coordinates all other services.

```typescript
const service = new MobileAppService();
await service.initialize();
await service.connectToRobot('robot-001');
```

## Screens

### Discovery Screen

Find and connect to robots on the network.

**data-testid:**
- `discovery-list` - Robot list
- `scan-btn` - Scan button
- `connect-btn-{robotId}` - Connect buttons

### Personality Mixer Screen

Adjust 9 personality parameters with sliders.

**I-MOBILE-001:** Robot responds within 200ms of slider change.

**data-testid:**
- `personality-mixer` - Main container
- `slider-energy`, `slider-tension`, etc. - Individual sliders
- `connection-status` - Connection indicator

### Neural Visualizer Screen

Real-time 2D/3D visualization of neural state.

**data-testid:**
- `neural-visualizer` - Main visualizer
- `viz-play-pause` - Animation control

### Gallery Screen

Browse and playback saved drawings.

**data-testid:**
- `gallery-grid` - Gallery grid
- `drawing-thumb-{id}` - Drawing thumbnails
- `playback-btn-{id}` - Playback buttons

## Testing

### Unit Tests

```bash
npm test
```

### E2E Tests (Detox)

```bash
# Build app for testing
detox build --configuration ios.sim.debug

# Run tests
npm run test:e2e
```

## Configuration

### app.json

- iOS bundle identifier: `com.mbot.controller`
- Android package: `com.mbot.controller`
- Permissions: Network access, WiFi state

### Settings

```typescript
interface AppSettings {
  autoReconnect: boolean;      // Enable auto-reconnect
  reconnectDelay: number;      // Delay in ms (max 5000)
  cacheExpiry: number;         // Hours (min 24)
  notificationsEnabled: boolean;
  theme: 'light' | 'dark';
}
```

## Network Discovery

### iOS

Requires `NSLocalNetworkUsageDescription` and `NSBonjourServices` in Info.plist.

Service type: `_mbot._tcp`

### Android

Requires `INTERNET` and `ACCESS_WIFI_STATE` permissions.

## Building

### iOS (EAS Build)

```bash
npm run build:ios
```

### Android (EAS Build)

```bash
npm run build:android
```

## Dependencies

- **expo** - React Native framework
- **@react-navigation** - Navigation
- **@react-native-async-storage** - Offline cache
- **react-native-svg** - Graphics rendering
- **react-native-vector-icons** - Icons

## Troubleshooting

### WebSocket Connection Fails

1. Check robot is on same WiFi network
2. Verify robot's IP address and port
3. Check firewall settings

### Discovery Not Finding Robots

1. Ensure mDNS is enabled on robot
2. Check network permissions
3. Try manual IP entry

### Offline Mode Not Working

1. Check cache settings (min 24 hours)
2. Verify AsyncStorage permissions
3. Clear cache and reconnect

## Contract References

- **Feature Contracts:** I-MOBILE-001, I-MOBILE-002, I-MOBILE-003
- **Journey Contract:** J-MOBILE-CONTROL
- **E2E Test:** `tests/journeys/mobile-control.journey.spec.ts`

## Related Issues

- #58 - WebSocket Real-Time Control
- #59 - Neural State Visualizer
- #60 - Personality Presets
- #64 - Drawing Gallery

## Future Enhancements

- Voice command integration (#89)
- Multi-robot control
- AR/VR features
- Social sharing
- App store publication
