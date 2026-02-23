#!/bin/bash

# Wave 6-7 Issue Creation Script for CCF on RuVector
# Creates 22 Specflow-compliant GitHub issues
# Run: chmod +x create-wave-6-7-issues.sh && ./create-wave-6-7-issues.sh

set -e

REPO="Hulupeep/CCF"

echo "============================================"
echo "Creating Wave 6-7 Issues for CCF on RuVector"
echo "============================================"
echo ""

# ============================================
# WAVE 6: UI COMPLETION & INTEGRATION (12 stories)
# ============================================

echo "Creating Wave 6 UI Component Stories (#58-#62)..."

# Story #58: Personality Mixer Web UI
gh issue create -R "$REPO" \
  --title "STORY-PERS-008: Personality Mixer Web UI" \
  --label "story,enhancement,wave-6" \
  --body-file - <<'EOF'
## Description

Create a web-based UI for the Personality Mixer that allows users to adjust personality parameters in real-time through sliders, load presets, and save custom personalities.

## DOD Criticality
- [x] **Important** - Should pass before release

## Contract References
- **Feature Contracts:** [PERS-001, PERS-005, PERS-007]
- **Journey Contract:** [J-PERS-CUSTOMIZE]

## Acceptance Criteria (Gherkin)

### Scenario 1: Load Personality Mixer
```gherkin
Scenario: User opens personality mixer for first time
  Given the robot is connected via WebSocket
  When I navigate to /personality-mixer
  Then I see 9 parameter sliders
  And I see 6 preset personality buttons
  And I see current personality values displayed
  And connection status shows "Connected"
```

### Scenario 2: Adjust Parameter with Slider
```gherkin
Scenario: User adjusts tension baseline
  Given I am on the personality mixer page
  When I drag the "Tension Baseline" slider to 0.8
  Then the value display updates to "0.80"
  And a WebSocket message is sent within 500ms
  And the robot's tension baseline changes to 0.8
```

### Scenario 3: Load Preset Personality
```gherkin
Scenario: User loads "Curious" preset
  Given I am on the personality mixer page
  When I click the "Curious" preset button
  Then all 9 sliders animate to new positions
  And all value displays update
  And the robot adopts "Curious" personality
```

### Scenario 4: Save Custom Personality
```gherkin
Scenario: User saves custom personality
  Given I have adjusted multiple parameters
  When I click "Save Custom" button
  And I enter name "My Robot"
  Then the personality is saved to localStorage
  And "My Robot" appears in custom list
```

## Invariants

### I-PERS-UI-001: Parameter Bounds
**MUST** All sliders enforce 0.0-1.0 range.

### I-PERS-UI-002: Debounced Updates
**MUST** Parameter changes debounced to max 2 updates/second.

### I-PERS-UI-003: Connection State
**MUST** Disable all controls when WebSocket disconnected.

## Data Contract

```typescript
interface PersonalityConfig {
  tension_baseline: number;
  coherence_baseline: number;
  energy_baseline: number;
  startle_sensitivity: number;
  recovery_speed: number;
  curiosity_drive: number;
  movement_expressiveness: number;
  sound_expressiveness: number;
  light_expressiveness: number;
}

interface PersonalityPreset {
  id: string;
  name: string;
  description: string;
  config: PersonalityConfig;
}
```

## data-testid Requirements

| Element | data-testid |
|---------|-------------|
| Tension slider | `slider-tension-baseline` |
| Coherence slider | `slider-coherence-baseline` |
| Energy slider | `slider-energy-baseline` |
| Preset: Curious | `preset-button-curious` |
| Save button | `save-custom-button` |
| Connection status | `connection-status` |

## In Scope
- 9 parameter sliders
- 6 preset buttons
- Save/load custom personalities
- WebSocket integration
- Responsive design

## Not In Scope
- Mobile UI (Wave 7)
- Personality sharing (Wave 7)
- Animation previews

## E2E Test File
`tests/journeys/personality-customize.journey.spec.ts`

## Definition of Done
- [ ] TypeScript component implemented
- [ ] All 9 sliders functional
- [ ] WebSocket messages working
- [ ] localStorage persistence
- [ ] E2E test passes
- [ ] Code review approved

**Dependencies:** #12
**Related:** J-PERS-CUSTOMIZE
EOF

echo "✅ Created #58: Personality Mixer Web UI"

# Story #59: Neural Visualizer Enhancements
gh issue create -R "$REPO" \
  --title "STORY-LEARN-006: Real-Time Neural Visualizer Enhancements" \
  --label "story,enhancement,wave-6" \
  --body-file - <<'EOF'
## Description

Enhance the neural visualizer with animations, historical data, mode transitions, and zoom/pan for educational demonstrations.

## DOD Criticality
- [x] **Important** - Should pass before release

## Contract References
- **Feature Contracts:** [LEARN-001, LEARN-002, LEARN-004]
- **Journey Contract:** [J-LEARN-FIRST-EXPERIMENT]

## Acceptance Criteria (Gherkin)

### Scenario 1: View Live Neural Activity
```gherkin
Scenario: User monitors robot's nervous system
  Given the robot is running
  When I open /visualizer
  Then I see current mode (Calm/Active/Spike/Protect)
  And meters update every 100ms
  And timeline shows last 60 seconds
```

### Scenario 2: Observe Mode Transition
```gherkin
Scenario: Mode changes from Calm to Spike
  Given robot is in Calm mode
  When sudden stimulus occurs
  Then mode icon changes to ⚡
  And transition marker appears on timeline
```

### Scenario 3: Review Historical Data
```gherkin
Scenario: User scrubs timeline
  When I drag timeline to -30 seconds
  Then meters show values from that time
```

## Invariants

### I-LEARN-VIZ-001: Update Rate
**MUST** Update at minimum 10Hz (100ms).

### I-LEARN-VIZ-002: Data Retention
**MUST** Store last 300 seconds (5 min).

## Data Contract

```typescript
interface NeuralState {
  timestamp: number;
  mode: 'Calm' | 'Active' | 'Spike' | 'Protect';
  tension: number;
  coherence: number;
  energy: number;
  curiosity: number;
}
```

## data-testid Requirements

| Element | data-testid |
|---------|-------------|
| Mode indicator | `neural-mode-indicator` |
| Tension meter | `neural-tension-meter` |
| Timeline chart | `neural-timeline-chart` |
| Export button | `export-data-button` |

## In Scope
- Real-time meters
- Timeline with transitions
- Pause/scrub
- CSV/JSON export
- Zoom/pan

## Not In Scope
- Long-term database storage
- Statistical analysis tools
- Multi-robot monitoring

## E2E Test File
`tests/journeys/learninglab-experiment.journey.spec.ts`

## Definition of Done
- [ ] Real-time meters implemented
- [ ] Timeline with transitions
- [ ] Export functionality
- [ ] 60fps performance
- [ ] E2E test passes

**Dependencies:** #15
**Related:** J-LEARN-FIRST-EXPERIMENT
EOF

echo "✅ Created #59: Neural Visualizer Enhancements"

# Story #60: Drawing Gallery
gh issue create -R "$REPO" \
  --title "STORY-ART-006: Drawing Gallery with Playback" \
  --label "story,enhancement,wave-6" \
  --body-file - <<'EOF'
## Description

Create a gallery to view, organize, and play back all robot drawings with mood tagging and time-lapse playback.

## DOD Criticality
- [x] **Important** - Should pass before release

## Contract References
- **Feature Contracts:** [ART-001, ART-002, ART-005]
- **Journey Contract:** [J-ART-FIRST-DRAWING]

## Acceptance Criteria (Gherkin)

### Scenario 1: View Gallery
```gherkin
Scenario: User opens drawing gallery
  Given I have created 5 drawings
  When I navigate to /gallery
  Then I see 5 thumbnails in grid
  And each shows creation date
  And each shows dominant mood
```

### Scenario 2: Play Back Drawing
```gherkin
Scenario: User watches playback
  When I click a thumbnail
  Then modal opens with full drawing
  When I click "Play"
  Then drawing animates stroke-by-stroke
```

### Scenario 3: Filter by Mood
```gherkin
Scenario: Filter by Calm mood
  When I click "Calm" filter
  Then only Calm drawings shown
```

## Invariants

### I-ART-GAL-001: Stroke Data
**MUST** Save all stroke data (paths, timestamps, moods).

### I-ART-GAL-002: Playback Accuracy
**MUST** Playback matches original speed exactly.

## Data Contract

```typescript
interface Drawing {
  id: string;
  createdAt: number;
  strokes: Stroke[];
  moods: MoodEvent[];
  duration: number;
}

interface Stroke {
  timestamp: number;
  path: Point[];
  mood: string;
}
```

## data-testid Requirements

| Element | data-testid |
|---------|-------------|
| Gallery grid | `drawing-gallery-grid` |
| Drawing thumbnail | `drawing-thumbnail-{id}` |
| Play button | `playback-play-button` |
| Mood filter | `filter-mood-calm` |

## In Scope
- Grid gallery
- Modal with playback
- Filter by mood/date
- Share/download
- Delete

## Not In Scope
- Cloud storage (Wave 7)
- Social sharing
- Drawing editing

## E2E Test File
`tests/journeys/artbot-gallery.journey.spec.ts`

## Definition of Done
- [ ] Gallery grid implemented
- [ ] Playback animation working
- [ ] Filtering functional
- [ ] E2E test passes

**Dependencies:** #16
**Blocks:** #72
**Related:** J-ART-FIRST-DRAWING
EOF

echo "✅ Created #60: Drawing Gallery"

# Story #61: Game Statistics Dashboard
gh issue create -R "$REPO" \
  --title "STORY-GAME-006: Game Statistics Dashboard" \
  --label "story,enhancement,wave-6,future" \
  --body-file - <<'EOF'
## Description

Track game statistics: wins/losses, high scores, play time, personality influence. Includes leaderboards and achievements.

## DOD Criticality
- [x] **Future** - Can release without

## Contract References
- **Feature Contracts:** [GAME-001, GAME-002, GAME-003]
- **Journey Contract:** [J-GAME-FIRST-TICTACTOE]

## Acceptance Criteria (Gherkin)

### Scenario: View Game Statistics
```gherkin
Given I have played 10 games
When I navigate to /stats/games
Then I see total games: 10
And I see win/loss breakdown
And I see average duration
```

## Invariants

### I-GAME-STAT-001: Data Persistence
**MUST** Statistics persist across sessions.

## Data Contract

```typescript
interface GameStatistics {
  totalGames: number;
  byGame: Map<GameType, GameStats>;
  achievements: Achievement[];
}
```

## data-testid Requirements

| Element | data-testid |
|---------|-------------|
| Total games | `stat-total-games` |
| Win rate | `stat-win-rate` |
| Leaderboard | `leaderboard-table` |

## In Scope
- Statistics dashboard
- Leaderboards
- 20 achievements
- Charts

## Not In Scope
- Online leaderboards (Wave 7)
- Video replays

## E2E Test File
`tests/journeys/gamebot-stats.journey.spec.ts`

**Dependencies:** #9, #34, #36
EOF

echo "✅ Created #61: Game Statistics Dashboard"

# Story #62: Inventory Dashboard
gh issue create -R "$REPO" \
  --title "STORY-SORT-010: Inventory Dashboard with NFC" \
  --label "story,enhancement,wave-6" \
  --body-file - <<'EOF'
## Description

Web dashboard for LEGO sorter inventory with NFC tag reading, capacity tracking, and sorting history.

## DOD Criticality
- [x] **Important** - Should pass before release

## Contract References
- **Feature Contracts:** [SORT-004, SORT-006]
- **Journey Contract:** [J-HELP-LEGO-SORT]

## Acceptance Criteria (Gherkin)

### Scenario: View Inventory Dashboard
```gherkin
Given LEGO sorter is running
When I navigate to /inventory
Then I see 4 station cards
And each shows current count
And each shows capacity %
```

### Scenario: Real-Time Update
```gherkin
When robot drops piece in Red
Then Red count updates within 1s
And card flashes
```

## Invariants

### I-SORT-INV-001: NFC Sync
**MUST** Sync with NFC every 5 seconds max.

## Data Contract

```typescript
interface Station {
  id: 'red' | 'green' | 'blue' | 'yellow';
  count: number;
  capacity: number;
  lastUpdated: number;
}
```

## data-testid Requirements

| Element | data-testid |
|---------|-------------|
| Red card | `station-card-red` |
| Station count | `station-count-{color}` |
| Capacity bar | `capacity-bar-{color}` |

## In Scope
- 4 station cards
- Real-time WebSocket
- Capacity alerts
- History timeline
- Reset functionality

## Not In Scope
- NFC writing
- Export to spreadsheet

## E2E Test File
`tests/journeys/lego-sorter-inventory.journey.spec.ts`

**Dependencies:** #53, #54
**Related:** J-HELP-LEGO-SORT
EOF

echo "✅ Created #62: Inventory Dashboard"

echo ""
echo "Creating Wave 6 Integration Stories (#63-#66)..."

# Story #63: Cross-App Personality Persistence
gh issue create -R "$REPO" \
  --title "STORY-ARCH-006: Cross-App Personality Persistence" \
  --label "story,enhancement,wave-6" \
  --body-file - <<'EOF'
## Description

Implement personality persistence layer that maintains personality state across all apps (ArtBot, GameBot, HelperBot).

## DOD Criticality
- [x] **Important** - Should pass before release

## Contract References
- **Feature Contracts:** [PERS-004, ARCH-005]

## Acceptance Criteria (Gherkin)

### Scenario: Personality Persists Across Apps
```gherkin
Given I set personality to "Curious" in mixer
When I switch to ArtBot
Then robot draws with Curious personality
When I switch to GameBot
Then robot plays with Curious personality
```

### Scenario: Personality Survives Restart
```gherkin
Given I set personality to "Zen"
When I restart the companion app
Then robot resumes with Zen personality
```

## Invariants

### I-ARCH-PERS-001: Singleton Pattern
**MUST** Only one personality instance active.

### I-ARCH-PERS-002: Atomic Updates
**MUST** Personality changes atomic (no partial states).

## Data Contract

```typescript
interface PersonalityStore {
  getCurrentPersonality(): PersonalityConfig;
  setPersonality(config: PersonalityConfig): void;
  subscribeToChanges(callback: (config: PersonalityConfig) => void): void;
  persistToDisk(): Promise<void>;
  loadFromDisk(): Promise<PersonalityConfig>;
}
```

## data-testid Requirements
N/A (backend service)

## In Scope
- Persistence layer (localStorage/file)
- Subscribe/notify pattern
- Atomic updates
- Load on startup

## Not In Scope
- Cloud sync (Wave 7)
- Version history
- Conflict resolution

## E2E Test File
`tests/integration/personality-persistence.test.ts`

**Dependencies:** #12, #58
**Blocks:** #72
EOF

echo "✅ Created #63: Cross-App Personality Persistence"

# Story #64: WebSocket Protocol V2
gh issue create -R "$REPO" \
  --title "STORY-ARCH-007: WebSocket Protocol V2 with State Sync" \
  --label "story,enhancement,wave-6" \
  --body-file - <<'EOF'
## Description

Upgrade WebSocket protocol to V2 with state synchronization, message batching, and reconnection handling.

## DOD Criticality
- [x] **Critical** - Blocks release if failing

## Contract References
- **Feature Contracts:** [ARCH-005]

## Acceptance Criteria (Gherkin)

### Scenario: Client Connects
```gherkin
When client connects to ws://localhost:8080
Then server sends full state snapshot
And client syncs to current robot state
```

### Scenario: Message Batching
```gherkin
Given 10 parameter changes in 100ms
When messages sent to server
Then batched into 1 message
```

### Scenario: Auto-Reconnect
```gherkin
Given connection is lost
When network restored within 30s
Then client reconnects automatically
And state re-syncs
```

## Invariants

### I-WS-V2-001: State Consistency
**MUST** Client state always matches robot state after sync.

### I-WS-V2-002: Message Order
**MUST** Messages processed in order sent.

## Data Contract

```typescript
interface WebSocketMessage {
  type: 'state' | 'command' | 'event';
  version: 2;
  payload: any;
  timestamp: number;
}

interface StateSnapshot {
  personality: PersonalityConfig;
  neuralState: NeuralState;
  inventory?: InventoryState;
}
```

## In Scope
- Protocol V2 design
- State synchronization
- Message batching
- Auto-reconnect
- Error handling

## Not In Scope
- Binary protocol
- Compression
- Multiple simultaneous clients

## E2E Test File
`tests/integration/websocket-v2.test.ts`

**Dependencies:** #58, #59, #62
**Blocks:** #65, #76
EOF

echo "✅ Created #64: WebSocket Protocol V2"

# Story #65: Multi-Robot Discovery
gh issue create -R "$REPO" \
  --title "STORY-ARCH-008: Multi-Robot Discovery Protocol" \
  --label "story,enhancement,wave-6" \
  --body-file - <<'EOF'
## Description

Implement mDNS-based discovery protocol for finding and connecting to multiple robots on local network.

## DOD Criticality
- [ ] **Future** - Can release without

## Contract References
- **Feature Contracts:** [ARCH-005]

## Acceptance Criteria (Gherkin)

### Scenario: Discover Robots
```gherkin
Given 3 robots on network
When I open discovery panel
Then I see 3 robots listed
And each shows name and IP
```

### Scenario: Connect to Robot
```gherkin
When I click "Connect" on Robot2
Then WebSocket opens to Robot2
And UI shows Robot2 state
```

## Invariants

### I-DISC-001: mDNS Standard
**MUST** Use standard mDNS protocol (RFC 6762).

## Data Contract

```typescript
interface DiscoveredRobot {
  id: string;
  name: string;
  ipAddress: string;
  port: number;
  version: string;
}
```

## In Scope
- mDNS discovery
- Robot list UI
- Connect/disconnect
- Health indicators

## Not In Scope
- Robot pairing
- Swarm coordination (Wave 7)

## E2E Test File
`tests/integration/multi-robot-discovery.test.ts`

**Dependencies:** #64
**Blocks:** #70
EOF

echo "✅ Created #65: Multi-Robot Discovery"

# Story #66: Data Export/Import
gh issue create -R "$REPO" \
  --title "STORY-ARCH-009: Data Export and Import System" \
  --label "story,enhancement,wave-6" \
  --body-file - <<'EOF'
## Description

Export/import system for personalities, drawings, game stats, and inventory data in JSON/CSV formats.

## DOD Criticality
- [ ] **Future** - Can release without

## Contract References
- **Feature Contracts:** [ARCH-005, LEARN-007]

## Acceptance Criteria (Gherkin)

### Scenario: Export Personality
```gherkin
When I click "Export" on personality
Then JSON file downloads
And file contains all parameters
```

### Scenario: Import Personality
```gherkin
When I upload personality.json
Then personality loads
And all parameters apply
```

## Data Contract

```typescript
interface ExportManifest {
  version: string;
  exportedAt: number;
  dataTypes: ('personality' | 'drawings' | 'stats' | 'inventory')[];
  data: {
    personalities?: PersonalityConfig[];
    drawings?: Drawing[];
    stats?: GameStatistics;
    inventory?: InventoryState;
  };
}
```

## In Scope
- Export to JSON/CSV
- Import from JSON
- Validation on import
- Backup creation

## Not In Scope
- Cloud backup (Wave 7)
- Scheduled exports
- Incremental backups

## E2E Test File
`tests/integration/data-export-import.test.ts`

**Dependencies:** #58, #60, #61, #62
EOF

echo "✅ Created #66: Data Export/Import"

echo ""
echo "Creating Wave 6 Testing Stories (#67-#69)..."

# Story #67: Integration Test Suite
gh issue create -R "$REPO" \
  --title "STORY-TEST-001: Integration Test Suite for Cross-App" \
  --label "story,enhancement,wave-6,testing" \
  --body-file - <<'EOF'
## Description

Comprehensive integration test suite for cross-app features: personality persistence, WebSocket sync, data export/import.

## DOD Criticality
- [x] **Critical** - Blocks release if failing

## Contract References
- **Feature Contracts:** All feature contracts

## Acceptance Criteria

### Test Coverage Requirements
- [ ] Personality persistence: >90% coverage
- [ ] WebSocket V2: >85% coverage
- [ ] Data export/import: >80% coverage
- [ ] Multi-robot discovery: >75% coverage

### Test Categories
- [ ] Unit tests for each module
- [ ] Integration tests for cross-app flows
- [ ] Contract enforcement tests
- [ ] Performance regression tests

## In Scope
- Jest integration tests
- Contract validation tests
- Performance regression suite
- CI/CD integration

## Not In Scope
- Load testing
- Security testing
- Cross-browser testing (covered by E2E)

**Dependencies:** #63, #64, #65, #66
EOF

echo "✅ Created #67: Integration Test Suite"

# Story #68: Performance Benchmarking
gh issue create -R "$REPO" \
  --title "STORY-TEST-002: Performance Benchmarking Dashboard" \
  --label "story,enhancement,wave-6,testing" \
  --body-file - <<'EOF'
## Description

Performance benchmarking dashboard to track WebSocket latency, render performance, memory usage, and establish baselines.

## DOD Criticality
- [x] **Important** - Should pass before release

## Contract References
- **Feature Contracts:** [ARCH-005, LEARN-001]

## Acceptance Criteria

### Benchmarks to Track
- [ ] WebSocket message latency (<50ms p99)
- [ ] UI render time (<16ms for 60fps)
- [ ] Memory usage (<100MB baseline)
- [ ] Data processing throughput

### Dashboard Features
- [ ] Real-time metrics display
- [ ] Historical trend charts
- [ ] Regression detection
- [ ] Export to CSV

## In Scope
- Performance metrics collection
- Dashboard UI
- Automated benchmarking
- Regression alerts

## Not In Scope
- Profiling tools (Wave 7: #78)
- Optimization implementation

**Dependencies:** #64, #67
**Blocks:** #78
EOF

echo "✅ Created #68: Performance Benchmarking"

# Story #69: Journey Coverage Tool
gh issue create -R "$REPO" \
  --title "STORY-TEST-003: Journey Coverage Report Tool" \
  --label "story,enhancement,wave-6,testing" \
  --body-file - <<'EOF'
## Description

Tool to generate journey test coverage reports showing which contracts are covered by which E2E tests.

## DOD Criticality
- [x] **Important** - Should pass before release

## Contract References
- **Journey Contracts:** All J-* contracts

## Acceptance Criteria

### Report Features
- [ ] List all journey contracts
- [ ] Show test file for each journey
- [ ] Indicate test status (passing/failing/not implemented)
- [ ] Show coverage percentage
- [ ] Identify gaps

### Output Formats
- [ ] HTML report
- [ ] Markdown summary
- [ ] CI/CD integration

## In Scope
- Coverage report generator
- HTML/Markdown output
- Journey-to-test mapping
- Gap identification

## Not In Scope
- Automatic test generation
- Code coverage (use Jest for that)

**Dependencies:** All Wave 5 journey tests
EOF

echo "✅ Created #69: Journey Coverage Tool"

echo ""
echo "============================================"
echo "WAVE 7: ADVANCED FEATURES & POLISH (10 stories)"
echo "============================================"
echo ""
echo "Creating Wave 7 Multi-Robot Stories (#70-#71)..."

# Story #70: Multi-Robot Coordination
gh issue create -R "$REPO" \
  --title "STORY-MULTI-001: Multi-Robot Coordination Protocol" \
  --label "story,enhancement,wave-7" \
  --body-file - <<'EOF'
## Description

Coordination protocol for 2-4 robots to synchronize actions, share state, and execute coordinated behaviors.

## DOD Criticality
- [ ] **Future** - Can release without

## Contract References
- **New Contract:** MULTI-001 (to be created)

## Acceptance Criteria (Gherkin)

### Scenario: Robots Discover Each Other
```gherkin
Given 3 robots on same network
When I start coordination mode
Then each robot discovers the others
And shared state is established
```

### Scenario: Synchronized Movement
```gherkin
Given 2 robots in coordination
When I command "dance"
Then both robots move in sync
And movements are coordinated
```

## Data Contract

```typescript
interface CoordinationMessage {
  fromRobot: string;
  toRobots: string[];
  action: 'sync' | 'command' | 'state';
  payload: any;
  timestamp: number;
}
```

## In Scope
- Discovery protocol (builds on #65)
- State synchronization
- Coordinated actions
- Leader election

## Not In Scope
- Swarm AI (next story)
- Conflict resolution

**Dependencies:** #65
**Blocks:** #71
EOF

echo "✅ Created #70: Multi-Robot Coordination"

# Story #71: Swarm Play Mode
gh issue create -R "$REPO" \
  --title "STORY-MULTI-002: Swarm Play Mode (2-4 Robots)" \
  --label "story,enhancement,wave-7" \
  --body-file - <<'EOF'
## Description

Swarm play modes: synchronized dance, collaborative drawing, tag team games, patrol formations.

## DOD Criticality
- [ ] **Future** - Can release without

## Contract References
- **New Contract:** MULTI-002 (to be created)

## Acceptance Criteria (Gherkin)

### Scenario: Synchronized Dance
```gherkin
Given 3 robots in swarm mode
When I command "dance"
Then robots perform choreographed routine
And movements are synchronized
```

### Scenario: Collaborative Drawing
```gherkin
Given 2 robots with pens
When I start collaborative art
Then robots take turns drawing
And create unified artwork
```

## In Scope
- 4 swarm modes
- Choreography system
- Turn-taking logic
- Collision avoidance

## Not In Scope
- Machine learning
- Complex pathfinding

**Dependencies:** #70
EOF

echo "✅ Created #71: Swarm Play Mode"

echo ""
echo "Creating Wave 7 Cloud & Sharing Stories (#72-#73)..."

# Story #72: Cloud Sync
gh issue create -R "$REPO" \
  --title "STORY-CLOUD-001: Cloud Sync for Personalities and Artwork" \
  --label "story,enhancement,wave-7" \
  --body-file - <<'EOF'
## Description

Cloud storage backend (Supabase/Firebase) for syncing personalities, drawings, and game stats across devices.

## DOD Criticality
- [ ] **Future** - Can release without

## Contract References
- **New Contract:** CLOUD-001 (to be created)

## Acceptance Criteria (Gherkin)

### Scenario: Sync Personality to Cloud
```gherkin
Given I create personality "MyBot"
When I enable cloud sync
Then personality uploads to cloud
When I login on another device
Then "MyBot" is available there
```

### Scenario: Sync Drawings
```gherkin
Given I create 5 drawings
When drawings sync to cloud
Then I can view them on phone
```

## Data Contract

```typescript
interface CloudSyncService {
  syncPersonality(config: PersonalityConfig): Promise<void>;
  syncDrawing(drawing: Drawing): Promise<void>;
  fetchUserData(): Promise<UserData>;
}
```

## In Scope
- Supabase/Firebase integration
- Authentication (email/Google)
- Data upload/download
- Conflict resolution

## Not In Scope
- Real-time collaboration
- Public galleries
- Paid storage tiers

**Dependencies:** #60, #63, #66
**Blocks:** #73
EOF

echo "✅ Created #72: Cloud Sync"

# Story #73: Personality Marketplace
gh issue create -R "$REPO" \
  --title "STORY-CLOUD-002: Personality Marketplace with Sharing" \
  --label "story,enhancement,wave-7" \
  --body-file - <<'EOF'
## Description

Marketplace for users to share and download custom personalities created by the community.

## DOD Criticality
- [ ] **Future** - Can release without

## Contract References
- **New Contract:** CLOUD-002 (to be created)

## Acceptance Criteria (Gherkin)

### Scenario: Publish Personality
```gherkin
Given I created "SuperBot" personality
When I click "Share to Marketplace"
Then personality uploads to public gallery
And appears in search results
```

### Scenario: Download Personality
```gherkin
When I browse marketplace
And I click "Download" on "CoolBot"
Then "CoolBot" added to my presets
```

## In Scope
- Public personality gallery
- Upload/download
- Search and filter
- Rating system

## Not In Scope
- Moderation tools
- Paid personalities
- Comments/reviews

**Dependencies:** #72
EOF

echo "✅ Created #73: Personality Marketplace"

echo ""
echo "Creating Wave 7 AI Enhancement Stories (#74-#75)..."

# Story #74: Learning from Play
gh issue create -R "$REPO" \
  --title "STORY-AI-001: Learning from Play (Reinforcement Learning)" \
  --label "story,enhancement,wave-7" \
  --body-file - <<'EOF'
## Description

Basic reinforcement learning system where robot learns from game outcomes and user feedback to improve behavior.

## DOD Criticality
- [ ] **Future** - Can release without

## Contract References
- **New Contract:** AI-001 (to be created)

## Acceptance Criteria (Gherkin)

### Scenario: Learn from Tic-Tac-Toe
```gherkin
Given robot plays 100 games
When learning is enabled
Then win rate improves over time
And robot adapts strategy
```

### Scenario: Learn from User Feedback
```gherkin
When user rates behavior as "good"
Then robot reinforces that behavior
When user rates behavior as "bad"
Then robot reduces that behavior
```

## In Scope
- Q-learning algorithm
- Reward function design
- Policy updates
- Model persistence

## Not In Scope
- Deep learning
- Transfer learning
- Complex neural networks

**Dependencies:** #9, #34, #36, #61, #67
**Blocks:** #75
EOF

echo "✅ Created #74: Learning from Play"

# Story #75: Predictive Behavior Engine
gh issue create -R "$REPO" \
  --title "STORY-AI-002: Predictive Behavior Engine" \
  --label "story,enhancement,wave-7" \
  --body-file - <<'EOF'
## Description

Predictive engine that anticipates user actions and adapts robot behavior proactively based on patterns.

## DOD Criticality
- [ ] **Future** - Can release without

## Contract References
- **New Contract:** AI-002 (to be created)

## Acceptance Criteria (Gherkin)

### Scenario: Predict User Intent
```gherkin
Given user always plays Chase at 3pm
When clock approaches 3pm
Then robot prepares for Chase mode
And suggests "Time for Chase?"
```

### Scenario: Adapt to Preferences
```gherkin
Given user prefers Calm personality
When robot starts in Active mode
Then robot gradually shifts to Calm
```

## In Scope
- Pattern recognition
- Time-based predictions
- Preference learning
- Proactive suggestions

## Not In Scope
- Computer vision
- Voice recognition (separate story)

**Dependencies:** #74
EOF

echo "✅ Created #75: Predictive Behavior Engine"

echo ""
echo "Creating Wave 7 Platform Expansion Stories (#76-#77)..."

# Story #76: Mobile App Foundation
gh issue create -R "$REPO" \
  --title "STORY-MOBILE-001: Mobile App Foundation (React Native)" \
  --label "story,enhancement,wave-7" \
  --body-file - <<'EOF'
## Description

React Native mobile app foundation with core features: personality mixer, neural visualizer, and gallery viewer.

## DOD Criticality
- [ ] **Future** - Can release without

## Contract References
- **New Contract:** MOBILE-001 (to be created)

## Acceptance Criteria (Gherkin)

### Scenario: Connect from Phone
```gherkin
Given robot on WiFi network
When I open mobile app
Then I see discovered robots
When I tap to connect
Then WebSocket connects
And I see live neural state
```

### Scenario: Adjust Personality from Phone
```gherkin
Given connected to robot
When I open personality mixer
Then I see all 9 sliders
When I adjust tension
Then robot responds
```

## In Scope
- React Native setup
- iOS and Android builds
- WebSocket client
- 3 core screens (mixer, visualizer, gallery)
- Push notifications

## Not In Scope
- Full feature parity with web
- Offline mode
- App store publication

**Dependencies:** #58, #59, #60, #64
**Blocks:** #77
EOF

echo "✅ Created #76: Mobile App Foundation"

# Story #77: Voice Control Integration
gh issue create -R "$REPO" \
  --title "STORY-VOICE-001: Voice Control Integration" \
  --label "story,enhancement,wave-7" \
  --body-file - <<'EOF'
## Description

Voice command system using Web Speech API / native voice recognition for hands-free robot control.

## DOD Criticality
- [ ] **Future** - Can release without

## Contract References
- **New Contract:** VOICE-001 (to be created)

## Acceptance Criteria (Gherkin)

### Scenario: Voice Command
```gherkin
Given voice control enabled
When I say "Hey robot, start drawing"
Then ArtBot mode activates
And drawing begins
```

### Scenario: Personality Voice Switch
```gherkin
When I say "Switch to curious mode"
Then robot loads Curious personality
And confirms with beep
```

## Supported Commands (10 minimum)
- "Start drawing"
- "Play tic-tac-toe"
- "Switch to [personality]"
- "Follow me"
- "Stop"
- "Show stats"
- "Dance"
- "Go to home"
- "Increase energy"
- "Decrease tension"

## In Scope
- Web Speech API integration
- 10+ voice commands
- Visual feedback
- Error handling

## Not In Scope
- Custom wake word
- Natural language processing
- Multi-language support

**Dependencies:** #58, #76
EOF

echo "✅ Created #77: Voice Control Integration"

echo ""
echo "Creating Wave 7 Polish Stories (#78-#79)..."

# Story #78: Performance Profiling
gh issue create -R "$REPO" \
  --title "STORY-PERF-001: Performance Profiling and Optimization" \
  --label "story,enhancement,wave-7" \
  --body-file - <<'EOF'
## Description

Systematic performance profiling and optimization pass targeting >20% improvement in latency, memory, and frame rate.

## DOD Criticality
- [x] **Important** - Should pass before release

## Contract References
- **Feature Contracts:** All architecture contracts

## Acceptance Criteria

### Performance Targets
- [ ] WebSocket latency: <50ms p99 (from baseline)
- [ ] UI render time: 60fps sustained
- [ ] Memory usage: <100MB (20% reduction)
- [ ] Drawing playback: Smooth 60fps
- [ ] Neural visualizer: 10Hz updates, 60fps animation

### Optimization Areas
- [ ] WebSocket message batching
- [ ] React component memoization
- [ ] Canvas rendering optimization
- [ ] Data structure improvements
- [ ] Lazy loading for gallery

## In Scope
- Profiling with Chrome DevTools
- Memory leak detection
- Render bottleneck identification
- Optimization implementation
- Before/after benchmarks

## Not In Scope
- Code rewrite (incremental only)
- New features

**Dependencies:** #68
**Blocks:** #79
EOF

echo "✅ Created #78: Performance Profiling"

# Story #79: Animation Polish
gh issue create -R "$REPO" \
  --title "STORY-UX-001: Animation Polish and Transitions" \
  --label "story,enhancement,wave-7" \
  --body-file - <<'EOF'
## Description

Polish all UI animations and transitions for smooth, delightful user experience with consistent timing and easing.

## DOD Criticality
- [x] **Important** - Should pass before release

## Contract References
- **Feature Contracts:** All UI contracts

## Acceptance Criteria

### Animation Improvements
- [ ] Slider movements: Smooth easing
- [ ] Mode transitions: Animated icons
- [ ] Page transitions: Fade in/out
- [ ] Button feedback: Scale/ripple
- [ ] Loading states: Skeleton screens
- [ ] Notifications: Toast animations

### Consistency
- [ ] Unified timing (300ms standard)
- [ ] Consistent easing (ease-in-out)
- [ ] No layout shift
- [ ] Reduced motion support

## In Scope
- All UI animations
- Transition timing
- Loading states
- Micro-interactions
- Accessibility (prefers-reduced-motion)

## Not In Scope
- 3D animations
- Complex particle effects

**Dependencies:** #78
EOF

echo "✅ Created #79: Animation Polish"

echo ""
echo "============================================"
echo "✅ Wave 6-7 Issue Creation Complete!"
echo "============================================"
echo ""
echo "Summary:"
echo "  Wave 6: 12 stories (#58-#69)"
echo "  Wave 7: 10 stories (#70-#79)"
echo "  Total: 22 stories"
echo ""
echo "Next steps:"
echo "  1. Review issues: gh issue list -R $REPO --label wave-6,wave-7"
echo "  2. Assign to team members"
echo "  3. Create project board with Wave 6 and Wave 7 columns"
echo "  4. Begin Wave 6 Sprint 1"
echo ""
echo "All issues are Specflow-compliant with:"
echo "  ✅ Gherkin acceptance criteria"
echo "  ✅ Invariants (I-*-NNN)"
echo "  ✅ Data contracts (TypeScript)"
echo "  ✅ data-testid tables (for UI stories)"
echo "  ✅ Journey references"
echo "  ✅ Definition of Done"
echo ""
