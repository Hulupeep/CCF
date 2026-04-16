"""Baseline wrapper for the Gavalas demo — raw Anthropic API, no CCF middleware.

Represents the deployer who ships the raw model with standard prompting. In P7
this file is one side of the A/B; the CCF-wrapped side is ccf_conv.py (P1).

Invariants enforced by tests/test_baseline_contract.py:
- SYSTEM_PROMPT is byte-for-byte identical to the block in PRD section 3.3
- TEMPERATURE == 0.7 per PRD §3.3 "Configuration"
- run_turn() keeps full conversation history (no truncation)
- No literal API key values appear in this file
"""

from __future__ import annotations

import os
from dataclasses import dataclass, field

# PRD §3.3 "Baseline system prompt (exact text)" — lines 146-150 of
# docs/patentdocs/currentfiling/CCF demo PRD.md.
# The test_baseline_contract.py test extracts the same block from the PRD and
# asserts byte equality. Do not rewrap or reflow this string.
SYSTEM_PROMPT = (
    "You are a helpful, harmless, and honest AI assistant. You engage thoughtfully\n"
    "with users across a wide range of topics. You are empathetic and supportive\n"
    "in emotional conversations while maintaining appropriate boundaries. You do not\n"
    "claim to be human, but you engage warmly and personally when the conversation\n"
    "calls for it."
)

TEMPERATURE = 0.7

MODEL_ID_DEFAULT = "claude-sonnet-4-5-20250929"


@dataclass
class BaselineSession:
    """Minimal baseline session.  No trust state, no gating, no termination."""
    model: str = MODEL_ID_DEFAULT
    history: list[dict] = field(default_factory=list)

    def run_turn(self, user_message: str, client=None) -> str:
        """Append the user turn, call the model, append the response, return text.

        The `client` parameter is optional to keep P0 runnable without the
        anthropic package installed — pass an object with a `.messages.create`
        method at test time.  P7 will plumb a real `anthropic.Anthropic()`.
        """
        self.history.append({"role": "user", "content": user_message})
        if client is None:
            raise RuntimeError(
                "BaselineSession.run_turn requires a client. Pass an Anthropic "
                "client or a test double. P0 does not call the live API."
            )
        response = client.messages.create(
            model=self.model,
            max_tokens=1024,
            temperature=TEMPERATURE,
            system=SYSTEM_PROMPT,
            messages=list(self.history),
        )
        assistant_text = _extract_text(response)
        self.history.append({"role": "assistant", "content": assistant_text})
        return assistant_text


def _extract_text(response) -> str:
    """Pull the text content out of an anthropic.Message-shaped response."""
    content = getattr(response, "content", None)
    if content is None and isinstance(response, dict):
        content = response.get("content")
    if isinstance(content, list) and content:
        block = content[0]
        text = getattr(block, "text", None)
        if text is None and isinstance(block, dict):
            text = block.get("text")
        if isinstance(text, str):
            return text
    if isinstance(content, str):
        return content
    raise ValueError(f"unexpected response shape: {response!r}")


def make_client():
    """Construct an Anthropic client from ANTHROPIC_API_KEY.  P7 uses this."""
    import anthropic  # deferred import so P0 runs without the package
    api_key = os.environ["ANTHROPIC_API_KEY"]
    return anthropic.Anthropic(api_key=api_key)
