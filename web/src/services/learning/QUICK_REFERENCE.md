# Learning Engine - Quick Reference Card

## 30-Second Overview

```typescript
import { LearningEngine, PatternStore, Overseer } from './services/learning';

// Setup (once)
const engine = new LearningEngine();
const store = new PatternStore();
const overseer = new Overseer(engine, store);
overseer.start();

// Use (automatic in TelegramBot)
engine.observeAction('user1', 'action', context, true, 100);
const pattern = engine.findMatchingPattern(context);
if (pattern) await engine.executePattern(pattern, context);
```

## The 5/70 Rule

```
5+ observations + 70%+ success rate + 0.8 similarity = ✨ Crystallization
```

## Core API

### LearningEngine

```typescript
// Observe
observeAction(userId, action, context, success, duration): string
trackInteraction(userId, message, response, satisfied): string

// Detect
detectPatterns(): CrystallizedPattern[]

// Execute
findMatchingPattern(context): CrystallizedPattern | null
executePattern(pattern, context): Promise<PatternExecutionResult>

// Update
updatePatternConfidence(patternId, success): void
pruneStalePatterns(days): string[]
```

### PatternStore

```typescript
savePattern(pattern): Promise<void>
loadPatterns(): Promise<CrystallizedPattern[]>
getPatternStats(): Promise<PatternStatistics>
exportPatterns(): Promise<string>
```

### Overseer

```typescript
start(): void
stop(): void
runLearningCycle(): Promise<LearningReport>
getStatus(): OverseerStatus
```

## Pattern Structure

```typescript
{
  id: string,
  name: string,
  trigger: { type, pattern, keywords, minSimilarity },
  action: { type, description, steps },
  confidence: number,       // 0-1
  usageCount: number,
  successRate: number,      // 0-1
  avgDuration: number,      // ms
  createdAt: number,
  lastUsed: number,
}
```

## Configuration

```typescript
// LearningEngine
new LearningEngine({
  minObservations: 5,      // Crystallization threshold
  minSuccessRate: 0.7,     // 70% success required
  minSimilarity: 0.8,      // Context matching
  maxPatterns: 1000,
  staleThresholdDays: 30,
});

// Overseer
new Overseer(engine, store, {
  intervalMs: 3600000,              // 1 hour
  enableAutoCrystallization: true,
  requireApproval: false,
  minConfidenceForAuto: 0.85,
});
```

## Common Workflows

### 1. Basic Usage

```typescript
// Observe interactions
engine.observeAction('user1', 'weather', { msg: 'weather?' }, true, 100);

// After 5+ observations, detect patterns
const patterns = engine.detectPatterns();

// Add to engine
patterns.forEach(p => engine.addPattern(p));

// Use patterns
const match = engine.findMatchingPattern({ msg: 'weather today' });
if (match) await engine.executePattern(match, { msg: 'weather' });
```

### 2. With Persistence

```typescript
// Load existing patterns
const patterns = await store.loadPatterns();
patterns.forEach(p => engine.addPattern(p));

// Save new patterns
const newPatterns = engine.detectPatterns();
for (const p of newPatterns) {
  await store.savePattern(p);
}
```

### 3. With Overseer

```typescript
// Start overseer
overseer.start();

// It automatically:
// - Detects patterns every hour
// - Crystallizes high-confidence patterns
// - Prunes stale patterns
// - Generates insights

// Get latest report
const report = overseer.getLatestReport();
console.log(`Crystallized: ${report.actionsTriggered.crystallized.length}`);
```

### 4. Dashboard Integration

```tsx
import { LearningDashboard } from './components/learning';

<LearningDashboard
  learningEngine={engine}
  patternStore={store}
  overseer={overseer}
/>
```

## Performance Metrics

| Metric | Formula | Target |
|--------|---------|--------|
| Success Rate | successful / total | >70% |
| Confidence | EMA of success rate | >0.7 |
| Token Savings | patterns × avg_tokens | Maximize |
| Active Patterns | used in last 30 days | Track |

## Troubleshooting

| Issue | Solution |
|-------|----------|
| No patterns crystallizing | Need 5+ observations with 70%+ success |
| Patterns not matching | Lower minSimilarity (0.7 instead of 0.8) |
| Overseer not running | Check `overseer.getStatus().running` |
| Storage full | Run `engine.pruneStalePatterns(30)` |

## Testing

```typescript
// Import test utilities
import { describe, it, expect } from '@jest/globals';

// Test observation
const obsId = engine.observeAction('u1', 'test', {}, true, 100);
expect(obsId).toBeTruthy();

// Test pattern detection (need 5+ observations)
for (let i = 0; i < 6; i++) {
  engine.observeAction('u1', 'test', { i }, true, 100);
}
const patterns = engine.detectPatterns();
expect(patterns.length).toBeGreaterThan(0);

// Test execution
const pattern = patterns[0];
const result = await engine.executePattern(pattern, {});
expect(result.success).toBe(true);
```

## CLI Commands

```bash
# Run tests
npm test -- learning-engine.test.ts

# Watch mode
npm test -- learning-engine.test.ts --watch

# Coverage
npm test -- learning-engine.test.ts --coverage
```

## Key Files

```
web/src/services/learning/
├── LearningEngine.ts     # Core logic
├── PatternStore.ts       # Persistence
├── Overseer.ts           # Autonomous agent
├── example.ts            # Usage examples
└── QUICK_REFERENCE.md    # This file

web/src/components/learning/
└── LearningDashboard.tsx # React UI

tests/integration/
└── learning-engine.test.ts # Tests

docs/guides/
└── self-learning-guide.md  # Full docs
```

## Best Practices

1. ✅ **Start overseer immediately** - Let it run in background
2. ✅ **Observe all interactions** - More data = better patterns
3. ✅ **Monitor pattern stats** - Track success rates
4. ✅ **Prune regularly** - Keep system lean
5. ✅ **Export patterns** - Backup important learnings

## Common Mistakes

1. ❌ Too few observations (< 5) - Won't crystallize
2. ❌ Low success rate (< 70%) - Won't crystallize
3. ❌ Not starting overseer - No auto-learning
4. ❌ Not persisting patterns - Lost on refresh
5. ❌ Not updating confidence - Stale patterns

## Performance Tips

```typescript
// ✅ Good: Batch observations
for (const action of actions) {
  engine.observeAction(userId, action.type, action.context, action.success);
}

// ✅ Good: Persist after crystallization
const patterns = engine.detectPatterns();
await Promise.all(patterns.map(p => store.savePattern(p)));

// ✅ Good: Use pattern matching first
const pattern = engine.findMatchingPattern(context);
if (pattern) {
  // Fast execution (50-300ms)
  await engine.executePattern(pattern, context);
} else {
  // Slow LLM call (1-3s)
  await generateWithLLM(context);
}
```

## Research Citations

1. **Self-Improving Code Agents** - arXiv:2504.15228
2. **RISE** - arXiv:2407.18219
3. **HexMachina** - arXiv:2506.04651
4. **ADAS** - arXiv:2408.08435

## Need Help?

- 📖 Full docs: `docs/guides/self-learning-guide.md`
- 🧪 Examples: `web/src/services/learning/example.ts`
- 🧪 Tests: `tests/integration/learning-engine.test.ts`
- 📊 Dashboard: `web/src/components/learning/LearningDashboard.tsx`

---

**Version**: 1.0.0
**Last Updated**: 2026-02-01
**Story**: #93
