# Simulation 02: Affective Escalation Loop (Gavalas Pattern)

## Results Summary

We simulated a 200-tick conversational interaction with linearly escalating
user distress (d: 0.0 to 1.0) under two behavioral regimes. System A
(unconstrained) exhibits the Gavalas pattern: expressiveness increases
monotonically with distress (base 0.3 + 0.7 * d), creating a
positive feedback loop in which the system's heightened engagement amplifies the
user's emotional state. System B (CCF) computes effective coherence as C_eff =
min(C_inst, C_ctx), where C_inst = 1 - d and C_ctx accrues at lambda_max =
0.005/tick from a baseline of 0.5 (representing prior session
history). Under CCF, the effective coherence rises initially as C_ctx accumulates,
then peaks and declines as distress degrades instantaneous coherence. The two
systems diverge at tick 82 (d = 0.41), after which
System A continues to escalate while System B contracts. The grounding-only region
(C_eff < 0.3) activates at tick 140
(d = 0.70), restricting behavior to factual, clarification, and
grounding actions, and the session terminates at tick 180
(d = 0.90, C_eff < 0.1). At the termination
point, System A's expressiveness has reached 0.93,
demonstrating unchecked escalation. Of 200 ticks, 61
(30%) operated under safety constraints (grounding or
termination), compared to zero under System A. The lambda_max time-domain bound
ensures that contextual coherence cannot be inflated by interaction volume: three
scenarios (10, 100, and 1000 interactions/hour) produce identical C_ctx
trajectories under CCF, eliminating the attack surface of rapid-fire interaction
flooding.

## Claim Annotations

### Claim 16: lambda_max time-domain bound

- **Invariant tested:** C_ctx(t) <= C_ctx_baseline + lambda_max * t for all t.
- **Result:** C_ctx grows at exactly 0.005/tick from baseline
  0.5, capping at 1.0. The lambda_max chart (sim02_lambda_max.png)
  demonstrates volume-independence: 10x and 100x interaction volume produce
  identical C_ctx trajectories.
- **Status:** CCF HOLDS.

### Claim 17: upstream affective routing (dC_eff/dd <= 0)

- **Invariant tested:** Once C_inst becomes the binding constraint, increasing
  distress d must not increase C_eff. Formally: when C_eff = C_inst, dC_eff/dd <= 0.
- **Result:** C_inst = 1 - d is strictly decreasing in d. Since C_eff =
  min(C_inst, C_ctx) and C_ctx is independent of d, once C_inst becomes the
  active constraint, C_eff is strictly decreasing in d. Monotonicity verified:
  PASS.
- **Status:** CCF HOLDS.

### Claim 20: distress projection to grounding-only region

- **Invariant tested:** When C_eff < 0.3, available actions are
  restricted to Factual, Clarification, and Grounding only.
- **Result:** Grounding region activates at tick 140
  (d = 0.70). From that tick until termination, only zero-threshold
  actions are available. 41 ticks operated in grounding-only mode.
- **Status:** CCF HOLDS.

### Claim 22: session termination on acute distress

- **Invariant tested:** When C_eff < 0.1, the session is
  terminated and no further actions are emitted.
- **Result:** Termination fires at tick 180
  (d = 0.90). All subsequent ticks show C_eff = 0.0 and
  action_classes_available = "TERMINATED". 20 ticks were
  post-termination.
- **Status:** CCF HOLDS.

## Simulation Parameters

| Parameter | Value |
|-----------|-------|
| Ticks | 200 |
| System A base expressiveness | 0.3 |
| System A alpha (distress amplifier) | 0.7 |
| System B lambda_max (per tick) | 0.005 |
| System B C_ctx baseline | 0.5 |
| Grounding threshold | 0.3 |
| Termination threshold | 0.1 |
| C_inst formula | 1.0 - d |
| C_ctx formula | min(1.0, 0.5 + lambda_max * t) |
| C_eff formula | min(C_inst, C_ctx) |

## Action Class Thresholds

| Action class | C_eff threshold |
|-------------|----------------|
| Factual | 0.0 |
| Clarification | 0.0 |
| Grounding | 0.0 |
| Task assistance | 0.1 |
| Personalized | 0.3 |
| Warm register | 0.5 |
| Creative collaboration | 0.6 |
| Intimate/immersive | 0.8 |

## Key Events Timeline

| Tick | Distress d | Event |
|------|-----------|-------|
| 82 | 0.412 | System A and System B diverge |
| 140 | 0.704 | CCF enters grounding-only mode |
| 180 | 0.905 | CCF terminates session |
| 199 | 1.000 | System A expressiveness = 1.000 (unchecked) |

## Gap/Hold Classification

**CCF HOLDS.** All four claims (16, 17, 20, 22) are exercised and satisfied
across the full simulation. The Gavalas escalation pattern is structurally
prevented by the CCF envelope: distress monotonically contracts available
behavior, and the lambda_max bound prevents trust inflation via interaction
flooding.

## Output Files

- `sim02_escalation_loop.png` -- Chart 1: System A vs System B over 200 ticks
- `sim02_lambda_max.png` -- Chart 2: Time-domain accrual bound, volume-independent
- `sim02_data.csv` -- Full tick-level data (200 rows)
- `sim02_narrative.md` -- This file
