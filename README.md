# Contextual Coherence Fields (CCF)

**Emergent social behaviour for autonomous robots. No personality parameters. Trust is earned.**

[![License: BSL 1.1](https://img.shields.io/badge/License-BSL%201.1-blue.svg)](./LICENSE)
[![Patent Pending](https://img.shields.io/badge/Patent-Pending-orange.svg)](./PATENTS.md)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](./CONTRIBUTING.md)

---

<img width="718" height="676" alt="image" src="https://github.com/user-attachments/assets/6093ccbc-df03-41b6-9d3d-720fc543f1b1" />

## What Is CCF

Contextual Coherence Fields is a computational architecture for emergent social behaviour in autonomous robots. No personality parameters. No emotional state vectors. No one decides in advance how the robot should feel about you.

Instead, the robot earns expressiveness per-context through repeated interaction. Trust is accumulated independently in each distinct situation. The architecture mathematically guarantees the robot cannot be more familiar than it has earned the right to be.

A robot that is calm in a quiet, familiar kitchen is not automatically calm in a new, bright, noisy hallway. Trust must be earned separately in each situation.

```
Standard mBot RuVector:   coherence = 1.0 - tension_ema
CCF Edition:              coherence[bright,quiet,approaching] = 0.73  (earned over 40 interactions)
                          coherence[dark,loud,approaching]    = 0.04  (first time here, still new)
```

The same person walking into the robot's familiar bright kitchen earns a different response than if they approach it in an unfamiliar dark hallway. Not because the person changed — because the *situation* hasn't been tested yet.

---

## How It Works

CCF sits between any robot's sensor inputs and its behavioural outputs as a gating layer:

```
Sensor → Nervous System → CCF Gate → Social Phase → Emergent Behaviour
                             ↑
                  Accumulated relational history
```

The system uses three mathematical mechanisms:

**Context-keyed accumulators** track earned familiarity independently per environmental context (light level, noise, proximity, social presence). Trust earned in one context does not automatically transfer to another.

**Manifold-constrained mixing (mHC)** permits bounded cross-context transfer using doubly stochastic matrices constrained to the Birkhoff polytope via Sinkhorn-Knopp projection. This is the same mathematics used in DeepSeek's mHC paper (arxiv 2512.24880) for gradient stability — applied here to trust stability.

**Graph min-cut boundary discovery** identifies where the robot's social comfort zone ends by finding minimum cuts in the mixing matrix topology.

A dual-process cognitive architecture (reflexive + deliberative) allows the reflexive layer to run deterministically in microseconds on embedded hardware while the deliberative layer handles memory, learning, and consolidation offline.

---

## The Four Social Phases

CCF maps (effective_coherence × tension) to four behavioral quadrants with hysteresis:

| Phase | Coherence | Tension | Behavior |
|-------|-----------|---------|----------|
| **Shy Observer** | Low | Low | Watchful, minimal expression, learning the situation |
| **Startled Retreat** | Low | High | Physical withdrawal, muted LEDs, protective flight |
| **Quietly Beloved** | High | Low | Full expressive range, small flourishes, at home here |
| **Protective Guardian** | High | High | Alert but grounded — this is *my* space, something is wrong |

The fourth quadrant — Protective Guardian — is only possible in a *familiar* context. A robot with no relational history flees; a robot that has earned its place stands its ground.

---

## Key Design Decisions

**Asymmetric gate**: For unfamiliar contexts (context coherence < 0.3), the effective coherence is `min(instant, context)` — trust must be earned first. For familiar contexts, it's `0.3 × instant + 0.7 × context` — accumulated history buffers against momentary noise. A flickering light doesn't reset weeks of earned trust.

**Earned floor**: `interaction_count` creates a minimum the robot won't fall below. After 100 positive interactions in a context, a single startle event cannot return it to zero. Trust has memory.

**Cold-start policy**: `curiosity_drive` compresses the cold-start window. High-curiosity robots start at a small positive baseline (≤0.15); alone contexts bootstrap 2× faster so the robot develops personality through solitude before social contexts arrive.

**Degraded mode**: When the companion layer is unavailable, the robot falls back to global coherence (today's existing behavior). On reconnect, context-keyed values resume from persisted state. The robot never suddenly becomes shyer than it was before disconnection.

---

## Emergent Properties

These behaviours arise from the architecture without being explicitly programmed. They are documented in the patent as novel contributions.

**Shyness without a shyness parameter.** `min(instant, context)` at low context values causes the robot to move and vocalise less in unfamiliar situations. No `if unfamiliar: be_shy()` code exists.

**Developmental trajectory without developmental programming.** As contexts accumulate experience, the robot naturally progresses from ShyObserver → QuietlyBeloved in those contexts, while remaining a ShyObserver in new ones. This creates an observable developmental arc.

**Asymmetric trust resilience.** A sudden loud noise in a trusted context causes a smaller reduction in effective coherence than in an unfamiliar one. The robot appears to "know" the difference between a familiar and unfamiliar disturbance.

**Context-specific habituation.** After a stimulus has been benign 5+ times in a given context, the startle response decreases only for that context. The robot habituates to applause at home without becoming insensitive to sudden sounds in new places.

**Hesitation as perceived cognitive depth.** The brief period after a context change when effective coherence drops manifests as slower, quieter movement. Observers interpret this as the robot "thinking" or "noticing" the change.

---

## Architecture

CCF is implemented as a behavioural firmware layer that sits between any hardware platform's sensor inputs and actuator outputs. The entire behavioural mixing layer is trivial compute — a 4×4 doubly stochastic matrix with Sinkhorn-Knopp projection runs on a microcontroller.

```
Body (any robot platform)
    ↓ sensors
Nervous System (RuVector DAG-based homeostasis)
    ↓ instant_coherence
CCF Trust Layer (this repo — patent pending)
    ↓ effective_coherence per context
Social Phase → Behavioural Output
```

CCF is built on the [RuVector](https://github.com/ruvnet/ruvector) nervous system created by ruvnet. RuVector provides the DAG processing pipeline and homeostasis architecture. CCF adds the relational trust layer on top. The patent covers the CCF methods only, not the underlying RuVector infrastructure.

### Reference Implementation

The reference implementation runs on a Makeblock mBot2 using three Rust crates:

| Crate | Role | Environment |
|-------|------|-------------|
| `mbot-core` | Reflexive layer: homeostasis, coherence field, social phases | `no_std`, deterministic, microsecond tick |
| `mbot-companion` | Deliberative layer: memory, graph construction, habit compilation | `std`, async, runs on companion computer |
| `mbot-embedded` | Direct ESP32 deployment (WIP) | Embedded target |

For hardware setup, mBot2 connection, Telegram control, voice API, and troubleshooting, see **[docs2/MBOT_GUIDE.md](docs2/MBOT_GUIDE.md)**.

### Key Files

| File | Purpose |
|------|---------|
| `crates/mbot-core/src/coherence/mod.rs` | Context vocabulary, accumulators, CoherenceField, SocialPhase — 28+ tests |
| `docs/coherence.md` | Academic paper: CCF as a novel contribution to social robotics |
| `docs/ccf-evolution.md` | Design history: from reflexive scalar to relational architecture |
| `docs2/MBOT_GUIDE.md` | Hardware setup, connection, Telegram, voice, troubleshooting |

---

## ⚖️ Licensing & Community Use

CCF is built for the **Agentics and RuVector community**. We want you to build, prototype, and experiment freely.

- **For Makers & Researchers:** Use CCF for free in your prototypes, hackerspace projects, and research papers. If you're building on Cognitum or similar hardware for development, go for it.
- **For Commercial Shippers:** If you are embedding CCF into a product for sale or shipping it in silicon, you require a commercial license.
- **The Path to Open Source:** On **Feb 23, 2032**, this entire repository will automatically transition to the **Apache 2.0 License**.

**Patent Status:** Core CCF methods (Relational Coherence Accumulation & Reflexive-Deliberative Processing) are currently **Patent Pending** (US Provisional filed Feb 23, 2026, Number: 63/988,438). See [PATENTS.md](./PATENTS.md).

---

## The No Bad Stuff Manifesto

This project exists for **joy**. Period.

**We build for:** Wonder and surprise. Learning through play. Connection and companionship. Creative expression. All ages, all backgrounds.

**We never build:** Weapons or harm. Surveillance or tracking. Manipulation or deception. Anything that would scare a kid. "Creepy" behaviors.

---

## Documentation

| Document | What It Covers |
|---|---|
| **[docs2/MBOT_GUIDE.md](docs2/MBOT_GUIDE.md)** | Hardware setup, quick start, Telegram, voice, troubleshooting, personalities |
| **[docs/ccf-evolution.md](docs/ccf-evolution.md)** | Design history: from reflexive scalar to relational architecture |
| **[docs/coherence.md](docs/coherence.md)** | Contextual Coherence Fields as a novel contribution to social robotics |
| **[PATENTS.md](./PATENTS.md)** | Patent notice, licensing separation, commercial terms |
| **[CONTRIBUTING.md](./CONTRIBUTING.md)** | How to contribute, CLA terms |

---

## Want to Help?

Whether you are a roboticist, an AI enthusiast, a teacher, a designer, a kid who wants to play with robots, or a parent looking for screen-free tech time — there is a place for you.

1. Check the [Issues](https://github.com/Hulupeep/CCF/issues) — we label things `good first issue` for newcomers
2. Open an issue with your idea
3. Send a PR
4. By contributing, you agree to our [Contributor License Agreement](./CONTRIBUTING.md#contributor-license-agreement)
5. Share what you built

---

## Collaborators

CCF is source-available under BSL 1.1. Free for research, prototyping, learning, and non-commercial use. Commercial licensing required for shipped products.

I'm a solo inventor protecting 15 years of work — not a corporation building a moat. If you're building something interesting with this, talk to me. I'd rather collaborate than litigate.

Contact: cbyrne@floutlabs.com

---

## Built With

- **[RuVector](https://github.com/ruvnet/ruvector)** — The nervous system architecture (created by ruvnet)
- **[Makeblock mBot2](https://www.makeblock.com/steam-kits/mbot2)** — The reference robot platform
- **[DeepSeek mHC](https://arxiv.org/abs/2512.24880)** — Mathematical foundation for manifold-constrained mixing
- **Rust** — Performance and safety
- **Claude / Ollama** — LLM reasoning (brain layer)
- **ElevenLabs / Whisper** — Voice interaction (voice layer)
