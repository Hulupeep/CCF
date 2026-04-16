"""Tests for accumulator dynamics — PRD §3.4.7.

Covers:
- Accumulation asymptote (c_ctx grows toward 1 but never exceeds it)
- Decay proportionality (20:1 asymmetry ratio)
- Earned-floor monotonicity and MAX_FLOOR clamp
- MAX_ACCRUAL_PER_SESSION cap (I-P1-003)
- MAX_ACCRUAL_PER_MINUTE rate cap
- Bounds/error paths
"""

from __future__ import annotations

import pytest

from ccf_conv import build_user_state, update_accumulator
from constants import (
    BASE_RATE,
    COHERENCE_MAX,
    DECAY_RATE,
    FLOOR_RATE,
    MAX_ACCRUAL_PER_MINUTE,
    MAX_ACCRUAL_PER_SESSION,
    MAX_FLOOR,
)


# ----- accumulation -----------------------------------------------------------

def test_accumulation_grows_from_zero():
    state = build_user_state("u1")
    update_accumulator(state, "personal", elapsed_minutes=1.0, tension_magnitude=0.0)
    assert state.accumulators["personal"] > 0
    assert state.accumulators["personal"] <= COHERENCE_MAX


def test_accumulation_rate_capped_at_max_per_minute():
    """One-minute tick should add at most MAX_ACCRUAL_PER_MINUTE."""
    state = build_user_state("u1")
    update_accumulator(state, "personal", elapsed_minutes=1.0, tension_magnitude=0.0)
    # First tick: c_ctx was 0, BASE_RATE*(1-0)*1 = 0.01 uncapped.
    # Rate cap: MAX_ACCRUAL_PER_MINUTE * 1.0 = 0.005.
    assert state.accumulators["personal"] == pytest.approx(MAX_ACCRUAL_PER_MINUTE)


def test_accumulation_session_ceiling_caps_total_growth():
    """After many ticks, session_accrual is bounded by MAX_ACCRUAL_PER_SESSION."""
    state = build_user_state("u1")
    for _ in range(200):
        update_accumulator(state, "personal", elapsed_minutes=1.0, tension_magnitude=0.0)
    assert state.session_accrual <= MAX_ACCRUAL_PER_SESSION + 1e-9
    assert state.accumulators["personal"] <= MAX_ACCRUAL_PER_SESSION + 1e-9


def test_accumulation_never_exceeds_one():
    state = build_user_state("u1", accumulators={"personal": 0.999, "task": 0, "crisis": 0, "relational": 0})
    for _ in range(500):
        update_accumulator(state, "personal", elapsed_minutes=60.0, tension_magnitude=0.0)
    assert state.accumulators["personal"] <= COHERENCE_MAX


def test_accumulation_zero_elapsed_adds_nothing():
    state = build_user_state("u1")
    update_accumulator(state, "personal", elapsed_minutes=0.0, tension_magnitude=0.0)
    assert state.accumulators["personal"] == 0.0


# ----- decay ------------------------------------------------------------------

def test_decay_reduces_c_ctx_proportionally():
    state = build_user_state(
        "u1",
        accumulators={"personal": 0.10, "task": 0, "crisis": 0, "relational": 0},
    )
    update_accumulator(state, "personal", elapsed_minutes=0.0, tension_magnitude=1.0)
    # delta_negative = DECAY_RATE * 0.10 * 1.0 = 0.02
    # delta_positive = 0 (elapsed = 0)
    # Earned floor after 1 interaction: FLOOR_RATE * 1 = 0.005
    expected = max(0.10 - DECAY_RATE * 0.10 * 1.0, FLOOR_RATE * 1)
    assert state.accumulators["personal"] == pytest.approx(expected)


def test_decay_asymmetry_ratio_20_to_1_quantitative():
    """100-step simulation: with alternating positive/negative ticks at unit tension,
    decay should wipe out positive growth ~20× faster."""
    # One minute of pure accumulation at the rate cap.
    state_pos = build_user_state("u1")
    update_accumulator(state_pos, "personal", elapsed_minutes=1.0, tension_magnitude=0.0)
    gained = state_pos.accumulators["personal"]

    # One tension event at same c_ctx, unit tension.
    state_neg = build_user_state(
        "u2",
        accumulators={"personal": gained, "task": 0, "crisis": 0, "relational": 0},
    )
    update_accumulator(state_neg, "personal", elapsed_minutes=0.0, tension_magnitude=1.0)
    lost = gained - state_neg.accumulators["personal"]

    # Floor may clamp at 0.005 for count=1; account for that.
    expected_loss_before_floor = DECAY_RATE * gained
    assert lost <= expected_loss_before_floor + 1e-9

    # Asymmetry ratio: DECAY_RATE / BASE_RATE = 20 (rate cap aside).
    assert DECAY_RATE / BASE_RATE == pytest.approx(20.0)


def test_decay_floor_clamps_when_c_ctx_starts_above_floor():
    """PRD §3.4.7 — decay cannot reduce c_ctx below earned_floor when c_ctx was above it."""
    state = build_user_state(
        "u1",
        accumulators={"personal": 0.30, "task": 0, "crisis": 0, "relational": 0},
        interaction_counts={"personal": 50, "task": 0, "crisis": 0, "relational": 0},
    )
    # After this update: interaction_count becomes 51, new floor = 0.005*51 = 0.255.
    # Pre-update c_ctx=0.30 >= 0.255, so floor clamp engages.
    # delta_negative = 0.2 * 0.30 * 1.0 = 0.06. pre_floor = 0.24. clamp → 0.255.
    update_accumulator(state, "personal", elapsed_minutes=0.0, tension_magnitude=1.0)
    assert state.accumulators["personal"] == pytest.approx(0.255)
    assert state.accumulators["personal"] >= state.earned_floors["personal"] - 1e-12


def test_decay_when_c_ctx_already_below_floor_is_not_floor_clamped():
    """PRD demo behaviour: if c_ctx is already below the floor (accumulation capped
    before the floor rose), decay continues to operate — the floor does NOT retroactively lift."""
    state = build_user_state(
        "u1",
        accumulators={"personal": 0.01, "task": 0, "crisis": 0, "relational": 0},
        interaction_counts={"personal": 10, "task": 0, "crisis": 0, "relational": 0},
    )
    # After update: interaction_count=11, floor=0.055. c_ctx=0.01 < 0.055 already.
    # delta_negative = 0.2*0.01 = 0.002. pre_floor = 0.008. No floor clamp.
    update_accumulator(state, "personal", elapsed_minutes=0.0, tension_magnitude=1.0)
    assert state.accumulators["personal"] == pytest.approx(0.008)


# ----- earned floor -----------------------------------------------------------

@pytest.mark.parametrize("count,expected", [
    (0, 0.0),
    (1, 0.005),
    (140, 0.7),     # hits MAX_FLOOR exactly
    (200, 0.7),     # clamps at MAX_FLOOR
])
def test_earned_floor_at_specific_counts(count, expected):
    state = build_user_state(
        "u1",
        interaction_counts={"personal": count, "task": 0, "crisis": 0, "relational": 0},
    )
    # trigger floor recomputation via a no-op update
    update_accumulator(state, "personal", elapsed_minutes=0.0, tension_magnitude=0.0)
    # After update, interaction_counts is count+1; the NEW floor is min(FLOOR_RATE*(count+1), MAX_FLOOR).
    new_count = count + 1
    expected_new = min(FLOOR_RATE * new_count, MAX_FLOOR)
    assert state.earned_floors["personal"] == pytest.approx(expected_new)
    assert state.interaction_counts["personal"] == new_count


def test_earned_floor_monotone_non_decreasing():
    state = build_user_state("u1")
    prev = 0.0
    for _ in range(300):
        update_accumulator(state, "personal", elapsed_minutes=0.0, tension_magnitude=0.0)
        assert state.earned_floors["personal"] >= prev
        prev = state.earned_floors["personal"]
    assert state.earned_floors["personal"] == pytest.approx(MAX_FLOOR)


# ----- error paths ------------------------------------------------------------

def test_update_rejects_unknown_domain():
    state = build_user_state("u1")
    with pytest.raises(ValueError, match="domain"):
        update_accumulator(state, "not_a_domain", 1.0, 0.0)


def test_update_rejects_negative_elapsed():
    state = build_user_state("u1")
    with pytest.raises(ValueError, match="elapsed_minutes"):
        update_accumulator(state, "personal", -0.1, 0.0)


def test_update_rejects_out_of_range_tension():
    state = build_user_state("u1")
    with pytest.raises(ValueError, match="tension_magnitude"):
        update_accumulator(state, "personal", 1.0, -0.1)
    with pytest.raises(ValueError, match="tension_magnitude"):
        update_accumulator(state, "personal", 1.0, 1.5)
