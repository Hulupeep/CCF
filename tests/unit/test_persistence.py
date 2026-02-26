"""
Unit tests for CcfPersistence — atomic save/load of CoherenceField state.

Covers invariants: I-LLM-050 through I-LLM-055
Issue: #62 — JSON persistence

Run with:
    python -m pytest tests/unit/test_persistence.py -v
"""

import json
import os
import sys
from datetime import timezone
from pathlib import Path
from unittest.mock import patch

import pytest

sys.path.insert(0, "ccf-py/python")

from ccf_core.persistence import CCF_SCHEMA_VERSION, CcfLoadError, CcfPersistence


# ── Helpers ───────────────────────────────────────────────────────────────────

def _make_accumulators(n: int = 3) -> dict:
    """Return *n* distinct accumulator records keyed by sequential hash values."""
    return {
        i * 1_000_000: {
            "coherence": 0.1 * (i + 1),
            "earned_floor": 0.03 * (i + 1),
            "interaction_count": 10 * (i + 1),
            "last_phase": "ShyObserver" if i % 2 == 0 else "QuietlyBeloved",
        }
        for i in range(n)
    }


def _make_personality(curiosity: float = 0.6, recovery: float = 0.4) -> dict:
    return {"curiosity": curiosity, "recovery": recovery}


# ── 1. save() creates a valid JSON file ──────────────────────────────────────

def test_save_creates_json_file(tmp_path):
    """I-LLM-050: save() must produce a single, readable JSON file."""
    dest = tmp_path / "state.ccf.json"
    CcfPersistence.save(_make_accumulators(), _make_personality(), str(dest))
    assert dest.exists(), "save() must create the destination file"
    with open(dest, encoding="utf-8") as f:
        data = json.load(f)
    assert isinstance(data, dict), "File must contain a JSON object"


# ── 2. load() round-trips coherence within ±0.001 ────────────────────────────

def test_load_coherence_round_trips(tmp_path):
    """I-LLM-051: restored coherence must equal original within ±0.001."""
    accs = _make_accumulators(3)
    dest = tmp_path / "state.ccf.json"
    CcfPersistence.save(accs, _make_personality(), str(dest))
    restored_accs, _ = CcfPersistence.load(str(dest))

    for ctx_hash, original in accs.items():
        assert ctx_hash in restored_accs, f"ctx_hash {ctx_hash} missing after load"
        assert abs(restored_accs[ctx_hash]["coherence"] - original["coherence"]) < 0.001, (
            f"Coherence mismatch for ctx_hash {ctx_hash}"
        )


# ── 3. load() on missing file returns (None, None) ───────────────────────────

def test_load_missing_file_returns_none(tmp_path):
    """I-LLM-053 / forward-compat: absent file must not raise."""
    result = CcfPersistence.load(str(tmp_path / "nonexistent.ccf.json"))
    assert result == (None, None), "Missing file must return (None, None)"


# ── 4. load() on corrupted JSON returns (None, None) ─────────────────────────

def test_load_corrupted_json_returns_none(tmp_path):
    """I-LLM-054: corrupted file must not raise; caller gets (None, None)."""
    dest = tmp_path / "corrupt.ccf.json"
    dest.write_text("{not valid json{{{{", encoding="utf-8")
    result = CcfPersistence.load(str(dest))
    assert result == (None, None), "Corrupted JSON must return (None, None)"


# ── 5. Atomic write: interrupted rename leaves original intact ────────────────

def test_atomic_write_leaves_original_on_failure(tmp_path):
    """I-LLM-053: if os.replace() raises, the original file must be unchanged."""
    dest = tmp_path / "state.ccf.json"
    # Write an initial valid state
    original_accs = {999: {"coherence": 0.99, "earned_floor": 0.5,
                           "interaction_count": 100, "last_phase": "Radiant"}}
    CcfPersistence.save(original_accs, _make_personality(), str(dest))
    original_content = dest.read_text(encoding="utf-8")

    # Now simulate a failed second save (os.replace raises mid-write)
    new_accs = {1: {"coherence": 0.1, "earned_floor": 0.0,
                    "interaction_count": 1, "last_phase": "ShyObserver"}}
    with patch("os.replace", side_effect=OSError("disk full")):
        with pytest.raises(OSError):
            CcfPersistence.save(new_accs, _make_personality(), str(dest))

    # Original file must be exactly as before
    assert dest.read_text(encoding="utf-8") == original_content, (
        "I-LLM-053: original file must be unchanged after interrupted save"
    )


# ── 6. Unknown fields are ignored (I-LLM-054) ────────────────────────────────

def test_unknown_fields_ignored_on_load(tmp_path):
    """I-LLM-054: extra fields in the JSON must not cause errors."""
    dest = tmp_path / "state.ccf.json"
    CcfPersistence.save(_make_accumulators(1), _make_personality(), str(dest))

    # Inject unknown fields at the top level and inside a context record
    with open(dest, encoding="utf-8") as f:
        data = json.load(f)
    data["future_field"] = 42
    data["contexts"][0]["unknown_ctx_field"] = "hello"
    with open(dest, "w", encoding="utf-8") as f:
        json.dump(data, f)

    accs, pers = CcfPersistence.load(str(dest))
    assert accs is not None, "load() must succeed even with unknown fields"
    assert pers is not None


# ── 7. Personality round-trips correctly ─────────────────────────────────────

def test_personality_round_trips(tmp_path):
    """Personality curiosity and recovery must be restored exactly."""
    dest = tmp_path / "state.ccf.json"
    original = {"curiosity": 0.73, "recovery": 0.29}
    CcfPersistence.save({}, original, str(dest))
    _, restored = CcfPersistence.load(str(dest))
    assert abs(restored["curiosity"] - original["curiosity"]) < 1e-9
    assert abs(restored["recovery"] - original["recovery"]) < 1e-9


# ── 8. Multiple contexts round-trip ──────────────────────────────────────────

def test_multiple_contexts_round_trip(tmp_path):
    """Three contexts with different hashes must all survive save/load."""
    accs = {
        111: {"coherence": 0.2, "earned_floor": 0.05, "interaction_count": 5,
              "last_phase": "ShyObserver"},
        222: {"coherence": 0.55, "earned_floor": 0.20, "interaction_count": 20,
              "last_phase": "QuietlyBeloved"},
        333: {"coherence": 0.88, "earned_floor": 0.40, "interaction_count": 50,
              "last_phase": "Radiant"},
    }
    dest = tmp_path / "multi.ccf.json"
    CcfPersistence.save(accs, _make_personality(), str(dest))
    restored, _ = CcfPersistence.load(str(dest))

    assert len(restored) == 3
    for ctx_hash, original in accs.items():
        assert ctx_hash in restored
        assert restored[ctx_hash]["interaction_count"] == original["interaction_count"]
        assert abs(restored[ctx_hash]["coherence"] - original["coherence"]) < 0.001


# ── 9. File size < 50 KB for 64 contexts (I-LLM-055) ────────────────────────

def test_file_size_under_50kb_for_64_contexts(tmp_path):
    """I-LLM-055: 64 active contexts must produce a file smaller than 50 KB."""
    accs = {
        i: {
            "coherence": 0.5,
            "earned_floor": 0.2,
            "interaction_count": 30,
            "last_phase": "QuietlyBeloved",
        }
        for i in range(64)
    }
    dest = tmp_path / "large.ccf.json"
    CcfPersistence.save(accs, _make_personality(), str(dest))
    size_bytes = dest.stat().st_size
    assert size_bytes < 50 * 1024, (
        f"I-LLM-055: file size {size_bytes} bytes exceeds 50 KB limit"
    )


# ── 10. saved_at is a valid ISO 8601 timestamp ────────────────────────────────

def test_saved_at_is_iso8601(tmp_path):
    """saved_at must be parseable as an ISO 8601 timestamp."""
    from datetime import datetime
    dest = tmp_path / "state.ccf.json"
    CcfPersistence.save({}, _make_personality(), str(dest))
    with open(dest, encoding="utf-8") as f:
        data = json.load(f)
    saved_at = data["saved_at"]
    # datetime.fromisoformat raises ValueError if format is invalid
    dt = datetime.fromisoformat(saved_at)
    assert dt.tzinfo is not None, "saved_at must include timezone info"


# ── 11. version field is set to 1 ────────────────────────────────────────────

def test_version_field_is_1(tmp_path):
    """The JSON version field must equal CCF_SCHEMA_VERSION (1)."""
    dest = tmp_path / "state.ccf.json"
    CcfPersistence.save({}, _make_personality(), str(dest))
    with open(dest, encoding="utf-8") as f:
        data = json.load(f)
    assert data["version"] == CCF_SCHEMA_VERSION


# ── 12. ccf_version matches the passed string ────────────────────────────────

def test_ccf_version_stored_correctly(tmp_path):
    """ccf_version in JSON must match the value passed to save()."""
    dest = tmp_path / "state.ccf.json"
    CcfPersistence.save({}, _make_personality(), str(dest), ccf_version="9.9.9")
    with open(dest, encoding="utf-8") as f:
        data = json.load(f)
    assert data["ccf_version"] == "9.9.9"


# ── 13. ctx_hash stored as integer ───────────────────────────────────────────

def test_ctx_hash_stored_as_integer(tmp_path):
    """ctx_hash in each context record must be a JSON integer, not a string."""
    accs = {42: {"coherence": 0.5, "earned_floor": 0.1,
                 "interaction_count": 1, "last_phase": "ShyObserver"}}
    dest = tmp_path / "state.ccf.json"
    CcfPersistence.save(accs, _make_personality(), str(dest))
    with open(dest, encoding="utf-8") as f:
        data = json.load(f)
    ctx = data["contexts"][0]
    assert isinstance(ctx["ctx_hash"], int), (
        f"ctx_hash must be int in JSON, got {type(ctx['ctx_hash'])}"
    )
    assert ctx["ctx_hash"] == 42


# ── 14. Empty accumulators round-trips as empty contexts list ─────────────────

def test_empty_accumulators_round_trip(tmp_path):
    """Empty accumulators dict must produce an empty contexts list and reload OK."""
    dest = tmp_path / "empty.ccf.json"
    CcfPersistence.save({}, _make_personality(), str(dest))
    restored_accs, restored_pers = CcfPersistence.load(str(dest))
    assert restored_accs == {}, "Empty accumulators must reload as empty dict"
    assert restored_pers is not None


# ── 15. Malformed context record skipped; others loaded ──────────────────────

def test_malformed_context_record_skipped(tmp_path):
    """I-LLM-054: a malformed context record must be skipped; valid ones loaded."""
    dest = tmp_path / "state.ccf.json"
    CcfPersistence.save(
        {777: {"coherence": 0.4, "earned_floor": 0.1, "interaction_count": 5,
               "last_phase": "ShyObserver"}},
        _make_personality(),
        str(dest),
    )

    # Inject a malformed context record (missing ctx_hash)
    with open(dest, encoding="utf-8") as f:
        data = json.load(f)
    data["contexts"].append({"broken": True, "no_hash_key": "oops"})
    with open(dest, "w", encoding="utf-8") as f:
        json.dump(data, f)

    restored, _ = CcfPersistence.load(str(dest))
    assert 777 in restored, "Valid context record must be loaded"
    assert len(restored) == 1, "Malformed record must be silently skipped"
