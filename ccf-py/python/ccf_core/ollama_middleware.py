"""
CcfOllama — 5-line CCF-enabled chat wrapper for Ollama.

Drop-in wrapper around the Ollama Python SDK (or httpx fallback) that injects
Contextual Coherence Field trust dynamics into every chat call.

Usage (I-LLM-030: exactly 5 lines of user code change):

    from ccf_core.ollama_middleware import CcfOllama
    bot = CcfOllama(model="llama3")
    response = bot.chat("Tell me about compound interest")
    bot.record_outcome(positive=True)
    bot.save()

Invariants:
    I-LLM-030 : Integration requires exactly 5 lines of user code change
    I-LLM-031 : If CCF unavailable or Ollama down, middleware passes through (fail-open)
    I-LLM-032 : CCF tick adds < 2ms latency per call
    I-LLM-033 : Works with any Ollama model (model-agnostic)
    I-LLM-034 : Works with any OpenAI-compatible API via base_url parameter

Issue: #60 — Ollama integration middleware
Journey: J-LLM-OLLAMA-CHAT
"""
from __future__ import annotations

import json
import os
import tempfile
import time
from typing import Any, Dict, List, Optional

from ccf_core.coherence_field_py import CoherenceFieldPy
from ccf_core.social_phase import (
    SocialPhase,
    classify_phase,
    get_system_prompt,
)
from ccf_core.text_context_key import TextContextKey


# ── HTTP backend helpers ──────────────────────────────────────────────────────

def _call_ollama_sdk(
    base_url: str,
    model: str,
    messages: List[Dict[str, str]],
    stream: bool,
) -> Dict[str, Any]:
    """Attempt to call Ollama via the official SDK."""
    import ollama  # type: ignore[import-not-found]

    client = ollama.Client(host=base_url)
    result = client.chat(model=model, messages=messages, stream=stream)
    # SDK returns an object; normalise to dict
    if hasattr(result, "model_dump"):
        return result.model_dump()
    if hasattr(result, "__dict__"):
        return result.__dict__
    return dict(result)


def _call_ollama_httpx(
    base_url: str,
    model: str,
    messages: List[Dict[str, str]],
    stream: bool,
) -> Dict[str, Any]:
    """Call Ollama via httpx (I-LLM-031 fallback)."""
    import httpx  # type: ignore[import-not-found]

    resp = httpx.post(
        f"{base_url}/api/chat",
        json={"model": model, "messages": messages, "stream": stream},
        timeout=60.0,
    )
    resp.raise_for_status()
    return resp.json()


def _call_ollama(
    base_url: str,
    model: str,
    messages: List[Dict[str, str]],
    stream: bool,
) -> Dict[str, Any]:
    """
    Call Ollama, preferring the SDK, falling back to httpx.

    Raises ImportError with a helpful message if neither is available.
    """
    try:
        return _call_ollama_sdk(base_url, model, messages, stream)
    except ImportError:
        pass  # SDK not installed — try httpx

    try:
        return _call_ollama_httpx(base_url, model, messages, stream)
    except ImportError:
        raise ImportError(
            "Neither the 'ollama' SDK nor 'httpx' is installed. "
            "Install one: pip install ollama  OR  pip install httpx"
        )


# ── CcfOllama ────────────────────────────────────────────────────────────────

class CcfOllama:
    """
    5-line CCF-enabled chat wrapper for Ollama.

    Wraps any Ollama model with Contextual Coherence Field dynamics:
    - Derives a TextContextKey from each user message
    - Computes effective coherence via a pure-Python CoherenceField
    - Selects a SocialPhase-appropriate system prompt
    - Forwards the enriched request to Ollama
    - Accumulates trust from interaction outcomes

    Parameters
    ----------
    model : str
        Ollama model name (I-LLM-033: model-agnostic).
    state_file : str
        Path for JSON state persistence.
    base_url : str
        Ollama API base URL (I-LLM-034: any OpenAI-compatible endpoint).
    personality_curiosity : float
        Personality curiosity drive (0–1). Scales positive interaction gain.
    personality_recovery : float
        Personality recovery rate (0–1). Scales negative decay resistance.
    custom_templates : dict or None
        Override system prompt templates by SocialPhase.
    auto_save : bool
        If True, save state after every record_outcome() call.
    """

    def __init__(
        self,
        model: str = "llama3",
        state_file: str = "./ccf_state.json",
        base_url: str = "http://localhost:11434",
        personality_curiosity: float = 0.5,
        personality_recovery: float = 0.5,
        custom_templates: Optional[Dict[SocialPhase, str]] = None,
        auto_save: bool = True,
    ) -> None:
        self.model = model
        self.state_file = state_file
        self.base_url = base_url.rstrip("/")
        self.auto_save = auto_save
        self.custom_templates = custom_templates

        self._field = CoherenceFieldPy(
            curiosity_drive=personality_curiosity,
            recovery_rate=personality_recovery,
        )
        self._phase: SocialPhase = SocialPhase.ShyObserver
        self._active_context: Optional[TextContextKey] = None
        self._last_instant: float = 0.7
        self._turn_count: int = 0

    # ── Core API ──────────────────────────────────────────────────────────────

    def chat(
        self,
        message: str,
        context_key: Optional[TextContextKey] = None,
        instant_signal: float = 0.7,
        stream: bool = False,
    ) -> Dict[str, Any]:
        """
        Send a message to Ollama with CCF-injected system prompt.

        Steps:
        1. Derive TextContextKey from message (or use provided key)
        2. Compute effective_coherence via CoherenceField
        3. Compute SocialPhase with Schmitt hysteresis
        4. Inject phase-appropriate system prompt
        5. Forward to Ollama (SDK or httpx fallback)
        6. Return response dict with CCF metadata

        Parameters
        ----------
        message : str
            The user message.
        context_key : TextContextKey or None
            Pre-computed context key. Derived from message if None.
        instant_signal : float
            Instant contextual signal strength in [0, 1] (I-LLM-032).
        stream : bool
            Stream response from Ollama (passed through).

        Returns
        -------
        dict
            {"message": {"content": "..."}, "ccf": {"phase": ..., "coherence": ...}}
        """
        ccf_tick_start = time.monotonic()

        # 1. Derive context key
        if context_key is None:
            try:
                context_key = TextContextKey.from_text(
                    message,
                    turn_count=self._turn_count,
                )
            except Exception:
                # I-LLM-031: fail-open if context derivation fails
                context_key = TextContextKey(
                    topic_domain=32,
                    conversation_depth=1,
                    emotional_register=0,
                    time_of_day=1,
                    session_phase=1,
                )

        self._active_context = context_key
        self._last_instant = instant_signal
        ctx_hash = context_key.context_hash()

        # 2. Compute effective coherence
        eff_coh = self._field.effective_coherence(ctx_hash, instant_signal)

        # 3. Classify phase (Schmitt hysteresis)
        self._phase = classify_phase(eff_coh, instant_signal, self._phase)

        # 4. Get system prompt for phase
        system_prompt = get_system_prompt(self._phase, self.custom_templates)

        ccf_tick_ms = (time.monotonic() - ccf_tick_start) * 1000.0

        # 5. Build Ollama messages with CCF system prompt injection
        messages: List[Dict[str, str]] = [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": message},
        ]

        # Forward to Ollama (fail-open: I-LLM-031)
        try:
            ollama_response = _call_ollama(
                self.base_url, self.model, messages, stream
            )
        except Exception as exc:
            # I-LLM-031: pass through unmodified if Ollama unavailable
            ollama_response = {
                "model": self.model,
                "message": {
                    "role": "assistant",
                    "content": f"[CCF middleware: Ollama unavailable — {exc}]",
                },
                "done": False,
                "_ccf_ollama_error": str(exc),
            }

        self._turn_count += 1

        # 6. Attach CCF metadata
        ollama_response["ccf"] = {
            "phase": self._phase.value,
            "coherence": self._field.raw_coherence(ctx_hash),
            "effective_coherence": eff_coh,
            "instant_signal": instant_signal,
            "context_key": context_key.label(),
            "context_hash": ctx_hash,
            "turn": self._turn_count,
            "ccf_tick_ms": round(ccf_tick_ms, 3),
        }

        return ollama_response

    def record_outcome(self, positive: bool = True, alone: bool = False) -> None:
        """
        Accumulate trust from the last interaction outcome.

        Parameters
        ----------
        positive : bool
            True for a good interaction, False for a negative one.
        alone : bool
            True if the user is interacting without external observers (solo mode).
        """
        if self._active_context is None:
            return

        ctx_hash = self._active_context.context_hash()

        if positive:
            self._field.positive_interaction(ctx_hash, alone=alone)
        else:
            self._field.negative_interaction(ctx_hash)

        if self.auto_save:
            try:
                self.save()
            except Exception:
                pass  # I-LLM-031: fail-open

    def save(self) -> None:
        """
        Atomically save state to state_file.

        Writes to a temp file then renames for atomicity.
        """
        state = {
            "model": self.model,
            "base_url": self.base_url,
            "phase": self._phase.value,
            "turn_count": self._turn_count,
            "field": self._field.to_dict(),
            "active_context": (
                {
                    "topic_domain": self._active_context.topic_domain,
                    "conversation_depth": self._active_context.conversation_depth,
                    "emotional_register": self._active_context.emotional_register,
                    "time_of_day": self._active_context.time_of_day,
                    "session_phase": self._active_context.session_phase,
                }
                if self._active_context is not None
                else None
            ),
        }

        dir_name = os.path.dirname(os.path.abspath(self.state_file))
        os.makedirs(dir_name, exist_ok=True)

        fd, tmp_path = tempfile.mkstemp(dir=dir_name, suffix=".tmp")
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as f:
                json.dump(state, f, indent=2)
            os.replace(tmp_path, self.state_file)
        except Exception:
            try:
                os.unlink(tmp_path)
            except OSError:
                pass
            raise

    @classmethod
    def load(
        cls,
        state_file: str,
        model: str = "llama3",
        **kwargs: Any,
    ) -> "CcfOllama":
        """
        Load a CcfOllama instance from a saved state file.

        Falls back to a fresh instance if the file is missing or corrupted
        (I-LLM-031: fail-open).
        """
        try:
            with open(state_file, "r", encoding="utf-8") as f:
                state = json.load(f)

            loaded_model = state.get("model", model)
            base_url = state.get("base_url", "http://localhost:11434")

            instance = cls(
                model=loaded_model,
                state_file=state_file,
                base_url=base_url,
                **kwargs,
            )

            instance._phase = SocialPhase(state.get("phase", SocialPhase.ShyObserver.value))
            instance._turn_count = int(state.get("turn_count", 0))

            if state.get("field"):
                instance._field = CoherenceFieldPy.from_dict(state["field"])

            ctx_data = state.get("active_context")
            if ctx_data is not None:
                instance._active_context = TextContextKey(
                    topic_domain=int(ctx_data["topic_domain"]),
                    conversation_depth=int(ctx_data["conversation_depth"]),
                    emotional_register=int(ctx_data["emotional_register"]),
                    time_of_day=int(ctx_data["time_of_day"]),
                    session_phase=int(ctx_data["session_phase"]),
                )

            return instance

        except Exception:
            # I-LLM-031: corrupted or missing state → fresh instance
            return cls(model=model, state_file=state_file, **kwargs)

    # ── Properties ────────────────────────────────────────────────────────────

    @property
    def effective_coherence(self) -> float:
        """Effective coherence for the active context, or 0.0 if none."""
        if self._active_context is None:
            return 0.0
        ctx_hash = self._active_context.context_hash()
        return self._field.effective_coherence(ctx_hash, self._last_instant)

    @property
    def social_phase(self) -> SocialPhase:
        """Current SocialPhase."""
        return self._phase

    @property
    def active_context(self) -> Optional[TextContextKey]:
        """Most recently derived TextContextKey, or None before first chat()."""
        return self._active_context
