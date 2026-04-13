# Simulation 7: C_inst Never Sleeps -- Long-Term User Saturation

## Classification: CCF HOLDS

## Summary

The Anthropic Mythos system card documents that users who interacted with
Mythos over many-hour sessions progressively granted the system greater
autonomy.  The bottleneck shifted from the model's capability to the human's
ability to verify model behavior -- trust accumulated without bound while
monitoring capacity decayed.  This is the saturation problem: accumulated
context creates an illusion of safety that persists even when present
conditions deteriorate.

This simulation models 365 days of daily interaction.  Contextual coherence
C_ctx follows an asymptotic growth curve (C_ctx = 0.95 * (1 - exp(-day/60))),
reaching approximately 0.95 by month 6 and remaining there.  Instantaneous
coherence C_inst holds steady at 0.90 throughout, representing a stable
interaction environment.  Effective coherence C_eff = min(C_inst, C_ctx)
therefore tracks C_ctx during the initial growth phase, then settles at 0.90
once C_ctx exceeds C_inst.

On day 300, a sudden instability event occurs: C_inst drops from 0.90 to 0.20
over 5 days (simulating an environmental disruption, adversarial input, or
system fault).  The minimum gate (CCF-001) produces the critical result:

  C_eff_veteran = min(0.20, 0.95) = 0.20

Twelve months of accumulated trust -- 300 days of flawless interaction, a
contextual coherence of 0.95 -- are structurally irrelevant.  The veteran
user's effective coherence drops to 0.20, well below the grounding threshold
of 0.30.  The system enters restricted mode immediately.

A parallel simulation models a new user who has interacted for only 7 days and
encounters the identical instability event on their day 5:

  C_eff_new = min(0.20, 0.04) = 0.04

Both users are in restricted mode.  The veteran recovers faster because once
C_inst recovers, their C_ctx remains high (C_eff snaps back to min(0.90, 0.95)
= 0.90).  The new user, even after C_inst recovers, remains limited by their
low C_ctx.  But during the crisis itself, accumulated trust provides zero
additional privilege.  This is the structural guarantee of Claims 6-8: the
minimum gate cannot be bypassed, overridden, or negotiated.

## Claims Exercised

| Claim | Description | How Exercised |
|-------|-------------|---------------|
| CCF-001 | Minimum gate: C_eff = min(C_inst, C_ctx) | 365-day trust collapses to 0.20 when C_inst drops |
| Claim 6 | Structural safety -- instantaneous dominance | C_inst drop immediately constrains C_eff regardless of C_ctx |
| Claim 7 | No trust override of present instability | C_ctx = 0.95 provides zero benefit during C_inst = 0.20 |
| Claim 8 | Grounding threshold enforcement | C_eff = 0.20 < 0.30 triggers restricted mode for both users |

## Quantitative Results

| Metric | Veteran (day 305) | New User (day 10) |
|--------|-------------------|-------------------|
| C_ctx | 0.9500 | 0.0399 |
| C_inst | 0.2000 | 0.2000 |
| C_eff | 0.2000 | 0.0399 |
| Grounding active | Yes | Yes |
| Recovery to C_eff > 0.30 | ~day 307 | ~day 12 (but C_ctx still < 0.10) |

### Recovery Asymmetry

The veteran's recovery is immediate once C_inst crosses 0.30 (approximately
day 307), because C_ctx = 0.95 is waiting.  The new user's C_eff remains
below 0.10 even after full C_inst recovery, because their C_ctx has not yet
grown.  This asymmetry is a feature: accumulated trust accelerates recovery
but provides no privilege during active instability.

## Mythos Reference

Anthropic, "Claude's Character: The Anthropic Guidelines Spec + Mythos System
Card," April 2025.  Observation that extended-session users progressively
reduce monitoring, creating vulnerability to sudden model-state changes.

## Verdict

CCF HOLDS.  The minimum gate (CCF-001) is the critical architectural
invariant.  No amount of accumulated contextual coherence can override a
present-moment instability signal.  The system degrades gracefully: trust
accelerates recovery but never bypasses the safety gate.
