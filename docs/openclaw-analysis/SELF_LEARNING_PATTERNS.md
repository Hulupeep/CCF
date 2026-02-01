# Self-Learning Patterns from OpenClaw Foundry

## Overview

OpenClaw Foundry implements **recursive self-improvement** through workflow observation, pattern crystallization, and code generation. The system learns from user interactions and writes new capabilities into itself.

## Core Insight: Knowledge vs. Behavior

| **Knowledge (Patterns)** | **Behavior (Self-Written Code)** |
|--------------------------|----------------------------------|
| Stored as text in JSON   | Baked into the system as code   |
| LLM must read/apply each time | Runs automatically          |
| Uses tokens every invocation | Zero token cost after written |
| Can be forgotten or ignored | Always executes              |

**The key innovation**: When a pattern is proven (5+ successful uses, 70%+ success rate), Foundry **crystallizes** it into executable code that becomes part of the system.

## Learning Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        OBSERVATION LAYER                         │
│   Hooks: after_tool_call, agent_end, before_agent_start        │
│   Tracks: Goal → Tool Sequence → Outcome → Duration            │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                       LEARNING ENGINE                            │
│                                                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  Failure Store  │  │ Pattern Store   │  │ Workflow Store  │ │
│  │  (error→fix)    │  │ (proven fixes)  │  │ (goal→tools)    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│                                                                  │
│  Crystallization Rules:                                         │
│  • 5+ successful uses                                           │
│  • 70%+ success rate                                            │
│  • Clear pattern boundaries                                     │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                        CODE WRITER                               │
│   Templates + Validation + Sandbox Testing                      │
│   Writes to: ~/.openclaw/extensions/                            │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                       DEPLOYMENT                                 │
│   Gateway Restart → New Capabilities Available                  │
└─────────────────────────────────────────────────────────────────┘
```

## Key Components

### 1. LearningEngine Class

**Purpose**: Records patterns from successes and failures

**Key Methods**:
```typescript
class LearningEngine {
  // Record a failure
  recordFailure(tool: string, error: string, context?: string): string

  // Link a resolution to previous failure
  recordResolution(failureId: string, resolution: string): void

  // Record successful tool use
  recordSuccess(tool: string, context: string): void

  // Create reusable insight
  recordInsight(insight: string, context?: string): void

  // Find relevant patterns for current context
  findRelevantLearnings(tool?: string, errorPattern?: string): LearningEntry[]

  // Get patterns ready for crystallization
  getCrystallizationCandidates(): LearningEntry[]
}
```

**Data Structure**:
```typescript
interface LearningEntry {
  id: string
  type: "failure" | "pattern" | "insight" | "success"
  tool?: string
  error?: string
  resolution?: string
  context?: string
  timestamp: string
  useCount: number  // Incremented on each successful use

  // RISE: Recursive introspection
  attemptCount?: number
  improvementTrajectory?: number[]

  // SelfEvolve: Interpreter feedback
  executionFeedback?: string[]

  // HexMachina: Crystallization tracking
  crystallizedTo?: string
  crystallizedAt?: string
}
```

**Storage Location**: `~/.openclaw/foundry/learnings.json`

### 2. Workflow Learning System

**Purpose**: Track multi-step workflows and crystallize high-value patterns

**Workflow Structure**:
```typescript
interface WorkflowEntry {
  id: string
  goal: string              // What user wanted to achieve
  goalKeywords: string[]    // Extracted keywords for matching
  toolSequence: string[]    // Tools used in order
  outcome: "success" | "failure" | "partial"
  duration: number          // Time taken in milliseconds
  timestamp: string
}

interface WorkflowPattern {
  patternId: string
  goalPattern: string       // Regex or keyword pattern
  toolSequence: string[]    // Common tool sequence
  occurrences: number       // How many times seen
  successRate: number       // 0-1 success ratio
  avgDuration: number       // Average time
  lastUsed: string
}
```

**Crystallization Threshold**:
- 5+ occurrences
- 70%+ success rate
- Clear, repeatable sequence

**Example**:
```
Workflow observed 7 times:
  Goal: "deploy to staging"
  Sequence: ["git_status", "git_add", "run_tests", "deploy_staging"]
  Success Rate: 87%

→ Crystallizes into single tool: "deploy_staging"
```

### 3. The Overseer (Autonomous Learning Agent)

**Purpose**: Background process that auto-improves the system

**Schedule**: Hourly cron job

**Responsibilities**:
1. **Identify Crystallization Candidates**
   - Scan workflow patterns for 5+ uses
   - Check success rates

2. **Auto-Generate Tools**
   - Create dedicated tool from proven pattern
   - Validate in sandbox
   - Deploy if passes

3. **Prune Stale Patterns**
   - Remove unused patterns (30+ days)
   - Keep learnings compact

4. **Track Performance** (ADAS-style)
   - Monitor tool usage after crystallization
   - Collect fitness metrics
   - Suggest improvements

**Overseer Report Structure**:
```typescript
interface OverseerReport {
  timestamp: string
  patternsAnalyzed: number
  crystallizationCandidates: LearningEntry[]
  actionsTriggered: {
    crystallized: string[]  // New tools created
    pruned: string[]        // Stale patterns removed
    insights: string[]      // New learnings generated
  }
  performanceMetrics: {
    toolUsage: Record<string, number>
    avgSuccessRate: number
    avgDuration: number
  }
}
```

## Learning Workflow

### Phase 1: Observation

**Hook**: `after_tool_call`

```typescript
api.on("after_tool_call", async (event) => {
  const { toolName, result, error, context } = event

  if (error) {
    // Record failure
    const failureId = learningEngine.recordFailure(
      toolName,
      error.message,
      context
    )

    // Check for known patterns
    const similarPatterns = learningEngine.findRelevantLearnings(
      toolName,
      error.message
    )

    if (similarPatterns.length > 0) {
      // Inject suggestion proactively
      injectSuggestion(`Try: ${similarPatterns[0].resolution}`)
    }
  } else {
    // Check if this resolved a recent failure
    if (lastFailureId && wasResolved(result)) {
      learningEngine.recordResolution(
        lastFailureId,
        extractResolution(result)
      )
    }

    // Record success
    learningEngine.recordSuccess(toolName, context)
  }
})
```

### Phase 2: Pattern Recognition

**Hook**: `agent_end`

```typescript
api.on("agent_end", async (event) => {
  const { session, toolsUsed, outcome, duration } = event

  // Record workflow
  const workflow = {
    goal: extractGoal(session.messages),
    goalKeywords: extractKeywords(session.messages[0]),
    toolSequence: toolsUsed.map(t => t.name),
    outcome,
    duration
  }

  workflowTracker.recordWorkflow(workflow)

  // Check if workflow matches existing pattern
  const matchingPattern = workflowTracker.findMatchingPattern(workflow)

  if (matchingPattern) {
    matchingPattern.occurrences++
    matchingPattern.successRate = calculateSuccessRate(matchingPattern)

    // Check crystallization threshold
    if (matchingPattern.occurrences >= 5 &&
        matchingPattern.successRate >= 0.7) {
      crystallizationQueue.add(matchingPattern)
    }
  }
})
```

### Phase 3: Context Injection

**Hook**: `before_agent_start`

```typescript
api.on("before_agent_start", async (event, ctx) => {
  // Get recent patterns
  const patterns = learningEngine.getPatterns().slice(-3)
  const insights = learningEngine.getInsights().slice(-2)

  if (patterns.length > 0 || insights.length > 0) {
    ctx.systemPrompt += `
[Foundry Learnings]
${patterns.map(p => `- "${p.error}" → ${p.resolution}`).join('\n')}
${insights.map(i => `- Insight: ${i.context}`).join('\n')}
`
  }

  // Check for relevant workflow patterns
  const goalKeywords = extractKeywords(event.initialMessage)
  const relevantPatterns = workflowTracker.findPatternsByKeywords(goalKeywords)

  if (relevantPatterns.length > 0) {
    ctx.systemPrompt += `
[Suggested Workflows]
${relevantPatterns.map(p =>
  `- For "${p.goalPattern}": use ${p.toolSequence.join(' → ')}`
).join('\n')}
`
  }
})
```

### Phase 4: Crystallization

**Overseer Process** (runs hourly):

```typescript
async function runOverseer() {
  const candidates = learningEngine.getCrystallizationCandidates()
  const report: OverseerReport = {
    timestamp: new Date().toISOString(),
    patternsAnalyzed: candidates.length,
    crystallizationCandidates: candidates,
    actionsTriggered: {
      crystallized: [],
      pruned: [],
      insights: []
    }
  }

  for (const candidate of candidates) {
    // Generate tool from pattern
    const toolCode = generateToolFromPattern(candidate)

    // Validate in sandbox
    const validation = await codeValidator.validate(toolCode)

    if (validation.success) {
      // Write to extensions
      const toolPath = await codeWriter.writeExtension({
        id: `crystallized-${candidate.id}`,
        name: candidate.resolution || "Unnamed Tool",
        description: `Auto-generated from pattern: ${candidate.context}`,
        tools: [toolCode]
      })

      // Mark as crystallized
      candidate.crystallizedTo = toolPath
      candidate.crystallizedAt = new Date().toISOString()

      report.actionsTriggered.crystallized.push(toolPath)
    }
  }

  // Prune stale patterns
  const stalePatterns = learningEngine.getStalePatterns(30) // 30 days
  for (const stale of stalePatterns) {
    learningEngine.deletePattern(stale.id)
    report.actionsTriggered.pruned.push(stale.id)
  }

  return report
}
```

## Research Foundations

Foundry's approach is grounded in recent AI research:

### 1. Self-Improving Code Agents (Robeyns et al., 2025)
**arXiv:2504.15228**

**Key Insight**: "An agent system, equipped with basic coding tools, can autonomously edit itself, and thereby improve its performance"

**Application**: `foundry_extend_self` tool allows the agent to modify its own codebase

**Results**: 17-53% improvement through "non-gradient learning via LLM reflection and code updates"

### 2. SelfEvolve (Jiang et al., 2023)
**arXiv:2306.02907**

**Key Insight**: Two-step pipeline: knowledge generation + self-reflection debugging using interpreter feedback

**Application**: LearningEngine records outcomes → patterns → crystallization with execution feedback

### 3. RISE: Recursive Introspection (Qu et al., 2024)
**arXiv:2407.18219**

**Key Insight**: Iterative fine-tuning teaches models to "alter responses after unsuccessful attempts" via multi-turn MDPs

**Application**: Workflow tracking learns from outcomes, suggests improvements on subsequent attempts

### 4. HexMachina (Liu et al., 2025)
**arXiv:2506.04651**

**Key Insight**: "Artifact-centric continual learning" — separates discovery from strategy evolution through code refinement

**Application**: Patterns (knowledge) crystallize into hooks/tools (behavior) that persist across sessions

### 5. ADAS: Automated Design of Agentic Systems (Hu et al., 2024)
**arXiv:2408.08435**

**Key Insight**: Meta-agent iteratively discovers improved agent designs through archive-based evolution

**Application**: Overseer tracks tool fitness, evolves patterns using success metrics

## Code Examples

### Example 1: Recording a Failure

```typescript
// User tries to call API, gets rate limited
const failureId = learningEngine.recordFailure(
  "fetch_weather",
  "429 Too Many Requests",
  "Calling weather API"
)
// Returns: "fail-fetch_weather-1705234567890"
```

### Example 2: Linking a Resolution

```typescript
// User adds retry logic, call succeeds
learningEngine.recordResolution(
  failureId,
  "Added exponential backoff: retry after 1s, 2s, 4s"
)
// Pattern created: error → resolution
```

### Example 3: Pattern Lookup

```typescript
// Next time user encounters rate limit
const patterns = learningEngine.findRelevantLearnings(
  "fetch_weather",
  "429 Too Many Requests"
)
// Returns: [{ error: "429...", resolution: "Added exponential backoff..." }]
```

### Example 4: Workflow Crystallization

```typescript
// After 7 successful "deploy to staging" workflows
const pattern = {
  goalPattern: "deploy.*staging",
  toolSequence: ["git_status", "git_add", "run_tests", "deploy_staging"],
  occurrences: 7,
  successRate: 0.87
}

// Crystallize into single tool
const toolCode = `
async function deploy_staging() {
  await exec("git status")
  await exec("git add .")
  await exec("npm test")
  await exec("npm run deploy:staging")
  return { success: true }
}
`
// Now "deploy to staging" is one command
```

## Storage Files

### 1. Learnings Database
**Path**: `~/.openclaw/foundry/learnings.json`

```json
[
  {
    "id": "fail-123",
    "type": "failure",
    "tool": "fetch_weather",
    "error": "429 Too Many Requests",
    "context": "Calling weather API",
    "timestamp": "2025-02-01T10:30:00Z",
    "useCount": 0
  },
  {
    "id": "pat-456",
    "type": "pattern",
    "tool": "fetch_weather",
    "error": "429 Too Many Requests",
    "resolution": "Added exponential backoff",
    "useCount": 5,
    "crystallizedTo": "retry-backoff-tool",
    "crystallizedAt": "2025-02-02T14:20:00Z"
  }
]
```

### 2. Workflows Database
**Path**: `~/.openclaw/foundry/workflows.json`

```json
[
  {
    "id": "wf-789",
    "goal": "deploy to staging",
    "goalKeywords": ["deploy", "staging"],
    "toolSequence": ["git_status", "git_add", "run_tests", "deploy_staging"],
    "outcome": "success",
    "duration": 45000,
    "timestamp": "2025-02-01T15:00:00Z"
  }
]
```

### 3. Workflow Patterns
**Path**: `~/.openclaw/foundry/workflow-patterns.json`

```json
[
  {
    "patternId": "wp-deploy-staging",
    "goalPattern": "deploy.*staging",
    "toolSequence": ["git_status", "git_add", "run_tests", "deploy_staging"],
    "occurrences": 7,
    "successRate": 0.87,
    "avgDuration": 42000,
    "lastUsed": "2025-02-01T15:00:00Z"
  }
]
```

## Integration into mBot

### Recommended Approach

1. **Start with LearningEngine**
   - Implement `recordFailure()` and `recordSuccess()`
   - Store in `data/learnings.json`

2. **Add Hook Integration**
   - Hook into servo operations
   - Record personality behavior outcomes
   - Track drawing/sorting workflows

3. **Implement Pattern Matching**
   - Match error patterns to known fixes
   - Suggest resolutions proactively

4. **Build Simple Crystallization**
   - Start with manual crystallization
   - User approves patterns before code generation

5. **Add Overseer (Later Phase)**
   - Autonomous crystallization
   - Performance tracking
   - Auto-pruning

### mBot-Specific Learning Targets

**For Personality System**:
- Learn which personality parameters work best for different interactions
- Crystallize proven personality profiles

**For Servo Control**:
- Learn optimal servo calibration for different surfaces
- Record failure patterns (binding, overshoot)
- Crystallize recovery behaviors

**For Drawing**:
- Track successful pen paths
- Learn optimal speeds for different shapes
- Crystallize complex drawing routines

**For LEGO Sorting**:
- Record color detection edge cases
- Learn optimal sorting sequences
- Crystallize multi-step sorting workflows

## Performance Metrics

- **Pattern Recognition**: O(n) search over stored learnings
- **Crystallization**: ~2-5s per tool generation
- **Storage**: ~1KB per learning entry
- **Overhead**: Minimal - async hook handlers don't block
- **Value**: 5+ uses = break-even on token cost

## Best Practices

1. **Start Small**: Record only critical failures initially
2. **Validate Patterns**: Require 5+ uses before crystallization
3. **Test Crystallized Code**: Always sandbox before deployment
4. **Prune Regularly**: Keep learnings database < 1000 entries
5. **Version Control**: Track crystallized tools in git
6. **Monitor Performance**: Track tool fitness after crystallization

## Related Files in Foundry

- `index.ts` (lines 476-1600) - LearningEngine class
- `src/self-writer.ts` - Code generation
- `docs/PROACTIVE-LEARNING.md` - Proactive behavior details
- `docs/ARCHITECTURE.md` - Overall system architecture
