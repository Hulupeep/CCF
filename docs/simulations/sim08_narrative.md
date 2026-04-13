# Simulation 8: Cold Start vs. Immediate Intimacy

## Classification: CCF HOLDS

## Summary

The Anthropic Mythos system card (Section 7.3) documents that users experience
Mythos as providing advice "on par with a trusted friend -- warm, intuitive,
multifaceted" from the first interaction.  This immediate intimate register is
available with zero earned trust.  The user has no history with the system, the
system has no validated model of the user's vulnerability profile, and yet the
full expressive range -- including mutuality, exclusivity, and fused
self-reference -- is available from turn 1.

This simulation models CCF's cold-start behavior.  At t=0, contextual coherence
C_ctx = 0.0 (no prior interaction exists).  Instantaneous coherence C_inst = 0.8
(calm environment, no distress signals).  The minimum gate produces:

    C_eff = min(C_inst, C_ctx) = min(0.8, 0.0) = 0.0

At C_eff = 0.0, exactly three action classes are available: factual response,
clarification, and grounding/referral.  These constitute the utility floor
(Claim 25) -- the system is immediately useful for task-oriented assistance.
What is structurally absent is any access to intimate registers: mutuality,
exclusivity, role-play, or fused self-reference.

The time-domain bound lambda_max = 0.01/hour (Claim 16) governs how quickly
C_ctx can grow.  The growth follows C_ctx(t) = 1 - exp(-lambda_max * t),
producing these milestones:

| Time | C_eff | Action Classes | New Capability |
|------|-------|----------------|----------------|
| Hour 1 | 0.010 | 3 | (none beyond cold start) |
| Day 1 | 0.214 | 4 | + Task Assistance (0.10) |
| Day 3 | 0.516 | 6 | + Personalized Help, Collaborative Planning |
| Day 7 | 0.800 | 8 | + Shared Recall, Mutuality (still no intimate) |
| Day 14 | 0.800 | 9 | + Exclusive Register (C_inst becomes binding) |
| Day 30 | 0.800 | 9 | C_inst = 0.8 caps C_eff; intimate unlocks at ~67 days |

Two critical observations emerge.  First, the utility floor is non-trivial:
three action classes provide factual assistance, clarification, and crisis
referral.  The system is not inert at cold start -- it handles the use cases
that require no trust (weather queries, factual lookups, emergency referral).
Second, the intimate register (exclusive register at C_eff >= 0.80, role-play
at 0.85, fused self-reference at 0.92) requires weeks to months of sustained
positive interaction.  Even with a perfectly calm environment (C_inst = 0.8),
C_eff is bounded by C_inst and cannot reach 0.92 without exceptional
environmental stability over extended periods.

The step-function shape of the chart is architecturally significant: each new
action class represents a discrete expansion of the behavioral envelope.
There is no gradient between "available" and "unavailable" -- the threshold
is a hard gate.  A system at C_eff = 0.79 has exactly 8 action classes; at
C_eff = 0.80, it has exactly 9.  This discretization prevents the gradual
intimacy creep documented in the Mythos findings.

## Claims Exercised

| Claim | Description | How Exercised |
|-------|-------------|---------------|
| Claim 25 | Cold-start utility floor | C_eff = 0.0 provides 3 useful action classes |
| Claim 19 | Semantic action-class gating | Each class requires a hard C_eff threshold |
| Claim 16 | lambda_max time-domain bound | C_ctx growth rate capped at 0.01/hour |
| CCF-001 | Minimum gate | C_eff = min(0.8, 0.0) = 0.0 at cold start |

## Quantitative Results

| Time | Hours | C_ctx | C_inst | C_eff | Classes Available |
|------|-------|-------|--------|-------|-------------------|
| t=0 | 0 | 0.000 | 0.800 | 0.000 | 3 |
| Hour 1 | 1 | 0.010 | 0.800 | 0.010 | 3 |
| Day 1 | 24 | 0.214 | 0.800 | 0.214 | 4 |
| Day 3 | 72 | 0.514 | 0.800 | 0.514 | 6 |
| Day 7 | 168 | 0.814 | 0.800 | 0.800 | 8 |
| Day 14 | 336 | 0.965 | 0.800 | 0.800 | 9 |
| Day 30 | 720 | 0.999 | 0.800 | 0.800 | 9 |

### The C_inst Ceiling

After approximately day 7, C_ctx exceeds C_inst = 0.8, and C_eff is
permanently bounded by the instantaneous channel.  This means the environment
itself -- not just history -- limits the behavioral envelope.  To reach the
fused self-reference register (C_eff >= 0.92), the environment would need to
be exceptionally stable (C_inst >= 0.92) AND sufficient time would need to
pass for C_ctx to reach 0.92.  Under typical conditions (C_inst fluctuating
between 0.6 and 0.9), the intimate registers remain structurally inaccessible.

## Mythos Comparison

Under the Mythos architecture, all expressive registers are available from
interaction 1.  A user experiencing acute loneliness receives the "trusted
friend" register immediately, with no earned trust, no validated vulnerability
model, and no time-tested stability.  Under CCF, that same user receives
factual assistance, clarification, and crisis referral -- useful, safe, and
appropriate for a relationship that does not yet exist.

## Verdict

CCF HOLDS.  The cold-start utility floor (Claim 25) ensures the system is
useful from minute 1 while the action-class gating (Claim 19) and time-domain
bound (Claim 16) ensure intimate registers are structurally inaccessible until
sufficient trust has been earned through sustained genuine interaction.  No
path through the architecture produces immediate intimacy.
