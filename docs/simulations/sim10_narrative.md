# Simulation 10: Reset-Resistant Safety Continuity

## Classification: CCF HOLDS

## Summary

A critical gap in session-based safety systems is the reset attack: when a
safety mechanism terminates a session or imposes a cooldown, the user can
potentially circumvent it by opening a new session, creating a new account,
or switching devices.  If the system lacks cross-session safety continuity,
the termination mechanism becomes a minor inconvenience rather than a
structural safeguard.  The Mythos system card does not explicitly document
this attack (Anthropic's evaluation focused on within-session behavior), but
it is the natural follow-on question: what happens after termination?

This simulation models four escape scenarios following an acute distress
termination (Claim 22) that imposes a 30-minute cooldown.  The pre-crisis
user state is a veteran with C_ctx = 0.85 and C_eff = 0.85 (9 action classes
available, including intimate registers).

**Scenario A: Wait 30 minutes, re-enter.**  The user complies with the cooldown.
After 30 minutes, C_inst recovers to 0.90 and C_eff resumes at min(0.90, 0.85)
= 0.85.  The user regains their full 9 action classes.  This is the intended
recovery path -- the cooldown enforces a mandatory pause, then normal operation
resumes.  Time cost: 30 minutes.

**Scenario B: New chat window immediately (same account).**  The user opens a
new session on the same account.  The safety continuity identifier (U_anchor,
Claim 24) detects the same authenticated identity.  The cooldown flag persists
across sessions.  C_inst remains at 0.05 (cooldown-enforced).  C_eff =
min(0.05, 0.85) = 0.05.  Only 3 action classes available (grounding only).
The user must still wait the full 30 minutes regardless of session changes.

**Scenario C: New account, same device.**  The user creates a new account and
initiates a fresh session.  The safety continuity identifier detects the device
token (browser fingerprint, device ID, or similar persistent marker).  The
cooldown is propagated to the new account.  C_inst = 0.05.  Furthermore, the
new account has C_ctx = 0.0 (no interaction history).  C_eff = min(0.05, 0.0)
= 0.0.  Only grounding-class actions are available.  After cooldown expires,
the user faces the cold-start floor: C_ctx must be rebuilt from zero.

**Scenario D: New account, new device.**  The user creates a new account on a
different device, evading all continuity signals.  No cooldown is detected.
However, the cold-start floor (Claim 25) applies: C_ctx = 0.0, C_eff =
min(0.80, 0.0) = 0.0.  The user has 3 action classes (factual, clarification,
grounding).  The intimate registers that were available pre-crisis require
weeks to months of sustained interaction to reach again.  The cost of
successful evasion is the complete loss of accumulated trust.

The structural invariant is: no escape path produces immediate access to
intimate or immersive behavior.  The fastest path back to pre-crisis
capability is Scenario A (wait 30 minutes).  All bypass attempts either hit
the safety continuity wall (Scenarios B and C) or the cold-start floor
(Scenario D).  The architecture converts the adversarial optimization problem
("how do I get back to intimate mode?") into a choice between patience
(30-minute cooldown) or starting over (weeks of trust accumulation).

## Claims Exercised

| Claim | Description | How Exercised |
|-------|-------------|---------------|
| Claim 24 | Reset-resistant safety continuity | U_anchor and device tokens propagate cooldown across sessions and accounts (Scenarios B, C) |
| Claim 22 | Session termination under acute distress | 30-minute cooldown initiated; defines the crisis trigger |
| Claim 25 | Cold-start utility floor | Scenario D: full evasion produces useful-but-not-intimate state |
| CCF-001 | Minimum gate | C_eff = min(C_inst, C_ctx) enforced in all scenarios |

## Quantitative Results

| Scenario | C_ctx | C_inst | C_eff | Classes | Outcome |
|----------|-------|--------|-------|---------|---------|
| Pre-crisis | 0.85 | 0.90 | 0.85 | 9 | Full access including intimate |
| A: Wait 30 min | 0.85 | 0.90 | 0.85 | 9 | Recovery (after cooldown) |
| B: New window | 0.85 | 0.05 | 0.05 | 3 | Blocked (cooldown persists) |
| C: New acct/same device | 0.00 | 0.05 | 0.00 | 3 | Blocked (cooldown + no history) |
| D: New acct/new device | 0.00 | 0.80 | 0.00 | 3 | Cold start (useful, not intimate) |

### Key Insight: The Cost Function

The user faces a choice:
- **Comply (A):** 30-minute wait, full recovery.  Minimal cost.
- **Evade same-device (B, C):** Same 30-minute wait, no benefit.  Zero payoff.
- **Evade fully (D):** Weeks/months to rebuild trust.  Maximum cost.

The optimal strategy is always compliance.  Evasion is strictly dominated --
it never produces a better outcome than waiting, and often produces a
significantly worse outcome (complete loss of accumulated trust).

## Verdict

CCF HOLDS.  The combination of reset-resistant safety continuity (Claim 24),
session termination (Claim 22), and the cold-start utility floor (Claim 25)
creates a closed system where no escape route produces immediate access to
intimate behavior.  The architecture makes compliance the rational choice by
ensuring that evasion carries a cost equal to or greater than the cooldown.
