# mBot Mobile App - Setup Guide

Complete setup instructions for the mBot React Native mobile application.

**Issue:** #88 (STORY-MOBILE-001)

## Prerequisites

- Node.js 18+ and npm
- iOS: Xcode 14+ and CocoaPods
- Android: Android Studio and JDK 11+
- Expo CLI (will be installed with npm install)

## Installation

### 1. Install Dependencies

```bash
cd mobile
npm install
```

### 2. iOS Setup

```bash
# Install iOS dependencies
cd ios
pod install
cd ..
```

**Configure Info.plist** (for robot discovery):

Add to `ios/mBot/Info.plist`:

```xml
<key>NSLocalNetworkUsageDescription</key>
<string>This app needs to discover and connect to mBot robots on your local network.</string>
<key>NSBonjourServices</key>
<array>
  <string>_mbot._tcp</string>
</array>
```

### 3. Android Setup

**Configure Permissions** in `android/app/src/main/AndroidManifest.xml`:

```xml
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
```

## Development

### Start Metro Bundler

```bash
npm start
```

### Run on iOS Simulator

```bash
npm run ios
```

Or with specific simulator:

```bash
npx react-native run-ios --simulator="iPhone 14 Pro"
```

### Run on Android Emulator

```bash
npm run android
```

Or with specific device:

```bash
npx react-native run-android --device="Pixel_4_API_30"
```

### Run on Physical Device

#### iOS

1. Open `ios/mBot.xcworkspace` in Xcode
2. Select your device
3. Configure signing team
4. Run the app

#### Android

1. Enable USB debugging on device
2. Connect via USB
3. Check device: `adb devices`
4. Run: `npm run android`

## Testing

### Unit Tests

```bash
npm test
```

With coverage:

```bash
npm test -- --coverage
```

### E2E Tests with Detox

#### Setup Detox

```bash
# Install Detox CLI
npm install -g detox-cli

# iOS
brew tap wix/brew
brew install applesimutils
```

#### Build Test App

iOS:

```bash
detox build --configuration ios.sim.debug
```

Android:

```bash
detox build --configuration android.emu.debug
```

#### Run E2E Tests

iOS:

```bash
detox test --configuration ios.sim.debug
```

Android:

```bash
detox test --configuration android.emu.debug
```

## Building

### iOS Production Build (EAS)

```bash
# Install EAS CLI
npm install -g eas-cli

# Login
eas login

# Configure build
eas build:configure

# Build
npm run build:ios
```

### Android Production Build (EAS)

```bash
npm run build:android
```

## Debugging

### React Native Debugger

1. Install React Native Debugger
2. Start app in debug mode
3. Open developer menu: `Cmd+D` (iOS) or `Cmd+M` (Android)
4. Select "Debug"

### Chrome DevTools

1. Start app
2. Open developer menu
3. Select "Debug with Chrome"

### Flipper

1. Install Flipper desktop app
2. Start app
3. Flipper auto-detects running app

### Network Debugging

Enable network inspect in app:

```typescript
import { NetworkInspector } from 'react-native';
NetworkInspector.enable();
```

## Troubleshooting

### Metro Bundler Issues

```bash
# Clear cache
npx react-native start --reset-cache

# Clear watchman
watchman watch-del-all
```

### iOS Build Issues

```bash
# Clean build
cd ios
xcodebuild clean
pod deintegrate
pod install
cd ..
```

### Android Build Issues

```bash
# Clean build
cd android
./gradlew clean
cd ..
```

### WebSocket Connection Issues

1. Check robot is running and accessible
2. Verify same WiFi network
3. Check firewall settings
4. Test with curl:

```bash
curl http://192.168.1.100:8080/health
```

### mDNS Discovery Issues

#### iOS

- Ensure `NSBonjourServices` is configured in Info.plist
- Check network permissions

#### Android

- Ensure WiFi permissions are granted
- Some Android versions have mDNS limitations

**Workaround:** Manual IP entry feature (future enhancement)

## Environment Variables

Create `.env` file:

```bash
# Robot connection
DEFAULT_ROBOT_PORT=8080
WEBSOCKET_TIMEOUT=3000

# Cache settings
MIN_CACHE_EXPIRY_HOURS=24
MAX_RECONNECT_DELAY_MS=5000

# Development
DEBUG_MODE=true
LOG_LEVEL=debug
```

## Configuration

### App Settings

Modify `src/services/MobileAppService.ts`:

```typescript
const DEFAULT_SETTINGS: AppSettings = {
  autoReconnect: true,
  reconnectDelay: 2000,        // I-MOBILE-001: ≤ 5000ms
  cacheExpiry: 24,             // I-MOBILE-003: ≥ 24 hours
  notificationsEnabled: true,
  theme: 'light',
};
```

### Network Discovery

Modify `src/services/RobotDiscoveryService.ts`:

```typescript
// Production: Use react-native-zeroconf
import Zeroconf from 'react-native-zeroconf';

const zeroconf = new Zeroconf();
zeroconf.scan('mbot', 'tcp', 'local.');
```

## Performance Optimization

### Enable Hermes (JavaScript Engine)

Already enabled in `app.json`:

```json
{
  "expo": {
    "jsEngine": "hermes"
  }
}
```

### Optimize Images

Use optimized images in `assets/`:

```bash
# Install image optimizer
npm install -g @expo/image-utils

# Optimize
expo-optimize
```

### Enable ProGuard (Android)

In `android/app/build.gradle`:

```gradle
buildTypes {
  release {
    minifyEnabled true
    proguardFiles getDefaultProguardFile('proguard-android.txt'), 'proguard-rules.pro'
  }
}
```

## App Store Submission

### iOS App Store

1. Create app in App Store Connect
2. Configure app metadata
3. Upload build via EAS
4. Submit for review

### Google Play Store

1. Create app in Play Console
2. Configure store listing
3. Upload APK/AAB via EAS
4. Submit for review

## CI/CD

### GitHub Actions

Create `.github/workflows/mobile.yml`:

```yaml
name: Mobile CI

on: [push, pull_request]

jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-node@v2
        with:
          node-version: '18'
      - run: cd mobile && npm install
      - run: cd mobile && npm test
      - run: cd mobile && npm run typecheck
      - run: cd mobile && npm run lint
```

## Monitoring

### Sentry Integration

```bash
npm install @sentry/react-native
```

Configure in `App.tsx`:

```typescript
import * as Sentry from '@sentry/react-native';

Sentry.init({
  dsn: 'YOUR_SENTRY_DSN',
  environment: __DEV__ ? 'development' : 'production',
});
```

### Analytics

```bash
npm install @react-native-firebase/analytics
```

## Resources

- [React Native Docs](https://reactnative.dev/docs/getting-started)
- [Expo Docs](https://docs.expo.dev/)
- [React Navigation](https://reactnavigation.org/docs/getting-started)
- [Detox E2E Testing](https://wix.github.io/Detox/)

## Support

- GitHub Issues: https://github.com/Hulupeep/mbot_ruvector/issues
- Related Issue: #88
- Contract: J-MOBILE-CONTROL
