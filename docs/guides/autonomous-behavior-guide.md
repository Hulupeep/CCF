# Autonomous Behavior Guide

Complete guide to mBot's autonomous proactive behavior system.

## Overview

The autonomous behavior system enables mBot to take proactive actions without explicit user commands. It uses a combination of:
- **Cron scheduling** - Time-based triggers
- **Event-driven actions** - React to system events
- **Context awareness** - Make decisions based on state

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Trigger Sources                         │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │     Cron     │  │   Events     │  │   Context    │     │
│  │   Schedule   │  │   System     │  │   Monitor    │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                   Autonomy Engine                            │
│                                                              │
│  • Action Registration                                      │
│  • Trigger Evaluation                                       │
│  • Personality-Driven Decisions                             │
│  • Safety System                                            │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                   Action Execution                           │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Send Msg   │  │   Log Data   │  │  Update UI   │     │
│  │  (Telegram)  │  │   Monitor    │  │  Dashboard   │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Autonomy Engine

**Location**: `web/src/services/autonomy/AutonomyEngine.ts`

The central coordinator for autonomous behavior.

**Key Features**:
- Action registration and lifecycle management
- Trigger evaluation (cron, event, context)
- Personality-driven decision making
- Safety system with approval flow
- Execution tracking and statistics

**Example Usage**:
```typescript
import { getAutonomyEngine } from './services/autonomy';

const engine = getAutonomyEngine({
  enabled: true,
  safeMode: false,
  maxConcurrentActions: 5,
  defaultCooldown: 60000,
});

// Register an action
engine.registerAction({
  id: 'my-action',
  name: 'My Custom Action',
  description: 'Does something proactive',
  trigger: {
    type: 'cron',
    condition: '0 8 * * *', // Daily at 8 AM
  },
  action: async (context) => {
    console.log('Action executed!');
  },
  enabled: true,
});

// Start the engine
engine.start();
```

### 2. Cron Scheduler

**Location**: `web/src/services/autonomy/CronScheduler.ts`

Time-based action triggers using cron expressions.

**Cron Expression Examples**:
```
'* * * * *'       - Every minute
'*/5 * * * *'     - Every 5 minutes
'0 * * * *'       - Every hour
'0 8 * * *'       - Daily at 8 AM
'0 8 * * 1-5'     - Weekdays at 8 AM
'0 20 * * 0'      - Sundays at 8 PM
```

**Example**:
```typescript
import { getCronScheduler, CronExamples } from './services/autonomy';

const scheduler = getCronScheduler();

scheduler.schedule(
  {
    id: 'morning-greeting',
    name: 'Morning Greeting',
    schedule: CronExamples.EVERY_DAY_8AM,
    enabled: true,
  },
  async () => {
    console.log('Good morning!');
  }
);
```

### 3. Event Bus

**Location**: `web/src/services/autonomy/EventBus.ts`

Publish-subscribe event system for event-driven triggers.

**Built-in Events**:
```typescript
EventTypes.USER_MESSAGE          - User sent message
EventTypes.USER_INACTIVE          - User inactive (no context trigger)
EventTypes.ROBOT_CONNECTED        - Robot came online
EventTypes.ROBOT_DISCONNECTED     - Robot went offline
EventTypes.ROBOT_IDLE             - Robot idle
EventTypes.PERSONALITY_CHANGED    - Personality switched
EventTypes.SENSOR_BATTERY_LOW     - Battery < 20%
EventTypes.SENSOR_BATTERY_CRITICAL - Battery < 10%
EventTypes.SYSTEM_STARTUP         - System started
```

**Example**:
```typescript
import { getEventBus, EventTypes } from './services/autonomy';

const eventBus = getEventBus();

// Listen for events
eventBus.on(EventTypes.SENSOR_BATTERY_LOW, async (event) => {
  console.log('Battery low!', event.data);
});

// Emit events
await eventBus.emit(EventTypes.USER_MESSAGE, {
  userId: '123',
  message: 'Hello!',
});
```

### 4. Context Monitor

**Location**: `web/src/services/autonomy/ContextMonitor.ts`

Tracks system state for context-aware decision making.

**Context Properties**:
```typescript
interface Context {
  // Time
  timeOfDay: 'morning' | 'afternoon' | 'evening' | 'night';
  dayOfWeek: string;
  hour: number;

  // User activity
  lastUserInteraction?: number;
  userPresent: boolean;
  messageCount: number;

  // Robot state
  robotStatus: 'online' | 'offline' | 'busy' | 'idle';
  currentActivity?: string;
  batteryLevel?: number;
}
```

**Example**:
```typescript
import { getContextMonitor } from './services/autonomy';

const contextMonitor = getContextMonitor();

// Update context
contextMonitor.recordUserInteraction();
contextMonitor.updateRobotStatus('busy', 'drawing');
contextMonitor.updateBatteryLevel(75);

// Query context
const ctx = contextMonitor.getContext();
console.log(`Time: ${ctx.timeOfDay}, Battery: ${ctx.batteryLevel}%`);

// Check conditions
if (contextMonitor.isInactive(60)) {
  console.log('User inactive for 1 hour');
}

if (contextMonitor.isTimeRange(8, 12)) {
  console.log('It\'s morning time!');
}
```

## Built-in Autonomous Actions

### 1. Good Morning Message

**ID**: `good-morning`
**Trigger**: Cron - Daily at 8 AM
**Action**: Send cheerful morning greeting

**Personality Variants**:
- High energy: "GOOD MORNING! ☀️ I'm SO ready!"
- Low energy: "Morning... Slowly waking up here."
- High curiosity: "Good morning! Want to try something new?"

### 2. Inactivity Check

**ID**: `inactivity-check`
**Trigger**: Context - 6+ hours user inactivity
**Action**: Check in with user
**Cooldown**: 6 hours

**Example Messages**:
- "Hey! Haven't heard from you in a while! Everything okay?"
- "Hi. Just checking in. All good?"

### 3. Battery Low Alert

**ID**: `battery-low-alert`
**Trigger**: Event - `sensor.battery_low`
**Action**: Notify when battery < 20%
**Cooldown**: 1 hour

**Message**: "⚠️ My battery is getting low (15%). I might need to recharge soon!"

### 4. Battery Critical Alert

**ID**: `battery-critical-alert`
**Trigger**: Event - `sensor.battery_critical`
**Action**: Urgent notification when battery < 10%
**Cooldown**: 30 minutes

**Message**: "🚨 URGENT: Battery critically low (8%)! I need to be charged NOW!"

### 5. Weekly Recap

**ID**: `weekly-recap`
**Trigger**: Cron - Sundays at 8 PM
**Action**: Send weekly activity summary
**Cooldown**: 7 days

**Content**:
- Messages exchanged
- Tasks completed
- Drawings created
- Games played
- Favorite activity

### 6. Idle Offer

**ID**: `idle-offer`
**Trigger**: Context - Robot idle + user recently active
**Action**: Proactively offer assistance
**Cooldown**: 30 minutes

**Example Offers**:
- "Want me to sort some LEGO pieces?"
- "I could practice some drawing patterns!"
- "Need help with anything?"
- "Want to play a game?"

## Creating Custom Actions

### Basic Action Template

```typescript
import { AutonomousAction } from './services/autonomy';

export const myCustomAction: AutonomousAction = {
  id: 'my-custom-action',
  name: 'My Custom Action',
  description: 'Does something custom',
  trigger: {
    type: 'cron', // or 'event' or 'context'
    condition: '0 9 * * *', // 9 AM daily
    cooldown: 86400000, // 24 hours
  },
  action: async (context) => {
    const { personality, systemContext } = context;

    // Your logic here
    console.log('Custom action executed!');
  },
  enabled: true,
  executionCount: 0,
  requiresApproval: false,
  metadata: {
    category: 'custom',
    tags: ['proactive'],
  },
};
```

### Cron-Based Action

```typescript
export const dailyReminder: AutonomousAction = {
  id: 'daily-reminder',
  name: 'Daily Reminder',
  description: 'Send daily reminder at 10 AM',
  trigger: {
    type: 'cron',
    condition: '0 10 * * *',
    cooldown: 86400000,
  },
  action: async (context) => {
    console.log('Time for your daily check-in!');
  },
  enabled: true,
  executionCount: 0,
};
```

### Event-Based Action

```typescript
export const userReturnedGreeting: AutonomousAction = {
  id: 'user-returned',
  name: 'User Returned Greeting',
  description: 'Greet user when they return',
  trigger: {
    type: 'event',
    condition: EventTypes.USER_RETURNED,
  },
  action: async (context) => {
    console.log('Welcome back!');
  },
  enabled: true,
  executionCount: 0,
};
```

### Context-Based Action

```typescript
export const eveningWindDown: AutonomousAction = {
  id: 'evening-wind-down',
  name: 'Evening Wind Down',
  description: 'Suggest winding down in evening',
  trigger: {
    type: 'context',
    condition: (ctx) => {
      return (
        ctx.timeOfDay === 'evening' &&
        ctx.robotStatus === 'idle' &&
        !ctx.currentActivity
      );
    },
    cooldown: 3600000, // 1 hour
  },
  action: async (context) => {
    console.log('It\'s evening. Time to wind down?');
  },
  enabled: true,
  executionCount: 0,
};
```

## Personality Integration

Actions can adapt to personality parameters:

```typescript
action: async (context) => {
  const { personality } = context;
  const { energy_baseline, curiosity_drive, playfulness } = personality;

  let message = '';

  if (energy_baseline > 0.7) {
    message = "I'M SO EXCITED! Let's do something fun!";
  } else if (energy_baseline < 0.3) {
    message = "Hey... what's up?";
  } else {
    message = "Hi! What should we do?";
  }

  if (curiosity_drive > 0.7) {
    message += " I'm curious what you're thinking...";
  }

  console.log(message);
}
```

## Safety Considerations

### Safe Mode

Enable safe mode to require approval for all actions:

```typescript
const engine = getAutonomyEngine({
  enabled: true,
  safeMode: true, // All actions require approval
});
```

### Action Approval

Mark specific actions as requiring approval:

```typescript
const dangerousAction: AutonomousAction = {
  id: 'dangerous-action',
  name: 'Dangerous Action',
  description: 'Requires approval',
  trigger: { type: 'event', condition: 'test' },
  action: async () => {
    // Sensitive operation
  },
  enabled: true,
  executionCount: 0,
  requiresApproval: true, // Always require approval
};
```

### Action Whitelisting

Actions marked with `requiresApproval: false` always execute:

```typescript
const safeAction: AutonomousAction = {
  id: 'safe-action',
  name: 'Safe Action',
  description: 'Always safe to execute',
  trigger: { type: 'cron', condition: '0 * * * *' },
  action: async () => {
    // Safe operation
  },
  enabled: true,
  executionCount: 0,
  requiresApproval: false, // Explicitly safe
};
```

## Telegram Integration

Actions can send messages via Telegram:

```typescript
import { TelegramBot } from './services/telegram/TelegramBot';

let telegramBot: TelegramBot | null = null;

export function setTelegramBot(bot: TelegramBot) {
  telegramBot = bot;
}

export const telegramAction: AutonomousAction = {
  id: 'telegram-message',
  name: 'Telegram Message',
  description: 'Send message via Telegram',
  trigger: {
    type: 'cron',
    condition: '0 8 * * *',
  },
  action: async (context) => {
    if (telegramBot) {
      // TODO: Implement sendAutonomousMessage
      // await telegramBot.sendAutonomousMessage(userId, 'Good morning!');
      console.log('Would send Telegram message');
    }
  },
  enabled: true,
  executionCount: 0,
};
```

## Testing

Run autonomy tests:

```bash
cd web
npm test -- autonomy-engine.test.ts
```

**Test Coverage**:
- ✅ Action registration and lifecycle
- ✅ Cron trigger execution
- ✅ Event trigger execution
- ✅ Context trigger evaluation
- ✅ Cooldown enforcement
- ✅ Personality-driven decisions
- ✅ Safety system blocking
- ✅ Statistics tracking

## Monitoring and Debugging

### Get Engine Statistics

```typescript
const engine = getAutonomyEngine();
const stats = engine.getStats();

console.log(`Running: ${stats.running}`);
console.log(`Total Actions: ${stats.totalActions}`);
console.log(`Enabled: ${stats.enabledActions}`);
console.log(`Total Executions: ${stats.totalExecutions}`);
```

### List All Actions

```typescript
const actions = engine.getAllActions();
actions.forEach((action) => {
  console.log(`${action.name} (${action.id})`);
  console.log(`  Enabled: ${action.enabled}`);
  console.log(`  Executions: ${action.executionCount}`);
  console.log(`  Last: ${action.lastExecuted || 'Never'}`);
});
```

### View Recent Events

```typescript
const eventBus = getEventBus();
const recent = eventBus.getAllRecentEvents(10);

recent.forEach((event) => {
  console.log(`${event.type}: ${JSON.stringify(event.data)}`);
});
```

### Monitor Context

```typescript
const contextMonitor = getContextMonitor();

contextMonitor.subscribe((ctx) => {
  console.log('Context updated:', ctx);
});
```

## Troubleshooting

### Actions Not Executing

1. **Check if engine is running**:
   ```typescript
   console.log(engine.getStats().running);
   ```

2. **Check if action is enabled**:
   ```typescript
   console.log(engine.getAction('action-id')?.enabled);
   ```

3. **Check cooldown**:
   ```typescript
   const action = engine.getAction('action-id');
   if (action?.lastExecuted) {
     const elapsed = Date.now() - action.lastExecuted;
     console.log(`Cooldown remaining: ${action.trigger.cooldown - elapsed}ms`);
   }
   ```

### Cron Not Firing

1. **Validate cron expression**:
   ```typescript
   import cron from 'node-cron';
   console.log(cron.validate('0 8 * * *')); // Should be true
   ```

2. **Check scheduler status**:
   ```typescript
   const scheduler = getCronScheduler();
   console.log(scheduler.getAllJobs());
   ```

### Events Not Triggering

1. **Check event bus handlers**:
   ```typescript
   const eventBus = getEventBus();
   console.log(eventBus.getHandlerCount(EventTypes.USER_MESSAGE));
   ```

2. **Verify event emission**:
   ```typescript
   await eventBus.emit(EventTypes.USER_MESSAGE, { test: true });
   ```

## Best Practices

1. **Use descriptive IDs**: `'battery-low-alert'` not `'action1'`
2. **Set appropriate cooldowns**: Prevent spam
3. **Consider personality**: Adapt behavior to personality
4. **Handle errors gracefully**: Wrap action logic in try-catch
5. **Test thoroughly**: Write integration tests
6. **Monitor execution**: Track statistics
7. **Respect user context**: Don't interrupt busy users
8. **Provide value**: Actions should be helpful, not annoying

## Next Steps

- Implement Telegram message sending in actions
- Add user preference controls (enable/disable actions)
- Create UI dashboard for monitoring autonomy
- Implement approval flow for sensitive actions
- Add action scheduling UI
- Create action marketplace for community actions

---

**Built with ❤️ for mBot RuVector AI**
