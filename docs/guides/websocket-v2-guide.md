import { useWebSocketV2 } from '@/hooks/useWebSocketV2';

const ws = useWebSocketV2('ws://localhost:4000', {
  batchWindow: 100, // milliseconds
  maxBatchSize: 10  // max messages per batch
});
```

---

## Auto-Reconnect

### Exponential Backoff

Connection retry schedule:

```
Attempt 1: Immediate
Attempt 2: 1 second wait
Attempt 3: 2 seconds wait
Attempt 4: 4 seconds wait
Attempt 5: 8 seconds wait
Attempt 6: 16 seconds wait
Attempt 7+: 30 seconds wait (max)
```

### Monitoring Reconnection

```typescript
const { connected, reconnecting, reconnectAttempts } = useWebSocketV2('ws://localhost:4000');

if (reconnecting) {
  console.log(`Reconnecting... attempt ${reconnectAttempts}`);
}
```

### Manual Reconnect

```typescript
const { reconnect } = useWebSocketV2('ws://localhost:4000');

// Force reconnect
reconnect();
```

---

## Message Ordering

### Contract I-WS-V2-002: Sequence Numbers

Messages include sequence numbers for ordering:

```typescript
{
  type: 'command',
  sequence: 123,
  payload: { /* ... */ }
}

{
  type: 'command',
  sequence: 124,
  payload: { /* ... */ }
}
```

### Out-of-Order Detection

```typescript
// Client automatically reorders messages
// If sequence 125 arrives before 124, it waits
```

---

## Event Subscription

### Subscribe to Events

```typescript
import { useWebSocketV2 } from '@/hooks/useWebSocketV2';

function EventListener() {
  const { subscribe } = useWebSocketV2('ws://localhost:4000');

  useEffect(() => {
    const unsubscribe = subscribe('game_won', (data) => {
      console.log('Game won!', data);
    });

    return unsubscribe;
  }, []);
}
```

### Available Events

- `game_won` - Game victory
- `game_lost` - Game defeat
- `stimulus_detected` - Sensory input
- `personality_changed` - Personality update
- `error` - Error occurred
- Custom events from your robot

---

## Server Implementation (Rust)

### Backend Setup

```rust
// In crates/mbot-companion/src/websocket_v2.rs
use websocket_v2::{ConnectionManager, MessageBatcher};

let manager = ConnectionManager::new();
let batcher = MessageBatcher::new(100); // 100ms window

// Handle connection
manager.on_connect(|conn| {
    // Send full state snapshot
    conn.send_state(get_full_state());
});

// Handle messages
manager.on_message(|msg| {
    match msg.type {
        "command" => handle_command(msg.payload),
        _ => {}
    }
});
```

---

## Performance

### Latency

- **LAN**: <10ms ping/pong
- **WiFi**: 10-50ms
- **Internet**: 50-200ms

### Throughput

- **Without batching**: ~100 msg/sec
- **With batching**: ~1000 msg/sec (10x improvement)

---

## Troubleshooting

### Connection fails

```typescript
// Check server is running
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" \
  -H "Host: localhost:4000" \
  -H "Origin: http://localhost:3000" \
  http://localhost:4000/
```

### State out of sync

```typescript
// Force re-sync
const { reconnect } = useWebSocketV2('ws://localhost:4000');
reconnect(); // Triggers full state sync
```

### Messages not batching

```typescript
// Verify batch window is set
const ws = useWebSocketV2('ws://localhost:4000', {
  batchWindow: 100 // Must be > 0
});
```

---

## Related Features

- [Multi-Robot Discovery](multi-robot-discovery-guide.md)
- [Multi-Robot Coordination](multi-robot-coordination-guide.md)
- [Mobile App](mobile-app-guide.md)
- [Cloud Sync](cloud-sync-guide.md)

---

## API Reference

See: [WebSocket V2 API](../api/WAVE_6_APIs.md#websocket-v2)

---

**Last Updated:** 2026-02-01
**Status:** Production Ready ✅
