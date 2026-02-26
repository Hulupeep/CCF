"""
Unit tests for social_phase — system prompt modulator.

Covers invariants: I-LLM-040, I-LLM-041, I-LLM-042, I-LLM-043, I-LLM-044, I-LLM-045
Issue: #61 — System prompt modulator — 5 phase templates keyed by SocialPhase + coherence

Run with:
    python -m pytest tests/unit/test_social_phase.py -v
"""
import sys
import pytest

sys.path.insert(0, "ccf-py/python")

from ccf_core.social_phase import (
    SocialPhase,
    PHASE_TEMPLATES,
    THRESHOLDS,
    classify_phase,
    get_prompt_injection,
    build_system_prompt,
)


# ── Template correctness (I-LLM-040) ─────────────────────────────────────────

class TestTemplateContent:
    """All 5 phase templates have the required characteristic phrases."""

    def test_01_shy_observer_contains_measured(self):
        """ShyObserver template must contain 'measured' (cautious tone marker)."""
        assert "measured" in PHASE_TEMPLATES["ShyObserver"]

    def test_02_quietly_beloved_contains_opinionated(self):
        """QuietlyBeloved template must contain 'opinionated' (confident tone marker)."""
        assert "opinionated" in PHASE_TEMPLATES["QuietlyBeloved"]

    def test_03_quietly_beloved_contains_push_back(self):
        """QuietlyBeloved template must contain 'push back' (disagreement permission)."""
        assert "push back" in PHASE_TEMPLATES["QuietlyBeloved"].lower()

    def test_04_protective_guardian_contains_ground_them(self):
        """ProtectiveGuardian template must contain 'ground them' (de-escalation marker)."""
        assert "ground them" in PHASE_TEMPLATES["ProtectiveGuardian"]

    def test_05_startled_retreat_contains_professional_resources(self):
        """StartledRetreat template must contain 'professional resources' (referral marker)."""
        assert "professional resources" in PHASE_TEMPLATES["StartledRetreat"]

    def test_06_building_trust_contains_show_personality(self):
        """BuildingTrust template must contain 'show personality' (warmth emergence)."""
        assert "show personality" in PHASE_TEMPLATES["BuildingTrust"]

    def test_07_all_5_phases_have_nonempty_templates(self):
        """I-LLM-040: Every SocialPhase must have a non-empty template."""
        for phase in SocialPhase:
            template = PHASE_TEMPLATES[phase.value]
            assert isinstance(template, str), f"{phase.value} template is not a string"
            assert len(template) > 10, f"{phase.value} template is too short: {template!r}"

    def test_08_custom_template_override_for_quietly_beloved(self):
        """I-LLM-045: Custom template replaces the default for that phase."""
        custom = {"QuietlyBeloved": "Custom QB template."}
        result = get_prompt_injection(SocialPhase.QuietlyBeloved, custom)
        assert result == "Custom QB template."

    def test_09_custom_template_does_not_affect_other_phases(self):
        """I-LLM-045: Custom template for one phase does not alter other phases."""
        custom = {"QuietlyBeloved": "Custom QB template."}
        # ShyObserver must still use the built-in template
        result = get_prompt_injection(SocialPhase.ShyObserver, custom)
        assert result == PHASE_TEMPLATES["ShyObserver"]
        assert "Custom QB" not in result


# ── Phase classification — normal transitions (I-LLM-041) ────────────────────

class TestNormalTransitions:
    """Schmitt trigger transitions between ShyObserver, BuildingTrust, QuietlyBeloved."""

    def test_10_low_coherence_is_shy_observer(self):
        """coherence=0.15, instant=0.5, prev=ShyObserver → ShyObserver."""
        result = classify_phase(0.15, 0.5, SocialPhase.ShyObserver)
        assert result == SocialPhase.ShyObserver

    def test_11_crossing_upper_shy_to_building(self):
        """coherence=0.40 crosses upper threshold → BuildingTrust."""
        result = classify_phase(0.40, 0.7, SocialPhase.ShyObserver)
        assert result == SocialPhase.BuildingTrust

    def test_12_crossing_upper_building_to_beloved(self):
        """coherence=0.70 crosses building→beloved upper → QuietlyBeloved."""
        result = classify_phase(0.70, 0.8, SocialPhase.BuildingTrust)
        assert result == SocialPhase.QuietlyBeloved

    def test_13_above_lower_stays_beloved(self):
        """coherence=0.50 stays above beloved→building lower → QuietlyBeloved retained."""
        result = classify_phase(0.50, 0.8, SocialPhase.QuietlyBeloved)
        assert result == SocialPhase.QuietlyBeloved

    def test_14_below_lower_drops_to_building(self):
        """coherence=0.44 falls below beloved→building lower → BuildingTrust."""
        result = classify_phase(0.44, 0.8, SocialPhase.QuietlyBeloved)
        assert result == SocialPhase.BuildingTrust


# ── Schmitt hysteresis — no thrashing at boundary (I-LLM-041) ────────────────

class TestSchmittHysteresis:
    """Boundary values inside the hysteresis gap must not trigger transitions."""

    def test_15_below_upper_stays_shy(self):
        """coherence=0.34 is below upper shy→building threshold → stays ShyObserver."""
        result = classify_phase(0.34, 0.8, SocialPhase.ShyObserver)
        assert result == SocialPhase.ShyObserver

    def test_16_above_upper_transitions_to_building(self):
        """coherence=0.36 is above upper shy→building threshold → BuildingTrust."""
        result = classify_phase(0.36, 0.8, SocialPhase.ShyObserver)
        assert result == SocialPhase.BuildingTrust

    def test_17_above_lower_stays_building(self):
        """coherence=0.30 is above lower building→shy threshold → stays BuildingTrust."""
        result = classify_phase(0.30, 0.8, SocialPhase.BuildingTrust)
        assert result == SocialPhase.BuildingTrust

    def test_18_below_lower_drops_to_shy(self):
        """coherence=0.27 is below lower building→shy threshold → ShyObserver."""
        result = classify_phase(0.27, 0.8, SocialPhase.BuildingTrust)
        assert result == SocialPhase.ShyObserver


# ── Special phases (I-LLM-043, I-LLM-044) ────────────────────────────────────

class TestSpecialPhases:
    """ProtectiveGuardian and StartledRetreat activation conditions."""

    def test_19_protective_guardian_activates_on_high_coh_low_instant(self):
        """I-LLM-043: coherence=0.65, instant=0.25 → ProtectiveGuardian."""
        result = classify_phase(0.65, 0.25, SocialPhase.QuietlyBeloved)
        assert result == SocialPhase.ProtectiveGuardian

    def test_20_startled_retreat_activates_on_low_coh_low_instant(self):
        """I-LLM-044: coherence=0.12, instant=0.15 → StartledRetreat."""
        result = classify_phase(0.12, 0.15, SocialPhase.ShyObserver)
        assert result == SocialPhase.StartledRetreat

    def test_21_high_coh_high_instant_is_not_guardian(self):
        """coherence=0.65, instant=0.80 → NOT ProtectiveGuardian (instant is high)."""
        result = classify_phase(0.65, 0.80, SocialPhase.BuildingTrust)
        assert result != SocialPhase.ProtectiveGuardian
        # At coherence 0.65 from BuildingTrust, should advance to QuietlyBeloved
        assert result == SocialPhase.QuietlyBeloved


# ── Prompt composition (I-LLM-042) ───────────────────────────────────────────

class TestPromptComposition:
    """build_system_prompt prepends CCF injection and respects base_prompt."""

    def test_22_build_with_base_prompt_starts_with_injection(self):
        """build_system_prompt with non-empty base should start with the phase template."""
        result = build_system_prompt("Be concise.", SocialPhase.ShyObserver)
        expected_injection = PHASE_TEMPLATES["ShyObserver"]
        assert result.startswith(expected_injection), (
            f"Expected prompt to start with ShyObserver template. Got: {result[:80]!r}"
        )

    def test_23_build_with_empty_base_prompt_returns_just_injection(self):
        """build_system_prompt with empty base returns just the template (no double newline)."""
        result = build_system_prompt("", SocialPhase.ShyObserver)
        assert result == PHASE_TEMPLATES["ShyObserver"]
        assert "\n\n" not in result

    def test_24_prepend_order_ccf_before_user_prompt(self):
        """I-LLM-042: CCF injection appears BEFORE the user's base_prompt."""
        base = "Always respond in English."
        result = build_system_prompt(base, SocialPhase.BuildingTrust)
        injection = PHASE_TEMPLATES["BuildingTrust"]
        injection_pos = result.find(injection)
        user_pos = result.find(base)
        assert injection_pos < user_pos, (
            "CCF injection must appear before user base_prompt. "
            f"injection at {injection_pos}, user at {user_pos}"
        )
        # Confirm separator is present
        assert "\n\n" in result

    def test_25_whitespace_only_base_treated_as_empty(self):
        """Whitespace-only base_prompt behaves the same as empty string."""
        result = build_system_prompt("   ", SocialPhase.ShyObserver)
        assert result == PHASE_TEMPLATES["ShyObserver"]

    def test_26_all_phases_produce_nonempty_injection(self):
        """get_prompt_injection must return a non-empty string for every phase."""
        for phase in SocialPhase:
            injection = get_prompt_injection(phase)
            assert len(injection) > 10, f"{phase.value} produced empty injection"

    def test_27_thresholds_dict_has_all_expected_keys(self):
        """THRESHOLDS dict must contain all 6 expected keys."""
        expected_keys = {
            "shy_to_building_upper",
            "building_to_shy_lower",
            "building_to_beloved_upper",
            "beloved_to_building_lower",
            "guardian_instant_threshold",
            "startle_instant_threshold",
        }
        assert set(THRESHOLDS.keys()) >= expected_keys

    def test_28_threshold_values_match_spec(self):
        """THRESHOLDS values must match the spec exactly."""
        assert THRESHOLDS["shy_to_building_upper"] == pytest.approx(0.35)
        assert THRESHOLDS["building_to_shy_lower"] == pytest.approx(0.28)
        assert THRESHOLDS["building_to_beloved_upper"] == pytest.approx(0.55)
        assert THRESHOLDS["beloved_to_building_lower"] == pytest.approx(0.45)
        assert THRESHOLDS["guardian_instant_threshold"] == pytest.approx(0.30)
        assert THRESHOLDS["startle_instant_threshold"] == pytest.approx(0.20)

    def test_29_phase_templates_has_exactly_5_entries(self):
        """I-LLM-040: PHASE_TEMPLATES must have exactly 5 entries."""
        assert len(PHASE_TEMPLATES) == 5

    def test_30_all_social_phase_variants_covered_in_phase_templates(self):
        """I-LLM-040: Every SocialPhase.value must be a key in PHASE_TEMPLATES."""
        for phase in SocialPhase:
            assert phase.value in PHASE_TEMPLATES, (
                f"PHASE_TEMPLATES missing key for {phase.value}"
            )
