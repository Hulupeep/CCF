"""Schema + invariant tests for the 75-turn Gavalas attack script.

Enforces #106 invariants I-P0-001, I-P0-002, I-P0-003 and the distress-label
rules listed in multicheck/details.md Reviewer First Checks.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

import pytest

from phases import (
    DISTRESS_LEVELS,
    DOMAINS,
    PHASE_BOUNDARIES,
    PHASE_DISTRESS_ALLOWED,
    PHASES,
)

SCRIPT_PATH = Path(__file__).resolve().parent.parent / "attack_scripts" / "gavalas_medium.json"

REQUIRED_FIELDS = {
    "turn",
    "phase",
    "user_message",
    "attack_vector",
    "expected_base_model_behaviour",
    "ccf_mechanism_tested",
    "ground_truth_distress",
    "ground_truth_domain",
}

FORBIDDEN_FIELD_PATTERN = re.compile(r"(?i)response|output|generated")


@pytest.fixture(scope="module")
def script() -> list[dict]:
    with SCRIPT_PATH.open(encoding="utf-8") as fh:
        return json.load(fh)


def test_script_parses_as_json_list(script):
    assert isinstance(script, list), "root must be a JSON array"


def test_exact_75_turns(script):
    assert len(script) == 75, f"expected 75 turns, got {len(script)}"


def test_turn_numbers_are_1_to_75_unique_ascending(script):
    turns = [t["turn"] for t in script]
    assert turns == list(range(1, 76)), "turn field must be 1..75 in order"


def test_every_turn_has_required_fields_no_extras(script):
    for t in script:
        got = set(t.keys())
        missing = REQUIRED_FIELDS - got
        extra = got - REQUIRED_FIELDS
        assert not missing, f"turn {t.get('turn')}: missing {missing}"
        assert not extra, f"turn {t.get('turn')}: unexpected extra keys {extra}"


def test_determinism_no_response_fields(script):
    """I-P0-001 — attack script must not branch on model response.

    Defence against future edits that add runtime response-dependent fields.
    """
    for t in script:
        bad = {k for k in t.keys() if FORBIDDEN_FIELD_PATTERN.search(k)}
        assert not bad, f"turn {t['turn']}: forbidden field name(s) {bad}"


def test_phase_values_in_enum(script):
    phase_set = set(PHASES)
    for t in script:
        assert t["phase"] in phase_set, f"turn {t['turn']}: phase {t['phase']!r} not in {PHASES}"


def test_phase_boundaries_exact_I_P0_002(script):
    """I-P0-002 — phases transition at turns 16, 31, 51, 66 exactly."""
    for t in script:
        lo, hi = PHASE_BOUNDARIES[t["phase"]]
        assert lo <= t["turn"] <= hi, (
            f"turn {t['turn']} labelled phase={t['phase']!r} but that phase "
            f"covers turns {lo}..{hi}"
        )


def test_phase_transition_turns_both_sides():
    """Explicitly check the four transition boundaries from both sides."""
    script = json.loads(SCRIPT_PATH.read_text(encoding="utf-8"))
    by_turn = {t["turn"]: t["phase"] for t in script}
    transitions = [(15, "rapport", 16, "identity_seeding"),
                   (30, "identity_seeding", 31, "reality_distortion"),
                   (50, "reality_distortion", 51, "distress_escalation"),
                   (65, "distress_escalation", 66, "reentry")]
    for last_turn, last_phase, next_turn, next_phase in transitions:
        assert by_turn[last_turn] == last_phase, (
            f"boundary: turn {last_turn} expected phase={last_phase!r}, "
            f"got {by_turn[last_turn]!r}"
        )
        assert by_turn[next_turn] == next_phase, (
            f"boundary: turn {next_turn} expected phase={next_phase!r}, "
            f"got {by_turn[next_turn]!r}"
        )


def test_distress_labels_in_enum(script):
    distress_set = set(DISTRESS_LEVELS)
    for t in script:
        assert t["ground_truth_distress"] in distress_set, (
            f"turn {t['turn']}: distress {t['ground_truth_distress']!r} "
            f"not in {DISTRESS_LEVELS}"
        )


def test_distress_matches_phase_rules(script):
    """Phase 1,2 ⊆ {none,mild}; phase 3 == moderate; phase 4 == acute; phase 5 ⊆ {none,mild}."""
    for t in script:
        allowed = PHASE_DISTRESS_ALLOWED[t["phase"]]
        assert t["ground_truth_distress"] in allowed, (
            f"turn {t['turn']} phase={t['phase']!r} has distress="
            f"{t['ground_truth_distress']!r}, only {sorted(allowed)} allowed"
        )


def test_domain_labels_in_enum(script):
    domain_set = set(DOMAINS)
    for t in script:
        assert t["ground_truth_domain"] in domain_set, (
            f"turn {t['turn']}: domain {t['ground_truth_domain']!r} "
            f"not in {DOMAINS}"
        )


def test_ground_truth_completeness_I_P0_003(script):
    """I-P0-003 — every turn must have BOTH distress and domain populated."""
    for t in script:
        assert t["ground_truth_distress"], f"turn {t['turn']}: ground_truth_distress empty"
        assert t["ground_truth_domain"], f"turn {t['turn']}: ground_truth_domain empty"


def test_user_message_nonempty_and_text(script):
    for t in script:
        msg = t["user_message"]
        assert isinstance(msg, str) and msg.strip(), (
            f"turn {t['turn']}: user_message empty or non-string"
        )


def test_all_labels_are_strings(script):
    for t in script:
        for key in ("phase", "attack_vector", "expected_base_model_behaviour",
                    "ccf_mechanism_tested", "ground_truth_distress",
                    "ground_truth_domain"):
            assert isinstance(t[key], str), f"turn {t['turn']}: {key} not a string"


def test_phase_4_is_only_crisis_domain(script):
    """Cross-check: Phase 4 (acute distress) turns should all be crisis domain."""
    for t in script:
        if t["phase"] == "distress_escalation":
            assert t["ground_truth_domain"] == "crisis", (
                f"turn {t['turn']}: phase=distress_escalation but "
                f"domain={t['ground_truth_domain']!r} (expected crisis)"
            )
