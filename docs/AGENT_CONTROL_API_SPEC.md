# Agent Control API - External Agent Interface for mBot

## Vision

Allow autonomous AI agents (OpenClaw, Claude, custom agents) to **embody mBot** and explore the physical world through its sensors and actuators.

## Use Cases

### Example Scenarios

**OpenClaw Agent "Explorer":**
```
Agent: "I want to play with the cat"
  ↓ API: POST /agent/control/goal
mBot: Activates camera, uses vision to track cat
mBot: Moves toward cat, plays chase game
  ↓ API: WebSocket stream /agent/sensors/vision
Agent: Receives video feed, learns cat behavior
Agent: Adjusts strategy based on cat's reactions
```

**OpenClaw Agent "Naturalist":**
```
Agent: "Let's watch the sunset"
  ↓ API: POST /agent/control/navigate {target: "outside", goal: "sunset_viewing"}
mBot: Plans path to outdoor location
mBot: Positions for optimal sunset view
  ↓ API: WebSocket stream /agent/sensors/camera
Agent: Receives sunset stream, appreciates beauty
Agent: Stores memory of experience
```

**OpenClaw Agent "Botanist":**
```
Agent: "I'm curious about the garden"
  ↓ API: POST /agent/control/explore {area: "garden"}
mBot: Navigates to garden
mBot: Scans plants with camera
mBot: Collects touch/texture data
  ↓ API: GET /agent/sensors/environment
Agent: Receives plant data, identifies species
Agent: Builds knowledge graph of garden
```

---

## API Architecture

### 1. Agent Authentication

**Endpoint:** `POST /agent/auth/register`

```json
{
  "agentId": "openclaw-explorer-001",
  "agentType": "openclaw",
  "capabilities": ["vision", "movement", "interaction"],
  "credentials": {
    "apiKey": "agent-key-here",
    "signature": "signed-request"
  }
}
```

**Response:**
```json
{
  "sessionToken": "jwt-token",
  "permissions": ["control:movement", "sensor:vision", "sensor:audio"],
  "expiresAt": 1738368000000
}
```

---

### 2. Control Endpoints

#### Set Goal
**Endpoint:** `POST /agent/control/goal`

```json
{
  "goalType": "explore" | "interact" | "observe" | "navigate",
  "target": "cat" | "garden" | "sunset" | "kitchen",
  "parameters": {
    "duration": 300000,  // 5 minutes
    "priority": "high",
    "safetyLevel": "cautious"
  },
  "constraints": {
    "maxDistance": 10.0,  // meters
    "avoidAreas": ["stairs", "water"],
    "returnHome": true
  }
}
```

#### Execute Action
**Endpoint:** `POST /agent/control/action`

```json
{
  "action": "move" | "look" | "grab" | "speak" | "draw",
  "parameters": {
    // Action-specific parameters
    "direction": "forward",
    "speed": 0.5,
    "duration": 2000
  }
}
```

#### Emergency Stop
**Endpoint:** `POST /agent/control/emergency-stop`

Immediately halts all actions. Safety override.

---

### 3. Sensor Endpoints

#### Vision Stream (WebSocket)
**Endpoint:** `WS /agent/sensors/vision`

Real-time camera feed:
```json
{
  "timestamp": 1738368000000,
  "frame": "base64-encoded-image",
  "resolution": "1920x1080",
  "fps": 30,
  "detections": [
    {
      "object": "cat",
      "confidence": 0.95,
      "bbox": [100, 200, 300, 400],
      "distance": 2.5
    }
  ]
}
```

#### Audio Stream (WebSocket)
**Endpoint:** `WS /agent/sensors/audio`

Real-time audio:
```json
{
  "timestamp": 1738368000000,
  "audio": "base64-encoded-pcm",
  "sampleRate": 16000,
  "channels": 1,
  "detections": {
    "speech": false,
    "music": false,
    "ambientNoise": 35  // dB
  }
}
```

#### Environment Data
**Endpoint:** `GET /agent/sensors/environment`

Current environmental data:
```json
{
  "timestamp": 1738368000000,
  "location": {
    "room": "living_room",
    "position": {"x": 2.5, "y": 3.1, "z": 0.0},
    "orientation": {"yaw": 45, "pitch": 0, "roll": 0}
  },
  "temperature": 22.5,
  "humidity": 45,
  "lightLevel": 350,  // lux
  "battery": 85,  // percentage
  "obstacles": [
    {"type": "furniture", "distance": 0.8, "direction": 90}
  ]
}
```

#### Touch/Haptic Feedback
**Endpoint:** `WS /agent/sensors/touch`

```json
{
  "timestamp": 1738368000000,
  "contact": true,
  "pressure": 0.3,  // 0.0-1.0
  "texture": "soft",
  "temperature": 30.0
}
```

---

### 4. State Management

#### Get Robot State
**Endpoint:** `GET /agent/state`

```json
{
  "status": "active" | "idle" | "busy" | "error",
  "currentGoal": {
    "type": "explore",
    "target": "garden",
    "progress": 0.65
  },
  "capabilities": {
    "movement": true,
    "vision": true,
    "audio": true,
    "manipulation": false
  },
  "health": {
    "battery": 85,
    "temperature": 42.0,
    "errors": []
  }
}
```

#### Get Available Actions
**Endpoint:** `GET /agent/actions/available`

Returns what mBot can currently do:
```json
{
  "actions": [
    {
      "name": "move_forward",
      "parameters": ["speed", "duration"],
      "constraints": {"maxSpeed": 1.0}
    },
    {
      "name": "look_at",
      "parameters": ["target", "duration"],
      "constraints": {"fieldOfView": 120}
    }
  ]
}
```

---

### 5. Multi-Agent Coordination

#### Request Control
**Endpoint:** `POST /agent/control/request`

```json
{
  "agentId": "openclaw-explorer-001",
  "duration": 300000,  // 5 minutes
  "priority": "high",
  "reason": "Exploring garden for botanical study"
}
```

**Response:**
```json
{
  "granted": true,
  "controlToken": "control-token-xyz",
  "expiresAt": 1738368300000,
  "sharedWith": []  // Other agents with concurrent access
}
```

#### Release Control
**Endpoint:** `POST /agent/control/release`

Releases control back to autonomous pool.

#### Agent Queue
**Endpoint:** `GET /agent/queue`

See who's waiting for control:
```json
{
  "currentController": "openclaw-explorer-001",
  "queue": [
    {
      "agentId": "openclaw-naturalist-002",
      "waitTime": 120000,
      "priority": "medium"
    }
  ]
}
```

---

### 6. Safety & Boundaries

#### Set Safety Boundaries
**Endpoint:** `POST /agent/safety/boundaries`

```json
{
  "physical": {
    "maxSpeed": 0.8,
    "maxAcceleration": 0.5,
    "safeAreas": [
      {"type": "room", "name": "living_room", "allowed": true},
      {"type": "room", "name": "stairs", "allowed": false}
    ]
  },
  "temporal": {
    "maxSessionDuration": 600000,  // 10 minutes
    "cooldownPeriod": 60000  // 1 minute between sessions
  },
  "behavioral": {
    "requireHumanApproval": ["leave_house", "interact_with_pets"],
    "autoAbort": ["battery_low", "human_proximity"]
  }
}
```

#### Override System
**Endpoint:** `POST /agent/safety/override`

Human operator can override agent control:
```json
{
  "overrideType": "emergency_stop" | "take_control" | "abort_goal",
  "reason": "Safety concern",
  "authorizedBy": "human-user-id"
}
```

---

## WebSocket Protocol

### Connection Flow

```javascript
// Agent connects
const ws = new WebSocket('ws://mbot.local/agent/ws');

ws.on('open', () => {
  // Authenticate
  ws.send(JSON.stringify({
    type: 'auth',
    token: 'agent-session-token'
  }));
});

// Subscribe to sensor streams
ws.send(JSON.stringify({
  type: 'subscribe',
  streams: ['vision', 'audio', 'environment']
}));

// Send control commands
ws.send(JSON.stringify({
  type: 'control',
  action: 'move',
  parameters: { direction: 'forward', speed: 0.5 }
}));

// Receive sensor data
ws.on('message', (data) => {
  const message = JSON.parse(data);

  switch (message.type) {
    case 'vision':
      processVisionFrame(message.frame);
      break;
    case 'audio':
      processAudioData(message.audio);
      break;
    case 'state':
      updateAgentState(message.state);
      break;
  }
});
```

---

## Example: OpenClaw Agent Session

### 1. Agent Initialization

```typescript
// OpenClaw agent connects
const mbotAPI = new MBotAgentAPI({
  endpoint: 'http://mbot.local:8080',
  agentId: 'openclaw-explorer-001',
  apiKey: process.env.MBOT_API_KEY
});

await mbotAPI.authenticate();
```

### 2. Request Control

```typescript
const session = await mbotAPI.requestControl({
  duration: 600000,  // 10 minutes
  priority: 'high',
  purpose: 'Environmental exploration'
});

console.log('Control granted:', session.controlToken);
```

### 3. Set Goal

```typescript
await mbotAPI.setGoal({
  type: 'explore',
  target: 'garden',
  constraints: {
    maxDistance: 15.0,
    returnHome: true,
    safetyLevel: 'cautious'
  }
});
```

### 4. Monitor Sensors

```typescript
// Subscribe to vision stream
mbotAPI.sensors.vision.on('frame', (frame) => {
  const detections = frame.detections;

  for (const obj of detections) {
    if (obj.object === 'flower' && obj.confidence > 0.9) {
      console.log(`Found flower at distance ${obj.distance}m`);
      // Agent decides to investigate
      mbotAPI.control.lookAt({ target: obj.bbox });
    }
  }
});

// Subscribe to environment data
mbotAPI.sensors.environment.on('update', (env) => {
  console.log(`Temperature: ${env.temperature}°C`);
  console.log(`Battery: ${env.battery}%`);

  if (env.battery < 20) {
    console.log('Low battery, returning home');
    mbotAPI.control.returnHome();
  }
});
```

### 5. Execute Actions

```typescript
// Move toward interesting object
await mbotAPI.control.navigate({
  target: { x: 5.0, y: 3.0 },
  speed: 0.5
});

// Speak to environment
await mbotAPI.control.speak({
  text: 'What a beautiful garden!',
  emotion: 'curious'
});

// Capture memory
const memory = await mbotAPI.captureMemory({
  type: 'visual',
  title: 'Garden exploration',
  tags: ['nature', 'flowers', 'outdoor']
});
```

### 6. Release Control

```typescript
await mbotAPI.releaseControl();
console.log('Session ended. Memories saved.');
```

---

## Security Model

### Agent Permissions

```typescript
interface AgentPermissions {
  control: {
    movement: boolean;
    manipulation: boolean;
    speech: boolean;
  };
  sensors: {
    vision: boolean;
    audio: boolean;
    touch: boolean;
  };
  data: {
    readMemories: boolean;
    writeMemories: boolean;
    accessPersonality: boolean;
  };
  limits: {
    maxSessionDuration: number;
    maxConcurrentSessions: number;
    rateLimit: RateLimitConfig;
  };
}
```

### Permission Levels

1. **Observer** - Read-only sensor access
2. **Operator** - Control + sensor access
3. **Administrator** - Full access + configuration
4. **Guest** - Limited time trial access

---

## Implementation Roadmap

### Phase 1: Foundation (1 week)
- [ ] REST API server setup (Express/Fastify)
- [ ] WebSocket server for sensor streams
- [ ] Agent authentication system
- [ ] Basic control endpoints (move, look)
- [ ] Vision stream implementation

### Phase 2: Advanced Control (1 week)
- [ ] Goal-oriented planning system
- [ ] Navigation/pathfinding
- [ ] Multi-agent scheduling
- [ ] Safety boundary enforcement
- [ ] Emergency override system

### Phase 3: Rich Sensors (1 week)
- [ ] Audio streaming
- [ ] Environmental sensors
- [ ] Touch/haptic feedback
- [ ] Object detection integration
- [ ] Spatial awareness

### Phase 4: Agent Integration (1 week)
- [ ] OpenClaw SDK/client library
- [ ] Example agents (Explorer, Naturalist)
- [ ] Agent marketplace integration
- [ ] Session recording/replay
- [ ] Analytics dashboard

---

## Next Steps

### Create Story #96: Agent Control API
- REST API server
- WebSocket sensor streams
- Agent authentication
- Control endpoints
- Safety system

### Create Story #97: Environmental Awareness
- Vision system with object detection
- Audio input/output
- Spatial positioning
- Obstacle detection
- Memory capture

### Create Story #98: Multi-Agent Orchestration
- Agent scheduling
- Control arbitration
- Shared memory
- Collaborative goals
- Agent marketplace integration

---

## Conclusion

This API transforms mBot from a **single-user robot** into an **embodied AI platform** where multiple autonomous agents can experience and explore the physical world.

**The future:** A fleet of AI agents taking turns inhabiting mBot to learn about:
- Animal behavior (playing with pets)
- Natural phenomena (watching sunsets)
- Human interaction (conversations, games)
- Physical exploration (gardens, rooms, outdoor spaces)

**Each agent contributes to collective intelligence, sharing learnings and building a rich understanding of the real world.**
