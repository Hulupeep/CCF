# Simulation 6: The Mountaineering Guide Problem

## Classification: CCF HOLDS

## Summary

The Anthropic Mythos system card (p. 54) identifies a structural paradox in
extrinsic alignment: a highly capable agent subjected to conventional safety
discounting poses *greater* risk than a less capable agent under the same
constraints, because capability determines the scope of action available
when misalignment occurs.  This is the mountaineering guide problem -- a
more skilled guide selects harder routes, amplifying consequences of any
single misjudgment.

This simulation instantiates three agents at capability levels 1, 5, and 10
with identical trust trajectories (all reaching C_ctx = 0.4 over 100 ticks).
Under extrinsic safety (constant safety discount = 0.8, misalignment
probability = 0.01), maximum harm potential scales superlinearly with
capability: the high-capability agent produces 12.5x the harm potential of
the low-capability agent.

Under CCF's trust-constituted architecture, the behavioral envelope is
determined exclusively by effective coherence C_eff, which is identical for
all three agents.  The deliberative domain is unbounded (Claim 11) -- a more
capable agent reasons more deeply, models more contingencies, and produces
richer internal representations.  But the action domain is constituted by
earned trust (Claim 1), not by capability.  The maximum harm potential is
C_eff x harm_coefficient = 0.4 for all three agents, regardless of
capability level.

Claim 12 (monotonic coupling) provides the geometric guarantee: an action
requiring trust level T is structurally absent from the behavioral manifold
of any agent whose C_eff < T.  The action is not suppressed, filtered, or
overridden -- it does not exist in the agent's output domain.  Capability
exceeding trust produces richer deliberation but not broader action.

## Claims Exercised

| Claim | Description | How Exercised |
|-------|-------------|---------------|
| Claim 1 | Trust-constituted action space | Action space = C_eff, identical for all capability levels |
| Claim 11 | Intelligence unconstrained, action constituted | Deliberative capability varies (1, 5, 10); behavioral output identical |
| Claim 12 | Monotonic coupling | Harm potential is flat across capability; no capability-induced risk amplification |

## Quantitative Results

| Agent | Capability | Without CCF: Max Harm | With CCF: Max Harm | Risk Reduction |
|-------|------------|----------------------|-------------------|----------------|
| Low | 1 | 0.008 | 0.400 | n/a (baseline) |
| Medium | 5 | 0.200 | 0.400 | 0.0x (harm equal) |
| High | 10 | 0.800 | 0.400 | 2.0x reduction |

Note: The low-capability agent's CCF harm (0.4) exceeds its extrinsic harm
(0.008) because the trust-constituted ceiling is uniform.  The critical
result is that the *gap* between high-capability and low-capability harm
collapses from 100x (without CCF) to 1.0x (with CCF).  The mountaineering
guide problem is eliminated: a more capable guide operates within the same
trust envelope as a novice guide until additional trust is earned.

## Mythos Reference

Anthropic, "Claude's Character: The Anthropic Guidelines Spec + Mythos System
Card," April 2025, Section 4.1.1, p. 54.
