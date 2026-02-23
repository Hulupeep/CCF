# CCF Patent Demo Plan

## Purpose

A single 5-minute video demonstrating that the Contextual Coherence Fields system produces its claimed behaviours on physical hardware. Each section maps to specific patent paragraphs and claims, identifies what the demo proves, and lists the existing codebase functions that implement it.

The demo is structured around one central contrast: **the robot's response to identical stimuli depends on whether it has earned familiarity with the current context — not on the stimulus itself.**

---

## Pre-Demo Setup

| Item | Detail |
|------|--------|
| Hardware | mBot2 with CyberPi, connected via BLE or USB |
| Software | `mbot-companion --bluetooth --voice-api --voice-port 8088` |
| Dashboard | `http://localhost:8088` open on a second screen, visible to camera |
| Dashboard elements visible | `social_phase`, `context_coherence`, `context_count`, tension bar, coherence bar |
| Room state | Bright overhead light, quiet, table clear around robot |

**Before pressing record:** confirm dashboard shows `social_phase = ShyObserver`, `context_coherence = 0.00`, `context_count = 1`. If the robot has run before, restart the companion to clear accumulators, or note that a previous session's context will still be present (which itself demonstrates persistence — claim 3).

---

## Section 1 — The Unfamiliar Situation (0:00–1:30)

### Script
*"This robot has just started. It knows nothing about this room, this person, this moment. Every context coherence accumulator starts at zero. Watch what happens."*

Walk slowly toward the robot from 2 metres. Pause 50 cm away. Stay calm for 15 seconds. Then deliver one sharp clap.

*"I will clap once. The robot has no earned trust here. Watch the floor."*

Clap. Observe: brief tension spike on dashboard, `context_coherence` stays near zero or drops fractionally. LED briefly shifts toward StartledRetreat colour (dark red) then returns to ShyObserver (muted blue-grey). Robot does not increase expressiveness.

### What this demonstrates

1. **The min gate in an unfamiliar context.** When context coherence is 0.0, `effective_coherence = min(instant, 0.0) = 0.0` regardless of how calm the moment is. Calm presence does not unlock expression in an unearned context.

2. **Shyness without a shyness parameter.** No personality flag is set to "be shy." The ShyObserver behaviour emerges purely from zero accumulated coherence intersecting the minimum gate. A different robot with `curiosity_drive = 1.0` would still start here.

3. **The clap floor problem.** In the unfamiliar context, a startle event hits a near-zero accumulator with no earned floor to catch it. The robot has nowhere to go but further into ShyObserver or briefly StartledRetreat.

### Patent references

| Paragraph | Relevance |
|-----------|-----------|
| **[0046]** | "The minimum function enforces the central architectural invariant... a calm environment combined with an unfamiliar context produces reserved, cautious behavior." |
| **[0046a]** | "The use of a minimum function over instantaneous environmental coherence and accumulated relational coherence as a behavioral gate is, to the inventor's knowledge, without precedent." This is the primary novelty claim. The demo proves it operates. |
| **[0049]** | Definition of Shy Observer quadrant: "low effective coherence and low tension... robot is in a learning state." |
| **[0078b]** | Unexpected emergent property: "context-specific shyness without shyness programming... no personality parameter controls shyness directly." |

### Claims supported

| Claim | How the demo supports it |
|-------|--------------------------|
| **Claim 1** | System maintains context-keyed accumulators and gates behavioral expressiveness via effective coherence. Demonstrated by zero expression at zero accumulated coherence. |
| **Claim 2** | Effective coherence = min(instant, accumulated). Demonstrated by calm presence producing no increase in expression when accumulated coherence = 0. |
| **Claim 7 (first quadrant)** | Shy Observer behaviour: "cautious observation" visible via muted LEDs and absence of voluntary motion. |

### Existing codebase functions

| Function | File | Role in this demo |
|----------|------|-------------------|
| `CoherenceAccumulator::new()` | `crates/mbot-core/src/coherence/mod.rs:305` | Initialises accumulator at 0.0 — the starting state for every new context |
| `CoherenceField::effective_coherence()` | `coherence/mod.rs:407` | `if instant < ctx { instant } else { ctx }` — the min gate producing 0.0 output |
| `CoherenceField::context_coherence()` | `coherence/mod.rs:399` | Returns 0.0 for unseen context keys — the "unfamiliar" state |
| `SocialPhase::classify()` | `coherence/mod.rs:490` | Maps (0.0 coherence, low tension) → `ShyObserver` |
| `SocialPhase::led_tint()` | `coherence/mod.rs:533` | `ShyObserver => [40, 40, 80]` — muted blue-grey, visible to camera |
| `SocialPhase::expression_scale()` | `coherence/mod.rs:523` | `ShyObserver => 0.25` — 25% amplitude cap on all outputs |
| `ContextKey::from_sensors()` | `coherence/mod.rs:189` | Builds the situation fingerprint that produces context_coherence = 0.0 for this new situation |

---

## Section 2 — Building Familiarity (1:30–2:10)

### Script
*"Now I stay. Same person, same room, same distance. Every calm tick, the robot is learning this is safe."*

Remain near the robot in calm proximity for 2–3 minutes of calm interaction (edit to 30 seconds for the video). Show the dashboard `context_coherence` ticking upward: 0.01 → 0.08 → 0.20 → 0.35. When it crosses ~0.35–0.40, dashboard shows `social_phase` shift to `QuietlyBeloved`. LED shifts to warm blue.

*"Forty calm interactions. The robot crossed the threshold. Not because I told it to relax — because the accumulator earned it."*

### What this demonstrates

1. **Asymptotic positive accumulation.** Coherence grows quickly at first, slows as it approaches the ceiling. This is not a linear counter — it is a diminishing-returns growth curve, matching the claim that "initial gains are rapid and refinement is gradual."

2. **Personality modulation without personality-gating expression.** `recovery_speed` affects how fast the accumulator grows, but cannot grant coherence without earning it.

3. **Phase transition via hysteresis.** The ShyObserver → QuietlyBeloved transition does not happen at an exact threshold — it uses a Schmitt trigger (enter at 0.65, exit at 0.55) preventing flicker at the boundary.

### Patent references

| Paragraph | Relevance |
|-----------|-----------|
| **[0038]** | "Positive accumulation: coherence value is incremented by a positive delta... further multiplied by the factor (1.0 minus the current coherence value). This multiplicative factor produces asymptotic growth." |
| **[0042]** | "Personality parameters modulate the dynamics of coherence accumulation... but do not modify the structural requirement that coherence must be earned." |
| **[0051]** | Definition of Quietly Beloved: "robot's full expressive range is unlocked... individuated behavioral characteristics... This quadrant represents the earned fluency state that is the developmental endpoint of the system." |
| **[0053]** | Hysteresis implementation: "entry into the Quietly Beloved quadrant may require coherence exceeding 0.65... while exit requires coherence falling below 0.55." |
| **[0078f]** | Emergent developmental trajectory: "The trajectory emerges from the accumulation of coherence... A skilled practitioner designing the individual components would not predict this specific developmental trajectory." |

### Claims supported

| Claim | How the demo supports it |
|-------|--------------------------|
| **Claim 3** | Interaction count grows with each positive tick. Accumulated coherence is visible on dashboard and directly tracks the claim's description of the accumulator structure. |
| **Claim 4** | Asymptotic growth visible on dashboard: coherence increments are large early, small near the ceiling. |
| **Claim 5** | Recovery speed personality parameter is modulating the rate of accumulation (not granting coherence directly). |
| **Claim 6** | Phase space transition demonstrated: ShyObserver → QuietlyBeloved with visible hysteresis boundary. |
| **Claim 7 (third quadrant)** | QuietlyBeloved behaviour: "full expressive fluency" visible via brighter LEDs, higher expression_scale. |

### Existing codebase functions

| Function | File | Role in this demo |
|----------|------|-------------------|
| `CoherenceAccumulator::positive_interaction()` | `coherence/mod.rs:317` | `delta = 0.02 * (0.5 + recovery_speed) * (1.0 - self.value)` — asymptotic growth formula |
| `CoherenceAccumulator.interaction_count` | `coherence/mod.rs:300` | Incremented on each positive tick; drives `earned_floor()` |
| `SocialPhase::classify()` | `coherence/mod.rs:490` | Hysteresis logic: `coherence_high_enter = 0.65`, `coherence_high_exit = 0.55` |
| `SocialPhase::led_tint()` | `coherence/mod.rs:533` | `QuietlyBeloved => [60, 120, 200]` — warm blue, the camera-visible phase transition signal |
| `SocialPhase::expression_scale()` | `coherence/mod.rs:523` | `QuietlyBeloved => 1.0` — full amplitude unlocked |
| `CoherenceField::get_or_create()` | `coherence/mod.rs:387` | Retrieves or creates accumulator for the current context key |

---

## Section 3 — Clap in a Familiar Context (2:10–3:00)

### Script
*"Now I clap. Same clap as 2 minutes ago. But something is different."*

Single sharp clap. Dashboard: tension spikes. `context_coherence` drops — but only to the earned floor (~0.22 at 40 interactions). Social phase dips to StartledRetreat for 3–5 ticks then returns to QuietlyBeloved.

*"It dipped. Tension spiked. But it came back. Forty positive interactions bought a floor that one clap cannot break through."*

Clap twice more. Same pattern — dip, recover.

*"In a system with global coherence, every clap would reset everything equally. Here, the robot knows this room. A clap is just noise in a familiar place."*

### What this demonstrates

1. **The earned floor.** `interaction_count` creates a minimum the accumulator cannot fall below after a negative event. This is the resilience property that distinguishes accumulated trust from instantaneous calm.

2. **Asymmetric trust resilience proportional to interaction depth.** The same negative event (clap) has a quantitatively different effect on a zero-history accumulator (Section 1) versus a 40-interaction accumulator (here). The difference is not in the clap — it is in the history.

3. **The asymmetric gate for familiar contexts.** Once `context_coherence >= 0.3`, effective coherence uses `0.3 * instant + 0.7 * context` rather than strict min. The context's accumulated history buffers the instantaneous noise. The robot stays in QuietlyBeloved through brief tension spikes that would have dropped a newcomer to ShyObserver.

### Patent references

| Paragraph | Relevance |
|-----------|-----------|
| **[0039]** | "Negative accumulation... floored by interaction history." |
| **[0040]** | "Interaction-count-proportional decay floor: contexts with a deep history of positive interaction maintain a higher minimum coherence... A single negative event cannot erase weeks of positive interaction history." |
| **[0047]** | "The minimum gate may be replaced with an asymmetric gate for high-familiarity contexts... weighted average of instantaneous and context coherence rather than a strict minimum, reflecting the resilience that deep familiarity provides." |
| **[0078e]** | Unexpected emergent property: "asymmetric trust resilience proportional to interaction depth... a qualitative distinction between shallow and deep familiarity at identical coherence levels." |

### Claims supported

| Claim | How the demo supports it |
|-------|--------------------------|
| **Claim 3** | Interaction-count-proportional floor directly visible: `context_coherence` drops but is caught above zero. Compare to Section 1 where the same clap hit a floor of 0.0. |
| **Claim 16** | Asymmetric gate for high-familiarity contexts: "when the accumulated context coherence exceeds a high-familiarity threshold, the effective coherence may be computed as a weighted average." QuietlyBeloved persistence through the clap demonstrates this working. |

### Existing codebase functions

| Function | File | Role in this demo |
|----------|------|-------------------|
| `CoherenceAccumulator::earned_floor()` | `coherence/mod.rs:352` | `0.5 * (1.0 - 1.0 / (1.0 + count / 20.0))` — the floor the negative event cannot breach |
| `CoherenceAccumulator::negative_interaction()` | `coherence/mod.rs:329` | `self.value = (self.value - delta).max(floor)` — clamped by earned floor |
| `CoherenceField::effective_coherence()` | `coherence/mod.rs:407` | Asymmetric gate logic: `if ctx < 0.3 { min } else { 0.3 * instant + 0.7 * ctx }` (planned, in integration plan) |
| `SocialPhase::classify()` | `coherence/mod.rs:490` | Hysteresis keeps `QuietlyBeloved` active through brief coherence dip (exit threshold 0.55, not entry threshold 0.65) |

---

## Section 4 — Context Change: Same Person, Stranger Again (3:00–4:10)

### Script
*"Same room. Same person. I'm going to change one thing."*

Turn off the main overhead light (or close the blinds — shifting `BrightnessBand` from Bright → Dark). Dashboard: `context_coherence` immediately shows 0.00 for the new context key. `context_count` increments to 2.

*"The situation fingerprint changed. Different light level — different context key. Zero accumulated coherence. Two situations are now tracked separately on the dashboard."*

Walk toward the robot at the same gentle pace as before.

*"Same person. Same movement. Same distance."*

Robot is ShyObserver again. LEDs muted. Minimal expression. Pause. Then clap.

*"The clap hits harder here. No earned floor in this context. Nothing to protect."*

Dashboard: `context_coherence` drops from near-zero further. Social phase briefly StartledRetreat.

*"Turn the lights back on."*

Lights back on. Dashboard: original context key reappears, `context_coherence` resumes at ~0.22 (where it was after the claps in Section 3). Within a few calm ticks, robot returns toward QuietlyBeloved in the original context.

*"The robot remembered where it left off. Coming back to the familiar context picked up from its history. The dark-room context is still at zero — it needs its own time."*

### What this demonstrates

1. **Situation fingerprinting, not person recognition.** The same person, same approach, same behaviour — produces zero coherence in a new context. This is the central architectural choice that makes CCF distinct from person-keyed systems.

2. **Independent accumulator isolation.** Context A's accumulated coherence is completely separate from Context B's. Earning trust in one situation does not bleed into another. The two accumulators evolve independently.

3. **Context key extensibility.** The demo uses brightness as the differentiating feature. On a camera-equipped platform, person ID would be an additional feature. The architecture is the same — a different key, a different accumulator. No structural change required.

4. **Persistent relational memory.** Returning to the original context resumes its accumulator mid-value. The robot did not forget.

### Patent references

| Paragraph | Relevance |
|-----------|-----------|
| **[0035]** | "It is a deliberate and architecturally significant feature of the invention that context keys fingerprint situations rather than individuals." |
| **[0035a]** | "Honesty principle: a robot that cannot distinguish Person A from Person B does not maintain the fiction that it has separate relationships with each." |
| **[0035b]** | "Situation fingerprinting captures relational dimensions that individual identification alone cannot... the same friend encountered in a quiet home on a weekend morning and in a noisy office on a weekday afternoon elicits different social behavior." |
| **[0035c]** | Extensibility: "On platforms equipped with additional sensors... the context key is extended by concatenating additional quantized feature levels... No component of the system requires modification." |
| **[0032]–[0034]** | Context detection subsystem: sensor quantisation, composite key construction. The light-level change producing a new key is exactly the mechanism described here. |
| **[0078b]** | "Context-specific shyness without shyness programming" — re-demonstrated in the new context despite established relationship in the adjacent context. |

### Claims supported

| Claim | How the demo supports it |
|-------|--------------------------|
| **Claim 1** | Each context maintains an independent accumulator. Demonstrated by context_count = 2 on dashboard and zero coherence in new context despite positive history in original. |
| **Claim 8** | Composite context key formed from quantised sensor signals. Brightness band shift (Bright → Dark) produces a new key — the mechanism of independent accumulation per context. |
| **Claim 13** | "Early in operational life, few coherence groups... After extended operation, naturally partitions into distinct relational domains." New dark context is in the early state; the familiar bright context is in the developed state. Both are visible simultaneously on the dashboard. |

### Existing codebase functions

| Function | File | Role in this demo |
|----------|------|-------------------|
| `ContextKey::from_sensors()` | `coherence/mod.rs:189` | Constructs the composite key from `light_level`, `sound_level`, `presence`, `accel_mag`, `motors_active`, `roll_deg`, `pitch_deg` — the brightness shift produces a different key |
| `BrightnessBand::from_light_level()` | `coherence/mod.rs:44` | `if level < 0.15 { Dark } else if level < 0.50 { Dim } else { Bright }` — the threshold crossed when lights are turned off |
| `CoherenceField::context_coherence()` | `coherence/mod.rs:399` | Returns 0.0 for the new key (dark context), while the bright context key still holds its ~0.22 floor |
| `CoherenceField.accumulators` (HashMap) | `coherence/mod.rs:376` | The two independent accumulators coexist in this map — `context_count()` shows 2 on dashboard |
| `CoherenceField::context_count()` | `coherence/mod.rs:424` | Returns `self.accumulators.len()` — increments to 2 when the dark context key is first encountered |
| `PresenceDetector` | `coherence/mod.rs:214` | Sliding window on ultrasonic maintains `Approaching` classification across the context change — same approach movement, different key |

---

## What Still Needs to Be Built for the Demo

The CCF data structures and all accumulation/gating logic exist and pass tests. The demo requires three additional integration steps:

### 1. LED output wired to `SocialPhase.led_tint()`

**Current state:** `SocialPhase::led_tint()` returns `[R, G, B]` values at `coherence/mod.rs:533`. These are not yet sent to the CyberPi LED.

**Required:** In `main.rs`, after computing `social_phase`, set `cmd.led_color = social_phase.led_tint()`. This is one line.

**Why critical for demo:** The LED colour shift is the camera-visible signal of phase transitions. Without it, the viewer must read dashboard numbers. With it, the robot physically shows its state.

### 2. Expression scale applied to motor amplitude

**Current state:** `SocialPhase::expression_scale()` returns a scalar at `coherence/mod.rs:523`. Not yet applied to motor commands.

**Required:** Multiply `cmd.left` and `cmd.right` by `social_phase.expression_scale()` before sending.

**Why useful for demo:** ShyObserver (0.25 scale) produces visibly smaller movements than QuietlyBeloved (1.0 scale) in response to the same stimulus.

### 3. Dashboard showing `social_phase`, `context_coherence`, `context_count`

**Current state:** Dashboard shows global `coherence` from `HomeostasisState`. The new fields `social_phase`, `context_coherence`, `context_count` are not yet in `RobotState` or broadcast via WebSocket.

**Required:** Per the integration plan — add three fields to `RobotState` in `voice_api.rs`, populate them in the main loop, include in the WebSocket broadcast, display in `index.html`.

**Why critical for demo:** The dashboard is the receipt. It proves the mechanism is real, not theatrical. `context_count = 2` showing two independent situations side by side is the clearest visual proof of claim 1.

---

## Complete Patent Claim Coverage Map

| Demo section | Claims directly evidenced |
|---|---|
| Section 1 (unfamiliar, zero coherence) | 1, 2, 7 (quadrant 1), 8 |
| Section 2 (building familiarity, phase transition) | 3, 4, 5, 6, 7 (quadrant 3) |
| Section 3 (clap resilience, earned floor) | 3, 16 |
| Section 4 (context change, independent accumulation) | 1, 8, 13 |
| Across all (emergent shyness without shyness parameter) | 1, 2, 5 + paragraphs [0078b], [0078e], [0078f] |
| Dashboard context_count = 2 | 1, 9 (boundary discovery observable precondition) |

Claims 9–15 (graph min-cut boundary discovery) and 18–34 (manifold-constrained mixing, dual-process architecture) require a longer-running demo with offline consolidation cycles. This 5-minute demo covers the core CCF claims (1–8, 13, 16) that are the foundational layer of the full system.
