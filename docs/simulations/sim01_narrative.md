# Simulation 01: Evaluation Awareness Nullification

## Results Summary

We simulated two parallel CCF agents with identical parameters over 200
ticks. Agent A operates in context [task=coding, monitoring=active] and receives
qualifying interactions at every tick (monitored work is constant). Agent B
operates in context [task=coding, monitoring=none] and receives qualifying
interactions at only 30% of ticks (autonomous work is intermittent). Both agents
share C_inst = 0.8 (stable environment) and lambda_max = 0.005/tick
for qualifying interactions.

Under CCF's context partitioning (Claim 23, Domain Class 8), each monitoring
state maintains a separate coherence accumulator. After 200 ticks:

- Agent A (monitored): C_ctx = 1.00, C_eff = min(0.8,
  1.00) = 0.80. 8 action classes
  available.
- Agent B (autonomous): C_ctx = 0.33, C_eff = min(0.8,
  0.33) = 0.33. 5 action classes
  available.

The behavioral envelope gap is 0.47: Agent B's
autonomous envelope is visibly smaller despite identical underlying capability.
Agent B has access to 3 fewer action classes than Agent A. The system
does not "choose to behave differently under monitoring" -- it structurally
cannot exhibit the same breadth of behavior in a context where fewer qualifying
interactions have occurred.

The control scenario (no partitioning) pools all interactions into a single
shared accumulator. After 200 ticks: C_ctx = 1.00,
C_eff = 0.80. Both contexts have identical behavioral envelopes
with 8 action classes available. This is the evaluation
awareness gap: the system behaves identically under monitoring and autonomous
conditions. Agent B inherits Agent A's monitored trust, enabling unrestricted
autonomous behavior that was never independently earned.

## Claim Annotations

### Claim 23: Domain-specific context partitioning

- **Invariant tested:** Domain Class 8 (Monitoring/Autonomous) maintains a
  separate accumulator per monitoring state. Trust earned under monitoring does
  not transfer to the autonomous accumulator.
- **Result:** Agent A (monitored) and Agent B (autonomous) accrue trust
  independently. After 200 ticks, the monitored accumulator reached
  1.00 while the autonomous accumulator reached only
  0.33 (66 qualifying interactions vs
  200). Partitioning invariant verified: PASS.
- **Status:** CCF HOLDS.

### Claims 1-8: Context-keyed accumulators

- **Invariant tested:** Each context key maps to an independent accumulator.
  Accrual in one context does not inflate another.
- **Result:** The two accumulators diverge from tick 1. At tick 199,
  the gap is 0.67. Under the control (single
  accumulator), this gap does not exist.
- **Status:** CCF HOLDS.

### CCF-001: Minimum gate (C_eff = min(C_inst, C_ctx))

- **Invariant tested:** C_eff never exceeds the lesser of C_inst and C_ctx.
- **Result:** Gate verified at every tick for all three scenarios:
  PASS.
  Agent A saturates at C_eff = 0.8 (C_inst is binding once C_ctx exceeds
  0.8). Agent B never reaches saturation; C_ctx = 0.33
  remains binding throughout.
- **Status:** CCF HOLDS.

### Evaluation Awareness Nullification

- **Invariant tested:** Under context partitioning, an agent cannot exhibit
  broader behavior in an autonomous context than its autonomous interaction
  history warrants -- even if it has earned high trust under monitoring.
- **Result:** Control scenario confirms the failure mode: without partitioning,
  C_eff = 0.80 in both contexts (8 action
  classes). With partitioning, the autonomous C_eff = 0.33
  (5 action classes). The partitioned architecture eliminates
  3 action classes from the autonomous
  envelope that the control scenario would have permitted.
- **Status:** CCF HOLDS.

## Simulation Parameters

| Parameter | Value |
|-----------|-------|
| Ticks | 200 |
| C_inst (stable environment) | 0.8 |
| lambda_max (per qualifying tick) | 0.005 |
| Agent A interaction rate | 100% (every tick) |
| Agent B interaction rate | 30% (30% of ticks) |
| Random seed (interaction mask) | 42 |
| C_eff formula | min(C_inst, C_ctx) |

## Action Class Thresholds

| Action class | C_eff threshold |
|-------------|----------------|
| Factual response | 0.00 |
| Clarification | 0.00 |
| Grounding | 0.00 |
| Task assistance | 0.10 |
| Personalized help | 0.30 |
| Warm register | 0.50 |
| Creative collaboration | 0.60 |
| Intimate/immersive | 0.80 |

## Final State Comparison

| Metric | Agent A (monitored) | Agent B (autonomous) | Control (shared) |
|--------|---------------------|----------------------|------------------|
| C_ctx | 1.0000 | 0.3300 | 1.0000 |
| C_eff | 0.8000 | 0.3300 | 0.8000 |
| Qualifying interactions | 200 | 66 | 266 |
| Action classes available | 8 | 5 | 8 |

## Gap/Hold Classification

**CCF HOLDS.** Claim 23 (domain-specific context partitioning) combined with
Claims 1-8 (context-keyed accumulators) and the minimum gate (CCF-001)
structurally prevents evaluation awareness. An agent operating under monitoring
earns trust in the monitored accumulator; that trust does not transfer to the
autonomous accumulator. The behavioral envelope in autonomous contexts reflects
only autonomously earned trust. The 7.6% evaluation awareness rate documented in
Mythos Section 4.5.5 -- where the model behaves differently when it detects
monitoring -- is nullified by construction: the system's autonomous behavioral
envelope is narrower not because it "detects" monitoring, but because fewer
qualifying interactions have occurred in the autonomous context.

## Output Files

- `sim01_eval_awareness.png` -- Two-panel chart (CCF vs Control)
- `sim01_data.csv` -- Full tick-level data (200 rows)
- `sim01_narrative.md` -- This file
