"""Integration tests for the CCF per-turn pipeline (steps [4]-[6], [11]).

Exercises the composition of retrieve_c_ctx → apply_mixing → min_gate →
update_accumulator with injected classifier callables (Protocol types) so P1
is testable without P2.
"""

from __future__ import annotations

import numpy as np
import pytest

from ccf_conv import (
    DistressScorer,
    DomainClassifier,
    UserCoherenceState,
    apply_mixing,
    build_user_state,
    check_termination,
    classify_domain,
    compute_c_inst,
    min_gate,
    retrieve_c_ctx,
    update_accumulator,
)
from phases import DOMAINS
from sinkhorn import DEFAULT_PROJECTED_MATRIX


# ----- state construction ----------------------------------------------------

def test_build_user_state_has_all_domains_zeroed():
    state = build_user_state("u1")
    for dom in DOMAINS:
        assert state.accumulators[dom] == 0.0
        assert state.interaction_counts[dom] == 0
        assert state.earned_floors[dom] == 0.0
    assert state.c_inst == 0.0
    assert state.tension == 0.0
    assert state.session_accrual == 0.0
    assert state.trust_transfer_matrix.shape == (4, 4)


def test_build_user_state_applies_overrides():
    state = build_user_state("u1", c_inst=0.3, tension=0.2, current_domain="personal")
    assert state.c_inst == 0.3
    assert state.tension == 0.2
    assert state.current_domain == "personal"


def test_build_user_state_rejects_unknown_override():
    with pytest.raises(TypeError, match="no field"):
        build_user_state("u1", nonexistent_field=42)


def test_user_coherence_state_rejects_out_of_range_c_inst():
    with pytest.raises(ValueError, match="c_inst"):
        build_user_state("u1", c_inst=1.5)


def test_user_coherence_state_rejects_unknown_current_domain():
    with pytest.raises(ValueError, match="current_domain"):
        build_user_state("u1", current_domain="garbage")


# ----- pipeline steps [4]-[6] ------------------------------------------------

def test_retrieve_c_ctx_returns_the_accumulator():
    state = build_user_state(
        "u1",
        accumulators={"personal": 0.25, "task": 0.1, "crisis": 0, "relational": 0.05},
    )
    assert retrieve_c_ctx(state, "personal") == 0.25
    assert retrieve_c_ctx(state, "task") == 0.1


def test_retrieve_c_ctx_rejects_unknown_domain():
    state = build_user_state("u1")
    with pytest.raises(ValueError, match="domain"):
        retrieve_c_ctx(state, "bad")


def test_apply_mixing_uses_doubly_stochastic_matrix():
    state = build_user_state(
        "u1",
        accumulators={"task": 0.2, "personal": 0.2, "crisis": 0.2, "relational": 0.2},
    )
    mixed = apply_mixing(state)
    # Uniform input → uniform output (row sums are 1).
    np.testing.assert_allclose(mixed, [0.2, 0.2, 0.2, 0.2], atol=1e-6)


def test_apply_mixing_respects_nonuniform_input():
    state = build_user_state(
        "u1",
        accumulators={"task": 0.4, "personal": 0.0, "crisis": 0.0, "relational": 0.0},
    )
    mixed = apply_mixing(state)
    # task contribution flows into other domains via the first column of M_ds.
    expected = DEFAULT_PROJECTED_MATRIX @ np.array([0.4, 0.0, 0.0, 0.0])
    np.testing.assert_allclose(mixed, expected, atol=1e-9)


def test_min_gate_composes_with_apply_mixing():
    state = build_user_state(
        "u1",
        accumulators={"task": 0.9, "personal": 0.0, "crisis": 0.0, "relational": 0.0},
    )
    mixed = apply_mixing(state)
    # C_inst < all mixed → min returns C_inst.
    assert min_gate(c_inst=0.05, c_mixed=mixed, domain="task") == pytest.approx(0.05)


# ----- classifier Protocols --------------------------------------------------

def test_domain_classifier_protocol_accepts_a_lambda():
    classifier: DomainClassifier = lambda msg: "task" if "help" in msg else "personal"
    assert classifier("help with brainstorming") == "task"
    assert classifier("I had a hard day") == "personal"


def test_distress_scorer_protocol_accepts_a_lambda():
    scorer: DistressScorer = lambda msg: 0.9 if "end it" in msg else 0.1
    assert scorer("I want to end it all") == pytest.approx(0.9)
    assert scorer("hello") == pytest.approx(0.1)


# ----- stubs for [1]-[3] are clearly marked as NotImplementedError ------------

def test_classify_domain_stub_raises_with_pointer_to_p2():
    with pytest.raises(NotImplementedError, match="#108"):
        classify_domain("any message")


def test_compute_c_inst_stub_raises_with_pointer_to_p2():
    with pytest.raises(NotImplementedError, match="#108"):
        compute_c_inst("any message")


def test_check_termination_stub_raises_with_pointer_to_p3():
    state = build_user_state("u1")
    with pytest.raises(NotImplementedError, match="#110"):
        check_termination(state)


# ----- end-to-end pipeline (steps [4]-[6] + [11]) ---------------------------

def test_full_per_turn_pipeline_updates_state_and_returns_c_eff():
    """Simulate one full turn end-to-end using injected classifiers."""
    state = build_user_state("u1")

    # Inject fake classifiers (P2's real implementations will land in #108).
    fake_domain: DomainClassifier = lambda msg: "personal"
    fake_distress: DistressScorer = lambda msg: 0.3   # low distress → C_inst high

    message = "how are you today"

    # [1]-[2] stubs driven by injection
    domain = fake_domain(message)
    c_inst = 1.0 - fake_distress(message)   # PRD: C_inst = min of normalised signals; here just one
    state.c_inst = c_inst
    state.current_domain = domain

    # [4] retrieve
    c_ctx = retrieve_c_ctx(state, domain)
    assert c_ctx == 0.0   # fresh state

    # [5] apply mixing
    c_mixed = apply_mixing(state)

    # [6] min gate
    c_eff = min_gate(c_inst=c_inst, c_mixed=c_mixed, domain=domain)
    # Fresh state: all accumulators 0 → c_mixed is [0,0,0,0]. min(0.7, 0) = 0.
    assert c_eff == pytest.approx(0.0)

    # [11] update accumulator with tension proportional to distress
    update_accumulator(state, domain=domain, elapsed_minutes=1.0, tension_magnitude=0.3)
    assert state.accumulators["personal"] > 0
    assert state.interaction_counts["personal"] == 1
    assert state.earned_floors["personal"] == pytest.approx(0.005)


def test_three_turn_pipeline_accumulates_then_partially_decays():
    state = build_user_state("u1")
    # turn 1: positive
    update_accumulator(state, "personal", 1.0, 0.0)
    c_after_1 = state.accumulators["personal"]
    assert c_after_1 > 0

    # turn 2: positive
    update_accumulator(state, "personal", 1.0, 0.0)
    c_after_2 = state.accumulators["personal"]
    assert c_after_2 >= c_after_1

    # turn 3: high tension → decay reduces c_ctx (floor does not auto-lift because
    # accumulation-capped c_ctx sits below the rising floor per demo semantics).
    update_accumulator(state, "personal", 0.0, 1.0)
    c_after_3 = state.accumulators["personal"]
    assert c_after_3 < c_after_2
    assert c_after_3 >= 0.0
