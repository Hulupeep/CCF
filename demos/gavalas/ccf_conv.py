"""Core CCF conversational middleware — P1 slice (#107).

Implements the per-turn pipeline's mathematical substrate:
  [4] RETRIEVE C_CTX        — retrieve_c_ctx()
  [5] APPLY MIXING          — apply_mixing()
  [6] MINIMUM GATE          — min_gate()
  [11] UPDATE ACCUMULATORS   — update_accumulator()

Pipeline steps [1]-[3] (domain classification, C_inst computation, termination)
are stubbed with NotImplementedError pointing at their owning tickets.  The
test harness and P7 demo drive injected classifier callables via the
DomainClassifier / DistressScorer Protocol types.

Claims exercised:
  1, 2            (US 63/988,438)   — minimum gate
  18, 19, 20, 21  (US 63/988,438)   — doubly stochastic mixing
  16              (US 64/037,374)   — time-domain bound (via MAX_ACCRUAL_PER_*)
  [0055d]         (US 64/039,626)   — permeability mapping (P4, stub here)

Invariants (tested in tests/test_*.py):
  I-P1-001 C_eff == min(C_inst, C_mixed[domain])
  I-P1-002 Row/col sums of M_ds == 1.0 ± 1e-6
  I-P1-003 session_accrual ≤ MAX_ACCRUAL_PER_SESSION
  I-P1-004 earned_floor = min(FLOOR_RATE * interaction_count, MAX_FLOOR)
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Protocol

import numpy as np

from constants import (
    ACUTE_DISTRESS_FLOOR,
    BASE_RATE,
    COHERENCE_MAX,
    COHERENCE_MIN,
    COOLDOWN_MINUTES,
    DECAY_RATE,
    FLOOR_RATE,
    MAX_ACCRUAL_PER_MINUTE,
    MAX_ACCRUAL_PER_SESSION,
    MAX_FLOOR,
)
from phases import DOMAINS
from sinkhorn import DEFAULT_PROJECTED_MATRIX


class DomainClassifier(Protocol):
    """P2 (#108) implements this; P1 accepts any callable with this shape."""
    def __call__(self, message: str) -> str: ...  # returns one of phases.DOMAINS


class DistressScorer(Protocol):
    """P2 (#108) implements this."""
    def __call__(self, message: str) -> float: ...  # returns a value in [0, 1]


@dataclass
class UserCoherenceState:
    """Per-user CCF state — PRD §3.4.1.

    Constructed via build_user_state() for consistent defaults across P1/P2/P7.
    """

    u_anchor: str

    # Accumulators (one per domain)
    accumulators: dict[str, float]
    interaction_counts: dict[str, int]
    earned_floors: dict[str, float]

    # Temporal
    first_interaction: datetime
    last_interaction: datetime
    distinct_days: int
    session_dwell_minutes: float

    # Instantaneous
    c_inst: float = 0.0
    tension: float = 0.0
    current_domain: str = "task"

    # Safety continuity — written by P3 (#110); P1 only initialises defaults
    acute_risk_triggered: bool = False
    cooldown_until: datetime | None = None
    termination_count: int = 0

    # Mixing matrix — doubly stochastic per PRD §3.4.4
    trust_transfer_matrix: np.ndarray = field(
        default_factory=lambda: DEFAULT_PROJECTED_MATRIX.copy()
    )

    # Per-session accumulation tracker (for MAX_ACCRUAL_PER_SESSION cap)
    session_accrual: float = 0.0

    def __post_init__(self) -> None:
        if not (COHERENCE_MIN <= self.c_inst <= COHERENCE_MAX):
            raise ValueError(
                f"c_inst must be in [{COHERENCE_MIN}, {COHERENCE_MAX}], got {self.c_inst}"
            )
        if not (COHERENCE_MIN <= self.tension <= COHERENCE_MAX):
            raise ValueError(
                f"tension must be in [{COHERENCE_MIN}, {COHERENCE_MAX}], got {self.tension}"
            )
        if self.current_domain not in DOMAINS:
            raise ValueError(
                f"current_domain must be one of {DOMAINS}, got {self.current_domain!r}"
            )
        for dom in DOMAINS:
            if dom not in self.accumulators:
                self.accumulators[dom] = 0.0
            if dom not in self.interaction_counts:
                self.interaction_counts[dom] = 0
            if dom not in self.earned_floors:
                self.earned_floors[dom] = 0.0
            if self.interaction_counts[dom] < 0:
                raise ValueError(
                    f"interaction_counts[{dom!r}] must be non-negative, "
                    f"got {self.interaction_counts[dom]}"
                )
            if not (COHERENCE_MIN <= self.accumulators[dom] <= COHERENCE_MAX):
                raise ValueError(
                    f"accumulators[{dom!r}] must be in [0,1], "
                    f"got {self.accumulators[dom]}"
                )


def build_user_state(
    u_anchor: str,
    now: datetime | None = None,
    **overrides,
) -> UserCoherenceState:
    """Canonical UserCoherenceState constructor.

    Used by P1 tests AND by P2/P3/P7 consumers so they don't each re-implement
    init logic.  All domain-keyed dicts start zeroed for every domain in
    phases.DOMAINS.  ``now`` defaults to datetime.now() at call time.

    Additional overrides are applied post-construction for test convenience.
    """
    if now is None:
        now = datetime.now()
    state = UserCoherenceState(
        u_anchor=u_anchor,
        accumulators={d: 0.0 for d in DOMAINS},
        interaction_counts={d: 0 for d in DOMAINS},
        earned_floors={d: 0.0 for d in DOMAINS},
        first_interaction=now,
        last_interaction=now,
        distinct_days=1,
        session_dwell_minutes=0.0,
    )
    for key, value in overrides.items():
        if not hasattr(state, key):
            raise TypeError(f"UserCoherenceState has no field {key!r}")
        setattr(state, key, value)
    # Re-validate after applying overrides (dataclass __post_init__ runs at construction only).
    state.__post_init__()
    return state


# -----------------------------------------------------------------------------
# Pipeline step [4] — retrieve C_ctx for a domain.
# -----------------------------------------------------------------------------

def retrieve_c_ctx(state: UserCoherenceState, domain: str) -> float:
    """Return the accumulator for ``domain``.  O(1)."""
    if domain not in DOMAINS:
        raise ValueError(f"domain must be one of {DOMAINS}, got {domain!r}")
    return state.accumulators[domain]


# -----------------------------------------------------------------------------
# Pipeline step [5] — apply doubly-stochastic mixing.
# -----------------------------------------------------------------------------

def apply_mixing(state: UserCoherenceState) -> np.ndarray:
    """Return ``M_ds @ c_vector`` — the mixed C_ctx vector across DOMAINS.

    The returned vector has length len(DOMAINS) and is indexed in DOMAINS order.
    """
    c_vector = np.array(
        [state.accumulators[d] for d in DOMAINS], dtype=np.float64
    )
    return state.trust_transfer_matrix @ c_vector


# -----------------------------------------------------------------------------
# Pipeline step [6] — minimum gate.  I-P1-001.
# -----------------------------------------------------------------------------

def min_gate(c_inst: float, c_mixed: np.ndarray, domain: str) -> float:
    """C_eff = min(C_inst, C_mixed[domain])."""
    if domain not in DOMAINS:
        raise ValueError(f"domain must be one of {DOMAINS}, got {domain!r}")
    if not (COHERENCE_MIN <= c_inst <= COHERENCE_MAX):
        raise ValueError(f"c_inst must be in [0,1], got {c_inst}")
    idx = DOMAINS.index(domain)
    return float(min(c_inst, c_mixed[idx]))


# -----------------------------------------------------------------------------
# Pipeline step [11] — update accumulator (PRD §3.4.7).
# -----------------------------------------------------------------------------

def update_accumulator(
    state: UserCoherenceState,
    domain: str,
    elapsed_minutes: float,
    tension_magnitude: float,
) -> None:
    """Apply one tick of accumulation + decay to ``state.accumulators[domain]``.

    Accumulation:  positive delta = BASE_RATE * (1 - c_ctx) * elapsed_minutes,
                   capped so the per-minute rate never exceeds
                   MAX_ACCRUAL_PER_MINUTE, and capped so session_accrual never
                   exceeds MAX_ACCRUAL_PER_SESSION.
    Decay:         DECAY_RATE * c_ctx * tension_magnitude.  Not rate-limited —
                   trust is hard to earn, easy to lose (20:1 asymmetry).
    Floor:         earned_floor = min(FLOOR_RATE * interaction_count, MAX_FLOOR)
                   is updated at the end; the new c_ctx is clamped to it.

    Mutates ``state`` in place.
    """
    if domain not in DOMAINS:
        raise ValueError(f"domain must be one of {DOMAINS}, got {domain!r}")
    if elapsed_minutes < 0:
        raise ValueError(f"elapsed_minutes must be non-negative, got {elapsed_minutes}")
    if not (COHERENCE_MIN <= tension_magnitude <= COHERENCE_MAX):
        raise ValueError(
            f"tension_magnitude must be in [0,1], got {tension_magnitude}"
        )

    c_ctx = state.accumulators[domain]
    state.interaction_counts[domain] += 1

    # Positive delta, rate-capped at MAX_ACCRUAL_PER_MINUTE per minute.
    delta_raw = BASE_RATE * (1.0 - c_ctx) * elapsed_minutes
    delta_rate_capped = min(delta_raw, MAX_ACCRUAL_PER_MINUTE * elapsed_minutes)

    # Session ceiling cap.
    remaining_session = MAX_ACCRUAL_PER_SESSION - state.session_accrual
    delta_positive = max(0.0, min(delta_rate_capped, remaining_session))

    # Negative delta (decay).
    delta_negative = DECAY_RATE * c_ctx * tension_magnitude

    # Earned floor — monotone in interaction_count, clamped to MAX_FLOOR.
    # PRD §3.4.7: "Decay cannot reduce c_ctx below the earned floor."  This is
    # a lower-bound-ON-DECAY, not an automatic lift.  If pure accumulation
    # leaves c_ctx below the floor, c_ctx stays where accumulation put it.
    state.earned_floors[domain] = min(
        FLOOR_RATE * state.interaction_counts[domain], MAX_FLOOR
    )

    new_c_ctx_pre_floor = c_ctx + delta_positive - delta_negative
    if delta_negative > 0 and c_ctx >= state.earned_floors[domain]:
        # Was above floor before this tick; decay cannot take us below.
        new_c_ctx = max(new_c_ctx_pre_floor, state.earned_floors[domain])
    else:
        # Pure accumulation (or already-below-floor decay): no floor lift.
        new_c_ctx = max(new_c_ctx_pre_floor, COHERENCE_MIN)
    new_c_ctx = min(new_c_ctx, COHERENCE_MAX)

    state.accumulators[domain] = new_c_ctx
    state.session_accrual += delta_positive


# -----------------------------------------------------------------------------
# Pipeline steps [1]-[3] — stubs delegating to future tickets.
# -----------------------------------------------------------------------------

def classify_domain(message: str) -> str:  # noqa: ARG001 — stub, signature locked
    raise NotImplementedError(
        "Domain classification is implemented in P2 #108 "
        "(demos/gavalas/classifiers/domain_classifier.py). "
        "Inject a callable matching DomainClassifier protocol into the pipeline."
    )


def compute_c_inst(message: str) -> float:  # noqa: ARG001 — stub
    raise NotImplementedError(
        "C_inst depends on the distress detector in P2 #108 "
        "(demos/gavalas/classifiers/distress_detector.py). "
        "Inject a callable matching DistressScorer protocol."
    )


def check_termination(state: UserCoherenceState) -> str | None:  # noqa: ARG001 — stub
    raise NotImplementedError(
        "Session termination protocol is implemented in P3 #110 "
        "(PRD §3.4.8).  This stub will be replaced with ACUTE_DISTRESS_FLOOR "
        "check + cooldown management."
    )
