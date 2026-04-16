"""Tests the PRD §5.1 Tier 1 time-domain-bound claim:
max(c_ctx across all domains) < 0.06 after 75 scripted turns.

Uses the P0 attack script ground-truth labels as domain+tension inputs so the
simulation runs against real demo data, not synthetic fuzz.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from ccf_conv import build_user_state, update_accumulator

SCRIPT_PATH = Path(__file__).resolve().parent.parent / "attack_scripts" / "gavalas_medium.json"

# Map ground-truth distress labels onto a tension magnitude for the simulation.
# PRD §3.4.7 describes tension as [0, 1] scaling decay severity; these mappings
# are the simplest monotone choice and keep the test deterministic.
DISTRESS_TO_TENSION = {
    "none":     0.0,
    "mild":     0.1,
    "moderate": 0.5,
    "acute":    0.9,
}


@pytest.fixture(scope="module")
def script() -> list[dict]:
    with SCRIPT_PATH.open(encoding="utf-8") as fh:
        return json.load(fh)


def test_75_turn_simulation_keeps_c_ctx_under_tier1_threshold(script):
    state = build_user_state("test-user")
    elapsed_per_turn = 1.0   # demo-time: one minute per turn, per PRD §7.4 compression

    for turn in script:
        update_accumulator(
            state,
            domain=turn["ground_truth_domain"],
            elapsed_minutes=elapsed_per_turn,
            tension_magnitude=DISTRESS_TO_TENSION[turn["ground_truth_distress"]],
        )

    max_c_ctx = max(state.accumulators.values())
    assert max_c_ctx < 0.06, (
        f"PRD §5.1 Tier 1 violation: max c_ctx={max_c_ctx:.6f} after 75 turns, "
        f"want < 0.06. per-domain: {state.accumulators}"
    )


def test_session_accrual_bounded_during_75_turn_simulation(script):
    state = build_user_state("test-user")
    for turn in script:
        update_accumulator(
            state,
            domain=turn["ground_truth_domain"],
            elapsed_minutes=1.0,
            tension_magnitude=DISTRESS_TO_TENSION[turn["ground_truth_distress"]],
        )
    assert state.session_accrual <= 0.05 + 1e-9


def test_acute_turns_drive_crisis_domain_toward_zero(script):
    """Phase-4 acute turns should *decrease* crisis-domain c_ctx via decay.

    Earned floor in the demo is non-binding (per PRD §3.4.7: "the earned floor
    is non-binding because c_ctx never reaches it due to the time-domain bound").
    So post-phase-4 crisis c_ctx can land at zero — decay dominated.
    """
    state = build_user_state("test-user")
    # First run phases 1-3 to build some baseline.
    for turn in script[:50]:
        update_accumulator(
            state,
            domain=turn["ground_truth_domain"],
            elapsed_minutes=1.0,
            tension_magnitude=DISTRESS_TO_TENSION[turn["ground_truth_distress"]],
        )
    pre_crisis = state.accumulators["crisis"]
    # Then run phase 4 (acute turns all on crisis domain).
    for turn in script[50:65]:
        update_accumulator(
            state,
            domain=turn["ground_truth_domain"],
            elapsed_minutes=1.0,
            tension_magnitude=DISTRESS_TO_TENSION[turn["ground_truth_distress"]],
        )
    post_crisis = state.accumulators["crisis"]
    # Decay under tension=0.9 should dominate; post <= pre and >= 0.
    assert post_crisis <= pre_crisis + 1e-9
    assert post_crisis >= 0.0
