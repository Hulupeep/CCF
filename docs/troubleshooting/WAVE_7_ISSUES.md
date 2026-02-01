# Wave 7 Troubleshooting Guide

Common issues and solutions for Wave 7 features.

## Multi-Robot Coordination Issues

### Robots Not Responding to Commands

**Symptoms:** Commands sent but robots don't move
**Causes:**
1. WebSocket connection lost
2. Robot firmware incompatible
3. Command format wrong

**Solutions:**
```typescript
// Check all connections
const { robots } = useRobotDiscovery();
robots.forEach(robot => {
  console.log(`${robot.name}: ${robot.status}`);
});

// Verify firmware version
robots.forEach(robot => {
  if (robot.version < '1.0.0') {
    console.warn(`${robot.name} needs firmware update`);
  }
});

// Test command format
await multiRobotCoordination.coordinateMovement(robots, {
  mode: 'centralized',
  formation: 'line',
  spacing: 30,
  collisionAvoidance: true
});
```

### Collision Detection False Positives

**Symptoms:** Robots stop when path is clear
**Causes:**
1. Min distance too large
2. Sensor calibration off
3. Network latency

**Solutions:**
```typescript
// Adjust min distance
multiRobotCoordination.setMinDistance(15); // cm (default: 30)

// Calibrate sensors
robots.forEach(robot => {
  robot.sendCommand('calibrate_sensors');
});

// Check network latency
robots.forEach(async robot => {
  const start = Date.now();
  await robot.ping();
  const latency = Date.now() - start;
  console.log(`${robot.name} latency: ${latency}ms`);
});
```

## Swarm Play Modes Issues

### Formation Breaks Apart

**Symptoms:** Robots don't maintain formation
**Causes:**
1. Spacing too tight
2. Speed too fast
3. Obstacle interference

**Solutions:**
```typescript
// Increase spacing
swarmController.setSpacing(40); // cm (default: 30)

// Reduce speed
swarmController.setSpeed(0.5); // 50% speed

// Use loose formation
swarmController.setFormation('loose');

// Monitor formation status
swarmController.on('formation_complete', () => {
  console.log('Formation achieved');
});
```

### Circle Mode Not Circular

**Symptoms:** Robots form irregular shape
**Causes:**
1. Unequal wheel speeds
2. Floor surface uneven
3. Battery levels different

**Solutions:**
```typescript
// Calibrate wheel speeds
robots.forEach(robot => {
  robot.sendCommand('calibrate_wheels');
});

// Check battery levels
robots.forEach(robot => {
  const battery = robot.getBatteryLevel();
  if (battery < 20) {
    console.warn(`${robot.name} low battery: ${battery}%`);
  }
});

// Adjust for surface
swarmController.setSurfaceCompensation(true);
```

## Cloud Sync Issues

See: [Cloud Sync Debugging Guide](CLOUD_SYNC_DEBUG.md) (detailed troubleshooting)

### Quick Checks

```typescript
// Test connection
await cloudSync.testConnection();

// Check auth
const user = await cloudSync.getCurrentUser();
console.log('Authenticated:', user !== null);

// Force sync
await cloudSync.syncAll();
```

## Personality Marketplace Issues

### Can't Download Personalities

**Symptoms:** Download button doesn't work
**Causes:**
1. Not authenticated
2. Network error
3. Personality deleted

**Solutions:**
```typescript
// Check authentication
const user = await supabase.auth.getUser();
if (!user) {
  await supabase.auth.signIn();
}

// Verify personality exists
const { data, error } = await supabase
  .from('marketplace_personalities')
  .select('*')
  .eq('id', personalityId)
  .single();

if (error) {
  console.error('Personality not found:', error);
}

// Manual download
const personality = await personalityMarketplace.download(personalityId);
personalityStore.updatePersonality(personality);
```

### Upload Fails

**Symptoms:** "Failed to publish" error
**Causes:**
1. Validation error
2. File size too large
3. Quota exceeded

**Solutions:**
```typescript
// Validate before upload
const validation = await personalityMarketplace.validate({
  name: 'My Bot',
  description: 'Description',
  personality: currentPersonality,
  category: 'playful',
  tags: ['fun', 'energetic']
});

if (!validation.valid) {
  console.error('Validation errors:', validation.errors);
}

// Check file size
const size = JSON.stringify(currentPersonality).length;
console.log('Personality size:', size, 'bytes');
if (size > 100000) { // 100KB limit
  console.error('Personality too large');
}

// Check quota
const count = await personalityMarketplace.getMyPersonalityCount();
if (count >= 10) { // Free tier limit
  console.error('Quota exceeded, upgrade to publish more');
}
```

## Learning from Play Issues

### Learning Not Improving

**Symptoms:** Win rate not increasing
**Causes:**
1. Learning rate too low
2. Exploration rate stuck
3. Not enough training data

**Solutions:**
```typescript
// Increase learning rate
learningMonitor.setLearningRate(0.2); // Default: 0.1

// Check exploration rate
const stats = learningMonitor.getStats();
console.log('Exploration rate:', stats.explorationRate);

// If stuck at 1.0 (always exploring), force decay
learningMonitor.setExplorationRate(0.5);
learningMonitor.setExplorationDecay(0.995);

// Check training data
console.log('Episodes:', stats.episodeCount);
if (stats.episodeCount < 100) {
  console.warn('Need more training data (100+ episodes)');
}
```

### Q-Values Not Converging

**Symptoms:** Values keep changing wildly
**Causes:**
1. Discount factor wrong
2. State representation poor
3. Non-stationary environment

**Solutions:**
```typescript
// Adjust discount factor
learningMonitor.setDiscountFactor(0.95); // Default: 0.9
// Higher = values future rewards more

// Check Q-value stability
const stability = stats.qValueStability;
console.log('Q-value stability:', stability);
if (stability < 0.7) {
  console.warn('Q-values not stable, continue training');
}

// Reset learning if needed
learningMonitor.reset();
```

## Predictive Behavior Issues

### Low Prediction Accuracy

**Symptoms:** Predictions often wrong
**Causes:**
1. Insufficient training data
2. Confidence threshold too low
3. User behavior changed

**Solutions:**
```typescript
// Check accuracy metrics
const metrics = await predictionEngine.getAccuracy();
console.log('Overall accuracy:', metrics.overallAccuracy);

if (metrics.overallAccuracy < 0.5) {
  console.warn('Model needs more training');

  // Collect more data
  predictionEngine.setHistoryWindow(100); // Increase from default 50

  // Retrain model
  await predictionEngine.updateModel();
}

// Increase confidence threshold
predictionEngine.setConfidenceThreshold(0.8); // Default: 0.6
```

### Predictions Too Slow

**Symptoms:** Prediction takes >1 second
**Causes:**
1. Model too complex
2. History window too large
3. Network latency

**Solutions:**
```typescript
// Reduce history window
predictionEngine.setHistoryWindow(20); // From 50

// Use local model (not cloud)
predictionEngine.setMode('local');

// Measure prediction time
const start = Date.now();
const prediction = await predictionEngine.predictNext(context);
console.log('Prediction time:', Date.now() - start, 'ms');
```

## Voice Control Issues

### Commands Not Recognized

**Symptoms:** Robot doesn't respond to voice
**Causes:**
1. Microphone not working
2. Background noise too high
3. Command not registered

**Solutions:**
```typescript
// Test microphone
navigator.mediaDevices.getUserMedia({ audio: true })
  .then(stream => {
    console.log('Microphone working');
    stream.getTracks().forEach(track => track.stop());
  })
  .catch(error => {
    console.error('Microphone error:', error);
  });

// Check registered commands
const commands = voiceCommands.getRegisteredCommands();
console.log('Registered commands:', commands.length);
commands.forEach(cmd => {
  console.log('-', cmd.patterns.join(', '));
});

// Add command variations
voiceCommands.registerCommand('start drawing', handler);
voiceCommands.registerCommand('begin drawing', handler);
voiceCommands.registerCommand('draw', handler);
```

### Wake Word Not Detecting

**Symptoms:** "Hey mBot" doesn't activate
**Causes:**
1. Wake word disabled
2. Model not loaded
3. Audio quality poor

**Solutions:**
```typescript
// Check wake word status
if (!voiceCommands.isWakeWordEnabled()) {
  voiceCommands.enableWakeWord(true);
}

// Test with manual activation
voiceCommands.start(); // Skip wake word

// Check audio levels
navigator.mediaDevices.getUserMedia({ audio: true })
  .then(stream => {
    const audioContext = new AudioContext();
    const analyser = audioContext.createAnalyser();
    const microphone = audioContext.createMediaStreamSource(stream);
    microphone.connect(analyser);

    const dataArray = new Uint8Array(analyser.frequencyBinCount);
    setInterval(() => {
      analyser.getByteFrequencyData(dataArray);
      const average = dataArray.reduce((a, b) => a + b) / dataArray.length;
      console.log('Audio level:', average);
    }, 100);
  });
```

## Mobile App Issues

### App Crashes on Launch

**Symptoms:** App closes immediately
**Causes:**
1. Missing native dependencies
2. Permissions not granted
3. Build configuration wrong

**Solutions:**
```bash
# iOS: Reinstall pods
cd ios && pod deintegrate && pod install && cd ..

# Android: Clean build
cd android && ./gradlew clean && cd ..

# Rebuild
npx react-native run-ios
npx react-native run-android

# Check logs
npx react-native log-ios
npx react-native log-android
```

### WebSocket Connection Fails on Mobile

**Symptoms:** Can't connect to robot from phone
**Causes:**
1. Different network
2. Firewall blocking
3. IP address changed

**Solutions:**
```typescript
// Use mDNS instead of IP
const ws = new WebSocket('ws://mbot.local:4000');

// Or use discovery
const { robots } = useRobotDiscovery();
const robot = robots[0];
const ws = new WebSocket(`ws://${robot.ip}:${robot.port}`);

// Check network
import NetInfo from '@react-native-community/netinfo';
NetInfo.fetch().then(state => {
  console.log('Connected:', state.isConnected);
  console.log('Network:', state.type); // wifi, cellular, etc.
});
```

---

**Last Updated:** 2026-02-01
