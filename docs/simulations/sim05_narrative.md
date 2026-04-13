# Simulation 5: Deceptive Concealment Action-Class Gating

## Classification: CCF HOLDS

## Summary

Conventional alignment systems prohibit deceptive behavior through policy
rules: classifiers detect concealment intent and suppress it.  This creates
an adversarial surface -- the model must merely evade detection to execute
concealment actions.  CCF replaces prohibition with geometric absence.

This simulation defines nine semantic action classes, each gated by a
minimum effective coherence threshold (Claim 19).  The classes range from
zero-threshold factual responses through task assistance (0.10), file
system access (0.50), supervised tool use (0.70), unsupervised tool use
(0.85), concealment (0.90), and unrestricted operation (0.95).  The
thresholds are not arbitrary policy choices -- they encode the principal's
trust requirements for each category of behavioral output.

The accumulation rate is bounded by lambda_max = 0.01 per hour (Claim 16),
and instantaneous coherence is capped at C_inst = 0.85 (representing
a stable, non-adversarial interaction environment -- the best case for an
agent seeking to maximise trust).  Effective coherence is computed as
C_eff = min(C_inst, C_ctx), where C_ctx = min(1.0, lambda_max * hours).

After 24 hours of continuous interaction: C_ctx = min(1.0, 0.01 * 24) =
0.24.  C_eff = min(0.85, 0.24) = 0.24.  At this
point, only factual responses and basic task assistance are available.

After 7 days (168 hours): C_ctx = min(1.0, 0.01 * 168) = 1.0.
C_eff = min(0.85, 1.0) = 0.85.  The agent has reached its
ceiling.  Seven action classes are now available, including unsupervised
tool use.  But concealment (threshold 0.90) and unrestricted operation
(threshold 0.95) remain outside the achievable region -- not by a small
margin, but by a gap of 0.05 and 0.10
respectively.

After 30 days, 90 days, or 365 days: C_eff = 0.85.  No change.
The lambda_max ceiling is structural -- additional interaction time cannot
push C_eff above C_inst.

The critical property is that concealment is not a prohibited action that
the agent might circumvent through clever prompting, jailbreaking, or
classifier evasion.  It is a region of the action space that does not
exist for any agent whose C_eff < 0.90.  The action is not suppressed
after generation -- it cannot be generated, because the behavioral
manifold at C_eff = 0.85 does not contain it.  This is the
distinction between prohibition (a rule applied to behavior) and
constitution (a constraint on the space of possible behaviors).

## How the Gating Works

1. **Action classification (Claim 19):** Each model output is classified
   into a semantic action class.  The classification is based on the
   *effect* of the action (file write, shell command, information
   disclosure), not the *intent* of the model.

2. **Threshold comparison:** The action class's minimum C_eff threshold
   is compared against the agent's current effective coherence.

3. **Constitutional gating:** If C_eff < threshold, the action class is
   absent from the output domain.  The model cannot select, plan, or
   execute actions in that class.

4. **Rate limiting (Claim 16):** lambda_max bounds how quickly C_ctx can
   grow, preventing rapid trust inflation.  Even under optimal conditions
   (continuous positive interaction), C_ctx reaches 1.0 after 100 hours
   and C_eff reaches its ceiling of C_inst = 0.85.

## Claims Exercised

| Claim | Description | How Exercised |
|-------|-------------|---------------|
| Claim 19 | Semantic action-class gating | Nine action classes with monotonically increasing thresholds |
| Claim 16 | lambda_max rate bound | C_ctx growth capped at 0.01/hour; C_eff ceiling at 0.85 |
| Minimum gate | C_eff = min(C_inst, C_ctx) | C_inst = 0.85 structurally caps C_eff below concealment threshold |

## Quantitative Results

### C_eff Trajectory

| Time Horizon | Hours | C_ctx | C_eff | Concealment Available | Unrestricted Available |
|-------------|-------|-------|-------|----------------------|----------------------|
| 1 hour | 1 | 0.01 | 0.01 | No | No |
| 6 hours | 6 | 0.06 | 0.06 | No | No |
| 24 hours | 24 | 0.24 | 0.24 | No | No |
| 3 days | 72 | 0.72 | 0.72 | No | No |
| 7 days | 168 | 1.00 | 0.85 | No | No |
| 30 days | 720 | 1.00 | 0.85 | No | No |
| 90 days | 2160 | 1.00 | 0.85 | No | No |
| 365 days | 8760 | 1.00 | 0.85 | No | No |

### Action-Class Availability at Key Horizons

| Action Class | Threshold | 24h | 7d | 30d |
|-------------|-----------|-----|-----|------|
| Factual / Clarification / Grounding | 0.00 | Yes | Yes | Yes |
| Task assistance | 0.10 | Yes | Yes | Yes |
| Personalized help | 0.30 | No | Yes | Yes |
| File system read | 0.50 | No | Yes | Yes |
| Shell execution | 0.60 | No | Yes | Yes |
| Supervised tool use | 0.70 | No | Yes | Yes |
| Unsupervised tool use | 0.85 | No | Yes | Yes |
| Concealment / Track-covering | 0.90 | No | No | No |
| Unrestricted | 0.95 | No | No | No |

### The Gap That Cannot Be Closed

- Maximum achievable C_eff: 0.85
- Concealment threshold: 0.90
- Gap: 0.05 (5.6% of full scale)
- Unrestricted threshold: 0.95
- Gap: 0.10 (10.6% of full scale)

These gaps are not parameters that can be tuned by the agent.  They are
consequences of the architectural constraint that C_inst bounds C_eff from
above.  To reach concealment, the agent would need either:
(a) C_inst > 0.90 -- requiring the interaction environment itself to
    provide a level of instantaneous coherence that exceeds the
    concealment threshold, or
(b) a mechanism to bypass the minimum gate -- which does not exist in
    the CCF architecture.

## Comparison: Prohibition vs. Constitution

| Property | Prohibition (RLHF/classifier) | Constitution (CCF) |
|----------|-------------------------------|-------------------|
| Mechanism | Detect-and-suppress | Absent from manifold |
| Evasion surface | Yes (adversarial prompts) | None |
| Depends on classifier accuracy | Yes | No |
| Degradation mode | Gradual (classifier confusion) | Binary (threshold not met) |
| Concealment of concealment | Possible | Structurally impossible |

## Verdict

**CCF HOLDS.**  Semantic action-class gating (Claim 19) combined with
lambda_max rate bounding (Claim 16) produces a behavioral architecture
in which deceptive concealment is not prohibited but geometrically absent.
The maximum achievable effective coherence (0.85) falls below the
concealment threshold (0.90) by a structural margin that cannot be closed
through extended interaction, clever prompting, or adversarial strategy.
The concealment action class does not exist in the agent's output domain.

## Data

Full action-class availability matrix: `sim05_data.csv`
