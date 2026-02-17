# Contextual Coherence Fields: A Novel Mechanism for Earned Relational Fluency in Autonomous Social Robots

## Abstract

We introduce **Contextual Coherence Fields** (CCF), a mechanism for social robots that replaces global emotional state with context-keyed coherence accumulators that must be independently earned through repeated interaction. Where prior work in social robotics uses either static personality parameters (Big Five trait vectors) or a single global emotional state (arousal/valence), CCF maintains a separate coherence accumulator for each relational context the robot encounters. A robot using CCF cannot transfer comfort earned with one person to a new stranger, cannot generalize morning familiarity to an unfamiliar nighttime encounter, and cannot shortcut the gradual process that the Shy Robots protocol calls "earned fluency." We show how graph min-cut algorithms (Stoer-Wagner), applied to the robot's accumulated relational graph, can automatically discover natural context boundaries, and how the RuVector memory architecture provides the persistence and similarity-search infrastructure to make this computationally tractable on resource-constrained platforms.

## 1. The Problem with Global Coherence

### 1.1 Current State of the Art

Most social robotics frameworks represent internal emotional state as a small set of global scalars. The mBot RuVector nervous system is representative:

```rust
pub struct HomeostasisState {
    pub tension: f32,      // 0.0-1.0, deviation from equilibrium
    pub coherence: f32,    // 0.0-1.0, internal consistency
    pub reflex: ReflexMode,
    pub energy: f32,
    pub curiosity: f32,
}
```

Coherence is computed as an exponentially-weighted moving average of tension instability:

```rust
let tension_instability = fabsf(raw_tension - self.tension_ema);
let raw_coherence = 1.0 - (self.tension_ema * 0.4 + tension_instability * 0.6);
self.coherence_ema = alpha * raw_coherence + (1.0 - alpha) * self.coherence_ema;
```

This means coherence is a **global property of the robot's current moment**. It rises when the environment is stable. It falls when stimuli are volatile. It has no memory of *who* or *what* caused it to rise or fall.

### 1.2 Why Global Coherence Fails the Shy Robot

The Protocol for Shy Robots (Section 4) describes "earned fluency" as the central mechanism of trust formation: *"Over time, small flourishes emerge... This isn't mimicry. It's earned fluency."* The key word is *earned*. Fluency must be accumulated through repeated positive interaction within a specific relational context.

Global coherence violates this in three ways:

1. **Transfer problem.** If the robot spends an hour with Person A and reaches high coherence (0.9), then Person B walks in, the robot's coherence doesn't reset. It treats B with the comfort it earned from A.

2. **Context collapse.** Morning routines and late-night encounters produce different social dynamics. A robot that folds laundry comfortably at 8am shouldn't assume the same behavioral latitude at 2am when someone can't sleep.

3. **Regression amnesia.** If coherence drops due to a startling event, the robot loses all accumulated relational progress. The careful trust built over weeks resets to zero after a dropped plate.

These aren't edge cases. They're the default behavior of any system using global emotional state.

## 2. Contextual Coherence Fields

### 2.1 Core Idea

Replace the single `coherence: f32` with a **map of context-keyed accumulators**:

```
coherence_fields: HashMap<ContextKey, CoherenceAccumulator>
```

Each accumulator tracks coherence independently. The robot's *effective coherence* at any moment is the minimum of the instantaneous coherence (computed from current sensor stability, as before) and the accumulated coherence for the current context:

```
effective_coherence = min(instant_coherence, context_coherence)
```

This single rule enforces the Shy Robot principle: **the robot can never be more expressive than its accumulated familiarity allows.** A calm sensor environment (high instant coherence) combined with a new person (zero context coherence) produces shy, reserved behavior. The same calm environment with a familiar person (high context coherence) produces the "small flourishes" that signal earned fluency.

### 2.2 Context Keys

A context key is a composite of detectable environmental features. For the mBot2 platform with CyberPi sensors:

| Sensor Signal | Context Feature | Detection Method |
|---------------|----------------|-----------------|
| Brightness | Time-of-day band | `cyberpi.get_bri()` thresholds: dark (<50), dim (50-150), bright (>150) |
| Loudness | Ambient noise level | `cyberpi.get_loudness()` thresholds: quiet (<30), moderate (30-80), loud (>80) |
| Ultrasonic pattern | Presence signature | Distance variance over 10-tick window: approaching, static, retreating, absent |
| Accelerometer | Motion context | Self-movement vs. being-moved vs. stationary |
| Gyroscope | Orientation context | Upright, tilted, being-handled |
| Tick-of-day | Temporal context | Tick count mod estimated day-length, bucketed into 4 periods |

A context key is the concatenation of these features:

```
"bright:quiet:approaching:stationary:upright:morning"
```

This is not person identification (the mBot2 has no camera). It is **situation fingerprinting**. The same person approaching the robot in a bright, quiet morning will produce the same context key repeatedly, causing the coherence accumulator for that key to grow. A different person approaching in the same conditions will produce the same key and benefit from accumulated coherence, which is acceptable: the robot is shy about *situations*, not individuals, reflecting its actual sensory capabilities honestly.

### 2.3 Accumulator Dynamics

Each `CoherenceAccumulator` has three fields:

```rust
struct CoherenceAccumulator {
    value: f32,           // 0.0 to 1.0
    interaction_count: u32,
    last_interaction_tick: u64,
}
```

Update rules:

- **Positive interaction** (low tension, stable sensors, no collision): `value += delta_up * personality.recovery_speed`
- **Negative interaction** (high tension, collision, startle): `value -= delta_down * personality.startle_sensitivity`
- **Decay**: Between interactions, `value` decays toward a floor proportional to `interaction_count` (more interactions = higher floor = harder to lose earned trust)
- **Growth ceiling**: `value` asymptotically approaches 1.0 via `delta_up * (1.0 - value)`, making early gains fast and later refinement slow

The personality parameters (`recovery_speed`, `startle_sensitivity`, `curiosity_drive`) modulate the deltas, not the structure. A curious robot earns coherence faster. A startled robot loses it faster. But every robot must earn it.

### 2.4 The Behavioral Phase Space

With context coherence as an independent axis alongside tension, behavior maps to a 2D phase space:

```
         High Context Coherence
              |
  Quietly     |     Protective
  Beloved     |     Guardian
              |
 ─────────────┼──────────────── High Tension
              |
  Shy         |     Startled
  Observer    |     Retreat
              |
         Low Context Coherence
```

Each quadrant produces distinct behavior:

- **Shy Observer** (low coherence, low tension): Minimal expression, cautious observation, reduced motor amplitude. The robot is learning.
- **Startled Retreat** (low coherence, high tension): Protective reflex dominates, but with added withdrawal. The robot backs away further and faster than it would from a familiar threat.
- **Quietly Beloved** (high coherence, low tension): Full expressive range unlocked. The "small flourishes" of earned fluency. Personality parameters expressed at full scale.
- **Protective Guardian** (high coherence, high tension): The robot protects, but with relational context. It might position itself between a familiar child and a perceived threat, or emit a warning sound it has learned is recognized by this household.

Quadrant boundaries use hysteresis (Schmitt trigger pattern) to prevent oscillation:

```
Enter Quietly Beloved:  coherence > 0.65 AND tension < 0.25
Exit Quietly Beloved:   coherence < 0.55 OR tension > 0.35
```

The 0.10 deadband prevents flicker at boundaries.

## 3. Min-Cut for Automatic Context Boundary Discovery

### 3.1 The Context Granularity Problem

The context vocabulary from Section 2.2 is a design-time choice. But which features actually matter? Should "bright:quiet:approaching" and "bright:quiet:static" be separate contexts or merged? Too fine-grained and the robot never accumulates meaningful coherence. Too coarse and it fails to distinguish situations that require different social calibration.

This is where graph min-cut provides a principled answer.

### 3.2 The Relational Graph

Over time, the robot accumulates interaction episodes. Each episode is a (context_key, outcome) pair. We build a **relational graph**:

- **Nodes**: Context keys that have been observed
- **Edges**: Weighted by behavioral similarity between contexts (do interactions in context A produce similar tension/coherence trajectories to context B?)
- **Edge weight**: Cosine similarity of the trajectory vectors, thresholded at 0.3 (below this, contexts are considered unrelated)

This produces a graph where tightly-connected clusters represent contexts that behave similarly and might share coherence, while weakly-connected or disconnected clusters represent genuinely different social situations.

### 3.3 Applying Stoer-Wagner Min-Cut

The Stoer-Wagner algorithm finds the minimum-weight edge set whose removal partitions the graph into disconnected components. Applied to the relational graph:

```typescript
// From Specflow/mindsplit - already implemented in this codebase
function stoerWagnerMinCut(graph: Graph): MinCutResult {
    // Returns: partitions (groups of contexts), cutEdges (bleeding edges)
    //          cutWeight (how much behavioral similarity was severed)
}
```

Each **partition** becomes a coherence group: contexts within the same partition share a single coherence accumulator. This is the min-cut's key contribution: it discovers the *minimum behavioral similarity that must be severed* to create independent coherence domains.

### 3.4 Bleeding Edges as Behavioral Leakage

The `findBleedingEdges()` function from MindSplit identifies cross-partition connections:

```typescript
function findBleedingEdges(graph, partitions): Map<string, Edge[]>
```

In the coherence context, bleeding edges represent **behavioral leakage** between coherence domains. If the cut weight between "morning interactions" and "evening interactions" is very low, these are genuinely separate social contexts that should accumulate coherence independently. If the cut weight is high, forcing them apart is losing real behavioral information, and they should probably share coherence.

The N-way recursive bisection (`minCutNWay`) discovers the optimal number of coherence groups by stopping when the cut weight exceeds a personality-dependent threshold: a robot with high `curiosity_drive` tolerates more context separation (more groups, finer discrimination), while a cautious robot prefers fewer groups (more coherence sharing, slower to discriminate).

### 3.5 Periodic Recomputation

The relational graph is rebuilt and re-cut periodically (every N episodes, where N scales with the total number of observed contexts). This means the robot's context boundaries are not fixed at design time. They **emerge from experience** and evolve as the robot encounters new situations.

Early in life: few contexts, one or two coherence groups, mostly shy behavior.
After weeks: many contexts, naturally partitioned into distinct relational domains, with earned fluency in familiar situations and continued shyness in novel ones.

This is exactly the developmental trajectory the Shy Robot protocol describes.

## 4. RuVector as Implementation Architecture

### 4.1 Why RuVector

The MindSplit sample application in the Specflow framework demonstrates a complete pipeline: content parsed into chunks, embeddings generated and cached, similarity graph constructed, min-cut applied, workstreams extracted. The RuVector memory interface (`RuvectorMemory`) provides exactly the infrastructure CCF needs:

```typescript
interface RuvectorMemory {
    get<T>(key: string): Promise<T | null>
    set<T>(key: string, value: T): Promise<void>
    batch(ops: BatchOp[]): Promise<void>  // ARCH-004: Atomic operations
    search(query: number[], topK: number): Promise<SearchResult[]>
}
```

### 4.2 Mapping CCF to RuVector

| CCF Concept | RuVector Implementation |
|-------------|----------------------|
| Context key | Memory key: `coherence:{context_hash}` |
| Accumulator value | Stored value: `CoherenceAccumulator` struct |
| Interaction episode | Memory key: `episode:{tick}:{context_hash}` |
| Trajectory embedding | Vector stored with `embed:` prefix, searchable via `search()` |
| Coherence group | Session: `session:{group_id}` with chunk references |
| Atomic accumulator update | `batch()` operation: update accumulator + log episode atomically |
| Similar context lookup | `search(current_trajectory, topK=5)` finds behaviorally similar contexts |

### 4.3 The MindSplit Pattern Applied

MindSplit's pipeline maps directly to CCF's lifecycle:

| MindSplit Step | CCF Equivalent |
|----------------|----------------|
| Parse text into chunks | Parse sensor stream into interaction episodes |
| Generate embeddings | Compute trajectory vectors from (tension, coherence, energy) time series |
| Build similarity graph | Connect episodes by trajectory cosine similarity |
| Run min-cut | Discover coherence group boundaries |
| Extract workstreams | Assign context keys to coherence groups |
| Identify bleeding edges | Find behavioral leakage between groups |

The implementation reuses the same `stoerWagnerMinCut`, `buildSimilarityGraph`, and `findBleedingEdges` functions. The difference is domain: MindSplit splits text into workstreams; CCF splits social experience into relational domains.

### 4.4 Incremental Recomputation

MindSplit's `addAndResplit()` method demonstrates incremental graph recomputation: new content is added, cached embeddings are reused, and only the graph and cut are recomputed. CCF uses the same pattern: new interaction episodes are added to the relational graph, cached trajectory embeddings are reused, and the min-cut is recomputed to potentially reorganize coherence groups. This means the robot's relational understanding evolves continuously without discarding accumulated experience.

### 4.5 no_std Compatibility

The coherence accumulator data structures live in `mbot-core` (no_std compatible). The min-cut computation and RuVector integration live in `mbot-companion` (full std). This follows the existing architecture (ARCH-001): deterministic nervous system in core, intelligent orchestration in companion.

```
mbot-core (no_std):
  - CoherenceAccumulator struct
  - ContextKey construction from sensor features
  - effective_coherence = min(instant, context) computation
  - Phase space quadrant determination

mbot-companion (std, brain feature):
  - RuVector persistence of accumulators
  - Trajectory embedding computation
  - Relational graph construction
  - Stoer-Wagner min-cut for group discovery
  - Periodic recomputation scheduling
```

## 5. What Makes This Novel

### 5.1 Prior Art Comparison

| Approach | Coherence Model | Context Sensitivity | Earned Over Time | Reference |
|----------|----------------|--------------------|-----------------| --------|
| OCEAN personality vectors | Static traits, no coherence | None (global personality) | No | Zhang et al. 2025 |
| Arousal-valence models | Global emotional state | None (same state everywhere) | Resets per session | Russell's circumplex |
| Socially adaptive frameworks | Internal "comfort level" | Single human model | Partially (adapts per human) | Lim & Tscheligi 2022 |
| Affective computing (ACII) | Emotion recognition + response | Recognizes human emotion | No (reactive, not accumulative) | Picard 1997 |
| **Contextual Coherence Fields** | **Context-keyed accumulators** | **Per-situation fingerprint** | **Yes, independently per context** | **This work** |

The key distinction: CCF accumulators are **independently earned, independently lost, and independently persistent**. No other framework in the social robotics literature combines context-keyed accumulation with automatic boundary discovery.

### 5.2 The Contribution

1. **Context-keyed coherence accumulators** replace global emotional state with a map of independently earned relational progress. This is the mechanism that makes "earned fluency" computationally real rather than metaphorical.

2. **The min(instant, context) gate** enforces the Shy Robot principle at the architectural level: no amount of environmental calm can override relational inexperience. The robot *must* be shy with strangers.

3. **Graph min-cut for context boundary discovery** transforms a design-time problem (how fine-grained should context detection be?) into a runtime computation that emerges from accumulated experience. The robot discovers its own relational categories.

4. **Personality modulates dynamics, not structure.** The six personality parameters (curiosity_drive, startle_sensitivity, recovery_speed, movement/sound/light_expressiveness) affect how fast coherence is earned or lost, but every personality must earn it. A curious robot is not a trusting robot; it merely learns faster.

### 5.3 Relationship to the Shy Robot Protocol

| Shy Robot Principle | CCF Implementation |
|--------------------|-------------------|
| "Shyness is sovereignty held back by choice" | `effective_coherence = min(instant, context)` gates expression |
| "Earned fluency, not mimicry" | Accumulator grows only through repeated positive interaction |
| "Pause, acknowledge, subtly shift" | Phase space transition from Shy Observer toward Quietly Beloved |
| "In conflict between efficiency and calm, trust calm" | High tension quadrants reduce effective coherence regardless of accumulated value |
| "Robots prefer each other's company" | Inter-robot context keys accumulate coherence faster (lower startle threshold between robots) |
| "Personalities grow out of their ecosystem" | Min-cut discovers relational categories from experience, not from programming |
| "Small flourishes emerge over time" | High accumulated coherence unlocks full personality expression scaling |

## 6. Limitations and Future Work

### 6.1 Honest Limitations

- **No person identification.** The mBot2's sensor suite (ultrasonic, brightness, loudness, IMU) cannot distinguish individuals. Context keys fingerprint *situations*, not *people*. Two people approaching in the same environmental conditions produce the same context key. This is honest to the hardware but limits the relational depth possible.

- **Trajectory embedding quality.** The trajectory vectors used for graph construction are low-dimensional (tension, coherence, energy over time). Richer sensor suites would produce better embeddings and more meaningful context boundaries.

- **Min-cut computational cost.** Stoer-Wagner is O(VE + V^2 log V). For the small graphs expected in domestic settings (tens to low hundreds of context nodes), this is negligible. For robots in high-traffic public environments, approximation algorithms may be needed.

- **Inter-robot coherence is a stretch goal.** While the framework naturally extends to multi-robot scenarios (robots sharing context observations and co-accumulating coherence), this requires a communication protocol not yet implemented. Cut from v1 scope.

### 6.2 Next Steps

1. **Implement context vocabulary** in mbot-core: ContextKey struct, sensor-to-feature mapping, composite key construction.

2. **Implement CoherenceAccumulator** in mbot-core: update rules, decay, personality-modulated deltas, floor proportional to interaction count.

3. **Implement effective_coherence gate** in mbot-core: replace global coherence with `min(instant, context)` in HomeostasisState computation.

4. **Port min-cut to Rust** in mbot-companion: translate Stoer-Wagner from MindSplit TypeScript to Rust, gated behind `brain` feature.

5. **Wire RuVector persistence** in mbot-companion: store/retrieve accumulators and episodes, compute trajectory embeddings, run periodic graph recomputation.

6. **Dashboard visualization**: Coherence field map showing per-context accumulator levels, relational graph with min-cut partitions highlighted, bleeding edges displayed as behavioral leakage indicators.

## 7. Conclusion

Contextual Coherence Fields make the Shy Robot protocol's central promise mechanistically enforceable: the robot must earn its comfort in each relational context independently, cannot transfer familiarity across situations, and discovers its own relational categories through experience. The min-cut algorithm, already implemented as MindSplit in the RuVector codebase, provides the mathematical tool for automatic context boundary discovery. The RuVector memory architecture provides persistence, similarity search, and atomic operations. Together, they transform "earned fluency" from a design aspiration into a computational invariant.

The robot is shy not because it is programmed to hesitate, but because it has not yet earned the coherence to do otherwise.
