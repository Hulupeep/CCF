# Self-Learning System Guide

## Overview

The mBot Self-Learning System enables the robot to observe user interactions, identify patterns, and crystallize frequent workflows into reusable behaviors. This system is based on OpenClaw Foundry's proven pattern crystallization architecture.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      OBSERVATION LAYER                           │
│   Tracks: User Messages → Bot Responses → Success/Failure       │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                    LEARNING ENGINE                               │
│                                                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  Observations   │  │ Pattern Store   │  │  Insights       │ │
│  │  (Temp)         │  │ (IndexedDB)     │  │  (Recent)       │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│                                                                  │
│  Crystallization Rules (from Foundry):                          │
│  • 5+ successful observations                                   │
│  • 70%+ success rate                                            │
│  • Cosine similarity > 0.8                                      │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                        OVERSEER                                  │
│   Autonomous Agent (runs every 1 hour)                          │
│   - Detect patterns                                             │
│   - Crystallize proven workflows                                │
│   - Prune stale patterns                                        │
│   - Generate insights                                           │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                    PATTERN EXECUTION                             │
│   Crystallized patterns execute automatically                   │
│   Zero token cost after crystallization                         │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. LearningEngine

The brain of the learning system.

**Key Methods:**

```typescript
// Observe user actions
observeAction(
  userId: string,
  action: string,
  context: Record<string, any>,
  success: boolean,
  duration?: number
): string

// Track message interactions
trackInteraction(
  userId: string,
  message: string,
  response: string,
  satisfaction: boolean
): string

// Detect patterns from observations
detectPatterns(): CrystallizedPattern[]

// Find matching pattern for context
findMatchingPattern(context: Record<string, any>): CrystallizedPattern | null

// Execute a crystallized pattern
executePattern(
  pattern: CrystallizedPattern,
  context: Record<string, any>
): Promise<PatternExecutionResult>

// Update pattern confidence
updatePatternConfidence(patternId: string, success: boolean): void
```

**Example Usage:**

```typescript
import { LearningEngine } from './services/learning/LearningEngine';

const engine = new LearningEngine();

// Observe an interaction
engine.observeAction(
  'user123',
  'weather_query',
  { message: 'What is the weather?' },
  true,
  120
);

// After 5+ similar observations, detect patterns
const patterns = engine.detectPatterns();

// Add pattern to engine
if (patterns.length > 0) {
  engine.addPattern(patterns[0]);
}

// Later, find and execute matching pattern
const match = engine.findMatchingPattern({ message: 'weather today' });
if (match) {
  const result = await engine.executePattern(match, { message: 'weather' });
  console.log('Pattern executed:', result.success);
}
```

### 2. PatternStore

IndexedDB persistence for patterns.

**Key Methods:**

```typescript
// Save pattern
savePattern(pattern: CrystallizedPattern): Promise<void>

// Load all patterns
loadPatterns(): Promise<CrystallizedPattern[]>

// Update pattern
updatePattern(id: string, updates: Partial<CrystallizedPattern>): Promise<void>

// Get statistics
getPatternStats(): Promise<PatternStatistics>

// Export/Import
exportPatterns(): Promise<string>
importPatterns(json: string): Promise<number>
```

**Example Usage:**

```typescript
import { PatternStore } from './services/learning/PatternStore';

const store = new PatternStore();

// Save a pattern
await store.savePattern(pattern);

// Load all patterns
const patterns = await store.loadPatterns();

// Get statistics
const stats = await store.getPatternStats();
console.log(`Total patterns: ${stats.totalPatterns}`);
console.log(`Avg success rate: ${stats.avgSuccessRate}`);
```

### 3. Overseer

Autonomous learning agent that runs periodic cycles.

**Key Methods:**

```typescript
// Start the overseer
start(): void

// Stop the overseer
stop(): void

// Run a single learning cycle manually
runLearningCycle(): Promise<LearningReport>

// Get all reports
getReports(): LearningReport[]

// Get status
getStatus(): OverseerStatus
```

**Configuration:**

```typescript
const overseer = new Overseer(engine, store, {
  intervalMs: 3600000, // 1 hour
  enableAutoCleanup: true,
  enableAutoCrystallization: true,
  requireApproval: false,
  minConfidenceForAuto: 0.85,
});

overseer.start();
```

**Learning Cycle Process:**

1. Analyze recent observations
2. Detect pattern candidates
3. Evaluate existing patterns
4. Crystallize high-confidence patterns (≥85%)
5. Prune stale patterns (30+ days unused)
6. Generate insights
7. Create recommendations

## The 5/70 Crystallization Rule

Based on OpenClaw Foundry research, patterns crystallize when:

1. **5+ Observations**: At least 5 successful uses of the same workflow
2. **70%+ Success Rate**: At least 70% of observations succeeded
3. **Context Similarity**: Similar contexts (cosine similarity > 0.8)

**Why these thresholds?**

- **5 observations**: Enough data to establish a pattern, not too high to miss useful workflows
- **70% success**: High confidence without being overly strict
- **0.8 similarity**: Balances pattern matching precision with recall

## Integration with Telegram Bot

The TelegramBot automatically observes all interactions:

```typescript
import { TelegramBot } from './services/telegram/TelegramBot';
import { LearningEngine } from './services/learning/LearningEngine';

const engine = new LearningEngine();
const bot = new TelegramBot(config, engine);

// Bot automatically:
// 1. Checks for matching patterns before generating responses
// 2. Observes all interactions (success/failure, duration)
// 3. Updates pattern confidence based on outcomes
```

**How it works:**

```typescript
// Inside handleMessage method:
const pattern = learningEngine.findMatchingPattern({ message, userId });

if (pattern) {
  // Use crystallized pattern (zero token cost!)
  const result = await learningEngine.executePattern(pattern, context);

  if (result.success) {
    // Pattern worked, update confidence positively
    learningEngine.updatePatternConfidence(pattern.id, true);
  } else {
    // Pattern failed, update confidence negatively
    learningEngine.updatePatternConfidence(pattern.id, false);
  }
} else {
  // No pattern found, generate new response
  response = await generateResponse(message);
}

// Always observe the interaction
learningEngine.observeAction(userId, 'message_response', context, success, duration);
```

## Learning Dashboard

React component to visualize learned patterns.

**Features:**

- Overseer status (running/stopped, last cycle)
- Performance metrics (total patterns, avg success rate, tokens saved)
- Active patterns list with expandable details
- Learning insights feed
- Latest learning cycle report

**Usage:**

```tsx
import { LearningDashboard } from './components/learning/LearningDashboard';

<LearningDashboard
  learningEngine={engine}
  patternStore={store}
  overseer={overseer}
/>
```

**Pattern Card:**

Shows individual pattern with:
- Name and description
- Success rate badge
- Usage count
- Confidence level
- Avg duration
- Improvement trajectory chart

## Pattern Data Structure

```typescript
interface CrystallizedPattern {
  id: string;
  name: string;
  description: string;

  trigger: {
    type: 'keyword' | 'action' | 'context' | 'composite';
    pattern: string | RegExp;
    keywords?: string[];
    minSimilarity?: number;
  };

  action: {
    type: 'response' | 'workflow' | 'tool_sequence' | 'custom';
    description: string;
    steps?: string[];
    responseTemplate?: string;
  };

  // Performance metrics
  confidence: number;        // 0-1 success rate
  usageCount: number;
  successRate: number;
  avgDuration: number;

  // Timestamps
  createdAt: number;
  lastUsed: number;
  crystallizedAt: number;

  // Evolution tracking
  version: number;
  improvementTrajectory?: number[]; // Historical success rates
}
```

## Performance Benefits

### Token Savings

Crystallized patterns execute **without LLM calls**, saving tokens:

```
Before crystallization:
  User: "What's the weather?"
  → LLM call (1000 tokens) → Response
  Cost: ~$0.003 per message

After crystallization (5+ uses):
  User: "What's the weather?"
  → Pattern match → Execute → Response
  Cost: $0.000 (zero tokens!)
```

**Estimated savings:**
- After 5 uses: Break-even point
- After 10 uses: 50% token reduction for that workflow
- After 100 uses: Significant cost savings

### Speed Improvement

Pattern execution is 2-10x faster than LLM generation:

```
LLM Response: 1-3 seconds
Pattern Execution: 50-300ms
```

## Troubleshooting

### Patterns Not Crystallizing

**Problem**: Observations made but no patterns detected.

**Solutions:**

1. Check observation count: Need 5+ similar observations
   ```typescript
   const stats = engine.getPerformanceStats();
   console.log(`Total observations: ${stats.totalObservations}`);
   ```

2. Check success rate: Need 70%+ success
   ```typescript
   const patterns = engine.detectPatterns();
   patterns.forEach(p => {
     console.log(`${p.name}: ${p.successRate * 100}% success`);
   });
   ```

3. Check context similarity: Contexts must be similar
   - Ensure keywords match
   - Review context vector generation

### Patterns Not Matching

**Problem**: Pattern exists but not matching new contexts.

**Solutions:**

1. Lower similarity threshold:
   ```typescript
   const engine = new LearningEngine({
     minSimilarity: 0.7, // Lower from 0.8
   });
   ```

2. Check pattern keywords:
   ```typescript
   const pattern = engine.getPattern(patternId);
   console.log('Pattern keywords:', pattern.trigger.keywords);
   ```

3. Test similarity calculation:
   ```typescript
   const pattern = engine.findMatchingPattern(context);
   console.log('Match found:', pattern ? 'Yes' : 'No');
   ```

### Overseer Not Running

**Problem**: Overseer started but not running cycles.

**Solutions:**

1. Check overseer status:
   ```typescript
   const status = overseer.getStatus();
   console.log('Running:', status.running);
   console.log('Total cycles:', status.totalCycles);
   ```

2. Check interval configuration:
   ```typescript
   const overseer = new Overseer(engine, store, {
     intervalMs: 60000, // 1 minute for testing
   });
   ```

3. Run manual cycle:
   ```typescript
   const report = await overseer.runLearningCycle();
   console.log('Cycle complete:', report);
   ```

## Best Practices

### 1. Start with Manual Crystallization

Don't enable auto-crystallization immediately. Review patterns first:

```typescript
const overseer = new Overseer(engine, store, {
  enableAutoCrystallization: false, // Disable auto
  requireApproval: true, // Require approval
});

// Review candidates manually
const report = await overseer.runLearningCycle();
console.log('Candidates:', report.crystallizationCandidates);

// Approve specific patterns
for (const candidate of report.crystallizationCandidates) {
  if (candidate.confidence > 0.9) {
    engine.addPattern(candidate);
    await store.savePattern(candidate);
  }
}
```

### 2. Monitor Pattern Performance

Track how patterns perform over time:

```typescript
// Get pattern stats
const stats = await store.getPatternStats();
console.log('Top patterns:', stats.topPatterns);

// Check individual pattern
const pattern = engine.getPattern(patternId);
console.log('Trajectory:', pattern.improvementTrajectory);
```

### 3. Prune Regularly

Remove stale patterns to keep the system lean:

```typescript
// Prune patterns unused for 30 days
const pruned = engine.pruneStalePatterns(30);
console.log(`Pruned ${pruned.length} patterns`);

// Or configure overseer to prune automatically
const overseer = new Overseer(engine, store, {
  enableAutoCleanup: true,
});
```

### 4. Export/Import Patterns

Backup and share learned patterns:

```typescript
// Export
const json = await store.exportPatterns();
localStorage.setItem('mbot-patterns-backup', json);

// Import
const json = localStorage.getItem('mbot-patterns-backup');
const imported = await store.importPatterns(json);
console.log(`Imported ${imported} patterns`);
```

## Research Foundations

This implementation is based on several key research papers:

### 1. Self-Improving Code Agents (Robeyns et al., 2025)
**arXiv:2504.15228**

> "An agent system, equipped with basic coding tools, can autonomously edit itself, and thereby improve its performance"

**Applied:** Patterns crystallize into executable code (though not self-modifying in our implementation).

### 2. RISE: Recursive Introspection (Qu et al., 2024)
**arXiv:2407.18219**

> "Iterative fine-tuning teaches models to alter responses after unsuccessful attempts"

**Applied:** Pattern confidence updates based on execution outcomes.

### 3. HexMachina (Liu et al., 2025)
**arXiv:2506.04651**

> "Artifact-centric continual learning — separates discovery from strategy evolution through code refinement"

**Applied:** Patterns (knowledge) crystallize into executable behaviors that persist.

### 4. ADAS: Automated Design of Agentic Systems (Hu et al., 2024)
**arXiv:2408.08435**

> "Meta-agent iteratively discovers improved agent designs through archive-based evolution"

**Applied:** Overseer tracks pattern fitness and evolves patterns over time.

## API Reference

### LearningEngine

```typescript
class LearningEngine {
  constructor(config?: Partial<LearningEngineConfig>)

  // Observation
  observeAction(userId: string, action: string, context: any, success: boolean, duration?: number): string
  trackInteraction(userId: string, message: string, response: string, satisfaction: boolean): string

  // Pattern Detection
  detectPatterns(): CrystallizedPattern[]
  shouldCrystallize(observations: WorkflowObservation[]): boolean
  crystallizePattern(observations: WorkflowObservation[]): CrystallizedPattern

  // Pattern Matching & Execution
  findMatchingPattern(context: any): CrystallizedPattern | null
  executePattern(pattern: CrystallizedPattern, context: any): Promise<PatternExecutionResult>

  // Pattern Management
  updatePatternConfidence(patternId: string, success: boolean): void
  pruneStalePatterns(thresholdDays?: number): string[]
  getPatterns(): CrystallizedPattern[]
  getPattern(id: string): CrystallizedPattern | undefined
  addPattern(pattern: CrystallizedPattern): void

  // Analytics
  getInsights(): LearningInsight[]
  getPerformanceStats(): PerformanceStats
}
```

### PatternStore

```typescript
class PatternStore {
  constructor()

  savePattern(pattern: CrystallizedPattern): Promise<void>
  loadPatterns(): Promise<CrystallizedPattern[]>
  updatePattern(id: string, updates: Partial<CrystallizedPattern>): Promise<void>
  deletePattern(id: string): Promise<void>

  getPatternsByUser(userId: string): Promise<CrystallizedPattern[]>
  getTopPatterns(limit: number): Promise<CrystallizedPattern[]>
  getPatternStats(): Promise<PatternStatistics>

  clearAll(): Promise<void>
  exportPatterns(): Promise<string>
  importPatterns(json: string): Promise<number>
}
```

### Overseer

```typescript
class Overseer {
  constructor(
    learningEngine: LearningEngine,
    patternStore: PatternStore,
    config?: Partial<OverseerConfig>
  )

  start(): void
  stop(): void

  runLearningCycle(): Promise<LearningReport>

  getReports(): LearningReport[]
  getLatestReport(): LearningReport | null
  getStatus(): OverseerStatus
}
```

## Future Enhancements

### 1. Multi-User Pattern Sharing

Allow patterns to be shared across users:

```typescript
interface SharedPattern extends CrystallizedPattern {
  sharedBy: string;
  usageByUser: Map<string, number>;
  globalSuccessRate: number;
}
```

### 2. Pattern Versioning

Track pattern evolution over time:

```typescript
interface PatternVersion {
  version: number;
  changes: string;
  timestamp: number;
  parentVersion?: number;
}
```

### 3. A/B Testing

Test pattern variants to find optimal implementations:

```typescript
interface PatternVariant {
  variantId: string;
  basePatternId: string;
  changes: Partial<CrystallizedPattern>;
  testGroup: 'A' | 'B';
  performance: PerformanceMetrics;
}
```

### 4. LLM Integration

Use LLM to generate pattern descriptions and improve matching:

```typescript
async function enhancePatternWithLLM(pattern: CrystallizedPattern): Promise<void> {
  const description = await llm.generate(
    `Describe this workflow pattern: ${JSON.stringify(pattern.trigger)}`
  );
  pattern.description = description;
}
```

## Conclusion

The Self-Learning System enables mBot to continuously improve from user interactions. By crystallizing proven patterns, the system becomes faster, cheaper, and more personalized over time - all based on solid AI research foundations from OpenClaw Foundry.

**Key Takeaways:**

- 5+ observations + 70% success rate = crystallization
- Patterns execute with zero token cost
- Overseer runs autonomous learning cycles
- Dashboard provides visibility into learned behaviors
- Based on proven AI research (4 papers cited)

Start small, monitor carefully, and watch your mBot learn and evolve!
