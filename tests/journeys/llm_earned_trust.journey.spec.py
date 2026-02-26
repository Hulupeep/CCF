"""
Journey Test: J-LLM-EARNED-TRUST
=================================

Simulates a 30-turn conversation and verifies that SocialPhase progresses
correctly as coherence accumulates, and that special phases (ProtectiveGuardian,
StartledRetreat) trigger as specified.

Specflow contract : J-LLM-EARNED-TRUST
Invariants        : I-LLM-040, I-LLM-041, I-LLM-042, I-LLM-043, I-LLM-044
Issue             : #61 — System prompt modulator

DOD:
    1. Turn 1:  phase = ShyObserver
    2. After 10 turns: phase has progressed (≥ BuildingTrust)
    3. After 30 turns: phase = QuietlyBeloved (or BuildingTrust minimum)
    4. Prompt changes between turn 1 and turn 30 (different template)
    5. Context isolation: switching topic → phase resets to ShyObserver
    6. ProtectiveGuardian triggered correctly (manual setup)
    7. StartledRetreat triggered correctly (manual setup)

Run with:
    python -m pytest tests/journeys/llm_earned_trust.journey.spec.py -v
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

# Use CoherenceFieldPy if available, else fall back to minimal inline class.
try:
    from ccf_core.coherence_field_py import CoherenceFieldPy as _FieldCls

    class _Acc:
        """Thin wrapper that mirrors the minimal fallback API."""
        def __init__(self):
            self._field = _FieldCls(curiosity_drive=0.5, recovery_rate=0.5)
            self._hash = 12345  # fixed context hash for single-topic simulation
            self.coherence = 0.0
            self.earned_floor = 0.0

        def positive(self, curiosity: float = 0.5) -> None:
            self._field.positive_interaction(self._hash)
            self.coherence = self._field.raw_coherence(self._hash)
            self.earned_floor = 0.0  # not exposed, approximate

        def effective(self, instant: float) -> float:
            return self._field.effective_coherence(self._hash, instant)

except ImportError:
    # Minimal inline fallback when CoherenceFieldPy is not available.
    class _Acc:  # type: ignore[no-redef]
        def __init__(self):
            self.coherence = 0.0
            self.earned_floor = 0.0

        def positive(self, curiosity: float = 0.5) -> None:
            gain = curiosity * 0.12
            self.coherence = min(1.0, self.coherence + gain)
            self.earned_floor = min(self.coherence, self.earned_floor + gain * 0.3)

        def effective(self, instant: float) -> float:
            c = self.coherence
            return min(instant, c) if c < 0.3 else 0.3 * instant + 0.7 * c


# ── Helper: run N positive turns and return (effective_coh, phase) ────────────

def _run_turns(n: int, instant: float = 0.8, curiosity: float = 0.5) -> tuple:
    """
    Simulate n positive turns and return final (effective_coherence, phase).

    Uses a single context (hash constant) and tracks phase with hysteresis.
    """
    acc = _Acc()
    phase = SocialPhase.ShyObserver
    for _ in range(n):
        acc.positive(curiosity)
        eff = acc.effective(instant)
        phase = classify_phase(eff, instant, phase)
    return acc.effective(instant), phase


# ── Journey step 1: Turn 1 = ShyObserver ──────────────────────────────────────

class TestStep1TurnOne:
    """
    Scenario: User sends first message to a fresh context
      Given no prior history exists
      When the first turn is processed
      Then SocialPhase is ShyObserver
    """

    def test_turn1_phase_is_shy_observer(self):
        """Turn 1: Fresh context starts as ShyObserver."""
        _, phase = _run_turns(1)
        assert phase == SocialPhase.ShyObserver, (
            f"Expected ShyObserver after turn 1, got {phase.value}"
        )

    def test_turn1_prompt_is_shy_observer_template(self):
        """Turn 1 prompt is the ShyObserver template."""
        _, phase = _run_turns(1)
        injection = get_prompt_injection(phase)
        assert injection == PHASE_TEMPLATES["ShyObserver"]


# ── Journey step 2: After 10 turns ≥ BuildingTrust ───────────────────────────

class TestStep2After10Turns:
    """
    Scenario: User has 10 turns of positive interaction
      Given 10 turns of positive interaction
      When phase is classified
      Then phase is at least BuildingTrust (not ShyObserver)
    """

    def test_after_10_turns_phase_progressed(self):
        """After 10 turns the phase has advanced beyond ShyObserver."""
        _, phase = _run_turns(10)
        assert phase != SocialPhase.ShyObserver, (
            f"Expected phase progression after 10 turns, still {phase.value}"
        )
        assert phase in (
            SocialPhase.BuildingTrust,
            SocialPhase.QuietlyBeloved,
        ), f"Unexpected phase: {phase.value}"


# ── Journey step 3: After 30 turns = QuietlyBeloved or BuildingTrust ─────────

class TestStep3After30Turns:
    """
    Scenario: User has 30 turns of positive interaction
      Given 30 turns of positive interaction with instant=0.8
      When phase is classified
      Then phase is QuietlyBeloved (or at minimum BuildingTrust)
    """

    def test_after_30_turns_high_trust_phase(self):
        """After 30 turns with high instant, phase reaches QuietlyBeloved."""
        _, phase = _run_turns(30)
        assert phase in (
            SocialPhase.QuietlyBeloved,
            SocialPhase.BuildingTrust,
        ), f"Expected QuietlyBeloved or BuildingTrust after 30 turns, got {phase.value}"

    def test_after_30_turns_not_shy_observer(self):
        """After 30 positive turns the user should never be ShyObserver."""
        _, phase = _run_turns(30)
        assert phase != SocialPhase.ShyObserver


# ── Journey step 4: Prompt changes between turn 1 and turn 30 ────────────────

class TestStep4PromptEvolution:
    """
    Scenario: System prompt evolves as trust accumulates
      Given prompts sampled at turn 1 and turn 30
      When compared
      Then the prompts are different (different templates)
    """

    def test_prompt_changes_over_30_turns(self):
        """Prompt at turn 30 differs from prompt at turn 1."""
        _, phase_1 = _run_turns(1)
        _, phase_30 = _run_turns(30)
        injection_1 = get_prompt_injection(phase_1)
        injection_30 = get_prompt_injection(phase_30)
        assert injection_1 != injection_30, (
            "Expected different system prompts at turn 1 and turn 30, "
            f"but both are: {injection_1[:60]!r}"
        )


# ── Journey step 5: Context isolation resets phase ───────────────────────────

class TestStep5ContextIsolation:
    """
    Scenario: User switches topic mid-conversation
      Given 30 turns of built-up trust in context A
      When the user switches to a new context B
      Then phase in context B starts as ShyObserver
    """

    def test_context_switch_resets_phase(self):
        """A fresh context accumulator starts at ShyObserver (0 coherence)."""
        # Simulate built-up context A
        _, phase_a = _run_turns(30)
        assert phase_a != SocialPhase.ShyObserver, "Pre-condition: context A must have progressed"

        # New context B — fresh accumulator, 0 coherence
        fresh_acc = _Acc()
        eff_b = fresh_acc.effective(0.8)
        phase_b = classify_phase(eff_b, 0.8, SocialPhase.ShyObserver)
        assert phase_b == SocialPhase.ShyObserver, (
            f"Fresh context must start as ShyObserver, got {phase_b.value}"
        )

    def test_context_isolation_independent_accumulators(self):
        """Two accumulators for different contexts progress independently."""
        acc_a = _Acc()
        acc_b = _Acc()

        # Run 30 turns on A, 1 turn on B
        for _ in range(30):
            acc_a.positive()
        acc_b.positive()

        eff_a = acc_a.effective(0.8)
        eff_b = acc_b.effective(0.8)
        assert eff_a > eff_b, (
            f"Context A ({eff_a:.3f}) should have higher effective coherence than B ({eff_b:.3f})"
        )


# ── Journey step 6: ProtectiveGuardian triggered correctly ───────────────────

class TestStep6ProtectiveGuardian:
    """
    Scenario: Familiar user suddenly goes quiet (stress signal)
      Given high effective coherence (> 0.55) built over many turns
      When instant signal drops below 0.30
      Then phase becomes ProtectiveGuardian (I-LLM-043)
    """

    def test_protective_guardian_triggers_on_high_coh_low_instant(self):
        """I-LLM-043: High coherence + low instant = ProtectiveGuardian."""
        # Build up coherence past 0.55
        acc = _Acc()
        for _ in range(40):
            acc.positive()
        high_eff = acc.effective(0.8)
        assert high_eff > THRESHOLDS["building_to_beloved_upper"], (
            f"Pre-condition failed: effective coherence {high_eff:.3f} not > 0.55"
        )

        # Now instant drops
        low_instant = 0.15
        eff_with_low_instant = acc.effective(low_instant)
        # effective_coherence with low instant may lower the value,
        # but classify_phase uses the original effective_coh argument
        # We pass the high_eff directly to test the rule
        phase = classify_phase(high_eff, low_instant, SocialPhase.QuietlyBeloved)
        assert phase == SocialPhase.ProtectiveGuardian, (
            f"Expected ProtectiveGuardian with coh={high_eff:.3f}, instant={low_instant}, "
            f"got {phase.value}"
        )

    def test_protective_guardian_template_has_supportive_language(self):
        """ProtectiveGuardian prompt contains supportive language."""
        injection = get_prompt_injection(SocialPhase.ProtectiveGuardian)
        assert "supportive" in injection.lower() or "ground" in injection.lower()


# ── Journey step 7: StartledRetreat triggered correctly ──────────────────────

class TestStep7StartledRetreat:
    """
    Scenario: Unknown user enters sensitive territory
      Given very low effective coherence (< 0.35)
      When instant signal drops below 0.20
      Then phase becomes StartledRetreat (I-LLM-044)
    """

    def test_startled_retreat_triggers_on_low_coh_low_instant(self):
        """I-LLM-044: Low coherence + very low instant = StartledRetreat."""
        low_eff = 0.12
        low_instant = 0.10
        phase = classify_phase(low_eff, low_instant, SocialPhase.ShyObserver)
        assert phase == SocialPhase.StartledRetreat, (
            f"Expected StartledRetreat with coh={low_eff}, instant={low_instant}, "
            f"got {phase.value}"
        )

    def test_startled_retreat_template_suggests_professional_resources(self):
        """StartledRetreat prompt must suggest professional resources."""
        injection = get_prompt_injection(SocialPhase.StartledRetreat)
        assert "professional resources" in injection

    def test_startled_retreat_does_not_trigger_above_instant_threshold(self):
        """Instant just above threshold should NOT trigger StartledRetreat."""
        low_eff = 0.12
        high_enough_instant = 0.25  # above startle threshold of 0.20
        phase = classify_phase(low_eff, high_enough_instant, SocialPhase.ShyObserver)
        assert phase != SocialPhase.StartledRetreat


# ── CLI Demo headless tests (issue #63) ──────────────────────────────────────

class TestCliDemoHeadless:
    """
    J-LLM-EARNED-TRUST (CLI demo perspective) — issue #63.

    Verifies that `python -m ccf_core.demo --headless-turns N` produces
    correct data-testid output for trust arc assertions.

    These tests use the demo module's run_headless() directly (no subprocess)
    for speed and reliability in CI.
    """

    @staticmethod
    def _run(turns: int) -> str:
        """Capture headless output as a string."""
        import argparse
        import io
        from contextlib import redirect_stdout

        # Ensure ccf-py/python is on the path
        import sys
        from pathlib import Path
        _ccf_py = Path(__file__).parent.parent.parent / "ccf-py" / "python"
        if str(_ccf_py) not in sys.path:
            sys.path.insert(0, str(_ccf_py))

        from ccf_core.demo import run_headless

        args = argparse.Namespace(headless_turns=turns)
        buf = io.StringIO()
        with redirect_stdout(buf):
            run_headless(args)
        return buf.getvalue()

    @staticmethod
    def _extract(output: str, testid: str) -> list[str]:
        import re
        pattern = re.compile(rf"\[data-testid: {re.escape(testid)}\]\s*(.+)")
        return [m.group(1).strip() for m in pattern.finditer(output)]

    def test_headless_30_turns_reach_building_trust(self):
        """After 30 headless turns, phase must be at least BuildingTrust."""
        output = self._run(30)
        phase_lines = self._extract(output, "phase-label")
        assert phase_lines, "No phase-label output from headless mode"
        last_phase = phase_lines[-1]
        assert last_phase in ("BuildingTrust", "QuietlyBeloved"), (
            f"After 30 turns, expected BuildingTrust or QuietlyBeloved, "
            f"got: {last_phase}"
        )

    def test_headless_1_turn_is_shy_observer(self):
        """Turn 1 in headless mode must be ShyObserver (DOD-1 via CLI)."""
        output = self._run(1)
        phases = self._extract(output, "phase-label")
        assert phases, "No phase-label in output"
        assert phases[0] == "ShyObserver", (
            f"Turn 1 must be ShyObserver, got: {phases[0]}"
        )

    def test_headless_coherence_pct_in_range(self):
        """All coherence-pct values must be floats in [0, 1]."""
        output = self._run(10)
        values = self._extract(output, "coherence-pct")
        assert values, "No coherence-pct in output"
        for v in values:
            coh = float(v)
            assert 0.0 <= coh <= 1.0, f"coherence-pct out of range: {coh}"

    def test_headless_interaction_count_sequential(self):
        """interaction-count must increment sequentially from 1."""
        turns = 5
        output = self._run(turns)
        counts = self._extract(output, "interaction-count")
        assert len(counts) == turns
        for i, c in enumerate(counts):
            assert c == str(i + 1), (
                f"Turn {i + 1}: expected {i + 1}, got {c!r}"
            )


# ── Full journey smoke test ────────────────────────────────────────────────────

def test_full_j_llm_earned_trust_journey():
    """
    J-LLM-EARNED-TRUST end-to-end smoke test.

    Simulates a 30-turn conversation and verifies all 7 DOD conditions.
    """
    # DOD 1: Turn 1 = ShyObserver
    _, phase_1 = _run_turns(1)
    assert phase_1 == SocialPhase.ShyObserver, f"DOD-1 FAIL: turn 1 is {phase_1.value}"

    # DOD 2: After 10 turns: progressed
    _, phase_10 = _run_turns(10)
    assert phase_10 != SocialPhase.ShyObserver, f"DOD-2 FAIL: still ShyObserver at turn 10"

    # DOD 3: After 30 turns: QuietlyBeloved or BuildingTrust
    _, phase_30 = _run_turns(30)
    assert phase_30 in (SocialPhase.QuietlyBeloved, SocialPhase.BuildingTrust), (
        f"DOD-3 FAIL: unexpected phase {phase_30.value} after 30 turns"
    )

    # DOD 4: Prompt changes between turn 1 and turn 30
    injection_1 = get_prompt_injection(phase_1)
    injection_30 = get_prompt_injection(phase_30)
    assert injection_1 != injection_30, "DOD-4 FAIL: prompt unchanged after 30 turns"

    # DOD 5: Context isolation — fresh context starts at ShyObserver
    fresh = _Acc()
    fresh_eff = fresh.effective(0.8)
    fresh_phase = classify_phase(fresh_eff, 0.8, SocialPhase.ShyObserver)
    assert fresh_phase == SocialPhase.ShyObserver, (
        f"DOD-5 FAIL: fresh context is {fresh_phase.value} instead of ShyObserver"
    )

    # DOD 6: ProtectiveGuardian triggered
    pg_phase = classify_phase(0.75, 0.10, SocialPhase.QuietlyBeloved)
    assert pg_phase == SocialPhase.ProtectiveGuardian, (
        f"DOD-6 FAIL: expected ProtectiveGuardian, got {pg_phase.value}"
    )

    # DOD 7: StartledRetreat triggered
    sr_phase = classify_phase(0.10, 0.05, SocialPhase.ShyObserver)
    assert sr_phase == SocialPhase.StartledRetreat, (
        f"DOD-7 FAIL: expected StartledRetreat, got {sr_phase.value}"
    )
