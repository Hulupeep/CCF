# OpenClaw Architecture Analysis

## Overview

OpenClaw is a personal AI assistant framework designed to run locally on user devices with multi-channel communication support. It provides a **Gateway-based architecture** that coordinates agents, channels, tools, and events through a WebSocket control plane.

## High-Level System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Communication Channels                        │
│   WhatsApp · Telegram · Discord · Slack · Signal · iMessage    │
│   Google Chat · Teams · WebChat · Matrix · BlueBubbles         │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Gateway (Control Plane)                     │
│                   ws://127.0.0.1:18789                          │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  Channels    │  │   Sessions   │  │    Tools     │         │
│  │  Registry    │  │   Manager    │  │   Registry   │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Hooks      │  │   Memory     │  │    Cron      │         │
│  │   System     │  │   Store      │  │   Scheduler  │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Agent Runtime (Pi)                          │
│    RPC Mode · Tool Streaming · Block Streaming                 │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Model Providers                              │
│         Anthropic (Claude) · OpenAI · Local Models              │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Gateway (Control Plane)

**Purpose**: Central coordination hub for all system interactions

**Key Responsibilities**:
- WebSocket server for real-time communication
- Channel lifecycle management (start/stop/probe)
- Session management and routing
- Tool execution coordination
- Event hook distribution
- Configuration management
- Control UI hosting

**Location**: `src/gateway/`

**Key Files**:
- `src/gateway/gateway.ts` - Main gateway server
- `src/gateway/protocol.ts` - WebSocket protocol definitions
- `src/gateway/configuration.ts` - Config management

### 2. Channels System

**Purpose**: Multi-platform message routing and normalization

**Architecture**: Plugin-based adapter system with standardized interfaces

**Channel Types**:
- **Direct messaging**: WhatsApp, Telegram, Signal, iMessage
- **Team chat**: Slack, Discord, Google Chat, Microsoft Teams
- **Extensions**: BlueBubbles, Matrix, Zalo
- **Web**: WebChat interface

**Channel Adapter Pattern**:
```typescript
interface ChannelAdapter {
  start(): Promise<void>
  stop(): Promise<void>
  probe(): Promise<ProbeResult>
  sendMessage(params: SendParams): Promise<SendResult>
  normalizeInbound(raw: any): NormalizedMessage
}
```

**Location**: `src/channels/`

**Key Files**:
- `src/channels/registry.ts` - Channel registration and lifecycle
- `src/channels/plugins/outbound/*.ts` - Platform-specific sending
- `src/channels/plugins/normalize/*.ts` - Message normalization
- `src/channels/session.ts` - Session routing logic

### 3. Session Model

**Purpose**: Conversation context isolation and management

**Session Types**:
- **main** - Direct 1:1 conversations
- **group** - Group chat threads
- **isolated** - Per-user isolation in groups

**Activation Modes**:
- `mention` - Respond only when mentioned
- `always` - Respond to all messages

**Location**: `src/concepts/session.ts`

### 4. Agent Runtime (Pi)

**Purpose**: LLM interaction orchestration

**Features**:
- RPC-based execution model
- Tool call streaming
- Block-level streaming for responses
- System prompt injection
- Context window management

**Location**: `src/agents/`

### 5. Tools System

**Purpose**: Extensible action capabilities

**Built-in Tools**:
- `browser` - Web automation via CDP
- `exec` - Command execution
- `canvas` - Visual workspace control
- `sessions_*` - Cross-session coordination
- `node.invoke` - Device-specific actions

**Tool Registration**:
```typescript
api.registerTool({
  name: "tool_name",
  description: "What it does",
  parameters: { /* JSON Schema */ },
  async execute(params) {
    // Implementation
    return { content: [...] }
  }
})
```

**Location**: `src/agents/tools/`

### 6. Hooks System

**Purpose**: Event-driven automation and customization

**Hook Events**:
- `command:new` - New session started
- `command:reset` - Session reset
- `command:stop` - Session stopped
- `agent:bootstrap` - Before workspace injection
- `gateway:startup` - After channels load
- `tool_result_persist` - Before tool result saved

**Hook Pattern**:
```typescript
api.on("event:name", async (event, ctx) => {
  // React to event
  // Modify context
  // Inject behaviors
})
```

**Location**: `src/automation/hooks/`

### 7. Skills Platform

**Purpose**: Modular capability packages

**Skill Format** (AgentSkills-compatible):
```yaml
---
name: skill-name
description: What it does
metadata: {"openclaw":{"emoji":"🔧","requires":{"env":["API_KEY"]}}}
---

# Skill Documentation
Instructions and usage here...
```

**Skill Types**:
- **Bundled** - Shipped with OpenClaw
- **Managed** - Installed from ClawHub registry
- **Workspace** - User-created local skills

**Location**: `~/.openclaw/skills/`

## Data Flow: Message Processing

```
1. Inbound Message
   ↓
2. Channel Adapter receives raw message
   ↓
3. Normalize to standard format
   ↓
4. Gateway routes to appropriate session
   ↓
5. Apply gating rules (allowlist, mention, etc.)
   ↓
6. Session manager determines activation
   ↓
7. Agent runtime processes with Pi
   ↓
8. Tool calls executed if needed
   ↓
9. Hooks triggered at lifecycle points
   ↓
10. Response generated
    ↓
11. Outbound adapter formats for channel
    ↓
12. Message delivered to user
```

## Security Model

### 1. DM Pairing

**Default**: Unknown senders must complete pairing flow
- User sends message → receives pairing code
- Admin approves with `openclaw pairing approve <channel> <code>`
- Sender added to allowlist

**Configuration**:
```json
{
  "channels": {
    "telegram": {
      "dm": {
        "policy": "pairing",  // or "open"
        "allowFrom": ["allowed_user_ids"]
      }
    }
  }
}
```

### 2. Execution Isolation

- Gateway runs tools in sandboxed environment
- Separate Node processes for extension validation
- TCC permission checks for macOS system access

### 3. Authentication

**Gateway Access**:
- `token` mode - Bearer token auth
- `password` mode - Shared password
- Tailscale identity headers (for Serve mode)

## Extension Points

### 1. Plugins

Extensions can:
- Register new tools
- Add hook handlers
- Inject system prompts
- Modify configuration

**Plugin Structure**:
```
~/.openclaw/extensions/{plugin-id}/
├── index.ts              # Main entry point
└── openclaw.plugin.json  # Manifest
```

### 2. Skills

Skills provide:
- Structured documentation
- API client code
- Browser automation workflows
- Domain expertise

### 3. Hooks

Standalone hooks:
```
~/.openclaw/hooks/{hook-name}/
├── HOOK.md       # Metadata + docs
└── handler.ts    # Event handler
```

## Communication Patterns

### 1. Direct Channel → Agent
User sends message directly to bot in supported channel

### 2. Group Channel → Agent
Agent activated by mention or configured rules

### 3. Agent → Agent (Cross-Session)
Use `sessions_send` tool for inter-agent coordination

### 4. Proactive/Scheduled
Cron jobs trigger agent actions at intervals

### 5. Webhook-Driven
External services trigger agent via webhooks

## Key Design Patterns

### 1. Channel Abstraction
All channels implement common adapter interface, making OpenClaw channel-agnostic

### 2. Session Isolation
Each conversation context is isolated, preventing cross-contamination

### 3. Tool Composability
Tools can call other tools, enabling complex workflows

### 4. Hook-Based Extension
Non-invasive customization through event hooks

### 5. Declarative Configuration
YAML/JSON config drives behavior without code changes

## Configuration Hierarchy

```
1. ~/.openclaw/openclaw.json   (main config)
2. Environment variables        (overrides)
3. CLI flags                    (highest priority)
```

## Persistence

**Gateway State**:
- `~/.openclaw/state/` - Session histories, memory
- `~/.openclaw/logs/` - Structured logging

**Channel State**:
- Channel-specific auth tokens/credentials
- Pairing allowlists
- Message queues for offline handling

## Performance Characteristics

- **Startup**: ~2-5 seconds for gateway + channels
- **Message Latency**: 50-500ms channel → agent → response
- **Concurrent Sessions**: 100+ simultaneous conversations
- **Memory**: ~200-500MB base, +50MB per active session

## Integration Points for mBot

### 1. Telegram Integration
**Copy Pattern**: `/home/xanacan/projects/code/openclaw/src/channels/plugins/outbound/telegram.ts`
- Uses grammY library for bot API
- Message chunking for 4000 char limit
- HTML format support
- Reply threading via `messageThreadId`

### 2. Autonomous Behavior (via Cron + Hooks)
**Pattern**: `gateway:startup` hook + cron scheduler
- Hook triggers on gateway start
- Cron schedules periodic checks
- Agent proactively sends messages via channels

### 3. Multi-Channel Abstraction
**Pattern**: Adapter interface + registry
- Define `MbotChannelAdapter` interface
- Implement adapters for Telegram, future channels
- Register with central routing system

### 4. Self-Learning (via Foundry Integration)
**Pattern**: Observe → Learn → Crystallize → Execute
- See SELF_LEARNING_PATTERNS.md for Foundry approach

## Technology Stack

- **Language**: TypeScript
- **Runtime**: Node.js ≥22
- **Messaging Libraries**:
  - WhatsApp: Baileys
  - Telegram: grammY
  - Discord: discord.js
  - Slack: Bolt
- **WebSocket**: Native Node.js WebSocket
- **Browser Control**: Chrome DevTools Protocol (CDP)
- **Build**: pnpm workspaces

## Deployment Models

1. **Local Development**: `pnpm gateway:watch`
2. **Daemon Service**: launchd (macOS) / systemd (Linux)
3. **Docker**: Multi-container setup
4. **Remote Gateway**: Linux VPS + Tailscale Serve

## Related Documentation

- **Telegram Integration**: See TELEGRAM_INTEGRATION.md
- **Autonomous Behavior**: See AUTONOMOUS_BEHAVIOR.md
- **Self-Learning**: See SELF_LEARNING_PATTERNS.md
- **Integration Stories**: See MBOT_INTEGRATION_STORIES.md
