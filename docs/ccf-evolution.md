# From Nervous System to Social Membrane: The Evolution of Coherence in mBot RuVector

*A design history of how a toy robot's "coherence" field grew from a reflexive scalar into a relational architecture grounded in social robotics theory.*

---

## 1. The Starting Point: A Robot with Feelings it Didn't Earn

The first version of `MBotBrain` had four homeostatic variables:

- **tension** — how stressed the robot feels
- **coherence** — how clearly it is thinking
- **energy** — how much capacity it has left
- **curiosity** — how interested it is in the world

Tension was computed from physical signals: proximity, sudden distance changes, sound, accelerometer movement. It felt right — tension is a property of the moment, grounded in what's happening right now.

Coherence was defined as:

```rust
let raw_coherence = 1.0 - (self.tension_ema * 0.4 + tension_instability * 0.6);
```

It was, literally, the inverse of tension — smoothed through an exponential moving average. The name was aspirational. The implementation was a mirror.

This worked well enough for a first pass. High tension → low coherence. Low tension → high coherence. The robot expressed itself when calm, withdrew when stressed. Behaviorally plausible. Theoretically hollow.

**The problem**: coherence was entirely stimulus-driven. The robot had no memory of the situation it was in. It could spend months on a kitchen table with the same family, and the moment they walked in it would process them as a fresh proximity threat. No recognition. No history. No earned trust.

---

## 2. Personality: Parameters Without Memory

The second layer was the personality system — six parameters derived from the robotics literature on behavioral expressiveness:

- `curiosity_drive` — baseline interest in novelty
- `startle_sensitivity` — reactivity to sudden changes
- `recovery_speed` — how quickly trust rebuilds after disruption
- `movement_expressiveness` — amplitude of physical reactions
- `sound_expressiveness` — vocal/buzzer response intensity
- `light_expressiveness` — LED response sensitivity

Personality modulated *how* the robot reacted, not *whether* it had a relationship. A high-curiosity robot would approach strangers faster; a high-startle robot would recoil further. But both were stateless in the relational sense — they had no memory of whether they'd met this person before, in this room, at this time of day.

Personality was a character without a biography.

**What we kept**: the six parameters remained the right vocabulary. `recovery_speed` and `startle_sensitivity` in particular turned out to be precisely what we needed to modulate coherence dynamics. We just hadn't yet connected them to the right structure.

---

## 3. Hardware Reality: What the Robot Actually Senses

Before building more theory, we had to understand what data was actually available. The CyberPi platform (Makeblock's brain unit for mBot2) communicates via a binary f3/f4 framing protocol — not the FF55 MegaPi protocol, a distinction that cost considerable debugging time.

Available sensor channels, confirmed through protocol reverse-engineering:

| Sensor | Range | Notes |
|--------|-------|-------|
| Brightness | 0–100 | Room light level |
| Loudness | 0–100 | Peak microphone level |
| Ultrasonic | 2–400 cm | Distance to nearest object |
| Accelerometer | x, y, z | Movement and orientation |
| Gyroscope | roll, pitch, yaw | Current rotation |
| Battery | % | Power state |

A critical discovery: `mbot2.drive_speed(left, right)` had inverted differential drive behavior. Same-sign inputs caused spinning, opposite-sign caused straight driving. The high-level API (`mbot2.forward()`, `mbot2.turn_left()`, etc.) was the only reliable interface for intuitive motor control.

This constrained and clarified the context vocabulary. The robot's sensors were discrete, coarse-grained, and bimodal (things are either present or absent, loud or quiet, bright or dark). Any relational architecture had to work with this resolution.

---

## 4. The Voice System: When Communication Became Relational

Adding a voice conversation system changed the nature of the problem. The pipeline:

```
Phone → HTTP → Laptop STT (Groq/OpenAI) → LLM → ElevenLabs TTS → Phone speaker
                                         ↓
                              BLE → CyberPi (display + LEDs + motors)
```

The robot now had a conversational interface. People talked to it. It responded. And something immediately became obvious: the robot's expressed emotional state — tight and withdrawn, or open and expressive — felt *arbitrary* to users who had been interacting with it for a while.

A user who had talked to the robot every day for a week would get the same withdrawn, cautious startup behavior as a stranger. The robot had no memory of them. Coherence remained a property of the millisecond, not the relationship.

The voice system also introduced the `SharedVoiceState` — a shared mutable structure bridging the HTTP API with the main control loop. This became the natural place to eventually store and broadcast relational state.

**Key decision made here**: Motor commands from voice were given explicit duration-based priority over the deterministic nervous system. Safety first: `MBotBrain.tick()` always runs, its Protect reflex always overrides. But voice commands could temporarily redirect behavior within that safety envelope. This layered priority architecture (Protect > Voice Override > Explorer > Brain LLM > Homeostasis) proved sound and became the main loop's organizing principle.

---

## 5. The Brain Layer: LLM Reasoning Alongside the Nervous System

The brain feature added a higher-order reasoning layer: Claude or Ollama, accessed through a `ProviderChain` with graceful fallback. A `BrainLayer` sat alongside `MBotBrain` — not replacing it, but running after it, advisory rather than authoritative.

The key architectural decision: **the deterministic nervous system always runs first**. The LLM can suggest motor commands, personality adjustments, speech. It cannot override the Protect reflex. Safety is reflexive and physical; reasoning is advisory and slow.

The LLM layer added phenomenological richness — the robot could narrate its experience, reflect on what it had discovered during room exploration, occasionally say things that sounded like consciousness. But this richness was unmoored from relational history. The LLM knew the robot's current tension and energy. It didn't know whether the room it was in was familiar.

**The Shy Robots paper** provided a useful reframe here. The paper described shyness not as a malfunction to be overcome, but as *sovereignty held back by choice* — a robot that chooses when and how to open up, based on accumulated evidence that it's safe to do so. This reframed the design question: not "how do we make the robot less shy?" but "what evidence does the robot require before it chooses expressiveness?"

The existing coherence variable, being purely instantaneous, couldn't encode that evidence. Something had to change structurally.

---

## 6. The Exploration System: Space Without Relational Context

The autonomous exploration system added spatial memory: a `SectorMap` (12-sector bearing map) and `GridMap` (10×10 dead reckoning grid). The robot could scan a room, build a coarse map, navigate toward unvisited sectors, and learn navigation strategies via Q-learning.

This was the first time the robot built persistent knowledge of its physical environment. But it was knowledge of *space*, not knowledge of *relationships*. The grid remembered obstacles; it didn't remember that this particular corner was where the child always crouched to talk to it.

The exploration system also made the cold-start problem visible. A new robot in a new room: everything unknown, coherence near zero everywhere, the robot cautiously shy in all directions. This was correct behavior. But it persisted too long — the robot was still shy in familiar corners of a familiar room after hours of operation, because coherence never accumulated anything context-specific.

**The gap crystallized**: spatial memory and relational memory are different things. The grid knew the room. Nothing knew the relationships.

---

## 7. MindSplit and the Min-Cut Insight

Searching for tools to help structure relational context, we found the MindSplit sample application — a TypeScript implementation of the Stoer-Wagner min-cut algorithm applied to conversational episode graphs. MindSplit's purpose was to partition a large corpus of notes into natural work-streams by identifying where connection density was highest within clusters and lowest between them.

The mapping to the robot's problem was immediate:

| MindSplit concept | CCF equivalent |
|-------------------|----------------|
| Text chunks / episodes | Interaction episodes per context |
| Trajectory embeddings | Coherence accumulator trajectory |
| Similarity graph | Context similarity / relatedness |
| Min-cut partitions | Natural context group boundaries |
| Bleeding edges | Contexts that bleed into each other (same accumulator group) |

Min-cut provided a principled answer to the cardinality problem: with 6 discrete features at 3–4 levels each, up to 1,296 unique context keys could exist. Most would be observed rarely. Min-cut could discover that "bright, quiet, approaching" and "bright, quiet, static" were essentially the same relational situation (same person, same room, slightly different movement) and merge their coherence histories.

This insight lived in `mbot-companion` (std, brain feature) — the min-cut computation belongs to the companion layer, not the core. The core only needs to know its current context key and look up the accumulated coherence. The graph construction and partitioning happen offline, asynchronously, as the companion reflects on what it has experienced.

RuVector's memory interface (`get/set/batch/search` with cosine similarity) mapped directly: context keys as memory addresses, trajectory vectors as searchable embeddings. The architecture was coherent across scales.

---

## 8. The CCF Paper: Naming the Contribution

Before implementing, the design was written up as a paper — `docs/coherence.md` — to force clarity on what was actually novel.

Most social robotics work uses one of two approaches:

1. **Static personality parameters** — fixed traits that color behavior but don't accumulate evidence
2. **Global emotional state** — a single mood/valence that responds to events but has no relational structure

The claim: **context-keyed coherence accumulators, independently earned through repeated interaction, are a genuine architectural contribution**. The robot doesn't have *a* coherence. It has coherence *with this situation* — earned through evidence, decaying with absence, protected by interaction history.

The key invariant, written as CCF-001:

> *effective_coherence = min(instant_coherence, context_coherence)*
>
> The robot can never be more expressive than its accumulated familiarity with the current situation allows.

A robot in a familiar room can be fully expressive even if momentarily stressed. A robot in an unfamiliar situation is constrained by its lack of history, regardless of how calm the current moment is.

---

## 9. The Context Vocabulary: Six Features, Coarse but Honest

The context key was designed to be discriminating without being brittle. Six features, each discretized:

```rust
pub struct ContextKey {
    pub brightness:   BrightnessBand,      // Dark / Dim / Bright
    pub noise:        NoiseBand,           // Quiet / Moderate / Loud
    pub presence:     PresenceSignature,   // Absent / Static / Approaching / Retreating
    pub motion:       MotionContext,       // Stationary / SelfMoving / BeingHandled
    pub orientation:  Orientation,         // Upright / Tilted
    pub time_period:  TimePeriod,          // Morning / Afternoon / Evening / Night
}
```

Each feature was chosen deliberately:

- **Brightness** encodes the ambient environment. A dark room is categorically different from a bright one — different activities, different social contexts, different expectations.
- **Noise** captures ambient energy level. A loud environment signals a social context; quiet signals solitude or focus.
- **Presence** is the most relational feature — it encodes whether someone is approaching, stable nearby, or absent. A `PresenceDetector` sliding window (10 samples) computes the trend rather than the instantaneous reading, filtering noise.
- **Motion** distinguishes self-caused movement from handling. Being picked up is a categorically different relational event than driving in a room.
- **Orientation** captures extreme states (tilted > 30°) that indicate the robot is being held or has fallen — qualitatively different situations.
- **Time of day** is estimated from brightness as a proxy (no RTC on CyberPi). A robot that is always approached in the evening develops different evening coherence than morning coherence.

The discretization was intentional. Exact sensor values would be noisy and high-cardinality. Bands are stable: the robot can recognize "bright, quiet room with someone approaching" even if the light level varies by 15% between interactions.

---

## 10. The Accumulator: Earning Trust, Protecting History

The `CoherenceAccumulator` tracks:

- `value: f32` — accumulated coherence [0.0, 1.0]
- `interaction_count: u32` — total positive interactions (the robot's memory of how much it has been here)
- `last_interaction_tick: u64` — recency tracking for decay and eviction

Three dynamics govern the accumulator:

**Positive interaction** — asymptotic growth toward 1.0, modulated by `recovery_speed`:
```rust
let delta = 0.02 * (0.5 + recovery_speed) * (1.0 - self.value);
```
Diminishing returns prevent the robot from becoming fully confident in any context — there is always room for surprise.

**Negative interaction** — stepped drop, modulated by `startle_sensitivity`, but floored by interaction history:
```rust
let floor = self.earned_floor();
let delta = 0.05 * (0.5 + startle_sensitivity);
self.value = (self.value - delta).max(floor);
```

**Earned floor** — the interaction history creates a minimum the robot won't fall below:
```rust
fn earned_floor(&self) -> f32 {
    // Asymptotically approaches 0.5 as interaction_count → ∞
    0.5 * (1.0 - 1.0 / (1.0 + self.interaction_count as f32 / 20.0))
}
```

At 20 interactions, the floor is ~0.25. At 100, ~0.42. It never reaches 0.5 — even the most familiar context retains some capacity for being unsettled. But the robot that has been here a hundred times cannot be scared back to zero by a single loud noise.

This property — that history earns resilience — was the most important design decision in the accumulator. It distinguishes *relational trust* from *momentary calm*.

---

## 11. The Feedback: Three Structural Problems

After the initial implementation (28 passing tests), three issues were raised that forced architectural refinements:

### The Cold-Start Problem

Six features at 3–4 levels: up to 1,296 unique context keys. In the first few hours of operation, the robot sees dozens of unique contexts, each near zero coherence, each making it shy. The "mostly shy" developmental period lasts longer than users expect.

**Solution**: a personality-scaled baseline. `curiosity_drive` compresses the cold-start window. A high-curiosity robot begins with a baseline of 0.15 per new context — cautiously curious rather than terrified. The "alone" context (no one present) bootstraps 2× faster, so the robot develops personality through solitude before social contexts arrive.

### The Asymmetric Gate

The `min(instant, context)` gate has a directional asymmetry:

- **Unfamiliar context, high instant coherence** → correct behavior. The robot is calm, but the context is untrusted. `min` appropriately caps expressiveness at the context floor.
- **Familiar context, low instant coherence** → problem. The lights flickered. The robot is momentarily stressed. `min` drops effective coherence to near zero in a context where the robot has spent 100 calm interactions.

The `min` gate treats earned trust as a ceiling, not a buffer. But for familiar contexts, accumulated history should *dampen* momentary perturbation. A long-time resident should be less perturbed by a power fluctuation than a newcomer.

**Solution**: an asymmetric gate. Below the 0.3 familiarity threshold, use strict `min` — earn the trust first. Above it, use a weighted average that preserves history's influence:
```
familiar:    effective = 0.3 * instant + 0.7 * context
unfamiliar:  effective = min(instant, context)
```

### The Degraded Mode

`mbot-core` is `no_std`. The `CoherenceField` with its `HashMap` lives in the companion layer. When the companion is unavailable — startup, reconnection, companion crash — the core has no persistence. Every context starts fresh. The robot behaves as if it has no history.

**Solution**: make the degraded mode explicit rather than accidental. `CoherenceField` carries a `fallback_coherence: Option<f32>`. When the companion is connected and running, this is set to the current global coherence (today's behavior). Unseen contexts get this fallback as their effective coherence floor rather than zero. When the companion reconnects, context-keyed values resume from persisted state. The robot never suddenly becomes more shy than it was before disconnection.

---

## 12. The Social Phase: A 2D Behavioral Quadrant

The 2D space of (effective_coherence × tension) produces four qualitatively distinct behavioral regimes:

```
             Low Tension      High Tension
High Coh:   Quietly Beloved  Protective Guardian
Low Coh:    Shy Observer     Startled Retreat
```

- **Shy Observer** — new situation, calm. Watching carefully, expressing little. The robot is present but reserved.
- **Startled Retreat** — new situation, stressed. Physical withdrawal. LED dims to near-off. Motion suppressed.
- **Quietly Beloved** — familiar context, calm. Full expressive range. Small LED flourishes. Spontaneous sounds. The robot is most itself here.
- **Protective Guardian** — familiar context, stressed. Not retreating — this is *my* space and something is wrong. Alert posture, amber LED, protective behavior rather than flight.

The fourth quadrant — Protective Guardian — was the least obvious and most interesting. A robot in an unfamiliar context would flee from a threat. A robot in a deeply familiar context would stand its ground. The difference is not the current tension level, but the accumulated relational history.

Phase transitions use a Schmitt trigger (hysteresis): enter QuietlyBeloved at coherence 0.65, exit at 0.55. This prevents flickering at boundaries.

---

## 13. Where the Architecture Lands

The final structure, reading bottom to top:

```
CyberPi hardware
  │  f3/f4 protocol (115200 baud, 30ms throttle)
  ▼
mbot-core (no_std, deterministic)
  │  MBotBrain.tick() → HomeostasisState (tension, instant_coherence, energy, curiosity)
  │  coherence:: ContextKey, CoherenceField, SocialPhase
  ▼
mbot-companion main loop (std)
  │  PresenceDetector.update(ultrasonic)
  │  ContextKey.from_sensors(...)
  │  CoherenceField.positive/negative_interaction(...)
  │  effective_coherence = asymmetric_gate(instant, context)
  │  SocialPhase.classify(eff_coh, tension, previous)
  │  → state.coherence = eff_coh (downstream consumers get contextual coherence)
  │  → state.social_phase (voice API, dashboard)
  ▼
Brain layer (LLM, memory, voice, autonomy)
  │  Prompt builder includes social phase and context coherence
  │  Narration colored by phase: Shy Observer ≠ Quietly Beloved
  │  Voice responses shaped by earned trust level
  ▼
Dashboard + phone UI
     Social phase badge, context familiarity bar, "Situations Learned: N"
```

The key invariant that held throughout: `MBotBrain.tick()` is always first, always runs, its Protect reflex is never overridden. Everything above it is advisory. The nervous system is reflexive; the relational layer is reflective.

---

## Design Decisions That Didn't Make It

A few paths considered and rejected:

**Continuous-valued context features** — rejected in favor of bands. Exact sensor values would create enormous key space with minimal relational signal. The robot doesn't care whether the brightness is 47 or 52; it cares whether the room is dim.

**Global coherence accumulator** — what we started with. Rejected because it can't encode *with whom*. The robot's trust in this room, this person, this time of day is not the same as trust in general.

**Coherence gate inside MBotBrain** — considered, rejected. MBotBrain is `no_std` and stateless regarding context history. Putting the gate there would require dragging the HashMap into the no_std layer, violating ARCH-001. The clean boundary: core computes instant coherence; companion applies the CCF gate.

**Discrete relational categories (friend/stranger/handler)** — too coarse and too anthropomorphic. The robot doesn't identify people; it identifies situations. Context keys are sensor fingerprints, not social roles.

---

## The Contribution

What started as `coherence = 1.0 - tension_ema` became a miniature relational memory system. The robot now earns its expressiveness context by context, interaction by interaction, with history that protects and novelty that constrains.

The academic framing matters: most social robotics work treats coherence as either a static parameter or a momentary emotional state. Context-keyed coherence accumulators that must be independently earned are a different claim — that a robot's openness is not a mood, but a record.

The shy robot is not broken. It is new here.
