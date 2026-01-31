# Multi-Robot Discovery Protocol

**Issue:** #77 - STORY-ARCH-008
**Contract:** ARCH-005 (Transport Layer Abstraction)
**Invariant:** I-DISC-001 (mDNS Standard Protocol - RFC 6762)
**DOD Criticality:** Future - Can release without

## Overview

Implements mDNS-based discovery protocol for finding and connecting to multiple mBot robots on the local network. Provides a real-time UI panel for discovering, connecting, and monitoring robot health.

## Architecture

### Browser Limitations

Browsers cannot directly perform mDNS queries due to security restrictions. This implementation provides:

1. **WebSocket-based discovery** - Primary method connecting to backend mDNS bridge
2. **HTTP polling fallback** - Secondary method if WebSocket unavailable
3. **Mock service** - Development and testing without real robots

### Components

```
┌─────────────────────────────────────────┐
│   RobotDiscovery Component (UI)         │
│   - Discovery panel                     │
│   - Robot cards                         │
│   - Connect/disconnect buttons          │
└────────────┬────────────────────────────┘
             │
             ├─── useRobotDiscovery (hook)
             │    └─── RobotDiscoveryService
             │         ├─── WebSocket: ws://localhost:8081/discovery
             │         └─── HTTP: /api/discovery/robots
             │
             └─── useRobotConnection (hook)
                  └─── WebSocket: ws://{robot-ip}:{port}
```

## Files Structure

### TypeScript Types
- `web/src/types/discovery.ts` - All discovery-related interfaces

### Services
- `web/src/services/robotDiscovery.ts` - Discovery service implementation
  - `RobotDiscoveryService` - Production service
  - `MockDiscoveryService` - Development/testing service

### React Hooks
- `web/src/hooks/useRobotDiscovery.ts` - Discovery state management
- `web/src/hooks/useRobotConnection.ts` - Robot connection management

### Components
- `web/src/components/RobotDiscovery.tsx` - Main UI component
- `web/src/components/RobotDiscovery.css` - Styles

### Tests
- `tests/integration/multi-robot-discovery.test.ts` - Integration tests
- `web/src/components/__tests__/RobotDiscovery.test.tsx` - Component tests

## Data Contract

```typescript
interface DiscoveredRobot {
  id: string;           // Unique robot identifier
  name: string;         // Human-readable name
  ipAddress: string;    // IPv4 address
  port: number;         // WebSocket port
  version: string;      // Software version (semver)
}

interface RobotWithState extends DiscoveredRobot {
  status: 'connected' | 'disconnected' | 'error' | 'discovering';
  lastSeen: number;     // Unix timestamp
  metadata?: {
    model?: string;
    firmware?: string;
    capabilities?: string[];
    uptime?: number;
  };
}
```

## Usage

### Basic Usage

```tsx
import { RobotDiscovery } from './components/RobotDiscovery';
import { MockDiscoveryService } from './services/robotDiscovery';

function App() {
  const discoveryService = new MockDiscoveryService();

  return (
    <RobotDiscovery
      discoveryService={discoveryService}
      onRobotConnect={(robotId) => console.log('Connected:', robotId)}
      onRobotDisconnect={(robotId) => console.log('Disconnected:', robotId)}
    />
  );
}
```

### Production Setup

```tsx
import { RobotDiscoveryService } from './services/robotDiscovery';

const discoveryService = new RobotDiscoveryService({
  serviceName: '_mbot._tcp.local',  // mDNS service name (RFC 6762)
  timeout: 5000,                     // Connection timeout
  pollingInterval: 3000,             // HTTP polling interval
});

<RobotDiscovery discoveryService={discoveryService} />
```

## Backend Requirements

The frontend requires a backend mDNS bridge service:

### WebSocket Discovery Endpoint

**URL:** `ws://localhost:8081/discovery`

**Messages from backend:**

```json
// Initial robot list
{
  "type": "robot_list",
  "robots": [
    {
      "id": "mbot-001",
      "name": "mBot Alpha",
      "ipAddress": "192.168.1.100",
      "port": 8081,
      "version": "1.0.0"
    }
  ]
}

// New robot discovered
{
  "type": "robot_discovered",
  "robot": { /* DiscoveredRobot */ }
}

// Robot lost
{
  "type": "robot_lost",
  "robotId": "mbot-001"
}

// Robot updated
{
  "type": "robot_updated",
  "robot": { /* DiscoveredRobot */ }
}
```

### HTTP Polling Endpoint (Fallback)

**URL:** `GET /api/discovery/robots`

**Response:**

```json
[
  {
    "id": "mbot-001",
    "name": "mBot Alpha",
    "ipAddress": "192.168.1.100",
    "port": 8081,
    "version": "1.0.0"
  }
]
```

## mDNS Implementation (Backend)

The backend should use standard mDNS libraries:

- **Node.js:** `multicast-dns`, `bonjour`
- **Rust:** `mdns-sd`, `libmdns`
- **Python:** `zeroconf`, `python-zeroconf`

### Service Registration (Robot Side)

```rust
// Robot advertises itself via mDNS
service_name = "_mbot._tcp.local"
port = 8081
txt_records = {
  "version": "1.0.0",
  "model": "mBot2",
  "firmware": "2.1.0",
  "capabilities": "drawing,personality,games"
}
```

### Service Discovery (Backend Side)

```rust
// Backend discovers robots and forwards to frontend
browse_for_service("_mbot._tcp.local")
  .on_service_discovered(|service| {
    send_to_frontend(DiscoveryEvent {
      type: "robot_discovered",
      robot: service.to_robot()
    })
  })
```

## Testing

### Run Integration Tests

```bash
npm test -- tests/integration/multi-robot-discovery.test.ts
```

### Run Component Tests

```bash
npm test -- web/src/components/__tests__/RobotDiscovery.test.tsx
```

### Manual Testing with Mock Service

```tsx
import { MockDiscoveryService } from './services/robotDiscovery';

const mockService = new MockDiscoveryService();
await mockService.start();

// Simulate discovering a new robot
mockService.simulateDiscovery({
  id: 'mbot-999',
  name: 'Test Robot',
  ipAddress: '192.168.1.200',
  port: 8081,
  version: '1.0.0'
});
```

## Gherkin Scenarios

### Scenario: Discover Robots

```gherkin
Given 3 robots on network
When I open discovery panel
Then I see 3 robots listed
And each shows name and IP
```

**Implementation:** `MockDiscoveryService` provides 3 robots by default.

### Scenario: Connect to Robot

```gherkin
When I click "Connect" on Robot2
Then WebSocket opens to Robot2
And UI shows Robot2 state
```

**Implementation:** Click connect button → `useRobotConnection.connect()` → WebSocket to robot IP.

## data-testid Selectors

| Element | data-testid | Purpose |
|---------|-------------|---------|
| Discovery panel | `robot-discovery-panel` | Main container |
| Discovery toggle | `discovery-toggle` | Start/stop discovery |
| Refresh button | `refresh-button` | Manual refresh |
| Robots list | `robots-list` | Container for robot cards |
| No robots message | `no-robots-message` | Empty state |
| Discovery error | `discovery-error` | Error display |
| Discovery stats | `discovery-stats` | Footer statistics |
| Robot card | `robot-card-{id}` | Individual robot |
| Robot name | `robot-name-{id}` | Robot name |
| Robot IP | `robot-ip-{id}` | IP address |
| Robot port | `robot-port-{id}` | Port number |
| Robot version | `robot-version-{id}` | Software version |
| Robot firmware | `robot-firmware-{id}` | Firmware version |
| Robot status | `robot-status-{id}` | Health indicator |
| Connect button | `connect-button-{id}` | Connect action |
| Disconnect button | `disconnect-button-{id}` | Disconnect action |

## Invariants

### I-DISC-001: mDNS Standard

**MUST** use standard mDNS protocol (RFC 6762).

**Implementation:**
- Service name format: `_<service>._<proto>.<domain>`
- Example: `_mbot._tcp.local`
- Standard port range: 1-65535
- IPv4 address format validation
- Semantic versioning (major.minor.patch)

**Validation:** See `tests/integration/multi-robot-discovery.test.ts` - "Discovery Protocol Compliance" suite.

## Integration with WebSocket V2

This feature integrates with Issue #76 (WebSocket V2):

1. Discovery finds robots via mDNS
2. Connect button opens WebSocket to robot
3. WebSocket V2 handles bidirectional communication
4. Connection state synced with discovery service

## Future Enhancements (Not in Scope)

- Robot pairing and authentication
- Swarm coordination (Wave 7)
- Robot grouping and favorites
- Network topology visualization
- Bandwidth and latency monitoring
- Automatic failover to backup robot

## Dependencies

- **#64** - Foundation
- **#76** - WebSocket V2 (parallel, coordinate when ready)

## Blocks

- **#70** - Multi-robot coordination features

## Status

✅ **Implementation Complete**
- Types and interfaces
- Discovery service (WebSocket + HTTP fallback)
- Mock service for development
- React hooks for discovery and connection
- UI component with all required elements
- Integration tests
- Component tests
- Full data-testid coverage

🔄 **Pending**
- Backend mDNS bridge implementation
- Real robot testing
- WebSocket V2 integration (depends on #76)

## Related Files

- Issue: https://github.com/Hulupeep/mbot_ruvector/issues/77
- Contract: `docs/contracts/feature_architecture.yml` (ARCH-005)
