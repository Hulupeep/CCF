# Mobile App Setup Guide

**Platform:** React Native (iOS & Android)
**Time:** 30 minutes
**Difficulty:** Advanced

## Prerequisites

- Node.js 18+
- React Native CLI
- Xcode (for iOS)
- Android Studio (for Android)
- CocoaPods (for iOS)

## Step 1: Create React Native Project

```bash
npx react-native init mBotMobile --template react-native-template-typescript
cd mBotMobile
```

## Step 2: Install Dependencies

```bash
npm install --save \
  @react-navigation/native \
  @react-navigation/stack \
  react-native-gesture-handler \
  react-native-reanimated \
  react-native-screens \
  react-native-safe-area-context \
  @react-native-async-storage/async-storage \
  @react-native-community/netinfo \
  react-native-webrtc

# For iOS
cd ios && pod install && cd ..
```

## Step 3: Copy Web Companion Code

```bash
# Copy services (with minimal modifications)
cp -r ../web/src/services ./src/
cp -r ../web/src/types ./src/
cp -r ../web/src/hooks ./src/

# Modify for React Native
# Replace localStorage with AsyncStorage
# Replace WebSocket with react-native-webrtc
```

## Step 4: Configure Navigation

```typescript
// App.tsx
import { NavigationContainer } from '@react-navigation/native';
import { createStackNavigator } from '@react-navigation/stack';

const Stack = createStackNavigator();

export default function App() {
  return (
    <NavigationContainer>
      <Stack.Navigator>
        <Stack.Screen name="Home" component={HomeScreen} />
        <Stack.Screen name="PersonalityMixer" component={PersonalityMixerScreen} />
        <Stack.Screen name="NeuralVisualizer" component={NeuralVisualizerScreen} />
        <Stack.Screen name="DrawingGallery" component={DrawingGalleryScreen} />
        <Stack.Screen name="GameStats" component={GameStatsScreen} />
      </Stack.Navigator>
    </NavigationContainer>
  );
}
```

## Step 5: Adapt Components for Mobile

### PersonalityMixer (Mobile)

```typescript
// components/PersonalityMixer.tsx
import { View, Text, Slider } from 'react-native';
import { personalityStore } from '../services/personalityStore';

export function PersonalityMixer() {
  const [personality, setPersonality] = useState(personalityStore.getCurrentPersonality());

  return (
    <View>
      {Object.entries(personality).map(([key, value]) => (
        <View key={key}>
          <Text>{key}</Text>
          <Slider
            value={value}
            onValueChange={(val) => {
              personalityStore.updateParameter(key, val);
              setPersonality(personalityStore.getCurrentPersonality());
            }}
            minimumValue={0}
            maximumValue={1}
          />
        </View>
      ))}
    </View>
  );
}
```

## Step 6: Configure Permissions

### iOS (info.plist)

```xml
<key>NSCameraUsageDescription</key>
<string>mBot needs camera access for visual recognition</string>
<key>NSMicrophoneUsageDescription</key>
<string>mBot needs microphone for voice control</string>
<key>NSBluetoothAlwaysUsageDescription</key>
<string>mBot uses Bluetooth to connect to robot</string>
<key>NSLocalNetworkUsageDescription</key>
<string>mBot discovers robots on local network</string>
```

### Android (AndroidManifest.xml)

```xml
<uses-permission android:name="android.permission.CAMERA" />
<uses-permission android:name="android.permission.RECORD_AUDIO" />
<uses-permission android:name="android.permission.BLUETOOTH" />
<uses-permission android:name="android.permission.BLUETOOTH_ADMIN" />
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
<uses-permission android:name="android.permission.INTERNET" />
```

## Step 7: Build & Run

### iOS

```bash
npx react-native run-ios
# or for specific device
npx react-native run-ios --device "iPhone Name"
```

### Android

```bash
npx react-native run-android
# or for specific device
adb devices
npx react-native run-android --deviceId=DEVICE_ID
```

## Step 8: Configure WebSocket

```typescript
// services/websocket-mobile.ts
import WebSocket from 'react-native-webrtc';

export function createWebSocket(url: string) {
  const ws = new WebSocket(url);

  // Same API as web WebSocket
  ws.onopen = () => console.log('Connected');
  ws.onmessage = (event) => console.log('Message:', event.data);
  ws.onerror = (error) => console.error('Error:', error);
  ws.onclose = () => console.log('Disconnected');

  return ws;
}
```

## Step 9: Offline Storage

```typescript
// services/storage-mobile.ts
import AsyncStorage from '@react-native-async-storage/async-storage';

export const storage = {
  async setItem(key: string, value: any) {
    await AsyncStorage.setItem(key, JSON.stringify(value));
  },

  async getItem(key: string) {
    const value = await AsyncStorage.getItem(key);
    return value ? JSON.parse(value) : null;
  },

  async removeItem(key: string) {
    await AsyncStorage.removeItem(key);
  },

  async clear() {
    await AsyncStorage.clear();
  }
};
```

## Step 10: Push Notifications (Optional)

```bash
npm install @react-native-firebase/app @react-native-firebase/messaging
```

```typescript
// services/notifications.ts
import messaging from '@react-native-firebase/messaging';

export async function setupNotifications() {
  const authStatus = await messaging().requestPermission();
  const enabled =
    authStatus === messaging.AuthorizationStatus.AUTHORIZED ||
    authStatus === messaging.AuthorizationStatus.PROVISIONAL;

  if (enabled) {
    const token = await messaging().getToken();
    console.log('FCM Token:', token);
  }

  messaging().onMessage(async (remoteMessage) => {
    console.log('Notification:', remoteMessage);
  });
}
```

## Troubleshooting

### iOS Build Fails

```bash
cd ios
pod deintegrate
pod install
cd ..
```

### Android Gradle Issues

```bash
cd android
./gradlew clean
cd ..
```

### Metro Bundler Issues

```bash
npx react-native start --reset-cache
```

---

**Last Updated:** 2026-02-01
