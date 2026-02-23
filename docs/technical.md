# Contextual Coherence Fields — Technical Reference

**Repository:** `github.com/Hulupeep/CCF`
**Created by:** Colm Byrne, Flout Labs (Flout Ltd), Cave, Clarinbridge, Co. Galway, Ireland
**Patent:** US Provisional Patent Application — *Contextual Coherence Fields with Dual-Process Cognitive Architecture and Manifold-Constrained Coherence Mixing*
**Language:** Rust (workspace of two crates)

---

## Overview

Contextual Coherence Fields (CCF) is a computational architecture for relational trust in situated robots. It replaces the traditional approach of a single global `coherence: f32` scalar — which represents how settled a robot is in its world — with a **map of independently earned per-context coherence values**. The key insight is that a robot which is calm in a quiet, familiar room is not automatically calm in a new, bright, noisy environment. Trust must be earned separately in each situation.

The system implements a **dual-process cognitive architecture** — a concept with precedent in cognitive science — where two processing layers operate in tandem:

- **Reflexive layer** (`mbot-core`): Deterministic, `no_std`, runs in microseconds. Handles all moment-to-moment behaviour: homeostasis, startle response, motor control.
- **Deliberative layer** (`mbot-companion`): Async, `std`, runs on a laptop or companion computer. Handles episode memory, graph construction, habit compilation, and suppression learning.

The deliberative layer sends learned structures back to the reflexive layer via **DownwardMessages**, progressively reshaping how the robot reacts to its world without ever breaking the real-time determinism of the reflex loop.

---

## Repository Structure

```
CCF/
├── crates/
│   ├── mbot-core/               # no_std reflexive layer
│   │   └── src/
│   │       ├── lib.rs           # HomeostasisState, MBotBrain, MBotSensors, MotorCommand
│   │       ├── coherence/
│   │       │   └── mod.rs       # All CCF data structures (ContextKey, CoherenceField, SocialPhase)
│   │       └── nervous_system/  # Stimulus detection, startle processing, suppression maps
│   └── mbot-companion/          # std deliberative layer
│       └── src/
│           ├── main.rs          # Main control loop — wires CCF into every tick
│           ├── protocol.rs      # f3/f4 binary framing (Makeblock HalocodeProtocol)
│           ├── transport.rs     # BLE / Serial / Simulated byte pipes
│           └── brain/           # Feature-gated deliberative subsystems
│               ├── episodes.rs          # Episode recording and TrajectoryVector
│               ├── relational_graph.rs  # Graph construction from episode similarity
│               ├── mincut.rs            # Stoer-Wagner min-cut algorithm
│               ├── group_manager.rs     # Context group identity from partitions
│               ├── recomputation.rs     # Scheduler for graph recomputation
│               ├── consolidation.rs     # Offline idle-time review cycles
│               ├── habits.rs            # Habit detection and compilation
│               ├── suppression_learner.rs  # Context-specific startle suppression
│               ├── suppression_sync.rs  # Syncs learned rules to reflexive layer
│               ├── coherence_persist.rs # Persistence of CoherenceField to disk
│               └── downward.rs          # DownwardMessage channel definition
├── docs/
│   ├── patent.md        # Full provisional patent application text
│   ├── demo-plan.md     # Video demo script with patent paragraph references
│   ├── ccf-evolution.md # Design history narrative
│   └── technical.md     # This document
└── tools/
    └── cyberpi_halocode_proto.py  # Standalone protocol diagnostic tool
```

---

## Reflexive Layer — `crates/mbot-core`

### `coherence/mod.rs` — CCF Data Structures

This is the core of the system. All types are `no_std` compatible (using `hashbrown::HashMap` instead of `std::collections::HashMap`).

#### Context Vocabulary

Six discrete sensor features form the context vocabulary. Each enum has 3 values, giving a vocabulary of 3⁶ = 729 possible contexts (of which typically 10–30 are observed in practice).

| Enum | Source Sensor | Values |
|------|---------------|--------|
| `BrightnessBand` | `MBotSensors.light_level` (0–1) | `Dark` (<0.15), `Dim` (0.15–0.5), `Bright` (>0.5) |
| `NoiseBand` | `MBotSensors.sound_level` (0–1) | `Quiet` (<0.15), `Moderate` (0.15–0.5), `Loud` (>0.5) |
| `PresenceSignature` | Ultrasonic trend via `PresenceDetector` | `Absent`, `Static`, `Approaching`, `Retreating` |
| `MotionContext` | Accelerometer magnitude + motor command | `Stationary`, `SelfMoving`, `BeingHandled` |
| `Orientation` | Roll/pitch from IMU | `Upright`, `Tilted` |
| `TimePeriod` | Wall clock hour | `Morning`, `Afternoon`, `Evening`, `Night` |

**Patent reference:** [0035a]–[0035c] — *"The context detection subsystem classifies the robot's current situational context by analyzing multiple streams of sensor data."*

#### `ContextKey`

```rust
pub struct ContextKey {
    pub brightness: BrightnessBand,
    pub noise: NoiseBand,
    pub presence: PresenceSignature,
    pub motion: MotionContext,
    pub orientation: Orientation,
    pub time_period: TimePeriod,
}
```

Implements `Hash + Eq`, making it usable as a `HashMap` key. Constructed via `ContextKey::from_sensors(light, sound, presence, accel_mag, motors_active, roll, pitch)`.

**Patent reference:** [0035] — *"Each unique combination of contextual features defines a distinct relational context."*

#### `PresenceDetector`

```rust
pub struct PresenceDetector { /* 10-sample ring buffer of ultrasonic readings */ }
impl PresenceDetector {
    pub fn update(&mut self, distance_cm: f32) -> PresenceSignature
}
```

Maintains a 10-sample sliding window of ultrasonic readings. Classifies trend as `Absent` (no reading), `Approaching` (decreasing distance), `Retreating` (increasing), or `Static`.

#### `CoherenceAccumulator`

```rust
pub struct CoherenceAccumulator {
    pub value: f32,              // Current coherence [0.0, 1.0]
    pub interaction_count: u32,  // Total interactions in this context
    pub last_interaction_tick: u64,
}
```

Tracks earned trust for a single context. Key methods:

- **`positive_interaction(recovery_speed, tick, alone: bool)`** — asymptotic growth toward 1.0; the `alone` flag doubles the delta when no one is present (bootstrapping from solitude is natural).
- **`negative_interaction(startle_sensitivity, tick)`** — reduces value but is **floored** by `earned_floor()`.
- **`earned_floor() -> f32`** — `0.5 * (1.0 - 1.0 / (1.0 + count/20.0))` — asymptotes to 0.5 as interaction count grows. A long-familiar context can never lose more than half its trust to a single bad event.
- **`decay(elapsed_ticks, personality_decay_rate)`** — gentle exponential decay during absence.

**Patent reference:** [0038] — *"The coherence accumulator maintains a per-context coherence value that increases through positive interactions and decreases through negative interactions."*
**Patent reference:** [0042] — *"The system applies the concept of earned floor..."*

#### `CoherenceField`

```rust
pub struct CoherenceField {
    contexts: HashMap<ContextKey, CoherenceAccumulator>,
    personality_baseline: f32,    // 0.15 * curiosity_drive for cold-start
    fallback_coherence: Option<f32>,  // degraded mode floor
}
```

The full map of context → accumulator. Key aspects:

- **MAX_CONTEXTS = 64** — LRU eviction removes least-recently-used entry when full.
- **`get_or_create(key)`** — returns existing accumulator or creates one initialized from `personality_baseline`.
- **`effective_coherence(instant, key) -> f32`** — applies the asymmetric gate (CCF-001):
  - If context coherence < 0.3 (unfamiliar): `min(instant, ctx)` — strict; earn trust first.
  - If context coherence ≥ 0.3 (familiar): `0.3 * instant + 0.7 * ctx` — buffered; history dampens noise.
  - If key unseen and fallback set: returns `max(0.0, fallback_coherence)` as degraded-mode floor.
- **`decay_all(elapsed_ticks)`** — called every 100 ticks to age all accumulators.
- **`set_fallback(val: Option<f32>)`** — sets degraded mode floor from global instant coherence.
- **`context_count() -> usize`** — number of distinct contexts seen.

**Patent reference:** [0039]–[0040] — *"The minimum function gate... is, to the inventor's knowledge, without precedent in prior art."*
**Patent reference:** [0047] — asymmetric trust-resilience for familiar contexts.

#### `SocialPhase`

```rust
pub enum SocialPhase {
    ShyObserver,        // Low coherence, low tension — watching, uncertain
    StartledRetreat,    // Low coherence, high tension — overwhelmed, withdrawing
    QuietlyBeloved,     // High coherence, low tension — calm, engaged, trusting
    ProtectiveGuardian, // High coherence, high tension — alert but stable, protective
}
```

Determined by a 2D phase space of `(effective_coherence × tension)`. Schmitt trigger hysteresis prevents rapid flickering at boundaries:

| Transition | Coherence threshold | Tension threshold |
|------------|--------------------|--------------------|
| Enter QuietlyBeloved / ProtectiveGuardian | ≥ 0.65 | — |
| Exit → ShyObserver / StartledRetreat | < 0.55 | — |
| Enter StartledRetreat / ProtectiveGuardian | ≥ 0.45 | — |
| Exit → calmer phase | < 0.35 | — |

- **`SocialPhase::classify(eff_coherence, tension, current) -> SocialPhase`** — stateful transition with hysteresis.
- **`SocialPhase::led_tint(&self) -> [u8; 3]`** — RGB colour for the mBot2 LEDs (blue-grey / dark red / warm blue / amber).
- **`SocialPhase::expression_scale(&self) -> f32`** — amplitude multiplier for motion and sound (0.2 for ShyObserver → 1.0 for QuietlyBeloved).

**Patent reference:** [0046], [0049] — *"The behavioral gating subsystem uses the effective coherence value to modulate the robot's behavioral outputs."*
**Patent reference:** [0051] — social phase quadrant classification.

---

## Main Control Loop — `crates/mbot-companion/src/main.rs`

The control loop runs at 20 Hz (configurable via `--freq`). Each tick:

1. **Read sensors** from CyberPi (BLE or Serial, using f3/f4 protocol).
2. **`MBotBrain.tick(sensors, personality) → (HomeostasisState, MotorCommand)`** — deterministic reflexive brain. Computes `instant_coherence`, tension, energy, and base motor command. This step is unchanged; CCF is applied *after*.
3. **Update `PresenceDetector`** with ultrasonic reading → `PresenceSignature`.
4. **Construct `ContextKey`** from current sensor bands.
5. **Update accumulator** for the current context: positive if tension < 0.5, negative if tension > 0.7.
6. **`CoherenceField.set_fallback(instant_coherence)`** — ensures degraded-mode floor is current.
7. **`CoherenceField.effective_coherence(instant, key)`** — applies asymmetric gate → `eff_coherence`.
8. **`SocialPhase::classify(eff_coherence, tension, prev)`** — transition with hysteresis.
9. **Override `state.coherence`** with `eff_coherence` for all downstream consumers (voice API, dashboard, brain layer).
10. **Write `state.social_phase`** and `state.context_coherence` for broadcast.
11. **Decay all contexts** every 100 ticks.
12. **Send motor command** to robot.

```
Sensors → MBotBrain.tick() → instant_coherence
                                    ↓
                             ContextKey
                                    ↓
                      CoherenceField.effective_coherence()
                                    ↓
                         SocialPhase.classify()
                                    ↓
                    LED tint + expression_scale + motor amplitude
```

**Patent reference:** [0032]–[0034] — dual-process architecture; [0078b] — emergent shyness without shyness programming.

---

## Deliberative Layer — `crates/mbot-companion/src/brain/`

Feature-gated under `#[cfg(feature = "brain")]`. All deliberative work is async and runs alongside the main loop without blocking the 20 Hz reflex cycle.

### `episodes.rs` — Episode Recording

```rust
pub struct InteractionEpisode {
    pub context_hash: u64,        // Hash of ContextKey
    pub start_tick: u64,
    pub end_tick: u64,
    pub outcome: EpisodeOutcome,  // Positive / Negative / Neutral
    pub trajectory: TrajectoryVector,
}

pub type TrajectoryVector = [f32; 12];
// Layout: [tension_mean, tension_variance, tension_trend, tension_peak,
//          coherence_mean, coherence_variance, coherence_trend, coherence_peak,
//          energy_mean, energy_variance, energy_trend, energy_peak]
```

`TrajectoryVector` is computed via `ChannelStats` — an online Welford accumulator that tracks mean, variance, trend slope, and peak across all ticks within an episode. This produces a 12-dimensional fingerprint of how the interaction *felt* over time, not just its endpoint.

**Patent reference:** [0053] — trajectory vector computation.

### `relational_graph.rs` — Graph Construction

```rust
pub struct RelationalGraph {
    nodes: Vec<(u64, TrajectoryVector)>,  // (context_hash, trajectory)
    edges: Vec<(usize, usize, f32)>,       // (i, j, cosine_similarity)
}
impl RelationalGraph {
    pub fn build(episodes: &[InteractionEpisode], min_episodes: usize, similarity_threshold: f32) -> Self
}
```

Nodes are contexts with at least `min_episodes` observations. Edges are drawn between contexts whose `TrajectoryVector`s have cosine similarity above `similarity_threshold`. The resulting graph represents how **relationally similar** different contexts feel — contexts with similar emotional fingerprints cluster together.

**Patent reference:** [0054]–[0056] — relational graph construction from trajectory similarity.

### `mincut.rs` — Stoer-Wagner Min-Cut

```rust
pub struct SWGraph { /* adjacency with f32 weights */ }
pub struct MinCutResult {
    pub cut_weight: f32,
    pub partition_a: Vec<usize>,
    pub partition_b: Vec<usize>,
}
impl SWGraph {
    pub fn stoer_wagner(&self) -> MinCutResult
    pub fn nway_bisection(graph, max_cut_weight) -> Vec<Partition>
}
```

The Stoer-Wagner algorithm (O(VE + V² log V)) finds the **minimum weight edge cut** that separates the relational graph into two partitions. `nway_bisection` applies this recursively until no cut below `max_cut_weight` exists, yielding N coherence groups.

The minimum cut naturally finds the **weakest relational boundary** in the context graph — the cut that requires severing the fewest, most dissimilar connections. This reveals which contexts genuinely belong together in the robot's experience, without any pre-defined clustering criterion.

**Patent reference:** [0057]–[0062] — min-cut boundary discovery; emergent mixing matrix structure [0078c].

### `group_manager.rs` — Context Group Identity

```rust
pub struct GroupManager {
    context_to_group: HashMap<u64, u64>,  // context_hash → group_id
}
impl GroupManager {
    pub fn update_from_partitions(&mut self, partitions: &[Partition])
    pub fn group_id(&self, context_hash: u64) -> Option<u64>
}
```

After min-cut, each partition is assigned a canonical `group_id` equal to the smallest context hash in the partition. `GroupManager` maintains the mapping and provides lookup. The group structure enables **cross-context coherence transfer**: if contexts A and B are in the same group, positive experience in A can partially warm B.

**Patent reference:** [0063] — group identity assignment.

### `recomputation.rs` — Scheduler

```rust
pub struct RecomputationScheduler {
    episodes_per_context: u32,      // Default: 10
    base_max_cut_weight: f32,       // Default: 0.3
    curiosity_drive: f32,           // Lowers threshold → finer discrimination
}
impl RecomputationScheduler {
    pub fn should_recompute(&self, total_episodes: u64, observed_contexts: usize) -> bool
    pub fn effective_cut_weight(&self) -> f32
}
```

Triggers graph recomputation every `episodes_per_context × observed_contexts` new episodes. `curiosity_drive` lowers `max_cut_weight`, causing a curious robot to make finer distinctions between contexts (more, smaller groups).

**Patent reference:** [0064] — personality-modulated recomputation.

### `consolidation.rs` — Offline Review Cycles

```rust
pub struct ConsolidationTrigger {
    idle_ticks_required: u64,   // 300 ticks (~15 seconds at 20 Hz)
    min_energy: f32,            // 0.5 — robot must not be exhausted
    max_tension: f32,           // 0.3 — robot must be calm
}
pub struct ConsolidationResult {
    pub habits_compiled: usize,
    pub suppression_rules_updated: usize,
    pub graph_recomputed: bool,
}
impl ConsolidationEngine {
    pub async fn run_cycle(&self, ...) -> ConsolidationResult
}
```

When the robot has been idle for 300+ ticks *and* is calm and energised, the consolidation engine runs:

1. Replays recent episodes through the relational graph builder.
2. Triggers min-cut recomputation if the scheduler says so.
3. Calls `HabitCompiler` to detect repeated sequences.
4. Reviews `SuppressionLearner` rules for any that should be promoted or removed.
5. Sends updated structures downstream via `DownwardMessage`.

This models the mammalian process of **offline memory consolidation** — the brain replaying experience during rest to strengthen patterns.

**Patent reference:** [0067]–[0071] — consolidation cycles; rest-phase learning.

### `habits.rs` — Habit Detection

```rust
pub struct SensorDistribution {
    loudness: WelfordStats,
    brightness: WelfordStats,
    distance: WelfordStats,
}
pub struct HabitCompiler {
    min_repetitions: u32,            // Must have seen sequence N times
    distribution_shift_zscore: f32,  // Threshold for "this is different now"
}
impl HabitCompiler {
    pub fn compile(&self, episodes: &[InteractionEpisode]) -> Vec<CompiledHabit>
}
```

Detects frequently-repeated sequences of actions across episodes. For each candidate habit, `SensorDistribution` tracks the typical sensor conditions (using online Welford statistics for mean and variance). Distribution shift — detected by z-score exceeding threshold — flags when a habit's context has changed and the habit may no longer apply.

Compiled habits are eventually sent to the reflexive layer as autonomous reflex routines, reducing deliberative overhead for known situations.

**Patent reference:** [0072]–[0074] — habit compilation from repeated episodes.

### `suppression_learner.rs` — Context-Specific Startle Suppression

```rust
pub struct SuppressionLearnerConfig {
    pub min_observations: u32,     // 5 — must see stimulus N times before deciding
    pub benign_threshold: f32,     // 0.8 — if 80% of observations were benign, create rule
    pub harmful_threshold: f32,    // 0.4 — if <40% benign, remove any existing rule
}
pub struct LearningResult {
    pub new_rules: Vec<SuppressionRule>,
    pub removed_rules: Vec<SuppressionRule>,
}
impl SuppressionLearner {
    pub fn observe(&mut self, stimulus: &ClassifiedStimulus, outcome: EpisodeOutcome)
    pub fn review(&mut self) -> LearningResult
}
```

After observing a stimulus (e.g., a loud clap) in a given context multiple times, the learner classifies whether that stimulus type is benign or harmful *in that context*. If benign 80%+ of the time, it creates a `SuppressionRule` that tells the reflexive layer to reduce the startle response multiplier for this stimulus+context combination. Suppression factors are clamped to [0.3, 1.0] by the reflexive layer — the robot never becomes fully immune, only less reactive.

This produces **context-specific habituation**: the robot learns that a clap in *this familiar room* is probably applause, while a clap in an unfamiliar environment remains startling.

**Patent reference:** [0075]–[0077] — suppression learning; [0078e] — emergent context-specific habituation.

### `suppression_sync.rs` — Reflexive Layer Synchronisation

Manages the channel that sends learned suppression rules from the deliberative `SuppressionLearner` to the reflexive `Startle` subsystem in `mbot-core`. On each consolidation cycle, updated rules are serialised into a `DownwardMessage::SuppressionMapUpdate` and applied to the live reflex loop at the next safe tick boundary.

### `coherence_persist.rs` — Persistence

Serialises `CoherenceField` (all context → accumulator mappings, interaction counts, earned floors) to disk between sessions. On startup, loads persisted state so the robot retains its relational history across power cycles. Uses a simple binary format compatible with `no_std` data structures.

**Patent reference:** [0052] — persistent coherence across sessions.

### `downward.rs` — Deliberative → Reflexive Communication

```rust
pub enum DownwardMessage {
    /// Updated suppression factor map: context_hash → factor [0.3, 1.0]
    SuppressionMapUpdate(HashMap<u64, f32>),

    /// Updated group map: context_hash → group_id
    CoherenceGroupMap(HashMap<u64, u64>),

    /// Consolidation completed — metadata for logging
    ConsolidationState {
        habits_compiled: usize,
        rules_updated: usize,
        graph_recomputed: bool,
        timestamp_ticks: u64,
    },
}
```

The one-way channel from deliberative to reflexive. The reflexive layer only accepts structured updates — it cannot be instructed to act arbitrarily, preserving determinism. This mirrors the neuroscience concept of **descending modulation**: the prefrontal cortex adjusts subcortical responses but does not override them directly.

**Patent reference:** [0033]–[0034] — deliberative-to-reflexive downward pathway; [0066] — downward message types.

---

## Data Flow Summary

```
Sensors (BLE/Serial)
        │
        ▼
MBotBrain.tick()  ←── Personality (curiosity, recovery, startle, ...)
        │               ← SuppressionMap (from DownwardMessage)
        │               ← CoherenceGroupMap (from DownwardMessage)
        │
        ▼
instant_coherence, tension, energy, motor_cmd_base
        │
        ▼
PresenceDetector.update() → PresenceSignature
        │
        ▼
ContextKey::from_sensors()
        │
        ▼
CoherenceField.effective_coherence(instant, key)
  ├── CCF-001 Asymmetric Gate:
  │     unfamiliar: min(instant, ctx)
  │     familiar:   0.3·instant + 0.7·ctx
  └── fallback_coherence if key unseen (degraded mode)
        │
        ▼
SocialPhase::classify(eff_coherence, tension, prev_phase)
  └── Schmitt trigger hysteresis (0.10 deadband)
        │
        ▼
expression_scale() × motor amplitude
led_tint() → CyberPi LEDs
state.coherence = eff_coherence  (broadcast to voice API / dashboard)

                    [Async, between ticks]
CoherenceAccumulator.positive/negative_interaction()
        │
        ▼ [every ~5 mins or 300+ idle ticks, calm + energised]
ConsolidationEngine.run_cycle()
  ├── RelationalGraph.build(episodes)
  ├── SWGraph.nway_bisection()
  ├── GroupManager.update_from_partitions()
  ├── HabitCompiler.compile()
  ├── SuppressionLearner.review()
  └── DownwardMessage → Startle subsystem
```

---

## Relationship to Patent Claims

| Patent Claim | Component | Key Method / Field |
|---|---|---|
| 1 — Context detection | `ContextKey::from_sensors()` | Six-feature vocabulary |
| 2 — Coherence accumulation | `CoherenceAccumulator` | `positive/negative_interaction()` |
| 3 — Earned floor | `CoherenceAccumulator::earned_floor()` | `0.5*(1-1/(1+n/20))` |
| 4 — Asymptotic accumulation | `positive_interaction()` | `delta * (1 - value)` |
| 5 — Temporal decay | `CoherenceAccumulator::decay()` | Exponential decay |
| 6 — Persistence | `coherence_persist.rs` | `CoherenceField` serialisation |
| 7 — Behavioural gating | `CoherenceField::effective_coherence()` | CCF-001 gate |
| 8 — Context key hashing | `ContextKey: Hash + Eq` | `HashMap` key |
| 13 — Cold-start baseline | `CoherenceField::new_with_personality()` | `0.15 * curiosity_drive` |
| 16 — Asymmetric gate | `effective_coherence()` | min() / weighted-average split at 0.3 |
| 17 — Social phase 2D space | `SocialPhase::classify()` | Coherence × tension quadrant |
| 18 — Hysteresis | `SocialPhase::classify()` | 0.10 deadband |
| 19 — LED / motion expression | `led_tint()`, `expression_scale()` | Phase-mapped outputs |
| 22 — Min-cut boundary discovery | `SWGraph::stoer_wagner()` | Stoer-Wagner O(VE+V²logV) |
| 23 — N-way bisection | `SWGraph::nway_bisection()` | Recursive bisection |
| 24 — Personality-modulated cut | `RecomputationScheduler::effective_cut_weight()` | `curiosity_drive` lowers threshold |
| 25 — Dual-process architecture | `MBotBrain` + `brain/` modules | no_std + std separation |
| 26 — Downward messages | `DownwardMessage` enum | Three message types |
| 27 — Suppression learning | `SuppressionLearner` | Context-specific startle reduction |
| 28 — Habit compilation | `HabitCompiler` | Episode sequence detection |
| 29 — Consolidation cycles | `ConsolidationEngine` | 300-tick idle trigger |
| 30 — Degraded mode | `CoherenceField::set_fallback()` | `fallback_coherence` field |

---

## Invariants

| ID | Rule | Enforced In |
|----|------|-------------|
| **CCF-001** | Asymmetric gate: unfamiliar uses min(), familiar uses weighted average | `effective_coherence()` |
| **CCF-002** | All accumulator values bounded [0.0, 1.0] | `positive_interaction()`, `earned_floor()` |
| **CCF-003** | Personality modulates deltas, never structure | `positive_interaction(recovery_speed)`, `new_with_personality(curiosity)` |
| **CCF-004** | Phase boundaries use Schmitt trigger hysteresis (0.10 deadband) | `SocialPhase::classify()` |
| **ARCH-001** | `mbot-core` is `no_std` compatible — no `std::` usage | `#![cfg_attr(not(feature = "std"), no_std)]` |
| **ARCH-002** | `MBotBrain.tick()` is deterministic — same inputs, same outputs | No async, no RNG, no system calls |
| **ARCH-003** | Kitchen Table Test — no harmful behaviours at full speed near children | `SafetyFilter` in motor path |

---

## Key Emergent Properties

These properties arise from the architecture without being explicitly programmed — they are highlighted in the patent as novel contributions:

1. **Shyness without shyness programming** [0078b]: `min(instant, ctx)` at low context values causes the robot to move and vocalise less in unfamiliar situations. No `if unfamiliar: be shy` code exists.

2. **Asymmetric trust resilience** [0078e]: Familiar contexts use the weighted average gate, so a sudden loud noise in a trusted context causes a smaller reduction in effective coherence than in an unfamiliar one. The robot appears to "know" the difference between a familiar and unfamiliar disturbance.

3. **Developmental trajectory without developmental programming** [0078f]: As contexts accumulate experience, the robot naturally progresses from ShyObserver → QuietlyBeloved in those contexts, while remaining a ShyObserver in new ones. This creates an observable developmental arc.

4. **Context-specific habituation** [0078e]: After the suppression learner has seen a stimulus (e.g., a clap) be benign 5+ times in a given context, the startle multiplier decreases *only for that context*. The robot habituates to applause at home without becoming insensitive to sudden sounds in new places.

5. **Hesitation as perceived cognitive depth** [0078d]: The brief period after a context change when effective coherence drops (before the new context accumulates trust) manifests as slower, quieter movement. Observers interpret this as the robot "thinking" or "noticing" the change.

---

*Technical reference for `github.com/Hulupeep/CCF`. All intellectual property is the work of Colm Byrne, Flout Labs (Flout Ltd), Cave, Clarinbridge, Co. Galway, Ireland. Patent pending.*
