# mBot Mobile App - Quick Start

**Issue:** #88 (STORY-MOBILE-001)

Get the mBot mobile app running in 5 minutes.

## Prerequisites

- Node.js 18+
- iOS: Xcode 14+ (Mac only)
- Android: Android Studio + JDK 11+

## Installation

```bash
# Navigate to mobile directory
cd mobile

# Install dependencies
npm install
```

## Run Development Server

### Start Metro Bundler

```bash
npm start
```

This opens the Expo Dev Tools in your browser.

### Run on iOS Simulator (Mac only)

```bash
npm run ios
```

Or press `i` in the Expo Dev Tools.

### Run on Android Emulator

```bash
npm run android
```

Or press `a` in the Expo Dev Tools.

### Run on Physical Device

1. Install **Expo Go** app from App Store or Play Store
2. Scan QR code from Expo Dev Tools
3. App loads on your device

## Quick Test

Once the app loads:

1. **Discovery Screen** - Tap "Scan Again" to find robots
2. **Personality Screen** - Adjust sliders (requires connection)
3. **Neural Screen** - View neural visualization (requires connection)
4. **Gallery Screen** - Browse drawings (requires connection)

## Mock Data

The app includes mock robot discovery:
- `mBot-Kitchen` at 192.168.1.100:8080
- `mBot-Lab` at 192.168.1.101:8080

To connect to a real robot, ensure:
- Robot is running WebSocket server on port 8080
- Robot and phone are on same WiFi network
- Robot broadcasts mDNS service `_mbot._tcp`

## Testing

### Run Unit Tests

```bash
npm test
```

### Run E2E Tests

```bash
# Build test app
detox build --configuration ios.sim.debug

# Run tests
detox test --configuration ios.sim.debug
```

## Troubleshooting

### Metro bundler issues

```bash
npm start -- --reset-cache
```

### iOS simulator not found

```bash
xcrun simctl list devices
npm run ios -- --simulator="iPhone 14"
```

### Android emulator issues

```bash
# List AVDs
emulator -list-avds

# Start specific AVD
emulator -avd Pixel_4_API_30
```

### Can't connect to robot

1. Check robot is running: `curl http://192.168.1.100:8080/health`
2. Verify same WiFi network
3. Check firewall settings
4. Try manual IP entry (future feature)

## Next Steps

1. **Connect to Real Robot**
   - Ensure robot WebSocket server is running
   - Robot must broadcast mDNS service
   - Test discovery and connection

2. **Test Features**
   - Personality adjustments
   - Neural visualization
   - Drawing gallery
   - Offline mode

3. **Build for Production**
   - iOS: `npm run build:ios`
   - Android: `npm run build:android`

## Documentation

- **Setup Guide:** `docs/SETUP.md`
- **Architecture:** `docs/ARCHITECTURE.md`
- **Full README:** `README.md`
- **Implementation Summary:** `IMPLEMENTATION_SUMMARY.md`

## Support

- GitHub Issue: #88
- Repository: https://github.com/Hulupeep/CCF
- Contract: J-MOBILE-CONTROL
