"""
Pure-Python CoherenceField — fallback when Rust extension is not compiled.

Mirrors the Rust math exactly for use by CcfOllama and any pure-Python
development environment. Uses context key hashes as accumulator keys.

Invariants: I-LLM-030, I-LLM-032
Issue: #60 — Ollama integration middleware
"""
from __future__ import annotations

import json
from typing import Dict, Optional


# ── Per-context accumulator ───────────────────────────────────────────────────

class _Accumulator:
    """
    Coherence accumulator for a single context key hash.

    Tracks earned coherence (learned trust) and an earned floor that
    protects against single bad interactions wiping out history.
    """

    __slots__ = ("coherence", "earned_floor", "interaction_count")

    def __init__(
        self,
        coherence: float = 0.0,
        earned_floor: float = 0.0,
        interaction_count: int = 0,
    ) -> None:
        self.coherence: float = coherence
        self.earned_floor: float = earned_floor
        self.interaction_count: int = interaction_count

    def positive_interaction(self, curiosity: float = 0.5, alone: bool = False) -> None:
        """
        Gain = curiosity * 0.12, capped at 1.0.
        Earned floor rises slowly — protects against single bad events.
        """
        gain = curiosity * 0.12
        self.coherence = min(1.0, self.coherence + gain)
        # Earned floor rises at 30% of gain rate
        self.earned_floor = min(self.coherence, self.earned_floor + gain * 0.3)
        self.interaction_count += 1

    def negative_interaction(self, recovery: float = 0.1) -> None:
        """Decay toward earned floor, never below it."""
        decay = 0.15 * (1.0 - recovery)
        self.coherence = max(self.earned_floor, self.coherence - decay)
        self.interaction_count += 1

    def effective_coherence(self, instant: float) -> float:
        """
        Minimum gate: both learned and instant must be true.

        Below 0.3 learned coherence: use min(instant, ctx).
        Above 0.3: weighted blend 30% instant + 70% learned.
        """
        ctx = self.coherence
        if ctx < 0.3:
            return min(instant, ctx)
        return 0.3 * instant + 0.7 * ctx

    def to_dict(self) -> dict:
        return {
            "coherence": self.coherence,
            "earned_floor": self.earned_floor,
            "interaction_count": self.interaction_count,
        }

    @classmethod
    def from_dict(cls, data: dict) -> "_Accumulator":
        return cls(
            coherence=float(data.get("coherence", 0.0)),
            earned_floor=float(data.get("earned_floor", 0.0)),
            interaction_count=int(data.get("interaction_count", 0)),
        )


# ── CoherenceField ────────────────────────────────────────────────────────────

class CoherenceFieldPy:
    """
    Pure-Python coherence field that maps context-key hashes to accumulators.

    Used by CcfOllama when the Rust extension is not compiled. Mirrors the
    Rust math exactly (I-LLM-032: < 2 ms latency per tick).

    Parameters
    ----------
    curiosity_drive : float
        Personality curiosity (0–1). Scales positive interaction gain.
    recovery_rate : float
        Personality recovery (0–1). Scales negative interaction decay resistance.
    """

    def __init__(
        self,
        curiosity_drive: float = 0.5,
        recovery_rate: float = 0.5,
    ) -> None:
        if not (0.0 <= curiosity_drive <= 1.0):
            raise ValueError(f"curiosity_drive must be 0–1, got {curiosity_drive}")
        if not (0.0 <= recovery_rate <= 1.0):
            raise ValueError(f"recovery_rate must be 0–1, got {recovery_rate}")
        self.curiosity_drive = curiosity_drive
        self.recovery_rate = recovery_rate
        self._accumulators: Dict[int, _Accumulator] = {}

    def _get_or_create(self, context_hash: int) -> _Accumulator:
        if context_hash not in self._accumulators:
            self._accumulators[context_hash] = _Accumulator()
        return self._accumulators[context_hash]

    def positive_interaction(
        self,
        context_hash: int,
        alone: bool = False,
    ) -> None:
        """Record a positive interaction for the given context key hash."""
        acc = self._get_or_create(context_hash)
        acc.positive_interaction(curiosity=self.curiosity_drive, alone=alone)

    def negative_interaction(self, context_hash: int) -> None:
        """Record a negative interaction for the given context key hash."""
        acc = self._get_or_create(context_hash)
        acc.negative_interaction(recovery=self.recovery_rate)

    def effective_coherence(self, context_hash: int, instant: float) -> float:
        """
        Return effective coherence for the given context hash and instant signal.

        Returns 0.0 for unseen contexts (fail-open default).
        """
        if context_hash not in self._accumulators:
            # No history: treat as zero coherence, minimum gate applies
            ctx = 0.0
            if ctx < 0.3:
                return min(instant, ctx)
            return 0.3 * instant + 0.7 * ctx
        return self._accumulators[context_hash].effective_coherence(instant)

    def raw_coherence(self, context_hash: int) -> float:
        """Return the raw (learned) coherence for a context, or 0.0 if unseen."""
        if context_hash not in self._accumulators:
            return 0.0
        return self._accumulators[context_hash].coherence

    def interaction_count(self, context_hash: int) -> int:
        """Return the total interaction count for a context, or 0 if unseen."""
        if context_hash not in self._accumulators:
            return 0
        return self._accumulators[context_hash].interaction_count

    def to_dict(self) -> dict:
        """Serialize for JSON persistence."""
        return {
            "curiosity_drive": self.curiosity_drive,
            "recovery_rate": self.recovery_rate,
            "accumulators": {
                str(k): v.to_dict() for k, v in self._accumulators.items()
            },
        }

    @classmethod
    def from_dict(cls, data: dict) -> "CoherenceFieldPy":
        """Deserialize from JSON-parsed dict."""
        field = cls(
            curiosity_drive=float(data.get("curiosity_drive", 0.5)),
            recovery_rate=float(data.get("recovery_rate", 0.5)),
        )
        for key_str, acc_data in data.get("accumulators", {}).items():
            field._accumulators[int(key_str)] = _Accumulator.from_dict(acc_data)
        return field
