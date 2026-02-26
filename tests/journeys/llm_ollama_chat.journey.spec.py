"""
Journey Test: J-LLM-OLLAMA-CHAT
================================

Tests that the CcfOllama 5-line integration pattern works end-to-end,
that system prompts are injected, CCF metadata is returned, phase transitions
occur across turns, and save/load round-trips preserve state.

Specflow contract : J-LLM-OLLAMA-CHAT
Invariants        : I-LLM-030, I-LLM-031, I-LLM-032, I-LLM-033, I-LLM-034
DOD               :
    - 5-line integration pattern works (mocked)
    - System prompt injection is present in request
    - CCF metadata in response
    - Phase transitions across multiple turns (mocked)
    - save/load round-trip preserves state

Issue: #60 — Ollama integration middleware

Run with:
    python -m pytest tests/journeys/llm_ollama_chat.journey.spec.py -v
"""
from __future__ import annotations

import json
import os
import sys
from typing import Any, Dict
from unittest.mock import MagicMock, patch

import pytest

sys.path.insert(0, "ccf-py/python")

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


# ── Step 1: 5-line integration pattern works ──────────────────────────────────

class TestStep1FiveLinePattern:
    """
    Scenario: Developer integrates CcfOllama in 5 lines (I-LLM-030)
      Given an Ollama model is available (mocked)
      When the developer uses the 5-line integration pattern
      Then the response contains expected content
      And CCF metadata is present
      And no exception is raised

    The 5 lines:
      from ccf_core.ollama_middleware import CcfOllama
      bot = CcfOllama(model="llama3")
      response = bot.chat("Tell me about compound interest")
      bot.record_outcome(positive=True)
      bot.save()
    """

    def test_five_line_pattern_no_exception(self, tmp_path):
        """I-LLM-030: 5-line integration must work without errors."""
        state_path = str(tmp_path / "ccf_state.json")

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            # Line 1 (import is at module level)
            # Line 2
            bot = CcfOllama(model="llama3", state_file=state_path, auto_save=False)
            # Line 3
            response = bot.chat("Tell me about compound interest")
            # Line 4
            bot.record_outcome(positive=True)
            # Line 5
            bot.save()

        assert response is not None
        assert "message" in response
        assert "ccf" in response

    def test_five_line_pattern_response_has_content(self, tmp_path):
        state_path = str(tmp_path / "ccf_state.json")

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            bot = CcfOllama(model="llama3", state_file=state_path, auto_save=False)
            response = bot.chat("What is compound interest?")
            bot.record_outcome(positive=True)
            bot.save()

        assert response["message"]["content"] == MOCK_RESPONSE["message"]["content"]

    def test_five_line_pattern_works_with_any_model(self, tmp_path):
        """I-LLM-033: model-agnostic — works with any Ollama model name."""
        for model_name in ["llama3", "mistral", "gemma2", "phi4"]:
            state_path = str(tmp_path / f"ccf_{model_name}.json")
            with patch(
                "ccf_core.ollama_middleware._call_ollama",
                return_value=dict(MOCK_RESPONSE),
            ):
                bot = CcfOllama(model=model_name, state_file=state_path, auto_save=False)
                response = bot.chat("Hello")
                bot.record_outcome(positive=True)
                bot.save()
            assert "ccf" in response


# ── Step 2: System prompt injection is present in request ────────────────────

class TestStep2SystemPromptInjection:
    """
    Scenario: CcfOllama injects a system prompt for every chat call
      Given a fresh CcfOllama instance
      When chat() is called with any message
      Then the Ollama request contains a system-role message
      And the system message content matches the current SocialPhase template
    """

    def test_system_prompt_injected_in_ollama_request(self):
        """System-role message must appear in every Ollama call."""
        captured = []

        def fake_call(base_url, model, messages, stream):
            captured.extend(messages)
            return dict(MOCK_RESPONSE)

        bot = CcfOllama(model="llama3", state_file="/tmp/ccf_test.json")
        with patch("ccf_core.ollama_middleware._call_ollama", side_effect=fake_call):
            bot.chat("Tell me about personal finance")

        roles = [m["role"] for m in captured]
        assert "system" in roles, "Ollama request must include a system-role message"

    def test_system_prompt_is_shy_observer_on_first_call(self):
        """Fresh instance → ShyObserver prompt injected (contains 'measured')."""
        captured = []

        def fake_call(base_url, model, messages, stream):
            captured.extend(messages)
            return dict(MOCK_RESPONSE)

        bot = CcfOllama(model="llama3", state_file="/tmp/ccf_test.json")
        with patch("ccf_core.ollama_middleware._call_ollama", side_effect=fake_call):
            bot.chat("Hello")

        sys_msgs = [m for m in captured if m["role"] == "system"]
        assert len(sys_msgs) >= 1
        assert "measured" in sys_msgs[0]["content"].lower(), (
            f"Expected 'measured' in ShyObserver prompt, got: {sys_msgs[0]['content']}"
        )

    def test_custom_template_used_when_provided(self):
        """Custom templates override default phase prompts."""
        custom = {SocialPhase.ShyObserver: "CUSTOM_PROMPT_FOR_TEST"}
        captured = []

        def fake_call(base_url, model, messages, stream):
            captured.extend(messages)
            return dict(MOCK_RESPONSE)

        bot = CcfOllama(
            model="llama3",
            state_file="/tmp/ccf_test.json",
            custom_templates=custom,
        )
        with patch("ccf_core.ollama_middleware._call_ollama", side_effect=fake_call):
            bot.chat("Hello")

        sys_msgs = [m for m in captured if m["role"] == "system"]
        assert sys_msgs[0]["content"] == "CUSTOM_PROMPT_FOR_TEST"


# ── Step 3: CCF metadata in response ─────────────────────────────────────────

class TestStep3CcfMetadata:
    """
    Scenario: Every chat() response includes CCF metadata
      Given a CcfOllama instance
      When chat() returns a response
      Then response["ccf"] is a dict
      And it contains phase, coherence, effective_coherence, instant_signal,
          context_key, context_hash, turn, ccf_tick_ms
    """

    def setup_method(self):
        self.bot = CcfOllama(model="llama3", state_file="/tmp/ccf_journey_test.json")

    def _chat(self, msg: str = "Hello", instant: float = 0.7) -> Dict[str, Any]:
        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            return self.bot.chat(msg, instant_signal=instant)

    def test_ccf_is_dict(self):
        response = self._chat()
        assert isinstance(response["ccf"], dict)

    def test_ccf_has_required_keys(self):
        response = self._chat()
        required = {
            "phase", "coherence", "effective_coherence",
            "instant_signal", "context_key", "context_hash",
            "turn", "ccf_tick_ms",
        }
        missing = required - set(response["ccf"].keys())
        assert not missing, f"CCF metadata missing keys: {missing}"

    def test_ccf_phase_is_valid_social_phase_string(self):
        response = self._chat()
        valid_phases = {p.value for p in SocialPhase}
        assert response["ccf"]["phase"] in valid_phases

    def test_ccf_coherence_in_unit_range(self):
        response = self._chat()
        coh = response["ccf"]["coherence"]
        assert 0.0 <= coh <= 1.0

    def test_ccf_turn_increments(self):
        resp1 = self._chat("First message")
        resp2 = self._chat("Second message")
        assert resp2["ccf"]["turn"] == resp1["ccf"]["turn"] + 1

    def test_ccf_tick_ms_is_non_negative(self):
        response = self._chat()
        assert response["ccf"]["ccf_tick_ms"] >= 0.0


# ── Step 4: Phase transitions across multiple turns ───────────────────────────

class TestStep4PhaseTransitions:
    """
    Scenario: User builds trust over multiple conversation turns
      Given a fresh CcfOllama instance
      When the user sends messages and records positive outcomes repeatedly
      Then the phase transitions from ShyObserver → BuildingTrust → QuietlyBeloved
      And each phase has the appropriate system prompt injected
    """

    def test_phases_progress_with_positive_outcomes(self):
        """Phase must progress from Shy → Building → Beloved with enough interactions."""
        bot = CcfOllama(
            model="llama3",
            state_file="/tmp/ccf_journey_phase_test.json",
            personality_curiosity=0.7,
            auto_save=False,
        )

        phases_seen = set()

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            for i in range(50):
                response = bot.chat("Finance question", instant_signal=0.7)
                phases_seen.add(response["ccf"]["phase"])
                bot.record_outcome(positive=True)

        # Should have passed through at least ShyObserver and reached higher phase
        assert SocialPhase.ShyObserver.value in phases_seen, "Must start at ShyObserver"
        assert SocialPhase.QuietlyBeloved.value in phases_seen, (
            f"Should reach QuietlyBeloved after 50 positive interactions. "
            f"Phases seen: {phases_seen}"
        )

    def test_startled_retreat_at_very_low_instant(self):
        """Very low instant_signal on fresh context → StartledRetreat."""
        bot = CcfOllama(model="llama3", state_file="/tmp/ccf_startle_test.json")

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            response = bot.chat("Something unexpected", instant_signal=0.05)

        assert response["ccf"]["phase"] == SocialPhase.StartledRetreat.value

    def test_protective_guardian_at_familiar_topic_under_stress(self):
        """ProtectiveGuardian fires at high coherence + low instant_signal."""
        bot = CcfOllama(
            model="llama3",
            state_file="/tmp/ccf_guardian_test.json",
            auto_save=False,
        )

        # Build high coherence manually
        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            bot.chat("Hello", instant_signal=0.7)

        # Force coherence high
        if bot.active_context:
            ctx_hash = bot.active_context.context_hash()
            acc = bot._field._get_or_create(ctx_hash)
            acc.coherence = 0.85
            acc.earned_floor = 0.60
            bot._phase = SocialPhase.QuietlyBeloved

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            response = bot.chat("Hello again", instant_signal=0.10)

        assert response["ccf"]["phase"] == SocialPhase.ProtectiveGuardian.value


# ── Step 5: save/load round-trip ─────────────────────────────────────────────

class TestStep5SaveLoadRoundTrip:
    """
    Scenario: State persists across CcfOllama sessions
      Given a CcfOllama instance with accumulated trust
      When save() is called and a new instance is loaded from the file
      Then social_phase matches the original
      And coherence for the active context is preserved
    """

    def test_save_load_preserves_phase(self, tmp_path):
        state_path = str(tmp_path / "ccf_round_trip.json")

        bot = CcfOllama(
            model="llama3",
            state_file=state_path,
            personality_curiosity=0.9,
            auto_save=False,
        )

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            for _ in range(30):
                bot.chat("Finance topic", instant_signal=0.7)
                bot.record_outcome(positive=True)

        phase_before = bot.social_phase
        bot.save()

        loaded = CcfOllama.load(state_path, model="llama3", auto_save=False)
        assert loaded.social_phase == phase_before, (
            f"Phase not preserved: {phase_before} → {loaded.social_phase}"
        )

    def test_save_load_preserves_coherence(self, tmp_path):
        state_path = str(tmp_path / "ccf_coherence_round_trip.json")

        bot = CcfOllama(
            model="llama3",
            state_file=state_path,
            auto_save=False,
        )

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            for _ in range(15):
                bot.chat("Finance topic")
                bot.record_outcome(positive=True)

        bot.save()
        ctx_hash_before = bot.active_context.context_hash() if bot.active_context else None
        coh_before = bot._field.raw_coherence(ctx_hash_before) if ctx_hash_before else 0.0

        loaded = CcfOllama.load(state_path, model="llama3", auto_save=False)
        if loaded.active_context and ctx_hash_before:
            loaded_hash = loaded.active_context.context_hash()
            coh_after = loaded._field.raw_coherence(loaded_hash)
            assert abs(coh_after - coh_before) < 1e-6, (
                f"Coherence not preserved: {coh_before:.6f} → {coh_after:.6f}"
            )

    def test_save_produces_valid_json(self, tmp_path):
        state_path = str(tmp_path / "ccf_json_test.json")

        bot = CcfOllama(
            model="llama3",
            state_file=state_path,
            auto_save=False,
        )

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            bot.chat("Hello")

        bot.save()

        with open(state_path, "r") as f:
            data = json.load(f)  # Must not raise

        assert data["model"] == "llama3"
        assert "field" in data
        assert "phase" in data

    def test_save_file_can_be_loaded_by_classmethod(self, tmp_path):
        state_path = str(tmp_path / "ccf_classmethod.json")

        bot = CcfOllama(
            model="llama3",
            state_file=state_path,
            auto_save=False,
        )

        with patch(
            "ccf_core.ollama_middleware._call_ollama",
            return_value=dict(MOCK_RESPONSE),
        ):
            bot.chat("Hello")
            bot.record_outcome(positive=True)

        bot.save()

        loaded = CcfOllama.load(state_path)
        assert isinstance(loaded, CcfOllama)
        assert isinstance(loaded.social_phase, SocialPhase)


# ── Full journey smoke test ───────────────────────────────────────────────────

def test_full_j_llm_ollama_chat_journey(tmp_path):
    """
    J-LLM-OLLAMA-CHAT end-to-end smoke test.

    1. 5-line integration (mocked Ollama)
    2. System prompt present in request
    3. CCF metadata in response
    4. Phase transitions across turns (30 positive → QuietlyBeloved)
    5. Save/load round-trip

    All Ollama calls are mocked — no real Ollama required.
    """
    state_path = str(tmp_path / "ccf_smoke.json")
    captured_messages = []

    def fake_call(base_url, model, messages, stream):
        captured_messages.extend(messages)
        return dict(MOCK_RESPONSE)

    # Step 1: 5-line pattern
    with patch("ccf_core.ollama_middleware._call_ollama", side_effect=fake_call):
        bot = CcfOllama(model="llama3", state_file=state_path, auto_save=False)
        response = bot.chat("Tell me about compound interest")
        bot.record_outcome(positive=True)
        bot.save()

    assert "ccf" in response, "Step 1: CCF metadata must be in response"
    assert "message" in response, "Step 1: message must be in response"

    # Step 2: System prompt was injected
    roles = [m["role"] for m in captured_messages]
    assert "system" in roles, "Step 2: system prompt must have been injected"

    # Step 3: CCF metadata completeness
    ccf = response["ccf"]
    assert "phase" in ccf
    assert "coherence" in ccf
    assert 0.0 <= ccf["coherence"] <= 1.0

    # Step 4: Phase transitions — run 30 more positive interactions
    with patch("ccf_core.ollama_middleware._call_ollama", side_effect=fake_call):
        for _ in range(30):
            bot.chat("Finance topic", instant_signal=0.7)
            bot.record_outcome(positive=True)

    assert bot.social_phase == SocialPhase.QuietlyBeloved, (
        f"Step 4: Expected QuietlyBeloved after 31 positive interactions, "
        f"got {bot.social_phase}"
    )

    # Step 5: Save/load round-trip
    phase_before = bot.social_phase
    bot.save()
    loaded = CcfOllama.load(state_path, model="llama3", auto_save=False)
    assert loaded.social_phase == phase_before, (
        f"Step 5: Phase not preserved after load: {phase_before} → {loaded.social_phase}"
    )
