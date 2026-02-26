"""
CCF persistence — atomic save/load of CoherenceField state.

Invariants: I-LLM-050 through I-LLM-055
Journey: J-LLM-PERSIST-RESTART
Issue: #62 — JSON persistence
"""

import json
import os
import tempfile
from datetime import datetime, timezone
from pathlib import Path

CCF_SCHEMA_VERSION = 1


class CcfLoadError(Exception):
    """Raised when state file has an incompatible schema version."""


class CcfPersistence:
    """Save/restore CoherenceField state as a single JSON file.

    Invariants enforced:
        I-LLM-050: Single JSON file — no database, no cloud.
        I-LLM-051: Restored field produces identical effective_coherence (±0.001).
        I-LLM-052: File is portable across machines.
        I-LLM-053: Save is atomic — write-then-rename.
        I-LLM-054: Unknown fields ignored gracefully (forward compat).
        I-LLM-055: File size < 50 KB for 64 active contexts.
    """

    @staticmethod
    def save(
        accumulators: dict,
        personality: dict,
        path: str,
        ccf_version: str = "0.1.4",
    ) -> None:
        """Atomic save: serialize state to temp file, then rename over destination.

        Args:
            accumulators: Mapping of ctx_hash (int) to dict with keys:
                coherence (float), earned_floor (float),
                interaction_count (int), last_phase (str).
            personality: Dict with keys ``curiosity`` and ``recovery``.
            path: Destination file path (parent directories created if absent).
            ccf_version: ccf-core version string embedded for forward-compat tagging.

        Raises:
            OSError: If the filesystem rejects the write or rename.
        """
        state = {
            "version": CCF_SCHEMA_VERSION,
            "ccf_version": ccf_version,
            "saved_at": datetime.now(timezone.utc).isoformat(),
            "personality": {
                "curiosity": float(personality.get("curiosity", 0.5)),
                "recovery": float(personality.get("recovery", 0.5)),
            },
            "contexts": [
                {
                    "ctx_hash": int(ctx_hash),
                    "coherence": float(acc["coherence"]),
                    "earned_floor": float(acc["earned_floor"]),
                    "interaction_count": int(acc["interaction_count"]),
                    "last_phase": str(acc.get("last_phase", "ShyObserver")),
                }
                for ctx_hash, acc in accumulators.items()
            ],
        }

        # Atomic write: temp file in same directory ensures same filesystem,
        # so os.replace() is a rename (not a cross-device copy).  I-LLM-053.
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)

        fd, tmp_path = tempfile.mkstemp(dir=path.parent, suffix=".ccf.tmp")
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as f:
                json.dump(state, f, indent=2)
            os.replace(tmp_path, path)
        except Exception:
            # Clean up orphaned temp file on any failure before re-raising.
            try:
                os.unlink(tmp_path)
            except OSError:
                pass
            raise

    @staticmethod
    def load(path: str) -> "tuple[dict | None, dict | None]":
        """Load state from a JSON file produced by :meth:`save`.

        Returns:
            ``(accumulators, personality)`` on success, where *accumulators* maps
            ``ctx_hash`` (int) to a dict and *personality* has keys ``curiosity``
            and ``recovery``.

            ``(None, None)`` if the file does not exist or is corrupted JSON.

        Raises:
            CcfLoadError: If ``version`` in the file is newer than
                :data:`CCF_SCHEMA_VERSION` (signals a forward-compat break).
        """
        path = Path(path)
        if not path.exists():
            return None, None

        try:
            with open(path, encoding="utf-8") as f:
                state = json.load(f)
        except (json.JSONDecodeError, OSError):
            # Corrupted or unreadable file → caller gets a fresh state.  I-LLM-054.
            return None, None

        version = state.get("version", 1)
        if version > CCF_SCHEMA_VERSION:
            raise CcfLoadError(
                f"State file version {version} is newer than supported version "
                f"{CCF_SCHEMA_VERSION}. Please upgrade ccf-core."
            )

        # I-LLM-054: unknown top-level fields are simply ignored.
        personality = {
            "curiosity": float(
                state.get("personality", {}).get("curiosity", 0.5)
            ),
            "recovery": float(
                state.get("personality", {}).get("recovery", 0.5)
            ),
        }

        accumulators: dict = {}
        for ctx_record in state.get("contexts", []):
            try:
                ctx_hash = int(ctx_record["ctx_hash"])
                accumulators[ctx_hash] = {
                    "coherence": float(ctx_record.get("coherence", 0.0)),
                    "earned_floor": float(ctx_record.get("earned_floor", 0.0)),
                    "interaction_count": int(
                        ctx_record.get("interaction_count", 0)
                    ),
                    "last_phase": str(
                        ctx_record.get("last_phase", "ShyObserver")
                    ),
                }
            except (KeyError, ValueError, TypeError):
                # Skip malformed context records.  I-LLM-054.
                continue

        return accumulators, personality
