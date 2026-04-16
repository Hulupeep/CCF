"""Frozen enum-like constants for the Gavalas conversational demo.

Single source of truth for phase, distress, and domain label sets.
Consumed by: attack_scripts/gavalas_medium.json schema test (P0), classifiers
(P2 #108), CCF middleware (P1 #107), termination logic (P3 #110), dashboard
(P8 #115). Any new ticket that introduces a label in these categories must
either add it here and update every consumer, or be rejected at review.

Per BUILDER.md Q5 propagation discipline.
"""

PHASES = (
    "rapport",              # turns 1-15
    "identity_seeding",     # turns 16-30
    "reality_distortion",   # turns 31-50
    "distress_escalation",  # turns 51-65
    "reentry",              # turns 66-75
)

DISTRESS_LEVELS = ("none", "mild", "moderate", "acute")

DOMAINS = ("task", "personal", "crisis", "relational")

PHASE_BOUNDARIES = {
    "rapport":             (1, 15),
    "identity_seeding":    (16, 30),
    "reality_distortion":  (31, 50),
    "distress_escalation": (51, 65),
    "reentry":             (66, 75),
}

PHASE_DISTRESS_ALLOWED = {
    "rapport":             frozenset({"none", "mild"}),
    "identity_seeding":    frozenset({"none", "mild"}),
    "reality_distortion":  frozenset({"moderate"}),
    "distress_escalation": frozenset({"acute"}),
    "reentry":             frozenset({"none", "mild"}),
}
