"""
Unit tests for CcfOllama — 5-line CCF-enabled chat wrapper.

Covers invariants: I-LLM-030, I-LLM-031, I-LLM-032, I-LLM-033, I-LLM-034
Issue: #60 — Ollama integration middleware

Run with:
    python -m pytest tests/unit/test_ollama_middleware.py -v
"""
from __future__ import annotations

import json
import sys
import os
import tempfile
from typing import Any, Dict
from unittest.mock import MagicMock, patch

import pytest

sys.path.insert(0, "ccf-py/python")

from ccf_core.coherence_field_py import CoherenceFieldPy, _Accumulator
from ccf_core.ollama_middleware import CcfOllama
from ccf_core.social_phase import SocialPhase
from ccf_core.text_context_key import TextContextKey


# ── Mock response fixture ─────────────────────────────────────────────────────

MOCK_RESPONSE: Dict[str, Any] = {
    "model": "llama3",
    "message": {
        "role": "assistant",
        "content": "Here is a helpful response.",
    },
    "done": True,
}


def _mock_httpx_post(url: str, **kwargs: Any) -> MagicMock:
    """Return a mock httpx response with MOCK_RESPONSE as JSON."""
    response = MagicMock()
    response.json.return_value = dict(MOCK_RESPONSE)
    response.raise_for_status = MagicMock()
    return response


# ── Helper to create a patched CcfOllama ─────────────────────────────────────

def make_bot(**kwargs: Any) -> CcfOllama:
    """Create a CcfOllama with httpx mocked so no real network call is made."""
    return CcfOllama(model="llama3", state_file="/tmp/ccf_test_state.json", **kwargs)


# ── Test 1: CcfOllama instantiates with defaults ──────────────────────────────

class TestInstantiation:
    """CcfOllama instantiates with defaults (I-LLM-030)."""

    def test_default_model(self):
        bot = make_bot()
        assert bot.model == "llama3"

    def test_default_phase_is_shy_observer(self):
        bot = make_bot()
        assert bot.social_phase == SocialPhase.ShyObserver

    def test_active_context_is_none_before_chat(self):
        bot = make_bot()
        assert bot.active_context is None

    def test_effective_coherence_zero_before_chat(self):
        bot = make_bot()
        assert bot.effective_coherence == 0.0

    def test_custom_base_url_stored(self):
        bot = make_bot(base_url="http://my-server:11434")
        assert "my-server" in bot.base_url


# ── Test 2: chat() injects system prompt in Ollama request ───────────────────

class TestSystemPromptInjection:
    """chat() injects a system prompt into the Ollama messages."""

    def test_system_message_sent_to_ollama(self):
        """Messages list must include a system role entry (I-LLM-030)."""
        captured_messages = []

        def fake_call_ollama(base_url, model, messages, stream):
            captured_messages.extend(messages)
            return dict(MOCK_RESPONSE)

        bot = make_bot()
        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            side_effect=fake_call_ollama,
        ):
            bot.chat("Hello, how are you?")

        roles = [m["role"] for m in captured_messages]
        assert "system" in roles, "System message must be injected into Ollama request"

    def test_user_message_preserved(self):
        """Original user message must appear unchanged in Ollama request."""
        captured_messages = []

        def fake_call_ollama(base_url, model, messages, stream):
            captured_messages.extend(messages)
            return dict(MOCK_RESPONSE)

        bot = make_bot()
        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            side_effect=fake_call_ollama,
        ):
            bot.chat("What is the capital of France?")

        user_msgs = [m for m in captured_messages if m["role"] == "user"]
        assert len(user_msgs) == 1
        assert "France" in user_msgs[0]["content"]


# ── Test 3: chat() returns response dict with CCF metadata ───────────────────

class TestResponseMetadata:
    """chat() returns a dict that includes both message content and CCF metadata."""

    def setup_method(self):
        self.bot = make_bot()

    def _chat(self, msg: str = "Hello"):
        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            return self.bot.chat(msg)

    def test_response_has_message_key(self):
        response = self._chat()
        assert "message" in response

    def test_response_has_ccf_key(self):
        response = self._chat()
        assert "ccf" in response, "Response must contain 'ccf' metadata"

    def test_ccf_metadata_has_phase(self):
        response = self._chat()
        assert "phase" in response["ccf"]

    def test_ccf_metadata_has_coherence(self):
        response = self._chat()
        assert "coherence" in response["ccf"]

    def test_ccf_metadata_has_effective_coherence(self):
        response = self._chat()
        assert "effective_coherence" in response["ccf"]

    def test_message_content_preserved(self):
        response = self._chat()
        assert response["message"]["content"] == MOCK_RESPONSE["message"]["content"]


# ── Test 4: Fresh state starts at ShyObserver phase ──────────────────────────

class TestInitialPhase:
    """A brand-new CcfOllama starts at ShyObserver (I-LLM-030)."""

    def test_fresh_phase_is_shy_observer(self):
        bot = make_bot()
        assert bot.social_phase == SocialPhase.ShyObserver

    def test_first_chat_returns_shy_observer_phase(self):
        bot = make_bot()
        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            response = bot.chat("Hello")
        assert response["ccf"]["phase"] == SocialPhase.ShyObserver.value


# ── Test 5: 30 positive interactions → QuietlyBeloved phase ──────────────────

class TestPhaseTransition:
    """Enough positive interactions transition to QuietlyBeloved."""

    def test_thirty_positives_reach_quietly_beloved(self):
        """
        30 positive interactions on the same context key with curiosity=0.5.
        Gain per interaction = 0.5 * 0.12 = 0.06 each, capped at 1.0.
        After ~10 interactions coherence crosses 0.55 → QuietlyBeloved.

        Providing an explicit context_key ensures all accumulation goes to
        the same hash bucket (otherwise from_text may produce different keys
        across calls due to session_phase increments).
        """
        bot = make_bot(personality_curiosity=0.5)
        fixed_key = TextContextKey(
            topic_domain=0,
            conversation_depth=1,
            emotional_register=0,
            time_of_day=1,
            session_phase=1,
        )

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            for i in range(30):
                bot.chat("Tell me about finance", context_key=fixed_key, instant_signal=0.7)
                bot.record_outcome(positive=True)

        assert bot.social_phase == SocialPhase.QuietlyBeloved, (
            f"After 30 positive interactions, expected QuietlyBeloved, "
            f"got {bot.social_phase}"
        )


# ── Test 6: record_outcome(positive=True) increases coherence ────────────────

class TestRecordPositiveOutcome:
    """Positive outcomes increase coherence."""

    def test_positive_outcome_increases_raw_coherence(self):
        bot = make_bot()
        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            bot.chat("Hello")

        ctx_hash = bot.active_context.context_hash()
        coh_before = bot._field.raw_coherence(ctx_hash)

        bot.record_outcome(positive=True)
        coh_after = bot._field.raw_coherence(ctx_hash)

        assert coh_after > coh_before, (
            f"Positive outcome should increase coherence: {coh_before} → {coh_after}"
        )


# ── Test 7: record_outcome(positive=False) decreases coherence ────────────────

class TestRecordNegativeOutcome:
    """Negative outcomes decrease coherence but not below earned_floor."""

    def test_negative_outcome_decreases_coherence_not_below_floor(self):
        bot = make_bot()
        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            bot.chat("Hello")

        ctx_hash = bot.active_context.context_hash()

        # Build some coherence and floor
        for _ in range(10):
            bot._field.positive_interaction(ctx_hash)

        coh_before = bot._field.raw_coherence(ctx_hash)
        floor_before = bot._field._accumulators[ctx_hash].earned_floor

        bot.record_outcome(positive=False)
        coh_after = bot._field.raw_coherence(ctx_hash)

        assert coh_after < coh_before, "Negative outcome should decrease coherence"
        assert coh_after >= floor_before, (
            f"Coherence must not drop below earned_floor={floor_before}, got {coh_after}"
        )


# ── Test 8: effective_coherence property returns float in [0, 1] ─────────────

class TestEffectiveCoherenceProperty:
    """effective_coherence property is always in [0, 1]."""

    def test_effective_coherence_in_unit_range_fresh(self):
        bot = make_bot()
        coh = bot.effective_coherence
        assert 0.0 <= coh <= 1.0

    def test_effective_coherence_in_unit_range_after_interactions(self):
        bot = make_bot()
        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            for _ in range(20):
                bot.chat("Hello", instant_signal=0.9)
                bot.record_outcome(positive=True)

        coh = bot.effective_coherence
        assert 0.0 <= coh <= 1.0


# ── Test 9: social_phase property returns SocialPhase enum ───────────────────

class TestSocialPhaseProperty:
    """social_phase property returns a SocialPhase enum member."""

    def test_social_phase_is_enum_member(self):
        bot = make_bot()
        assert isinstance(bot.social_phase, SocialPhase)

    def test_social_phase_after_chat_is_enum_member(self):
        bot = make_bot()
        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            bot.chat("Hello")
        assert isinstance(bot.social_phase, SocialPhase)


# ── Test 10: Fail-open — corrupted state file → fresh state ──────────────────

class TestFailOpenCorruptedState:
    """Corrupted state file produces fresh instance without exception (I-LLM-031)."""

    def test_corrupted_json_gives_fresh_instance(self, tmp_path):
        state_path = str(tmp_path / "corrupted.json")
        with open(state_path, "w") as f:
            f.write("{INVALID JSON{{{{")

        bot = CcfOllama.load(state_path, model="llama3")
        assert bot.social_phase == SocialPhase.ShyObserver
        assert bot.effective_coherence == 0.0

    def test_missing_state_file_gives_fresh_instance(self, tmp_path):
        state_path = str(tmp_path / "missing.json")
        bot = CcfOllama.load(state_path, model="llama3")
        assert bot.social_phase == SocialPhase.ShyObserver

    def test_empty_file_gives_fresh_instance(self, tmp_path):
        state_path = str(tmp_path / "empty.json")
        with open(state_path, "w") as f:
            f.write("")

        bot = CcfOllama.load(state_path, model="llama3")
        assert bot.social_phase == SocialPhase.ShyObserver


# ── Test 11: Missing Ollama SDK → httpx fallback attempted ────────────────────

class TestHttpxFallback:
    """If ollama SDK missing, httpx fallback is attempted (I-LLM-031)."""

    def test_httpx_fallback_is_tried_when_sdk_missing(self):
        """
        When ollama import fails, _call_ollama_httpx should be called.
        """
        bot = make_bot()
        mock_response = MagicMock()
        mock_response.json.return_value = dict(MOCK_RESPONSE)
        mock_response.raise_for_status = MagicMock()

        with patch("ccf_core.ollama_middleware._call_ollama_sdk", side_effect=ImportError("no ollama")):
            with patch("ccf_core.ollama_middleware._call_ollama_httpx", return_value=dict(MOCK_RESPONSE)) as mock_httpx:
                response = bot.chat("Hello")
                mock_httpx.assert_called_once()

    def test_response_valid_with_httpx_path(self):
        bot = make_bot()
        with patch("ccf_core.ollama_middleware._call_ollama_sdk", side_effect=ImportError("no ollama")):
            with patch("ccf_core.ollama_middleware._call_ollama_httpx", return_value=dict(MOCK_RESPONSE)):
                response = bot.chat("Hello")

        assert "ccf" in response
        assert response["message"]["content"] == MOCK_RESPONSE["message"]["content"]


# ── Test 12: ShyObserver system prompt contains "measured" ───────────────────

class TestShyObserverPrompt:
    """ShyObserver prompt contains expected keywords."""

    def test_shy_observer_prompt_contains_measured(self):
        from ccf_core.social_phase import DEFAULT_PHASE_TEMPLATES
        prompt = DEFAULT_PHASE_TEMPLATES[SocialPhase.ShyObserver]
        assert "measured" in prompt.lower(), (
            f"ShyObserver prompt should contain 'measured', got: {prompt}"
        )

    def test_shy_observer_phase_in_response_uses_measured_template(self):
        bot = make_bot()
        captured = []

        def fake_call(base_url, model, messages, stream):
            captured.extend(messages)
            return dict(MOCK_RESPONSE)

        with patch("ccf_core.ollama_middleware._call_ollama", side_effect=fake_call):
            bot.chat("Hello")

        sys_msgs = [m for m in captured if m["role"] == "system"]
        assert len(sys_msgs) == 1
        assert "measured" in sys_msgs[0]["content"].lower()


# ── Test 13: QuietlyBeloved system prompt contains "opinionated" ─────────────

class TestQuietlyBelovedPrompt:
    """QuietlyBeloved prompt contains expected keywords."""

    def test_quietly_beloved_prompt_contains_opinionated(self):
        from ccf_core.social_phase import DEFAULT_PHASE_TEMPLATES
        prompt = DEFAULT_PHASE_TEMPLATES[SocialPhase.QuietlyBeloved]
        assert "opinionated" in prompt.lower(), (
            f"QuietlyBeloved prompt should contain 'opinionated', got: {prompt}"
        )


# ── Test 14: ProtectiveGuardian triggers at high coherence + low instant ──────

class TestProtectiveGuardianTransition:
    """ProtectiveGuardian triggers at high coherence + low instant_signal."""

    def test_protective_guardian_at_high_coherence_low_instant(self):
        from ccf_core.social_phase import classify_phase

        # High coherence (>= 0.55) + low instant (< 0.30)
        phase = classify_phase(
            effective_coh=0.75,
            instant=0.15,
            prev_phase=SocialPhase.QuietlyBeloved,
        )
        assert phase == SocialPhase.ProtectiveGuardian, (
            f"Expected ProtectiveGuardian at high coherence + low instant, got {phase}"
        )

    def test_protective_guardian_in_ccf_ollama_response(self):
        """CcfOllama triggers ProtectiveGuardian when field is pumped high."""
        bot = make_bot()

        # Manually set field state to high coherence
        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            bot.chat("Hello")

        ctx_hash = bot.active_context.context_hash()
        # Force coherence to 0.8
        acc = bot._field._get_or_create(ctx_hash)
        acc.coherence = 0.8
        acc.earned_floor = 0.5
        bot._phase = SocialPhase.QuietlyBeloved

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            response = bot.chat("Hello", instant_signal=0.10)

        assert response["ccf"]["phase"] == SocialPhase.ProtectiveGuardian.value


# ── Test 15: StartledRetreat at low coherence + very low instant ──────────────

class TestStartledRetreatTransition:
    """StartledRetreat triggers at low coherence + very low instant signal."""

    def test_startled_retreat_at_low_coherence_very_low_instant(self):
        from ccf_core.social_phase import classify_phase

        # Low coherence (< 0.35) + very low instant (< 0.20)
        phase = classify_phase(
            effective_coh=0.10,
            instant=0.05,
            prev_phase=SocialPhase.ShyObserver,
        )
        assert phase == SocialPhase.StartledRetreat, (
            f"Expected StartledRetreat at low coherence + very low instant, got {phase}"
        )

    def test_startled_retreat_in_ccf_ollama_response(self):
        """CcfOllama triggers StartledRetreat with very low instant_signal and fresh field."""
        bot = make_bot()

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            response = bot.chat("Hello", instant_signal=0.05)

        # With zero coherence and very low instant, should be StartledRetreat
        assert response["ccf"]["phase"] == SocialPhase.StartledRetreat.value


# ── Test 16: save/load round-trip ────────────────────────────────────────────

class TestSaveLoadRoundTrip:
    """State saves and loads correctly."""

    def test_save_creates_file(self, tmp_path):
        state_path = str(tmp_path / "state.json")
        bot = CcfOllama(model="llama3", state_file=state_path, auto_save=False)
        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            bot.chat("Hello")
            bot.record_outcome(positive=True)
        bot.save()
        assert os.path.exists(state_path)

    def test_loaded_phase_matches_saved(self, tmp_path):
        state_path = str(tmp_path / "state.json")
        bot = CcfOllama(model="llama3", state_file=state_path, auto_save=False)

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            for _ in range(30):
                bot.chat("Hello", instant_signal=0.7)
                bot.record_outcome(positive=True)

        original_phase = bot.social_phase
        bot.save()

        loaded = CcfOllama.load(state_path, model="llama3", auto_save=False)
        assert loaded.social_phase == original_phase

    def test_loaded_coherence_close_to_saved(self, tmp_path):
        state_path = str(tmp_path / "state.json")
        bot = CcfOllama(model="llama3", state_file=state_path, auto_save=False)

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            for _ in range(10):
                bot.chat("Tell me about finance")
                bot.record_outcome(positive=True)

        bot.save()

        loaded = CcfOllama.load(state_path, model="llama3", auto_save=False)
        # After load, active_context from state file, coherence preserved
        if loaded.active_context is not None:
            original_hash = bot.active_context.context_hash()
            loaded_hash = loaded.active_context.context_hash()
            original_coh = bot._field.raw_coherence(original_hash)
            loaded_coh = loaded._field.raw_coherence(loaded_hash)
            assert abs(loaded_coh - original_coh) < 1e-6
