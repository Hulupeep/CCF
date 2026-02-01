# Wave 6 Troubleshooting Guide

Common issues and solutions for Wave 6 features.

## Personality Mixer Issues

### Sliders Not Responding

**Symptoms:** Sliders disabled or don't move
**Causes:**
1. WebSocket disconnected
2. Component disabled prop
3. Invalid initial values

**Solutions:**
```typescript
// Check connection
const { connected } = usePersonalityWebSocket('ws://localhost:4000');
if (!connected) {
  console.error('WebSocket not connected');
}

// Verify component props
<PersonalityMixer disabled={false} />

// Check initial values
const personality = personalityStore.getCurrentPersonality();
console.log('All values valid:', Object.values(personality).every(v => v >= 0 && v <= 1));
```

### Changes Not Persisting

**Symptoms:** Personality resets after refresh
**Causes:**
1. localStorage full
2. localStorage disabled
3. Not calling save()

**Solutions:**
```typescript
// Check localStorage quota
try {
  localStorage.setItem('test', 'test');
  localStorage.removeItem('test');
} catch (e) {
  console.error('localStorage not available:', e);
}

// Manual save
personalityStore.save();

// Check saved data
const saved = localStorage.getItem('mbot-personality');
console.log('Saved personality:', saved);
```

## Neural Visualizer Issues

### Low Frame Rate

**Symptoms:** Animation stutters, <60fps
**Causes:**
1. Too many data points
2. Complex rendering
3. Browser throttling

**Solutions:**
```typescript
// Reduce update rate
<NeuralVisualizer updateRate={10} /> // 10Hz instead of 20Hz

// Reduce retention time
<NeuralVisualizer retentionTime={120} /> // 2 min instead of 5 min

// Check frame rate
let lastFrame = performance.now();
function checkFPS() {
  const now = performance.now();
  const fps = 1000 / (now - lastFrame);
  console.log('FPS:', fps);
  lastFrame = now;
  requestAnimationFrame(checkFPS);
}
checkFPS();
```

### WebSocket Data Not Showing

**Symptoms:** Meters show zero
**Causes:**
1. Wrong message format
2. WebSocket disconnected
3. Invalid data values

**Solutions:**
```typescript
// Verify message format
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

// Check WebSocket status
const ws = new WebSocket('ws://localhost:4000');
ws.onopen = () => console.log('Connected');
ws.onerror = (error) => console.error('Error:', error);
ws.onmessage = (event) => console.log('Message:', event.data);
```

## Drawing Gallery Issues

### Gallery Shows "No Drawings"

**Symptoms:** Empty gallery despite saved drawings
**Causes:**
1. IndexedDB quota exceeded
2. Filters too restrictive
3. Database corruption

**Solutions:**
```typescript
// Check IndexedDB
const db = await window.indexedDB.open('mbot-drawings');
db.onsuccess = () => {
  const transaction = db.result.transaction(['artworks'], 'readonly');
  const store = transaction.objectStore('artworks');
  const countRequest = store.count();
  countRequest.onsuccess = () => {
    console.log('Drawing count:', countRequest.result);
  };
};

// Remove filters
<DrawingGallery moodFilter="all" searchQuery="" />

// Check storage quota
navigator.storage.estimate().then((estimate) => {
  console.log('Used:', estimate.usage);
  console.log('Quota:', estimate.quota);
});
```

### Playback Speed Wrong

**Symptoms:** Animation too fast/slow
**Causes:**
1. Missing timestamps
2. Wrong time calculation
3. Playback speed setting

**Solutions:**
```typescript
// Verify timestamps
const drawing = await artworkStorage.getDrawing(id);
console.log('First stroke:', drawing.strokes[0].timestamp);
console.log('Last stroke:', drawing.strokes[drawing.strokes.length - 1].timestamp);
console.log('Duration:', drawing.duration);

// Reset playback speed
<DrawingPlayback speed={1} /> // 1x = original speed
```

## Game Statistics Issues

### Achievements Not Unlocking

**Symptoms:** Progress shows but not unlocked
**Causes:**
1. Achievement logic error
2. localStorage not updated
3. Cross-tab sync issue

**Solutions:**
```typescript
// Check achievement progress
const achievement = achievements.find(a => a.id === 'first-steps');
console.log('Progress:', achievement.progress);
console.log('Unlocked:', achievement.unlocked);

// Force unlock (for testing)
achievement.unlocked = true;
achievement.unlockedAt = Date.now();
gameStorage.saveAchievements(achievements);

// Sync across tabs
window.addEventListener('storage', (e) => {
  if (e.key === 'mbot-achievements') {
    reloadAchievements();
  }
});
```

## Inventory Dashboard Issues

### NFC Sync Timeout

**Symptoms:** "NFC sync took >5s" error
**Causes:**
1. NFC reader not responding
2. Network latency
3. Database slow

**Solutions:**
```typescript
// Increase timeout (for testing only, violates I-SORT-INV-001)
<InventoryDashboard nfcTimeout={10000} />

// Check NFC reader status
fetch('http://nfc-reader.local/status')
  .then(r => r.json())
  .then(status => console.log('NFC status:', status));

// Manual sync
await inventoryStorage.syncWithNFC();
```

### Stock Alerts Not Showing

**Symptoms:** Low stock but no alert
**Causes:**
1. Threshold misconfigured
2. Alert dismissed
3. Component prop disabled

**Solutions:**
```typescript
// Check thresholds
const station = await inventoryStorage.getStation('red');
console.log('Count:', station.count);
console.log('Low threshold:', station.lowStockThreshold);
console.log('Critical threshold:', station.criticalThreshold);

// Enable alerts
<InventoryDashboard showAlerts={true} />
```

## WebSocket V2 Issues

### Connection Fails

**Symptoms:** "Connection refused" or timeout
**Causes:**
1. Server not running
2. Wrong URL
3. Firewall blocking

**Solutions:**
```bash
# Check server is running
curl http://localhost:4000

# Test WebSocket
wscat -c ws://localhost:4000

# Check firewall
sudo ufw status
sudo ufw allow 4000/tcp
```

### Auto-Reconnect Not Working

**Symptoms:** Stays disconnected
**Causes:**
1. Max attempts reached
2. Reconnect disabled
3. Network issue

**Solutions:**
```typescript
// Check reconnect config
const ws = useWebSocketV2('ws://localhost:4000', {
  reconnectAttempts: Infinity, // Unlimited
  reconnectInterval: 1000 // Start at 1 second
});

// Monitor reconnection
ws.onReconnect((attempt) => {
  console.log('Reconnect attempt:', attempt);
});
```

### State Out of Sync

**Symptoms:** UI shows different values than robot
**Causes:**
1. Missing state snapshot
2. Sequence gap
3. Message loss

**Solutions:**
```typescript
// Force re-sync
const { reconnect } = useWebSocketV2('ws://localhost:4000');
reconnect();

// Check sequence numbers
ws.onMessage((msg) => {
  console.log('Sequence:', msg.sequence);
});
```

---

**Last Updated:** 2026-02-01
