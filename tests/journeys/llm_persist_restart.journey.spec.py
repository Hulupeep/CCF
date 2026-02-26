"""
Journey Test: J-LLM-PERSIST-RESTART
=====================================

Verifies that CCF state survives a simulated process restart.

Specflow contract: J-LLM-PERSIST-RESTART
Invariants: I-LLM-050 through I-LLM-055
Issue: #62 — JSON persistence

DOD:
    - Full save/load cycle preserves effective_coherence within ±0.001
    - last_phase is preserved faithfully across restarts
    - File is portable: only saved_at can vary without breaking load
    - Atomic write: os.replace() failure leaves original file unchanged
    - No state file → fresh field, no error
    - Three simulated sessions accumulate trust cumulatively

Run with:
    python -m pytest tests/journeys/llm_persist_restart.journey.spec.py -v
"""

import json
import os
import sys
from pathlib import Path
from unittest.mock import patch

import pytest

sys.path.insert(0, "ccf-py/python")

from ccf_core.persistence import CcfPersistence
from ccf_core.coherence_field_py import CoherenceFieldPy


# ── Helpers ───────────────────────────────────────────────────────────────────

_CTX_HASH = 1_234_567  # stable synthetic context hash for all journey tests


def _accumulate(field: CoherenceFieldPy, ctx_hash: int, n: int) -> None:
    """Run *n* positive interactions on *field* for *ctx_hash*."""
    for _ in range(n):
        field.positive_interaction(ctx_hash)


def _snapshot_accs(field: CoherenceFieldPy, ctx_hash: int) -> dict:
    """Extract accumulator dict for one context hash from a CoherenceFieldPy."""
    acc = field._accumulators.get(ctx_hash)
    if acc is None:
        return {
            "coherence": 0.0,
            "earned_floor": 0.0,
            "interaction_count": 0,
            "last_phase": "ShyObserver",
        }
    return {
        "coherence": acc.coherence,
        "earned_floor": acc.earned_floor,
        "interaction_count": acc.interaction_count,
        "last_phase": "QuietlyBeloved",  # fixed phase label used in journey
    }


def _restore_field(accs: dict, personality: dict) -> CoherenceFieldPy:
    """Reconstruct a CoherenceFieldPy from loaded persistence dicts."""
    field = CoherenceFieldPy(
        curiosity_drive=personality["curiosity"],
        recovery_rate=personality["recovery"],
    )
    from ccf_core.coherence_field_py import _Accumulator
    for ctx_hash, rec in accs.items():
        field._accumulators[ctx_hash] = _Accumulator(
            coherence=rec["coherence"],
            earned_floor=rec["earned_floor"],
            interaction_count=rec["interaction_count"],
        )
    return field


# ── Scenario 1: Full cycle — 30 interactions, save, reload, check coherence ──

class TestFullSaveRestoreCycle:
    """
    Scenario: Full save/load cycle preserves coherence
      Given a CoherenceFieldPy accumulating 30 positive interactions
      When state is saved to disk and reloaded into a fresh field
      Then effective_coherence of the restored field equals original within ±0.001
    """

    def test_effective_coherence_preserved(self, tmp_path):
        """I-LLM-051: restored field effective_coherence within ±0.001."""
        field = CoherenceFieldPy(curiosity_drive=0.5, recovery_rate=0.5)
        _accumulate(field, _CTX_HASH, 30)

        original_ec = field.effective_coherence(_CTX_HASH, instant=0.8)
        original_raw = field.raw_coherence(_CTX_HASH)

        accs = {_CTX_HASH: _snapshot_accs(field, _CTX_HASH)}
        personality = {"curiosity": field.curiosity_drive, "recovery": field.recovery_rate}

        dest = tmp_path / "session.ccf.json"
        CcfPersistence.save(accs, personality, str(dest))

        loaded_accs, loaded_pers = CcfPersistence.load(str(dest))
        restored = _restore_field(loaded_accs, loaded_pers)

        restored_ec = restored.effective_coherence(_CTX_HASH, instant=0.8)
        assert abs(restored_ec - original_ec) < 0.001, (
            f"I-LLM-051 violated: original={original_ec:.6f}, "
            f"restored={restored_ec:.6f}, delta={abs(restored_ec - original_ec):.6f}"
        )

    def test_raw_coherence_preserved(self, tmp_path):
        """Raw (learned) coherence must also round-trip within ±0.001."""
        field = CoherenceFieldPy(curiosity_drive=0.7, recovery_rate=0.3)
        _accumulate(field, _CTX_HASH, 20)
        original_raw = field.raw_coherence(_CTX_HASH)

        accs = {_CTX_HASH: _snapshot_accs(field, _CTX_HASH)}
        personality = {"curiosity": field.curiosity_drive, "recovery": field.recovery_rate}

        dest = tmp_path / "session.ccf.json"
        CcfPersistence.save(accs, personality, str(dest))
        loaded_accs, loaded_pers = CcfPersistence.load(str(dest))
        restored = _restore_field(loaded_accs, loaded_pers)

        assert abs(restored.raw_coherence(_CTX_HASH) - original_raw) < 0.001


# ── Scenario 2: Phase preserved across restart ────────────────────────────────

class TestPhasePreserved:
    """
    Scenario: Saved QuietlyBeloved phase reloads as QuietlyBeloved
      Given a context whose last_phase is set to "QuietlyBeloved"
      When state is saved and reloaded
      Then last_phase is "QuietlyBeloved" in the restored accumulator dict
    """

    def test_last_phase_round_trips(self, tmp_path):
        accs = {
            _CTX_HASH: {
                "coherence": 0.62,
                "earned_floor": 0.18,
                "interaction_count": 30,
                "last_phase": "QuietlyBeloved",
            }
        }
        personality = {"curiosity": 0.5, "recovery": 0.5}
        dest = tmp_path / "phase.ccf.json"
        CcfPersistence.save(accs, personality, str(dest))
        loaded_accs, _ = CcfPersistence.load(str(dest))

        assert loaded_accs[_CTX_HASH]["last_phase"] == "QuietlyBeloved", (
            f"last_phase not preserved: got {loaded_accs[_CTX_HASH]['last_phase']}"
        )

    def test_multiple_phases_preserved(self, tmp_path):
        """Different phases per context must all be preserved."""
        accs = {
            1: {"coherence": 0.1, "earned_floor": 0.0, "interaction_count": 2,
                "last_phase": "ShyObserver"},
            2: {"coherence": 0.62, "earned_floor": 0.18, "interaction_count": 30,
                "last_phase": "QuietlyBeloved"},
            3: {"coherence": 0.91, "earned_floor": 0.50, "interaction_count": 80,
                "last_phase": "Radiant"},
        }
        dest = tmp_path / "multi_phase.ccf.json"
        CcfPersistence.save(accs, {"curiosity": 0.5, "recovery": 0.5}, str(dest))
        loaded_accs, _ = CcfPersistence.load(str(dest))

        for ctx_hash, original in accs.items():
            assert loaded_accs[ctx_hash]["last_phase"] == original["last_phase"]


# ── Scenario 3: Portability — only saved_at may vary ─────────────────────────

class TestPortability:
    """
    Scenario: File is portable across machines
      Given a saved state file
      When only the saved_at timestamp is modified (simulating a copy to another machine)
      Then load() still returns the correct state
    """

    def test_modified_saved_at_loads_ok(self, tmp_path):
        """I-LLM-052: modifying saved_at must not break load()."""
        accs = {
            _CTX_HASH: {
                "coherence": 0.5,
                "earned_floor": 0.1,
                "interaction_count": 15,
                "last_phase": "QuietlyBeloved",
            }
        }
        personality = {"curiosity": 0.6, "recovery": 0.4}
        dest = tmp_path / "portable.ccf.json"
        CcfPersistence.save(accs, personality, str(dest))

        # Simulate copy to another machine with different clock
        with open(dest, encoding="utf-8") as f:
            data = json.load(f)
        data["saved_at"] = "2030-01-01T00:00:00+00:00"
        with open(dest, "w", encoding="utf-8") as f:
            json.dump(data, f)

        loaded_accs, loaded_pers = CcfPersistence.load(str(dest))
        assert loaded_accs is not None, "load() must succeed after saved_at change"
        assert abs(loaded_accs[_CTX_HASH]["coherence"] - 0.5) < 0.001


# ── Scenario 4: Atomic write — os.replace failure leaves original intact ──────

class TestAtomicWrite:
    """
    Scenario: Interrupted save leaves original file unchanged
      Given an existing valid state file
      When os.replace() raises during a new save
      Then the original file is byte-for-byte identical to before
    """

    def test_original_unchanged_after_interrupted_save(self, tmp_path):
        """I-LLM-053: failed atomic write must not corrupt existing state."""
        dest = tmp_path / "state.ccf.json"

        original_accs = {
            _CTX_HASH: {
                "coherence": 0.75,
                "earned_floor": 0.30,
                "interaction_count": 40,
                "last_phase": "QuietlyBeloved",
            }
        }
        CcfPersistence.save(original_accs, {"curiosity": 0.5, "recovery": 0.5}, str(dest))
        original_bytes = dest.read_bytes()

        new_accs = {
            999: {
                "coherence": 0.1,
                "earned_floor": 0.0,
                "interaction_count": 1,
                "last_phase": "ShyObserver",
            }
        }
        with patch("os.replace", side_effect=OSError("simulated disk full")):
            with pytest.raises(OSError):
                CcfPersistence.save(new_accs, {"curiosity": 0.5, "recovery": 0.5}, str(dest))

        assert dest.read_bytes() == original_bytes, (
            "I-LLM-053: original file corrupted by interrupted save"
        )

    def test_no_orphan_temp_files_after_failure(self, tmp_path):
        """Temp file must be cleaned up when os.replace() fails."""
        dest = tmp_path / "state.ccf.json"
        with patch("os.replace", side_effect=OSError("fail")):
            with pytest.raises(OSError):
                CcfPersistence.save(
                    {1: {"coherence": 0.1, "earned_floor": 0.0,
                         "interaction_count": 1, "last_phase": "ShyObserver"}},
                    {"curiosity": 0.5, "recovery": 0.5},
                    str(dest),
                )
        # The only file in tmp_path must NOT be a dangling .ccf.tmp
        leftover_tmps = list(tmp_path.glob("*.ccf.tmp"))
        assert leftover_tmps == [], (
            f"Orphaned temp file(s) after failed save: {leftover_tmps}"
        )


# ── Scenario 5: No state file → fresh field, no error ────────────────────────

class TestFreshStart:
    """
    Scenario: Process starts with no prior state file
      Given no state file exists at the configured path
      When load() is called
      Then (None, None) is returned — caller initialises a fresh field
      And no exception is raised
    """

    def test_missing_file_returns_none_none(self, tmp_path):
        result = CcfPersistence.load(str(tmp_path / "doesnotexist.ccf.json"))
        assert result == (None, None)

    def test_caller_can_build_fresh_field_from_none(self, tmp_path):
        """Demonstrate that (None, None) allows graceful fresh-start initialisation."""
        accs, pers = CcfPersistence.load(str(tmp_path / "missing.ccf.json"))
        # Caller defaults
        if accs is None:
            accs = {}
        if pers is None:
            pers = {"curiosity": 0.5, "recovery": 0.5}

        field = CoherenceFieldPy(
            curiosity_drive=pers["curiosity"],
            recovery_rate=pers["recovery"],
        )
        # Fresh field has no accumulators; effective_coherence returns 0
        assert field.effective_coherence(_CTX_HASH, instant=0.8) == 0.0


# ── Scenario 6: Three sessions accumulate trust cumulatively ─────────────────

class TestMultiSessionAccumulation:
    """
    Scenario: Multiple restart cycles accumulate trust
      Given three separate simulated sessions, each running 10 positive interactions
      When each session saves its state and the next session restores it
      Then coherence after session 3 is greater than after session 1
      And coherence grows monotonically across sessions
    """

    def test_coherence_grows_across_sessions(self, tmp_path):
        """Cumulatively accumulated coherence must grow across restarts.

        3 interactions per session with curiosity=0.5 gives gain 0.06/tick.
        After 3 ticks: ~0.18, after 6: ~0.36, after 9: ~0.54 — well below
        the 1.0 cap, so all three readings are strictly increasing.
        """
        dest = tmp_path / "multi_session.ccf.json"
        coherence_readings = []

        for session_num in range(3):
            # Restore from previous session (or start fresh)
            if dest.exists():
                accs, pers = CcfPersistence.load(str(dest))
            else:
                accs, pers = None, None

            if accs is None:
                accs = {}
            if pers is None:
                pers = {"curiosity": 0.5, "recovery": 0.5}

            field = _restore_field(accs, pers)

            # Run 3 interactions per session (gain = 0.5 * 0.12 * 3 = 0.18/session)
            _accumulate(field, _CTX_HASH, 3)
            coherence_readings.append(field.raw_coherence(_CTX_HASH))

            # Save and "restart"
            updated_accs = {_CTX_HASH: _snapshot_accs(field, _CTX_HASH)}
            CcfPersistence.save(updated_accs, pers, str(dest))

        # Coherence must grow monotonically
        assert coherence_readings[1] > coherence_readings[0], (
            f"Session 2 coherence {coherence_readings[1]:.4f} must exceed "
            f"session 1 coherence {coherence_readings[0]:.4f}"
        )
        assert coherence_readings[2] > coherence_readings[1], (
            f"Session 3 coherence {coherence_readings[2]:.4f} must exceed "
            f"session 2 coherence {coherence_readings[1]:.4f}"
        )

    def test_interaction_count_accumulates_across_sessions(self, tmp_path):
        """interaction_count must increase with each session."""
        dest = tmp_path / "count_session.ccf.json"
        counts = []

        for _ in range(3):
            if dest.exists():
                accs, pers = CcfPersistence.load(str(dest))
            else:
                accs, pers = None, None
            if accs is None:
                accs = {}
            if pers is None:
                pers = {"curiosity": 0.5, "recovery": 0.5}

            field = _restore_field(accs, pers)
            _accumulate(field, _CTX_HASH, 5)
            counts.append(field.interaction_count(_CTX_HASH))

            updated_accs = {_CTX_HASH: _snapshot_accs(field, _CTX_HASH)}
            updated_accs[_CTX_HASH]["interaction_count"] = field.interaction_count(_CTX_HASH)
            CcfPersistence.save(updated_accs, pers, str(dest))

        assert counts[0] == 5
        assert counts[1] == 10
        assert counts[2] == 15
