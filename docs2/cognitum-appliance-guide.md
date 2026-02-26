# Cognitum v0 Appliance — Practical Guide for CCF Integration

This document covers the Cognitum v0 desktop appliance as a deployment target for CCF:
what the hardware provides, how the sensor port works, and what CCF does with data
that comes through it.

For the software stack diagram (Layers 1–3), see `cognitum-integration-architecture.md`.
For the MCP gate protocol (permit/defer/deny), see `cog.md`.

---

## What the Appliance Is

The Cognitum v0 is a desktop computing unit designed to sit beside the user rather
than in a cloud. It ships in March 2026, is CES 2026 Innovation Awards honoured, and
is positioned as "the world's first agentic appliance."

Physical form factor: desktop box, Gigabit Ethernet, USB power, under 15 watts
continuous draw. No internet connection required after initial setup. All compute
and learning stays on-device.

The unit contains **7 agentic tiles** — independent WASM compute cores that run in
parallel. Each tile is ~32 MB of working memory and can spawn or terminate in 50ms.
For CCF this means CoherenceField state, sensor processing, and the Coherence Gate
can each run on a dedicated tile without contention.

---

## The Sensor Port

The v0 appliance exposes a **sensor expansion port** on the rear panel. This is the
primary integration point for physical devices, robots, and environmental sensor
arrays.

### What It Accepts

The port speaks the Cognitum universal protocol: a compact 33-byte binary frame at
up to 9.85 million operations per second. Any device that can produce this frame
format can stream sensor data into the appliance in real time.

The frame structure (from the protocol documentation) carries:

```
[ 1 byte: sensor_type ]
[ 4 bytes: sensor_id  ]
[ 4 bytes: timestamp_ms ]
[ 8 bytes: value (f64) ]
[ 4 bytes: flags ]
[ 12 bytes: reserved / metadata ]
= 33 bytes total
```

Latency from frame arrival to tile processing: sub-millisecond. At full rate the
port sustains about 300,000 sensor readings per second while the tiles are otherwise
busy — comfortably above what any single robot or sensor array produces.

### Sensor Categories Relevant to CCF

CCF's `SensorVocabulary` maps physical readings to relational signals. The following
sensor categories feed CCF's `ContextKey` and drive `positive_interaction` /
`negative_interaction` calls on the `CoherenceField`:

| Category | Example hardware | CCF signal |
|----------|-----------------|------------|
| Proximity / distance | IR rangefinder, lidar | Presence: absent / static / approaching / retreating |
| Ambient light | photodiode, camera luma | `light_level` 0.0–1.0 |
| Sound level | microphone envelope | `sound_level` 0.0–1.0 |
| IMU / motion | accelerometer, gyro | `accel_mag`, `roll_deg`, `pitch_deg` |
| Motor state | encoder feedback | `motors_active` bool |
| Touch / contact | capacitive, force | positive interaction event |
| Sustained attention | camera + face detect | positive interaction event (extended) |
| Rejection signal | sharp motion away, lid close | negative interaction event |

The mBot2 robot covers the first five categories natively. Touch and attention
detection require the mBot2 camera module or an attached sensor.

---

## How CCF Uses the Sensor Stream

### Step 1 — Sensor frame arrives at the port

The Cognitum firmware routes the frame to the tile assigned to CCF's sensor
vocabulary adapter. This adapter runs as a WASM module (or Rust compiled to the
tile ABI) and implements `SensorVocabulary`.

### Step 2 — ContextKey is constructed

The adapter aggregates recent sensor readings into a `ContextKey`. CCF does not
react to individual frames; it derives a context snapshot from a rolling window
(typically 100–500 ms). This prevents noise spikes from triggering phase transitions.

```rust
// Cognitum sensor adapter — pseudocode
impl SensorVocabulary for CognitumSensors {
    fn context_hash_u32(&self) -> u32 {
        // Hash the quantised sensor tuple into a stable context ID
        fnv1a(self.light_bucket, self.sound_bucket, self.presence, ...)
    }

    fn cosine_similarity(&self, other: &Self) -> f32 {
        // Compare two context snapshots for warm-start seeding
        dot(self.feature_vec(), other.feature_vec())
    }
}
```

### Step 3 — Interaction is scored

If the context window contains a qualifying interaction (sustained proximity,
approach, touch, attention hold), the adapter calls:

```rust
field.positive_interaction(&context_key, &personality, tick, alone);
```

If the window contains a rejection signal (sharp retreat, cover, physical push):

```rust
field.negative_interaction(&context_key, &personality, tick);
```

CCF's `CoherenceField` updates the accumulated coherence score for this context,
applies homeostatic decay to all contexts, and recomputes the current `SocialPhase`.

### Step 4 — SocialPhase governs the response

The tile running the CCF brain emits `(effective_coherence, SocialPhase)` to the
LLM inference tile on every prompt call. The LLM sees the user's relational history
as two values — no retraining, no model changes.

The Cognitum Coherence Gate (`permit_action`) checks `SocialPhase` and `tension`
before allowing any external side-effect (speech output, motor command, API call).
`ProtectiveGuardian` state raises the gate threshold; `ShyObserver` state lowers
output intensity even on `Permit` verdicts.

---

## Tile Assignment for CCF Workloads

The v0 appliance has 7 tiles. A sensible CCF allocation:

| Tile | Workload |
|------|----------|
| 0 | Sensor ingestion + ContextKey construction |
| 1 | CoherenceField update + SocialPhase classification |
| 2 | Coherence Gate (TileZero / permit arbiter) |
| 3 | LLM inference |
| 4 | CCF_SEG persistence (`.rvf` read/write) |
| 5–6 | Available for application logic, audio, vision |

This leaves two tiles free for the robot's application layer (game logic, sorting
control, voice synthesis) while the CCF and gate functions run without competing
for compute.

---

## mBot2 as a Sensor Node

The mBot2 connects to the appliance via Gigabit Ethernet or USB. It streams sensor
frames through the expansion port at roughly 50 Hz (20 ms cadence), covering:

- **Proximity**: ultrasonic rangefinder (front), IR side sensors
- **Light**: ambient light sensor (top panel)
- **Sound**: microphone (records presence, not content)
- **IMU**: 6-axis (accel + gyro) for roll, pitch, motors_active
- **Motor encoders**: left/right velocity → motors_active bool

At 50 Hz and 33 bytes per frame, mBot2 generates about 1,650 bytes per second of
sensor data. This is well within the port's capacity. The CCF sensor tile processes
each frame in under 1 ms; the CoherenceField update runs in under 2 ms.

The end-to-end latency from a physical proximity event on the mBot2 to a
`SocialPhase` update available to the LLM tile is under **5 ms** on the v0
appliance. For comparison, a cloud round-trip for the same computation would be
50–200 ms.

---

## Persistence Across Power Cycles

CCF state — accumulated coherence scores, decay parameters, SocialPhase thresholds
— is stored in a `.rvf` cognitive container via the `CCF_SEG` segment. The `.rvf`
file lives on the appliance's local storage and is loaded by tile 4 on every boot.

This means:

- A user who achieved `QuietlyBeloved` phase yesterday wakes to a device that still
  knows them.
- A guest who approaches an established user's device starts from `ShyObserver`, not
  from the owner's accumulated state.
- A power outage does not reset six months of earned trust.

The `.rvf` format supports copy-on-write branching: you can snapshot the CCF state,
test new Personality parameters against the snapshot, and roll back without touching
the live state.

---

## Air-Gapped Operation

CCF's offline-first invariant (ARCH-005) and the Cognitum appliance's no-internet
design are a direct match. Once configured:

- No sensor data leaves the premises
- No coherence state is transmitted to a cloud server
- No model updates require a network call
- The Cognitum self-learning firmware adjusts tile weights from local interaction
  history, not from uploaded telemetry

This makes the appliance suitable for healthcare, education, and home deployments
where privacy is a non-negotiable requirement.

---

## What Is Not Yet Documented

The following details are not available in the public Cognitum introduction materials
and will need to be confirmed with the Cognitum team before implementation:

- Exact sensor port connector type (USB-C, RJ45, proprietary)
- Frame format specification (33-byte layout above is inferred from protocol docs)
- Tile ABI for loading WASM sensor adapters
- `.rvf` write API from within a running tile
- Per-tile memory limits (32 MB stated; whether this is shared or per-tile TBD)
- Authentication model between the sensor port and the gate tile

These details are available in the Cognitum *V0 Appliance* technical deck (hardware
architecture, protocol stack, silicon benchmarks) and the *Agentic OS* deck (agent
runtime, MCP integration). Requesting those decks from the Cognitum team is the
next step before writing production sensor adapter code.
