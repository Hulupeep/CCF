# mBot Integration Stories - OpenClaw Patterns

## Overview

This document contains Specflow-compliant GitHub issue templates for integrating OpenClaw and Foundry patterns into mBot. Each story includes Gherkin scenarios, acceptance criteria, technical approach, and estimated complexity.

---

## Story #92: Telegram Bot Integration

### Epic
Communication & Channels

### Description
Integrate Telegram bot communication using patterns from OpenClaw, enabling mBot to receive commands and send responses via Telegram.

### DOD Criticality
- [x] **Important** - Should pass before release

### Contract References
- **Feature Contracts:** [COMM-001: Message Transport]
- **Journey Contract:** [J-COMM-TELEGRAM-FIRST-MSG]

### Acceptance Criteria (Gherkin)

**Scenario 1**: User sends first message to mBot via Telegram
```gherkin
Given mBot Telegram bot is running
And user has not paired their Telegram account
When user sends "hello" to mBot bot
Then mBot responds with pairing code
And mBot does not process the message until pairing approved
```

**Scenario 2**: Admin approves Telegram pairing
```gherkin
Given user received pairing code "ABC123"
When admin runs `mbot telegram pair approve ABC123`
Then user is added to allowlist
And subsequent messages from user are processed normally
```

**Scenario 3**: Paired user sends command
```gherkin
Given user Telegram account is paired
When user sends "draw a circle" via Telegram
Then mBot receives normalized message
And mBot executes drawing command
And mBot sends response back to Telegram chat
```

**Scenario 4**: Long response chunking
```gherkin
Given user requests detailed system status
When response text exceeds 4000 characters
Then mBot splits response into multiple Telegram messages
And sends chunks in correct order
And preserves message formatting (Markdown → HTML)
```

**Scenario 5**: Media attachment handling
```gherkin
Given user sends a photo via Telegram
When mBot processes the message
Then mBot downloads the photo
And stores it temporarily for analysis
And responds acknowledging the photo
```

### data-testid Requirements

N/A (Backend integration, no UI elements)

### E2E Test File
`tests/journeys/telegram-first-message.journey.spec.ts`

### Technical Approach (from OpenClaw)

**1. Dependencies**:
```bash
npm install grammy
```

**2. Bot Setup**:
- Store bot token in `.env` as `TELEGRAM_BOT_TOKEN`
- Create `TelegramAdapter` class implementing message interface
- Reference: `/home/xanacan/projects/code/openclaw/src/channels/plugins/outbound/telegram.ts`

**3. Key Components**:
```rust
pub struct TelegramAdapter {
    bot_token: String,
    allowed_users: Vec<i64>,
    pairing_codes: HashMap<String, i64>,
}

impl TelegramAdapter {
    pub async fn start(&self) -> Result<()>
    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<()>
    pub async fn handle_inbound(&self, update: Update) -> Result<NormalizedMessage>
}
```

**4. Message Normalization**:
```rust
pub struct NormalizedMessage {
    pub channel: String,           // "telegram"
    pub chat_id: String,
    pub user_id: String,
    pub text: String,
    pub timestamp: u64,
    pub media_urls: Option<Vec<String>>,
}
```

**5. Outbound Formatting**:
- Markdown → Telegram HTML conversion
- 4000 character chunking
- Reply threading support

**6. Security**:
- DM pairing policy by default
- Allowlist stored in `data/telegram_allowlist.json`
- Admin-only pairing approval

### Estimated Complexity
**Medium** - 3-5 days

**Breakdown**:
- Bot setup & basic sending: 1 day
- Inbound normalization: 1 day
- Pairing flow: 1 day
- Message chunking & media: 1 day
- Testing & docs: 1 day

### Dependencies
- None (can be implemented independently)

### Related Issues
- #93 (Self-learning may use Telegram for notifications)
- #95 (Multi-channel adapter builds on this)

---

## Story #93: Self-Learning from User Interactions

### Epic
Intelligence & Learning

### Description
Implement Foundry-style self-learning that observes servo operations, personality behaviors, and user interactions to automatically improve mBot's performance over time.

### DOD Criticality
- [x] **Important** - Should pass before release

### Contract References
- **Feature Contracts:** [LEARN-001: Pattern Learning, LEARN-002: Crystallization]
- **Journey Contract:** [J-LEARN-FIRST-PATTERN]

### Acceptance Criteria (Gherkin)

**Scenario 1**: Record servo operation failure
```gherkin
Given mBot attempts to move servo to position 90°
And servo stalls due to obstruction
When failure is detected
Then failure is recorded with context: servo_id, target_position, error_type
And failure ID is generated
```

**Scenario 2**: Link resolution to failure
```gherkin
Given previous servo failure with ID "fail-servo-123"
And user manually adjusts servo mount
And retry succeeds
When success is detected
Then resolution is linked to failure ID
And pattern is created: "servo stall" → "check mount clearance"
```

**Scenario 3**: Pattern suggestion on similar failure
```gherkin
Given pattern exists: "servo stall" → "check mount clearance" (used 3 times)
When similar servo stall occurs
Then mBot proactively suggests: "Try: check mount clearance"
And user can accept or provide different fix
```

**Scenario 4**: Workflow crystallization
```gherkin
Given "draw circle" workflow executed successfully 5+ times
And workflow uses sequence: ["move_to_start", "pen_down", "draw_arc", "pen_up"]
And success rate is 85%
When crystallization threshold met
Then new optimized function "draw_circle_optimized" is generated
And subsequent "draw circle" commands use optimized version
```

**Scenario 5**: Learning persistence across restarts
```gherkin
Given mBot has learned 10 patterns
When mBot system restarts
Then all 10 patterns are loaded from storage
And patterns continue to be used
```

### data-testid Requirements

| Element | data-testid | Purpose |
|---------|-------------|---------|
| Learning log viewer | `learning-log` | Display recorded learnings |
| Pattern list | `pattern-list` | Show crystallization candidates |
| Pattern detail | `pattern-detail-{id}` | View specific pattern details |

### E2E Test File
`tests/journeys/first-learning-pattern.journey.spec.ts`

### Technical Approach (from Foundry)

**1. LearningEngine Structure**:
```rust
pub struct LearningEngine {
    learnings: Vec<LearningEntry>,
    storage_path: PathBuf,
}

pub struct LearningEntry {
    pub id: String,
    pub entry_type: LearningType,  // Failure, Pattern, Success, Insight
    pub operation: Option<String>,
    pub error: Option<String>,
    pub resolution: Option<String>,
    pub context: Option<String>,
    pub timestamp: u64,
    pub use_count: u32,
    pub crystallized_to: Option<String>,
}
```

**2. Core Methods**:
```rust
impl LearningEngine {
    pub fn record_failure(&mut self, operation: &str, error: &str, context: &str) -> String;
    pub fn record_resolution(&mut self, failure_id: &str, resolution: &str);
    pub fn record_success(&mut self, operation: &str, context: &str);
    pub fn find_relevant_patterns(&self, operation: &str, error_pattern: &str) -> Vec<&LearningEntry>;
    pub fn get_crystallization_candidates(&self) -> Vec<&LearningEntry>;
    pub fn save(&self) -> Result<()>;
    pub fn load(&mut self) -> Result<()>;
}
```

**3. Integration Points**:
- Hook into servo command execution (before/after)
- Track personality behavior outcomes
- Monitor drawing operation success rates
- Record LEGO sorting accuracy

**4. Storage**:
- File: `data/learnings.json`
- Format: JSON array of LearningEntry objects
- Auto-save after each learning event

**5. Crystallization Rules**:
- Threshold: 5+ successful uses
- Success rate: ≥70%
- Pattern must be repeatable

**6. Proactive Suggestions**:
- On failure, search for matching patterns
- Display suggestion via Telegram (if #92 complete)
- Log whether user accepted suggestion

### Estimated Complexity
**Medium-High** - 5-7 days

**Breakdown**:
- LearningEngine core: 2 days
- Integration hooks: 2 days
- Pattern matching logic: 1 day
- Crystallization (basic): 1 day
- Testing & docs: 1 day

### Dependencies
- None (can start immediately)
- Optional: #92 (for Telegram notifications of learnings)

### Related Issues
- #12 (Personality system can learn preferred parameters)
- #94 (Autonomous behavior uses learnings for decision-making)

### Reference Files
- `/home/xanacan/projects/code/openclaw-foundry/index.ts` (lines 476-1600) - LearningEngine
- `/home/xanacan/projects/code/openclaw-foundry/docs/PROACTIVE-LEARNING.md`

---

## Story #94: Autonomous Proactive Behavior

### Epic
Autonomy & Intelligence

### Description
Enable mBot to take autonomous actions based on context (cron schedules, idle detection, battery monitoring) without explicit user commands.

### DOD Criticality
- [ ] **Future** - Can release without

### Contract References
- **Feature Contracts:** [AUTO-001: Autonomous Actions, AUTO-002: Decision Making]
- **Journey Contract:** [J-AUTO-FIRST-PROACTIVE-MSG]

### Acceptance Criteria (Gherkin)

**Scenario 1**: Scheduled morning greeting
```gherkin
Given current time is 8:00 AM
And cron job "morning-greeting" is enabled
When cron scheduler triggers job
Then mBot sends greeting message via Telegram
And greeting is personality-appropriate
```

**Scenario 2**: Battery low notification
```gherkin
Given mBot battery level drops below 20%
When battery monitor detects low level
Then mBot autonomously sends notification to user
And notification includes current battery percentage
And mBot suggests charging
```

**Scenario 3**: Idle behavior suggestion
```gherkin
Given mBot has been idle for 5 minutes
And user is detected nearby (sensor)
When idle timeout triggers
Then mBot offers proactive assistance
And suggestion matches current personality mode
```

**Scenario 4**: Autonomous decision-making
```gherkin
Given mBot has multiple tasks in queue
And battery is at 15%
When autonomous decision engine evaluates context
Then mBot prioritizes battery-critical tasks
And defers non-urgent tasks
And notifies user of decision
```

**Scenario 5**: Learning-based proactive action
```gherkin
Given mBot learned pattern: "servo stalls in cold weather"
And temperature sensor reads < 15°C
And user hasn't used mBot today
When startup sequence begins
Then mBot proactively runs servo warmup routine
And notifies user: "Running warmup due to cold temperature"
```

### data-testid Requirements

| Element | data-testid | Purpose |
|---------|-------------|---------|
| Cron job list | `cron-jobs` | View scheduled tasks |
| Autonomous decision log | `decision-log` | History of autonomous actions |
| Add cron job button | `add-cron-job` | Create new scheduled task |

### E2E Test File
`tests/journeys/first-proactive-message.journey.spec.ts`

### Technical Approach (from OpenClaw)

**1. Cron Scheduler**:
```rust
use tokio_cron_scheduler::{JobScheduler, Job};

pub struct CronManager {
    scheduler: JobScheduler,
    jobs: HashMap<String, Job>,
}

impl CronManager {
    pub async fn add_job(&mut self, id: &str, schedule: &str, action: Box<dyn Fn()>) -> Result<()>;
    pub async fn remove_job(&mut self, id: &str) -> Result<()>;
    pub async fn start(&self) -> Result<()>;
}
```

**2. Decision Engine**:
```rust
pub struct DecisionEngine {
    context_manager: ContextManager,
    personality: PersonalityProfile,
}

pub struct AutonomousContext {
    pub battery_level: f32,
    pub last_interaction: u64,
    pub user_present: bool,
    pub time_of_day: u8,
    pub current_task: Option<String>,
}

impl DecisionEngine {
    pub async fn should_act(&self) -> Option<AutonomousAction>;
    pub async fn plan_action(&self, context: &AutonomousContext) -> Option<Action>;
    pub async fn execute_action(&self, action: Action) -> Result<()>;
}
```

**3. Background Task Queue**:
```rust
pub struct TaskQueue {
    pending: Vec<BackgroundTask>,
    running: Option<BackgroundTask>,
}

pub struct BackgroundTask {
    pub id: String,
    pub priority: Priority,  // High, Medium, Low
    pub task_type: TaskType,
    pub data: serde_json::Value,
    pub run_at: Option<u64>,
}
```

**4. Context Awareness**:
- Battery monitoring (every 5 minutes)
- Idle detection (no activity for 5 minutes)
- User presence detection (sensor integration)
- Time-of-day awareness

**5. Personality-Driven Behavior**:
```rust
pub struct PersonalityProfile {
    pub proactiveness: f32,    // 0-1, how often to initiate
    pub chattiness: f32,       // 0-1, verbosity
    pub curiosity: f32,        // 0-1, ask questions vs. wait
}

impl PersonalityProfile {
    pub fn should_initiate(&self, context: &AutonomousContext) -> bool {
        let base_chance = self.proactiveness;
        let adjusted = self.adjust_for_context(base_chance, context);
        rand::random::<f32>() < adjusted
    }
}
```

**6. Cron Job Examples**:
- `0 8 * * *` - Morning greeting (8 AM daily)
- `*/30 * * * *` - Battery check (every 30 min)
- `0 20 * * *` - Evening summary (8 PM daily)
- `0 0 * * 0` - Weekly maintenance reminder (Sunday midnight)

### Estimated Complexity
**High** - 7-10 days

**Breakdown**:
- Cron scheduler: 2 days
- Decision engine: 3 days
- Context manager: 2 days
- Background task queue: 2 days
- Personality integration: 1 day
- Testing & docs: 2 days

### Dependencies
- #92 (Telegram - needed for proactive messaging)
- #93 (Self-learning - provides patterns for decisions)
- #12 (Personality system - drives behavior tone)

### Related Issues
- All features benefit from autonomous monitoring

### Reference Files
- `/home/xanacan/projects/code/openclaw/src/automation/cron-jobs/`
- `/home/xanacan/projects/code/openclaw-foundry/docs/PROACTIVE-LEARNING.md`

---

## Story #95: Multi-Channel Communication Adapter

### Epic
Communication & Channels

### Description
Build a channel-agnostic communication abstraction layer that allows mBot to communicate via multiple channels (Telegram, Discord, Slack, etc.) using a unified interface.

### DOD Criticality
- [ ] **Future** - Can release without

### Contract References
- **Feature Contracts:** [COMM-002: Channel Abstraction]
- **Journey Contract:** [J-COMM-MULTI-CHANNEL]

### Acceptance Criteria (Gherkin)

**Scenario 1**: Send message via abstract interface
```gherkin
Given mBot has Telegram and Discord channels configured
When mBot calls `channel_manager.send_message("telegram", chat_id, "Hello")`
Then message is sent via Telegram adapter
And message format is normalized for Telegram (HTML)
```

**Scenario 2**: Receive message from any channel
```gherkin
Given user sends "draw circle" via Discord
When Discord adapter receives message
Then message is normalized to standard format
And mBot processes command identically to Telegram
```

**Scenario 3**: Channel-specific features
```gherkin
Given user requests interactive buttons
And current channel is Telegram
When mBot sends response
Then inline keyboard buttons are included (Telegram-specific)
And buttons work correctly
```

**Scenario 4**: Fallback on unsupported features
```gherkin
Given user requests interactive buttons
And current channel is SMS (no button support)
When mBot sends response
Then buttons are converted to numbered list
And user can reply with number
```

**Scenario 5**: Multi-channel broadcasting
```gherkin
Given mBot needs to send system alert
When mBot calls `channel_manager.broadcast_all("System restarting")`
Then message is sent to all configured channels
And each channel receives properly formatted message
```

### data-testid Requirements

| Element | data-testid | Purpose |
|---------|-------------|---------|
| Channel list | `channel-list` | Show configured channels |
| Channel status | `channel-status-{id}` | Display channel health |
| Add channel button | `add-channel` | Configure new channel |

### E2E Test File
`tests/journeys/multi-channel.journey.spec.ts`

### Technical Approach (from OpenClaw)

**1. Channel Adapter Trait**:
```rust
pub trait ChannelAdapter: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn send_text(&self, to: &str, text: &str) -> Result<SendResult>;
    async fn send_media(&self, to: &str, text: &str, media_url: &str) -> Result<SendResult>;
    async fn normalize_inbound(&self, raw: serde_json::Value) -> Result<NormalizedMessage>;
}
```

**2. Channel Registry**:
```rust
pub struct ChannelRegistry {
    adapters: HashMap<String, Box<dyn ChannelAdapter>>,
}

impl ChannelRegistry {
    pub fn register(&mut self, id: &str, adapter: Box<dyn ChannelAdapter>);
    pub async fn send(&self, channel: &str, to: &str, message: &Message) -> Result<()>;
    pub async fn broadcast(&self, message: &Message) -> Result<Vec<SendResult>>;
    pub fn get_status(&self, channel: &str) -> Option<ChannelStatus>;
}
```

**3. Message Normalization**:
```rust
pub struct NormalizedMessage {
    pub channel: String,
    pub chat_id: String,
    pub user_id: String,
    pub text: String,
    pub timestamp: u64,
    pub media_urls: Option<Vec<String>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct Message {
    pub text: String,
    pub media_urls: Option<Vec<String>>,
    pub buttons: Option<Vec<Button>>,
    pub channel_specific: HashMap<String, serde_json::Value>,
}
```

**4. Supported Channels (Initial)**:
- Telegram (from #92)
- Discord (future)
- Slack (future)
- WebSocket/HTTP API (for web interface)

**5. Channel-Specific Handlers**:
```rust
pub struct TelegramAdapter {
    bot: Bot,
}

pub struct DiscordAdapter {
    client: serenity::Client,
}

// Each implements ChannelAdapter trait
```

**6. Feature Detection**:
```rust
pub struct ChannelCapabilities {
    pub supports_buttons: bool,
    pub supports_media: bool,
    pub supports_threads: bool,
    pub max_message_length: usize,
}
```

### Estimated Complexity
**High** - 5-7 days

**Breakdown**:
- Trait definition & registry: 2 days
- Telegram adapter (refactor from #92): 1 day
- Message normalization: 1 day
- Broadcasting & fallbacks: 1 day
- Testing & docs: 2 days

### Dependencies
- #92 (Telegram adapter serves as first implementation)

### Related Issues
- #94 (Autonomous behavior uses channels for notifications)
- All communication features benefit from abstraction

### Reference Files
- `/home/xanacan/projects/code/openclaw/src/channels/registry.ts`
- `/home/xanacan/projects/code/openclaw/src/channels/plugins/outbound/*.ts`

---

## Implementation Roadmap

### Phase 1: Foundation (Stories #92, #93)
**Duration**: 2-3 weeks

**Goals**:
- Basic Telegram communication
- Learning engine operational
- Pattern recording working

**Deliverables**:
- mBot can send/receive Telegram messages
- Servo failures are recorded and patterns suggested
- Basic pairing flow works

### Phase 2: Intelligence (Story #94)
**Duration**: 2 weeks

**Goals**:
- Autonomous behavior functional
- Cron scheduling working
- Context-aware decisions

**Deliverables**:
- mBot sends scheduled messages
- Battery monitoring proactive
- Idle behavior suggestions

### Phase 3: Expansion (Story #95 + Future)
**Duration**: 1-2 weeks

**Goals**:
- Multi-channel support
- Advanced crystallization
- Comprehensive learning

**Deliverables**:
- Abstract channel interface
- Discord support (optional)
- Workflow crystallization working

## Testing Strategy

### Unit Tests
- Each adapter in isolation
- Learning engine pattern matching
- Decision engine logic
- Message normalization

### Integration Tests
- Telegram → mBot → Response flow
- Learning → Pattern → Suggestion flow
- Cron → Decision → Action flow

### E2E Journey Tests
- Complete user interactions
- Pairing flow
- First learning pattern
- First proactive message

## Metrics for Success

### Story #92 (Telegram)
- ✅ Pairing flow completion rate > 95%
- ✅ Message delivery success rate > 99%
- ✅ Response time < 2 seconds

### Story #93 (Learning)
- ✅ Failure recording success rate 100%
- ✅ Pattern suggestion accuracy > 70%
- ✅ Crystallization triggers correctly

### Story #94 (Autonomous)
- ✅ Cron job execution accuracy > 99%
- ✅ Decision engine response time < 500ms
- ✅ User satisfaction with proactive messages > 80%

### Story #95 (Multi-Channel)
- ✅ Channel abstraction works for 2+ channels
- ✅ Message normalization preserves intent
- ✅ Broadcasting delivery success > 95%

## Documentation Requirements

Each story must include:
1. **User Guide**: How to use the feature
2. **Developer Guide**: How to extend/modify
3. **Configuration Reference**: All config options
4. **Troubleshooting**: Common issues and fixes

## Related Documentation

- **OPENCLAW_ARCHITECTURE.md** - Overall system design
- **TELEGRAM_INTEGRATION.md** - Detailed Telegram patterns
- **SELF_LEARNING_PATTERNS.md** - Learning engine details
- **AUTONOMOUS_BEHAVIOR.md** - Proactive behavior patterns
