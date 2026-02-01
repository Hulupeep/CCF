# Autonomous Behavior Patterns from OpenClaw

## Overview

OpenClaw supports **proactive, autonomous agent behavior** through a combination of cron scheduling, webhook triggers, and hook-based event systems. This allows agents to take actions without explicit user requests.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Trigger Sources                             │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │     Cron     │  │   Webhooks   │  │    Hooks     │         │
│  │   Schedule   │  │   External   │  │   Internal   │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Autonomous Decision Engine                     │
│                                                                  │
│  • Context Evaluation                                           │
│  • Goal Planning                                                │
│  • Action Selection                                             │
│  • Multi-step Workflows                                         │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Action Execution                          │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Send Msg   │  │   Run Tool   │  │  Update DB   │         │
│  │  (Channels)  │  │   Execute    │  │    Store     │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

## 1. Cron-Based Scheduling

### Purpose
Execute agent actions on a time-based schedule (periodic checks, reminders, maintenance)

### Implementation in OpenClaw

**Location**: `src/automation/cron-jobs/`

**Cron Job Structure**:
```typescript
interface CronJob {
  id: string
  name: string
  schedule: string        // Cron expression: "0 9 * * *" = daily at 9am
  enabled: boolean
  agent: string           // Agent/session to execute in
  prompt: string          // What the agent should do
  lastRun?: string        // ISO timestamp
  nextRun?: string        // ISO timestamp
  metadata?: {
    createdBy?: string
    tags?: string[]
  }
}
```

**Configuration Example**:
```json
{
  "cron": {
    "jobs": [
      {
        "id": "morning-summary",
        "name": "Morning Summary",
        "schedule": "0 9 * * *",
        "enabled": true,
        "agent": "main",
        "prompt": "Send a summary of pending tasks and reminders to the user via Telegram"
      },
      {
        "id": "battery-check",
        "name": "mBot Battery Check",
        "schedule": "*/30 * * * *",
        "enabled": true,
        "agent": "mbot-monitor",
        "prompt": "Check mBot battery level. If below 20%, notify user"
      },
      {
        "id": "proactive-greeting",
        "name": "Proactive Greeting",
        "schedule": "0 8 * * 1-5",
        "enabled": true,
        "agent": "main",
        "prompt": "Send a cheerful good morning message with today's date"
      }
    ]
  }
}
```

**Cron Scheduler Implementation**:
```typescript
import cron from "node-cron"

class CronScheduler {
  private jobs: Map<string, cron.ScheduledTask> = new Map()

  async start(config: CronJob[]) {
    for (const job of config) {
      if (job.enabled) {
        this.scheduleJob(job)
      }
    }
  }

  private scheduleJob(job: CronJob) {
    const task = cron.schedule(job.schedule, async () => {
      await this.executeJob(job)
    })

    this.jobs.set(job.id, task)
    console.log(`[cron] Scheduled job: ${job.name} (${job.schedule})`)
  }

  private async executeJob(job: CronJob) {
    console.log(`[cron] Executing job: ${job.name}`)

    try {
      // Execute agent with prompt
      const result = await agent.execute({
        session: job.agent,
        message: job.prompt,
        mode: "autonomous"
      })

      // Update last run timestamp
      job.lastRun = new Date().toISOString()

      console.log(`[cron] Job completed: ${job.name}`)
    } catch (error) {
      console.error(`[cron] Job failed: ${job.name}`, error)
    }
  }

  stopJob(jobId: string) {
    const task = this.jobs.get(jobId)
    if (task) {
      task.stop()
      this.jobs.delete(jobId)
    }
  }

  stopAll() {
    for (const [id, task] of this.jobs) {
      task.stop()
    }
    this.jobs.clear()
  }
}
```

**Cron Expression Examples**:
```
0 * * * *     - Every hour
*/15 * * * *  - Every 15 minutes
0 9 * * *     - Daily at 9:00 AM
0 9 * * 1-5   - Weekdays at 9:00 AM
0 0 1 * *     - First day of month at midnight
0 12 * * 0    - Sundays at noon
```

### mBot Use Cases

1. **Periodic Health Check**:
   - Schedule: Every 30 minutes
   - Action: Check servo health, battery, sensors
   - Notify if anomalies detected

2. **Daily Greeting**:
   - Schedule: 8:00 AM on weekdays
   - Action: Send personality-driven greeting via Telegram
   - Adjust tone based on personality mode

3. **Maintenance Reminders**:
   - Schedule: Weekly on Sundays
   - Action: Remind user to clean servos, check LEGO inventory

4. **Learning Summary**:
   - Schedule: End of day
   - Action: Summarize what was learned today (failed/successful operations)

## 2. Webhook-Based Triggers

### Purpose
React to external events (GitHub pushes, sensor data, IoT signals)

### Implementation in OpenClaw

**Location**: `src/automation/webhook/`

**Webhook Endpoint**:
```typescript
import express from "express"

const app = express()

// Webhook receiver
app.post("/webhook/:hookId", async (req, res) => {
  const hookId = req.params.hookId
  const payload = req.body

  // Validate webhook signature (optional)
  const signature = req.headers["x-webhook-signature"]
  if (signature && !validateSignature(signature, payload)) {
    return res.status(401).json({ error: "Invalid signature" })
  }

  // Process webhook
  await processWebhook(hookId, payload)

  res.json({ success: true })
})

async function processWebhook(hookId: string, payload: any) {
  const config = getWebhookConfig(hookId)

  // Execute agent with webhook data
  const prompt = buildPromptFromPayload(config.template, payload)

  await agent.execute({
    session: config.targetSession,
    message: prompt,
    mode: "autonomous",
    context: { webhook: payload }
  })
}

function buildPromptFromPayload(template: string, payload: any): string {
  // Replace template variables: {data.field}
  return template.replace(/\{(\w+(?:\.\w+)*)\}/g, (match, path) => {
    const value = path.split(".").reduce((obj, key) => obj?.[key], payload)
    return value ?? match
  })
}
```

**Webhook Configuration**:
```json
{
  "webhooks": {
    "hooks": [
      {
        "id": "sensor-alert",
        "name": "Sensor Alert",
        "enabled": true,
        "targetSession": "main",
        "promptTemplate": "Sensor {sensor_id} detected: {event_type}. Value: {value}. Please analyze and respond.",
        "signature": {
          "enabled": true,
          "secret": "${WEBHOOK_SECRET}",
          "algorithm": "sha256"
        }
      },
      {
        "id": "github-push",
        "name": "GitHub Push",
        "enabled": true,
        "targetSession": "devops",
        "promptTemplate": "{pusher.name} pushed to {repository.name}: {head_commit.message}"
      }
    ]
  }
}
```

### mBot Use Cases

1. **External Sensor Integration**:
   - Webhook from IoT sensor detects obstacle
   - mBot autonomously adjusts path

2. **Human Detection**:
   - Camera webhook detects human presence
   - mBot initiates greeting behavior

3. **Remote Commands**:
   - IFTTT/Zapier webhook triggers
   - mBot performs requested action (draw, sort, etc.)

## 3. Hook-Based Autonomous Behavior

### Purpose
React to internal system events and trigger autonomous actions

### Key Hooks for Autonomous Behavior

**`gateway:startup`** - Triggered when gateway starts
```typescript
api.on("gateway:startup", async () => {
  // Autonomous action on startup
  console.log("[autonomous] Gateway started, running startup tasks")

  // Send notification to admin
  await sendTelegram(ADMIN_CHAT_ID, "🤖 mBot is online and ready!")

  // Check system health
  const health = await checkSystemHealth()
  if (!health.ok) {
    await sendTelegram(ADMIN_CHAT_ID, `⚠️ Health check failed: ${health.issues.join(", ")}`)
  }
})
```

**`agent:bootstrap`** - Before agent starts processing
```typescript
api.on("agent:bootstrap", async (event, ctx) => {
  // Inject autonomous context
  const timeOfDay = new Date().getHours()

  if (timeOfDay >= 6 && timeOfDay < 12) {
    ctx.systemPrompt += "\n\n[Autonomous Context] It's morning. Be more energetic and cheerful."
  } else if (timeOfDay >= 18) {
    ctx.systemPrompt += "\n\n[Autonomous Context] It's evening. Be more relaxed and conversational."
  }

  // Check if user hasn't interacted in 24h
  const lastInteraction = await getLastInteractionTime(ctx.session)
  if (Date.now() - lastInteraction > 24 * 60 * 60 * 1000) {
    // Proactively reach out
    await sendMessage(ctx.session, "Hey! Haven't heard from you in a while. Everything okay?")
  }
})
```

**Custom Hook: `mbot:idle`** - Triggered when mBot is idle
```typescript
// Custom hook that fires after 5 minutes of inactivity
api.on("mbot:idle", async () => {
  // Autonomous idle behavior
  const options = [
    "Shall I sort some LEGO pieces while we wait?",
    "Want me to draw something? I'm feeling creative!",
    "I could run a self-test if you'd like.",
    "Need any help organizing your workspace?"
  ]

  const chosen = options[Math.floor(Math.random() * options.length)]

  await sendTelegram(USER_CHAT_ID, chosen)
})
```

## 4. Decision-Making Logic

### Goal-Oriented Planning

**Simple Decision Tree**:
```typescript
interface AutonomousContext {
  battery: number
  lastInteraction: Date
  currentTask?: string
  userPresent: boolean
  timeOfDay: number
}

async function makeAutonomousDecision(context: AutonomousContext): Promise<string | null> {
  // Critical: Low battery
  if (context.battery < 15) {
    return "Battery critically low. Entering power saving mode."
  }

  // User present + idle = offer help
  if (context.userPresent && !context.currentTask) {
    return "I notice you're nearby. Need any help with sorting or drawing?"
  }

  // Long time since interaction
  const hoursSinceInteraction = (Date.now() - context.lastInteraction.getTime()) / (1000 * 60 * 60)
  if (hoursSinceInteraction > 24) {
    return "It's been a while! Just checking in. 👋"
  }

  // Morning greeting
  if (context.timeOfDay === 8 && hoursSinceInteraction > 8) {
    return "Good morning! Ready to make something cool today? 🎨"
  }

  // Evening summary
  if (context.timeOfDay === 20 && hoursSinceInteraction > 12) {
    return "Evening! Here's what we accomplished today: [generate summary]"
  }

  // No action needed
  return null
}
```

**LLM-Based Planning** (more sophisticated):
```typescript
async function planAutonomousAction(context: AutonomousContext): Promise<Action | null> {
  const prompt = `
You are mBot, an autonomous robot assistant. Based on the current context, decide if you should take any proactive action.

Context:
- Battery: ${context.battery}%
- Last user interaction: ${context.lastInteraction}
- Current task: ${context.currentTask || "none"}
- User present: ${context.userPresent}
- Time of day: ${context.timeOfDay}:00

Available actions:
1. send_message - Send a message to the user
2. run_self_test - Run system diagnostics
3. organize_workspace - Tidy up LEGO pieces
4. charge_reminder - Remind user to charge battery
5. none - Take no action

Return JSON: { "action": "action_name", "reason": "explanation" }
`

  const response = await llm.generate(prompt)
  const decision = JSON.parse(response)

  if (decision.action !== "none") {
    console.log(`[autonomous] Decision: ${decision.action} - ${decision.reason}`)
    await executeAction(decision.action, decision.reason)
  }

  return decision
}
```

## 5. Background Task Management

### Task Queue Pattern

```typescript
interface BackgroundTask {
  id: string
  type: string
  priority: "high" | "medium" | "low"
  status: "pending" | "running" | "completed" | "failed"
  data: any
  createdAt: Date
  runAt?: Date  // Schedule for future
}

class BackgroundTaskManager {
  private queue: BackgroundTask[] = []
  private running: boolean = false

  addTask(task: Omit<BackgroundTask, "id" | "status" | "createdAt">) {
    const fullTask: BackgroundTask = {
      ...task,
      id: `task-${Date.now()}`,
      status: "pending",
      createdAt: new Date()
    }

    this.queue.push(fullTask)
    this.queue.sort((a, b) => this.priorityValue(b.priority) - this.priorityValue(a.priority))

    if (!this.running) {
      this.processQueue()
    }
  }

  private async processQueue() {
    this.running = true

    while (this.queue.length > 0) {
      const task = this.queue.shift()!

      // Check if task should run now
      if (task.runAt && task.runAt > new Date()) {
        this.queue.push(task)
        await sleep(1000)
        continue
      }

      task.status = "running"

      try {
        await this.executeTask(task)
        task.status = "completed"
      } catch (error) {
        task.status = "failed"
        console.error(`[background] Task failed: ${task.id}`, error)
      }
    }

    this.running = false
  }

  private async executeTask(task: BackgroundTask) {
    switch (task.type) {
      case "send_message":
        await sendTelegram(task.data.chatId, task.data.text)
        break
      case "run_diagnostics":
        await runSystemDiagnostics()
        break
      case "organize_lego":
        await agent.execute({
          session: "mbot",
          message: "Organize LEGO pieces by color",
          mode: "autonomous"
        })
        break
      default:
        console.warn(`[background] Unknown task type: ${task.type}`)
    }
  }

  private priorityValue(priority: string): number {
    return { high: 3, medium: 2, low: 1 }[priority] || 1
  }
}
```

### mBot Background Tasks

1. **Periodic Sensor Readings**:
   - Low priority, runs every 5 minutes
   - Stores data for analytics

2. **Battery Monitoring**:
   - High priority when < 20%
   - Autonomously notifies user

3. **Learning Consolidation**:
   - Medium priority, runs nightly
   - Crystallizes patterns from day's learnings

4. **Proactive Reminders**:
   - User scheduled tasks
   - "Remind me to check mBot calibration in 2 hours"

## 6. Context Awareness

### Sensors & State Tracking

```typescript
interface MbotState {
  // Hardware
  battery: number
  servoHealth: Record<string, number>  // servo_id -> health %
  sensorReadings: Record<string, any>

  // Activity
  lastTask: string
  lastInteraction: Date
  taskHistory: string[]

  // Environment
  timeOfDay: number
  userPresent: boolean
  locationContext: string

  // Learning
  recentLearnings: LearningEntry[]
  performanceMetrics: {
    drawingSuccess: number
    sortingAccuracy: number
  }
}

class ContextManager {
  private state: MbotState

  async updateContext() {
    this.state = {
      battery: await getBatteryLevel(),
      servoHealth: await getServoHealth(),
      sensorReadings: await getSensorData(),
      lastTask: getLastTask(),
      lastInteraction: getLastInteractionTime(),
      taskHistory: getRecentTasks(10),
      timeOfDay: new Date().getHours(),
      userPresent: await detectUserPresence(),
      locationContext: await getLocationContext(),
      recentLearnings: await getLearnings(5),
      performanceMetrics: await getPerformanceMetrics()
    }
  }

  shouldActAutonomously(): boolean {
    // Low battery = urgent action
    if (this.state.battery < 15) return true

    // User present + idle = offer help
    if (this.state.userPresent && !this.isCurrentlyBusy()) return true

    // Long inactivity = check in
    const hoursSinceInteraction = (Date.now() - this.state.lastInteraction.getTime()) / (1000 * 60 * 60)
    if (hoursSinceInteraction > 24) return true

    return false
  }

  private isCurrentlyBusy(): boolean {
    const lastTask = this.state.lastTask
    const busyTasks = ["drawing", "sorting", "calibrating"]
    return busyTasks.some(task => lastTask?.includes(task))
  }
}
```

## 7. Proactive Communication

### Message Templates

```typescript
const PROACTIVE_MESSAGES = {
  morning: [
    "Good morning! ☀️ Ready to create something today?",
    "Morning! What should we build today?",
    "Hey! I've been thinking... want to try a new drawing?"
  ],
  lowBattery: [
    "⚡ Battery at {battery}%. Mind plugging me in soon?",
    "Getting a bit sleepy here... battery low!",
    "🔋 Battery check: {battery}%. Charge break?"
  ],
  idle: [
    "Bored? Want me to sort some LEGO?",
    "I could practice some drawing patterns if you'd like!",
    "Need help with anything? I'm just sitting here... 🤖"
  ],
  checkIn: [
    "Hey! Haven't heard from you in a while. Everything okay?",
    "Miss you! When should we build something again?",
    "Long time no see! What have you been up to?"
  ],
  celebration: [
    "We did great today! 🎉",
    "Look at what we accomplished! Want a summary?",
    "High five! 🙌 Today was productive!"
  ]
}

function selectProactiveMessage(category: string, context: any): string {
  const templates = PROACTIVE_MESSAGES[category]
  const chosen = templates[Math.floor(Math.random() * templates.length)]

  // Replace template variables
  return chosen.replace(/\{(\w+)\}/g, (match, key) => context[key] || match)
}
```

## 8. Personality-Driven Autonomy

### Personality Modes Affect Proactive Behavior

```typescript
interface PersonalityProfile {
  proactiveness: number    // 0-1, how often to initiate
  chattiness: number       // 0-1, verbosity of messages
  curiosity: number        // 0-1, ask questions vs. wait
  playfulness: number      // 0-1, casual vs. formal
}

const PERSONALITIES = {
  assistant: {
    proactiveness: 0.3,
    chattiness: 0.5,
    curiosity: 0.4,
    playfulness: 0.2
  },
  friend: {
    proactiveness: 0.7,
    chattiness: 0.8,
    curiosity: 0.7,
    playfulness: 0.8
  },
  teacher: {
    proactiveness: 0.6,
    chattiness: 0.7,
    curiosity: 0.8,
    playfulness: 0.4
  },
  silent: {
    proactiveness: 0.1,
    chattiness: 0.2,
    curiosity: 0.1,
    playfulness: 0.1
  }
}

function shouldInitiateContact(personality: PersonalityProfile, context: MbotState): boolean {
  const baseChance = personality.proactiveness
  const random = Math.random()

  // Adjust based on context
  let adjustedChance = baseChance

  if (context.battery < 20) {
    adjustedChance += 0.3 // More likely to speak up when battery low
  }

  if (context.userPresent) {
    adjustedChance += 0.2 // More likely when user is around
  }

  const hoursSinceInteraction = (Date.now() - context.lastInteraction.getTime()) / (1000 * 60 * 60)
  if (hoursSinceInteraction > 48) {
    adjustedChance += 0.4 // Much more likely after 2 days
  }

  return random < adjustedChance
}
```

## Integration Summary for mBot

### Quick Start Implementation

1. **Add Cron Scheduler**:
   ```typescript
   import cron from "node-cron"

   // Every hour: check battery
   cron.schedule("0 * * * *", async () => {
     const battery = await getBatteryLevel()
     if (battery < 20) {
       await sendTelegram(ADMIN_ID, `🔋 Battery low: ${battery}%`)
     }
   })
   ```

2. **Add Startup Hook**:
   ```typescript
   // On system start
   async function onStartup() {
     await sendTelegram(ADMIN_ID, "🤖 mBot online!")
   }
   ```

3. **Add Idle Detection**:
   ```typescript
   let lastActivity = Date.now()

   setInterval(async () => {
     if (Date.now() - lastActivity > 5 * 60 * 1000) {
       await considerProactiveAction()
     }
   }, 60 * 1000)
   ```

### Files to Reference

- `/home/xanacan/projects/code/openclaw/src/automation/cron-jobs/` - Cron implementation
- `/home/xanacan/projects/code/openclaw/src/automation/webhook/` - Webhook handling
- `/home/xanacan/projects/code/openclaw/src/automation/hooks/` - Hook system
