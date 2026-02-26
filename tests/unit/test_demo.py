"""
Unit tests for ccf_core.demo — headless mode.

Tests: I-LLM-060 through I-LLM-065
Journey: J-LLM-EARNED-TRUST

Run:
    python -m pytest tests/unit/test_demo.py -v --tb=short
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

import pytest

# Make ccf_core importable from repo root
_REPO_ROOT = Path(__file__).parent.parent.parent
_CCF_PY = _REPO_ROOT / "ccf-py" / "python"
sys.path.insert(0, str(_CCF_PY))

from ccf_core.demo import parse_args, run_headless
from ccf_core.social_phase import SocialPhase


# ── Helpers ───────────────────────────────────────────────────────────────────

def _extract_testid(output: str, testid: str) -> list[str]:
    """Return all values for a given data-testid from headless output."""
    pattern = re.compile(rf"\[data-testid: {re.escape(testid)}\]\s*(.+)")
    return [m.group(1).strip() for m in pattern.finditer(output)]


def _run_headless_captured(turns: int) -> str:
    """Run headless mode and capture printed output."""
    import io
    from contextlib import redirect_stdout

    args = argparse.Namespace(headless_turns=turns)
    buf = io.StringIO()
    with redirect_stdout(buf):
        run_headless(args)
    return buf.getvalue()


# ── Tests ─────────────────────────────────────────────────────────────────────

class TestHeadlessModeBasic:
    """I-LLM-065: --no-dashboard headless / scripted mode."""

    def test_headless_1_turn_runs_without_error(self):
        """--headless-turns 1 completes without exception."""
        output = _run_headless_captured(1)
        assert output, "Expected non-empty output from headless mode"

    def test_headless_1_turn_phase_label_present(self):
        """After 1 turn: [data-testid: phase-label] is in output."""
        output = _run_headless_captured(1)
        values = _extract_testid(output, "phase-label")
        assert values, "Expected [data-testid: phase-label] in headless output"
        assert values[0] == "ShyObserver", (
            f"Turn 1 should be ShyObserver, got: {values[0]}"
        )

    def test_headless_1_turn_coherence_is_float_in_range(self):
        """After 1 turn: [data-testid: coherence-pct] is a float in [0, 1]."""
        output = _run_headless_captured(1)
        values = _extract_testid(output, "coherence-pct")
        assert values, "Expected [data-testid: coherence-pct] in headless output"
        coh = float(values[0])
        assert 0.0 <= coh <= 1.0, f"coherence-pct out of range [0,1]: {coh}"

    def test_headless_1_turn_ctx_hash_is_hex(self):
        """After 1 turn: [data-testid: ctx-hash-display] is a hex string."""
        output = _run_headless_captured(1)
        values = _extract_testid(output, "ctx-hash-display")
        assert values, "Expected [data-testid: ctx-hash-display] in headless output"
        # Should be parseable as hex
        int(values[0], 16)

    def test_headless_interaction_count_matches_turn_number(self):
        """[data-testid: interaction-count] matches the turn number."""
        turns = 5
        output = _run_headless_captured(turns)
        counts = _extract_testid(output, "interaction-count")
        assert len(counts) == turns, (
            f"Expected {turns} interaction-count entries, got {len(counts)}"
        )
        for i, count_str in enumerate(counts):
            assert count_str == str(i + 1), (
                f"Turn {i + 1}: expected interaction-count={i + 1}, got {count_str!r}"
            )


class TestPhasProgression:
    """I-LLM-061: Phase transitions are visually distinct."""

    def test_headless_30_turns_phase_has_progressed(self):
        """After 30 turns of finance conversation, phase must have progressed."""
        output = _run_headless_captured(30)
        values = _extract_testid(output, "phase-label")
        assert values, "No phase-label output"

        # Final phase should not still be ShyObserver
        last_phase = values[-1]
        valid_progressed = {"BuildingTrust", "QuietlyBeloved"}
        assert last_phase in valid_progressed, (
            f"After 30 turns expected BuildingTrust or QuietlyBeloved, "
            f"got: {last_phase}"
        )

    def test_headless_30_turns_coherence_increases(self):
        """Coherence after 30 turns should be higher than after 1 turn."""
        output_1 = _run_headless_captured(1)
        output_30 = _run_headless_captured(30)

        coh_1 = float(_extract_testid(output_1, "coherence-pct")[0])
        coh_30_all = _extract_testid(output_30, "coherence-pct")
        coh_30 = float(coh_30_all[-1])

        assert coh_30 > coh_1, (
            f"Coherence should increase over 30 turns: {coh_1:.4f} -> {coh_30:.4f}"
        )

    def test_headless_all_phases_are_valid(self):
        """All emitted phase-label values must be valid SocialPhase names."""
        output = _run_headless_captured(30)
        values = _extract_testid(output, "phase-label")
        valid = {p.value for p in SocialPhase}
        for phase_name in values:
            assert phase_name in valid, (
                f"Invalid phase name in output: {phase_name!r}. Valid: {valid}"
            )


class TestNoDashboardMode:
    """I-LLM-065: --no-dashboard exits cleanly."""

    def test_parse_args_no_dashboard(self):
        """--no-dashboard flag is parsed correctly."""
        args = parse_args(["--no-dashboard"])
        assert args.no_dashboard is True

    def test_parse_args_headless_turns(self):
        """--headless-turns N is parsed correctly."""
        args = parse_args(["--headless-turns", "10"])
        assert args.headless_turns == 10

    def test_parse_args_defaults(self):
        """Default args are sane."""
        args = parse_args([])
        assert args.model == "llama3"
        assert args.no_dashboard is False
        assert args.headless_turns == 0


class TestDemoClassInit:
    """Test CcfDemo initialises correctly without running the interactive loop."""

    def test_demo_init_no_state_file(self, tmp_path):
        """CcfDemo initialises with a fresh field when state file absent."""
        from ccf_core.demo import CcfDemo

        state_file = str(tmp_path / "test_state.json")
        demo = CcfDemo(
            model="llama3",
            state_file=state_file,
            show_dashboard=False,
        )
        assert demo.turn_count == 0
        assert demo.current_phase == SocialPhase.ShyObserver
        assert demo.current_coherence == 0.0
        assert demo.field is not None

    def test_demo_process_turn_headless(self, tmp_path):
        """_process_turn() runs without error in no-dashboard mode."""
        from ccf_core.demo import CcfDemo

        state_file = str(tmp_path / "test_state.json")
        demo = CcfDemo(
            model="llama3",
            state_file=state_file,
            show_dashboard=False,
        )
        # Should not raise even if Ollama is unavailable
        demo._process_turn("what is compound interest")
        assert demo.turn_count == 1
        assert demo.current_key is not None
        assert 0.0 <= demo.current_coherence <= 1.0
