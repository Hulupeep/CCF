# OpenClaw & Foundry Analysis for mBot Integration

## Overview

This directory contains comprehensive analysis of OpenClaw and OpenClaw Foundry patterns for integrating advanced communication, autonomous behavior, and self-learning capabilities into mBot.

## Repository Information

### OpenClaw
- **Repository**: https://github.com/openclaw/openclaw
- **Local Path**: `/home/xanacan/projects/code/openclaw`
- **Description**: Personal AI assistant framework with multi-channel communication
- **Key Features**: Gateway architecture, channel adapters, hooks system, tool execution
- **Technology**: TypeScript, Node.js ≥22, grammY (Telegram), WebSockets

### OpenClaw Foundry
- **Repository**: https://github.com/lekt9/openclaw-foundry
- **Local Path**: `/home/xanacan/projects/code/openclaw-foundry`
- **Description**: Self-writing meta-extension that learns from workflows and writes code into itself
- **Key Features**: Workflow observation, pattern crystallization, autonomous improvement
- **Technology**: TypeScript, AgentSkills format, LLM-based code generation

## Analysis Documents

### 1. [OPENCLAW_ARCHITECTURE.md](./OPENCLAW_ARCHITECTURE.md)
**Purpose**: Overall system architecture and design patterns

**Key Sections**:
- Gateway-based control plane architecture
- Channel adapter plugin system
- Session management and isolation
- Agent runtime and tool execution
- Hooks system for event-driven automation
- Skills platform (AgentSkills format)
- Security model (DM pairing, authentication)
- Data flow and lifecycle management

**Why It Matters for mBot**:
- Learn how to build modular, channel-agnostic communication
- Understand plugin-based extension patterns
- Security best practices (pairing flow, allowlists)
- Tool execution coordination

**Key Takeaways**:
- Use adapter pattern for multi-channel support
- Hooks provide non-invasive extension points
- Session isolation prevents cross-contamination
- Gateway serves as single coordination point

---

### 2. [SELF_LEARNING_PATTERNS.md](./SELF_LEARNING_PATTERNS.md)
**Purpose**: Foundry's recursive self-improvement mechanisms

**Key Sections**:
- Knowledge vs. Behavior distinction
- LearningEngine class and data structures
- Workflow observation and tracking
- Pattern crystallization (5+ uses, 70%+ success)
- The Overseer (autonomous learning agent)
- Research foundations (arXiv papers)
- Integration examples for mBot

**Why It Matters for mBot**:
- Automatically improve servo calibration from failures
- Learn optimal personality parameters
- Crystallize proven drawing/sorting routines
- Reduce token cost by converting patterns to code

**Key Takeaways**:
- Record failures + resolutions = reusable patterns
- 5+ successful uses → crystallize into permanent code
- Context injection makes LLM aware of past learnings
- Learning persists across restarts in JSON storage

**Research Papers**:
- Self-Improving Coding Agent (arXiv:2504.15228)
- SelfEvolve (arXiv:2306.02907)
- RISE: Recursive Introspection (arXiv:2407.18219)
- HexMachina (arXiv:2506.04651)
- ADAS (arXiv:2408.08435)

---

### 3. [TELEGRAM_INTEGRATION.md](./TELEGRAM_INTEGRATION.md)
**Purpose**: Complete guide to Telegram bot integration

**Key Sections**:
- grammY framework setup and configuration
- Inbound message normalization
- Outbound message formatting (Markdown → HTML)
- Message chunking (4000 char limit)
- Multi-account support
- DM pairing security flow
- Inline keyboards and buttons
- Media handling (photos, documents, voice)

**Why It Matters for mBot**:
- Enable remote control via Telegram
- Send proactive notifications
- Receive drawing commands remotely
- Share photos/videos of mBot's work

**Key Takeaways**:
- grammY is modern, type-safe Telegram library
- Always normalize messages to internal format
- Chunk long messages at paragraph boundaries
- DM pairing prevents spam/abuse
- Bot tokens must be stored securely

**Code References**:
- `/openclaw/src/channels/plugins/outbound/telegram.ts`
- `/openclaw/src/channels/plugins/normalize/telegram.ts`
- `/openclaw/ui/src/ui/views/channels.telegram.ts`

---

### 4. [AUTONOMOUS_BEHAVIOR.md](./AUTONOMOUS_BEHAVIOR.md)
**Purpose**: Proactive, autonomous agent behavior patterns

**Key Sections**:
- Cron-based scheduling (time-triggered actions)
- Webhook-based triggers (event-driven actions)
- Hook-based autonomous behavior (internal events)
- Decision-making logic (rule-based and LLM-based)
- Background task management
- Context awareness (sensors, state tracking)
- Proactive communication strategies
- Personality-driven autonomy

**Why It Matters for mBot**:
- Proactive battery warnings
- Scheduled maintenance reminders
- Autonomous greeting behaviors
- Context-aware assistance offers
- Idle behavior suggestions

**Key Takeaways**:
- Cron jobs enable time-based autonomy
- Context awareness drives better decisions
- Personality profiles affect proactiveness
- Background task queue prevents blocking
- Webhooks connect to external events

**Use Cases for mBot**:
- Hourly battery check → notify if low
- Morning greeting (8 AM daily)
- Idle detection → offer to help
- Temperature-based servo warmup
- Proactive learning summary (evening)

---

### 5. [MBOT_INTEGRATION_STORIES.md](./MBOT_INTEGRATION_STORIES.md)
**Purpose**: Specflow-compliant GitHub issue templates

**Stories Included**:

#### Story #92: Telegram Bot Integration
- **Complexity**: Medium (3-5 days)
- **DOD**: Important
- **Key Features**: Pairing flow, message chunking, media handling
- **Dependencies**: None

#### Story #93: Self-Learning from User Interactions
- **Complexity**: Medium-High (5-7 days)
- **DOD**: Important
- **Key Features**: LearningEngine, pattern recording, crystallization
- **Dependencies**: None (optional: #92 for notifications)

#### Story #94: Autonomous Proactive Behavior
- **Complexity**: High (7-10 days)
- **DOD**: Future
- **Key Features**: Cron scheduler, decision engine, context awareness
- **Dependencies**: #92 (Telegram), #93 (Self-learning), #12 (Personality)

#### Story #95: Multi-Channel Communication Adapter
- **Complexity**: High (5-7 days)
- **DOD**: Future
- **Key Features**: Channel abstraction, adapter pattern, broadcasting
- **Dependencies**: #92 (Telegram serves as first implementation)

**Each Story Includes**:
- Gherkin acceptance criteria
- data-testid requirements
- E2E test file specifications
- Technical approach with code examples
- Estimated complexity and breakdown
- Dependency tracking
- Reference files from OpenClaw/Foundry

---

## Implementation Roadmap

### Phase 1: Foundation (2-3 weeks)
**Stories**: #92, #93

**Goals**:
- Telegram communication working
- Learning engine recording failures
- Basic pattern suggestions

**Success Criteria**:
- ✅ Send/receive Telegram messages
- ✅ Pairing flow functional
- ✅ Servo failures recorded
- ✅ Patterns suggested on similar failures

---

### Phase 2: Intelligence (2 weeks)
**Stories**: #94

**Goals**:
- Autonomous behavior operational
- Cron scheduling working
- Context-aware decisions

**Success Criteria**:
- ✅ Scheduled messages sent
- ✅ Battery monitoring proactive
- ✅ Idle suggestions triggered
- ✅ Decision engine evaluates context

---

### Phase 3: Expansion (1-2 weeks)
**Stories**: #95 + enhancements

**Goals**:
- Multi-channel support
- Advanced crystallization
- Comprehensive learning

**Success Criteria**:
- ✅ Abstract channel interface works
- ✅ 2+ channels operational
- ✅ Workflow crystallization functional
- ✅ Broadcasting works

---

## Quick Start Guide

### For Telegram Integration (Story #92)

1. **Install grammY**:
   ```bash
   cargo add grammy  # Or npm install grammy for Node.js
   ```

2. **Create bot via @BotFather**:
   - Message @BotFather on Telegram
   - Create new bot, get token

3. **Store token securely**:
   ```bash
   export TELEGRAM_BOT_TOKEN="your_token_here"
   ```

4. **Implement basic bot**:
   ```rust
   use grammy::Bot;

   let bot = Bot::new(env::var("TELEGRAM_BOT_TOKEN")?);

   bot.on_message(|ctx| async move {
       let text = ctx.message.text.unwrap_or_default();
       ctx.reply(&format!("You said: {}", text)).await?;
       Ok(())
   });

   bot.start().await?;
   ```

5. **Reference**: See TELEGRAM_INTEGRATION.md for complete implementation

---

### For Self-Learning (Story #93)

1. **Create LearningEngine**:
   ```rust
   pub struct LearningEngine {
       learnings: Vec<LearningEntry>,
       storage_path: PathBuf,
   }
   ```

2. **Record failures**:
   ```rust
   let failure_id = learning_engine.record_failure(
       "servo_move",
       "Stall detected at 90°",
       "Moving arm servo to drawing position"
   );
   ```

3. **Link resolutions**:
   ```rust
   learning_engine.record_resolution(
       &failure_id,
       "Reduced speed to 50% near target position"
   );
   ```

4. **Find patterns**:
   ```rust
   let patterns = learning_engine.find_relevant_patterns(
       "servo_move",
       "Stall detected"
   );
   ```

5. **Reference**: See SELF_LEARNING_PATTERNS.md for complete implementation

---

### For Autonomous Behavior (Story #94)

1. **Setup cron scheduler**:
   ```rust
   use tokio_cron_scheduler::{JobScheduler, Job};

   let scheduler = JobScheduler::new().await?;

   scheduler.add(Job::new("0 * * * *", |_uuid, _lock| {
       // Hourly battery check
       tokio::spawn(async {
           check_battery_and_notify().await;
       });
   })?).await?;

   scheduler.start().await?;
   ```

2. **Implement decision engine**:
   ```rust
   pub async fn should_act_autonomously(context: &ContextState) -> bool {
       // Low battery = urgent
       if context.battery < 15.0 { return true; }

       // Long inactivity = check in
       if context.hours_since_interaction() > 24 { return true; }

       false
   }
   ```

3. **Reference**: See AUTONOMOUS_BEHAVIOR.md for complete implementation

---

## Key Files from OpenClaw

### Architecture
- `src/gateway/gateway.ts` - Gateway implementation
- `src/gateway/protocol.ts` - WebSocket protocol
- `src/channels/registry.ts` - Channel management

### Telegram
- `src/channels/plugins/outbound/telegram.ts` - Sending messages
- `src/channels/plugins/normalize/telegram.ts` - Receiving messages
- `src/telegram/format.ts` - Markdown → HTML conversion

### Autonomous
- `src/automation/cron-jobs/` - Cron scheduling
- `src/automation/hooks/` - Hook system
- `src/automation/webhook/` - Webhook handling

### Foundry
- `index.ts` (lines 476-1600) - LearningEngine
- `src/self-writer.ts` - Code generation
- `docs/ARCHITECTURE.md` - System overview
- `docs/PROACTIVE-LEARNING.md` - Learning details

---

## Testing Strategy

### Unit Tests
- Telegram adapter in isolation (mock bot API)
- Learning engine pattern matching
- Decision engine logic
- Message normalization

### Integration Tests
- Telegram → mBot → Response flow
- Learning → Pattern → Suggestion flow
- Cron → Decision → Action flow
- Channel registry routing

### E2E Journey Tests
- `tests/journeys/telegram-first-message.journey.spec.ts`
- `tests/journeys/first-learning-pattern.journey.spec.ts`
- `tests/journeys/first-proactive-message.journey.spec.ts`
- `tests/journeys/multi-channel.journey.spec.ts`

---

## Metrics for Success

### Telegram Integration
- Pairing flow completion: >95%
- Message delivery success: >99%
- Response time: <2 seconds

### Self-Learning
- Failure recording: 100%
- Pattern suggestion accuracy: >70%
- Crystallization triggers: 100%

### Autonomous Behavior
- Cron execution accuracy: >99%
- Decision engine latency: <500ms
- User satisfaction: >80%

### Multi-Channel
- Abstraction coverage: 2+ channels
- Normalization accuracy: 100%
- Broadcasting success: >95%

---

## Dependencies

### External Libraries
- **grammY** - Telegram bot framework (TypeScript/Node.js)
- **tokio-cron-scheduler** - Cron jobs (Rust)
- **serde** - JSON serialization (Rust)

### mBot Internal
- Personality system (#12) - Drives autonomous tone
- Servo control - Learning targets
- Sensor integration - Context awareness

---

## Security Considerations

### Telegram
1. **Bot Token**: Never commit to version control
2. **DM Pairing**: Enable by default (prevent spam)
3. **Allowlist**: Store in secure config file
4. **Webhook Signatures**: Validate all webhook payloads

### Learning Engine
1. **No Sensitive Data**: Never record API keys, passwords
2. **User Data**: Store only operational context
3. **Crystallized Code**: Sandbox validation before deployment

### Autonomous Behavior
1. **Action Limits**: Rate limit proactive messages
2. **User Override**: Always allow disabling autonomy
3. **Audit Log**: Track all autonomous actions

---

## Troubleshooting

### Telegram Not Connecting
- Check bot token validity: `curl https://api.telegram.org/bot<TOKEN>/getMe`
- Verify network connectivity
- Check firewall rules (outbound HTTPS)

### Learning Patterns Not Saving
- Check file permissions on `data/learnings.json`
- Verify JSON format validity
- Ensure disk space available

### Cron Jobs Not Triggering
- Validate cron expression: https://crontab.guru/
- Check system timezone settings
- Verify scheduler is started

---

## Resources

### OpenClaw Documentation
- Official Docs: https://docs.openclaw.ai/
- GitHub: https://github.com/openclaw/openclaw
- Discord: https://discord.gg/clawd

### Foundry Documentation
- README: https://github.com/lekt9/openclaw-foundry/blob/main/README.md
- Architecture: `docs/ARCHITECTURE.md` (in Foundry repo)
- Proactive Learning: `docs/PROACTIVE-LEARNING.md` (in Foundry repo)

### Telegram Bot API
- grammY Docs: https://grammy.dev/
- Telegram Bot API: https://core.telegram.org/bots/api
- @BotFather: https://t.me/BotFather

### Research Papers
1. **Self-Improving Coding Agent**: https://arxiv.org/abs/2504.15228
2. **SelfEvolve**: https://arxiv.org/abs/2306.02907
3. **RISE (Recursive Introspection)**: https://arxiv.org/abs/2407.18219
4. **HexMachina**: https://arxiv.org/abs/2506.04651
5. **ADAS**: https://arxiv.org/abs/2408.08435

---

## Next Steps

1. **Review Analysis Docs**: Read all 5 analysis files
2. **Create GitHub Issues**: Use templates from MBOT_INTEGRATION_STORIES.md
3. **Start with #92**: Telegram integration (foundation)
4. **Then #93**: Self-learning (intelligence)
5. **Finally #94**: Autonomous behavior (proactive agent)

---

## Questions?

Refer to the specific analysis documents for detailed implementation guidance. Each document includes:
- Architecture diagrams
- Code examples
- Configuration templates
- Integration checklists
- Reference file locations

**Happy integrating! 🤖🦞**
