# WebSocket V2 Protocol Specification

**Version:** 2.0.0
**Status:** Production Ready
**Last Updated:** 2026-02-01

## Overview

WebSocket V2 is the real-time communication protocol between the web companion app and mBot robot. It includes state synchronization, message batching, auto-reconnection, and ordered message delivery.

## Connection

### Endpoint

```
ws://localhost:4000/
wss://robot.local:4443/ (production)
```

### Connection Flow

```
Client                           Server
  |                                |
  |---- HTTP Upgrade Request ----->|
  |<--- 101 Switching Protocols ---|
  |                                |
  |<----- State Snapshot ----------|
  |                                |
  |---- Ping (heartbeat) --------->|
  |<--- Pong --------------------- |
  |                                |
  |---- Commands/Events ---------->|
  |<--- State Updates -------------|
```

### Heartbeat

- Client sends ping every 30 seconds
- Server responds with pong within 5 seconds
- Client reconnects if pong not received

## Message Format

### Base Message Structure

```typescript
interface WebSocketMessage {
  type: 'state' | 'command' | 'event' | 'batch' | 'ping' | 'pong';
  version: 2; // Protocol version
  payload: any;
  timestamp: number; // Unix milliseconds
  sequence?: number; // For ordering (I-WS-V2-002)
}
```

### Message Types

#### 1. State Message

Full or partial state update from server.

```typescript
{
  type: 'state',
  version: 2,
  payload: {
    personality: {
      curiosity: 0.8,
      energy: 0.7,
      // ... other parameters
    },
    neuralState: {
      tension: 0.45,
      energy: 0.67,
      coherence: 0.82,
      curiosity: 0.34
    },
    inventory: {
      red: 25,
      green: 30,
      blue: 18,
      yellow: 22
    },
    gameState: { /* optional */ }
  },
  timestamp: 1738454321000
}
```

#### 2. Command Message

Client sends command to robot.

```typescript
{
  type: 'command',
  version: 2,
  payload: {
    command: 'update_personality',
    params: {
      personality: { /* ... */ }
    }
  },
  timestamp: 1738454321100,
  sequence: 123
}
```

**Available Commands:**
- `update_personality` - Update personality parameters
- `start_game` - Start a game (params: game type)
- `stop_game` - Stop current game
- `start_drawing` - Begin drawing session
- `stop_drawing` - End drawing session
- `sort_pieces` - Start LEGO sorting
- `sync_inventory` - Sync inventory with NFC
- `custom` - Custom command (params.action)

#### 3. Event Message

Server sends event notification.

```typescript
{
  type: 'event',
  version: 2,
  payload: {
    event: 'game_won',
    data: {
      game: 'tictactoe',
      score: 100,
      duration: 45000
    }
  },
  timestamp: 1738454322000
}
```

**Common Events:**
- `game_won` - Game victory
- `game_lost` - Game defeat
- `stimulus_detected` - Sensory input (sound, touch, visual)
- `personality_changed` - Personality was updated
- `drawing_complete` - Drawing session ended
- `inventory_updated` - Inventory counts changed
- `error` - Error occurred

#### 4. Batch Message

Multiple messages bundled together (10:1 efficiency).

```typescript
{
  type: 'batch',
  version: 2,
  payload: {
    messages: [
      { type: 'command', /* ... */ },
      { type: 'command', /* ... */ },
      // ... up to 10 messages
    ]
  },
  timestamp: 1738454323000
}
```

**Batching Rules:**
- Window: 100ms (configurable)
- Max size: 10 messages per batch
- Automatic on client and server
- Preserves message order via sequence numbers

#### 5. Ping/Pong Messages

Heartbeat for connection health.

```typescript
// Ping (client -> server)
{
  type: 'ping',
  version: 2,
  payload: {},
  timestamp: 1738454324000
}

// Pong (server -> client)
{
  type: 'pong',
  version: 2,
  payload: {},
  timestamp: 1738454324005
}
```

## State Synchronization

### Initial Snapshot (I-WS-V2-001)

When client connects, server sends full state snapshot:

```typescript
// First message after connection
{
  type: 'state',
  version: 2,
  payload: {
    snapshot: true, // Indicates full state
    personality: { /* complete personality */ },
    neuralState: { /* current brain state */ },
    inventory: { /* all stations */ },
    gameState: { /* if game active */ },
    metadata: {
      robotId: 'mbot-001',
      version: '1.0.0',
      uptime: 3600000
    }
  },
  timestamp: 1738454325000
}
```

### Delta Updates

After initial snapshot, only changes are sent:

```typescript
// Update single field
{
  type: 'state',
  version: 2,
  payload: {
    delta: true,
    field: 'personality.curiosity',
    value: 0.9,
    previous: 0.8
  },
  timestamp: 1738454326000
}
```

### Consistency Guarantee

Contract I-WS-V2-001: Client state always matches robot after sync.

- Client applies all state updates atomically
- If update fails, client requests full re-sync
- Server tracks last-synced sequence per client

## Message Ordering

### Sequence Numbers (I-WS-V2-002)

All commands and critical events include sequence numbers:

```typescript
// Messages sent in order
{
  sequence: 100,
  type: 'command',
  payload: { command: 'update_personality', /* ... */ }
}

{
  sequence: 101,
  type: 'command',
  payload: { command: 'start_game', /* ... */ }
}

{
  sequence: 102,
  type: 'command',
  payload: { command: 'stop_game', /* ... */ }
}
```

### Out-of-Order Handling

Client maintains a buffer for out-of-order messages:

```typescript
// If 103 arrives before 102:
// 1. Buffer message 103
// 2. Wait for 102 (timeout: 5 seconds)
// 3. Process 102, then 103
// 4. If timeout, request re-sync from sequence 102
```

## Auto-Reconnection

### Exponential Backoff

```typescript
const reconnectSchedule = [
  0,      // Attempt 1: Immediate
  1000,   // Attempt 2: 1 second
  2000,   // Attempt 3: 2 seconds
  4000,   // Attempt 4: 4 seconds
  8000,   // Attempt 5: 8 seconds
  16000,  // Attempt 6: 16 seconds
  30000,  // Attempt 7+: 30 seconds (max)
];
```

### Reconnection Flow

```
1. Connection lost
2. Trigger onDisconnect callback
3. Start reconnection timer
4. Attempt reconnect
5. If successful:
   - Request state snapshot
   - Resume normal operation
6. If failed:
   - Increment attempt counter
   - Wait (exponential backoff)
   - Goto step 4
```

### State After Reconnect

```typescript
// After reconnect, server sends fresh snapshot
{
  type: 'state',
  version: 2,
  payload: {
    snapshot: true,
    reconnected: true,
    lastSequence: 105, // Last sequence server received
    // ... full state ...
  },
  timestamp: 1738454330000
}

// Client compares lastSequence to its sent messages
// Resends any messages with sequence > 105
```

## Error Handling

### Error Message

```typescript
{
  type: 'error',
  version: 2,
  payload: {
    code: 'INVALID_COMMAND',
    message: 'Command "invalid_cmd" not recognized',
    sequence: 104, // Which message caused the error
    details: { /* additional context */ }
  },
  timestamp: 1738454331000
}
```

### Error Codes

- `INVALID_COMMAND` - Command not recognized
- `INVALID_PARAMS` - Command parameters invalid
- `STATE_CONFLICT` - State update conflict
- `SEQUENCE_GAP` - Missing sequence numbers detected
- `RATE_LIMIT` - Too many messages (>100/sec)
- `AUTH_REQUIRED` - Authentication needed
- `INTERNAL_ERROR` - Server error

## Performance

### Latency Targets

| Network | Ping/Pong | Command Response |
|---------|-----------|------------------|
| LAN | <10ms | <20ms |
| WiFi | 10-50ms | 20-100ms |
| Internet | 50-200ms | 100-500ms |

### Throughput

- **Without batching**: ~100 msg/sec
- **With batching**: ~1000 msg/sec (10x improvement)
- **Rate limit**: 100 commands/sec per client

### Message Size

- Max message size: 1MB
- Typical message: 100-1000 bytes
- Batch message: 1-10KB

## Security

### Authentication

```typescript
// Include auth token in initial connection
const ws = new WebSocket('ws://localhost:4000', {
  headers: {
    'Authorization': `Bearer ${authToken}`
  }
});

// Or send auth message after connect
ws.send(JSON.stringify({
  type: 'auth',
  version: 2,
  payload: {
    token: authToken
  },
  timestamp: Date.now()
}));
```

### Encryption

- Use WSS (WebSocket Secure) in production
- TLS 1.3 minimum
- Certificate pinning recommended

## Implementation Examples

### Client (TypeScript/React)

```typescript
import { useWebSocketV2 } from '@/hooks/useWebSocketV2';

function MyComponent() {
  const { connected, state, sendCommand, subscribe } = useWebSocketV2(
    'ws://localhost:4000',
    {
      batchWindow: 100, // ms
      maxBatchSize: 10,
      reconnectAttempts: Infinity,
      heartbeatInterval: 30000 // ms
    }
  );

  useEffect(() => {
    const unsubscribe = subscribe('game_won', (data) => {
      console.log('Game won!', data);
    });

    return unsubscribe;
  }, [subscribe]);

  const handleUpdatePersonality = () => {
    sendCommand('update_personality', {
      personality: { curiosity: 0.9, /* ... */ }
    });
  };

  return (
    <div>
      <p>Status: {connected ? 'Connected' : 'Disconnected'}</p>
      <button onClick={handleUpdatePersonality}>Update</button>
    </div>
  );
}
```

### Server (Rust)

```rust
use websocket_v2::{ConnectionManager, MessageBatcher};

let manager = ConnectionManager::new();
let batcher = MessageBatcher::new(Duration::from_millis(100));

// Handle new connection
manager.on_connect(|conn| {
    // Send initial state snapshot
    let state = get_full_state();
    conn.send_state(state, true /* snapshot */);
});

// Handle incoming messages
manager.on_message(|conn, msg| {
    match msg.msg_type.as_str() {
        "command" => {
            let cmd = msg.payload["command"].as_str()?;
            handle_command(conn, cmd, msg.payload["params"]);
        },
        "ping" => {
            conn.send_pong();
        },
        _ => {}
    }
});

// Handle disconnection
manager.on_disconnect(|conn_id| {
    cleanup_connection(conn_id);
});
```

---

**Last Updated:** 2026-02-01
**Status:** Production Ready ✅
