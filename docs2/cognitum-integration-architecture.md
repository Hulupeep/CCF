# CCF Integration Architecture for Cognitum Hardware

## Overview

CCF sits at Layer 2 of Cognitum hardware, gating all agent-to-user interactions through earned relational trust. It operates between the raw sensor events produced by Cognitum silicon and the LLM response generation layer above, continuously shaping what the device knows about each user's relational state and making that knowledge available as structured context for every inference call.

## Three-Layer Stack

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 3: Cloud / LLM Response Generation                   │
│           Receives: effective_coherence + SocialPhase        │
│           as enriched context on every prompt                │
└──────────────────────────┬──────────────────────────────────┘
                           ↑ effective_coherence + SocialPhase
┌─────────────────────────────────────────────────────────────┐
│  Layer 2: CCF Behavioural Layer       ← THIS IS ccf-core    │
│           CoherenceField<V> + SocialPhase                    │
│           Persisted via CCF_SEG in .rvf                      │
└──────────────────────────┬──────────────────────────────────┘
                           ↑ Raw sensor events → SensorVocabulary impl
┌─────────────────────────────────────────────────────────────┐
│  Layer 1: Cognitum v0 Chip / V1 Thumb Drive                 │
│           Raw sensor events, I/O bus, local compute          │
└─────────────────────────────────────────────────────────────┘
```

## What CCF Adds at Each Layer

**Layer 1 — Cognitum hardware**
- No changes to silicon or firmware required
- CCF consumes sensor events via a thin `SensorVocabulary` adapter
- Adapter maps Cognitum event types to CCF's `positive_interaction` / `negative_interaction` calls

**Layer 2 — CCF Behavioural Layer**
- Maintains `CoherenceField<V>`: a continuous scalar tracking earned relational trust per user
- Tracks `SocialPhase`: which of the five relational phases the device is currently in (stranger, acquaintance, companion, confidant, bonded)
- Applies homeostatic decay: trust accumulates through interaction and decays slowly without it, preventing artificial lock-in
- Persists state to `.rvf` format via `CCF_SEG` — survives power cycles

**Layer 3 — LLM / Cloud**
- Receives `effective_coherence` (0.0–1.0) and `SocialPhase` as structured context
- Can condition response tone, disclosure depth, and personalisation on relational state
- No model retraining required — CCF operates as a prompt-layer enrichment signal

## Integration Path

1. **Implement SensorVocabulary** — Write a Cognitum-specific adapter that maps hardware sensor events (touch, voice proximity, sustained attention, rejection signals) to CCF's vocabulary calls. Typically 100–200 lines of Rust.

2. **Instantiate CoherenceField** — Create one `CoherenceField<CognitumSensor>` per user identity. On first boot this initialises to the stranger-phase baseline. On subsequent boots it loads from `.rvf` via `CCF_SEG`.

3. **Call positive/negative_interaction per user event** — Route each meaningful user event through the field. CCF handles scoring, phase transitions, and decay automatically.

4. **Feed effective_coherence + SocialPhase to Layer 3** — Append CCF state to the system prompt or context block before each LLM call. The Layer 3 model sees a user's relational history expressed as two values; no other integration is required.

## Persistence

CCF state is stored in the `.rvf` (RuVector File) format under the `CCF_SEG` segment key. The segment is a compact binary representation of the full `CoherenceField` state: decay parameters, phase thresholds, and the accumulated coherence value.

On reboot, warm-start loading reads `CCF_SEG` before the first user interaction. An established user who has reached companion or confidant phase is immediately recognised as such — the device does not reset to stranger behaviour between sessions. This is the mechanism that makes long-term users feel known rather than re-onboarded.

## The Retention Argument

Cognitum's current story is performance. CCF is the retention story.

A device that behaves differently with you after 90 days than on day 1 creates switching cost that speed benchmarks cannot. Users who have accumulated relational state with a device are not comparing it to alternatives on spec sheets — they are comparing the continuity of a relationship against the cost of starting over. CCF makes that continuity legible to the hardware and to the LLM layer above it, without requiring any changes to Cognitum silicon.
