"""Contract tests for baseline.py.

Ensures the PRD §3.3 system prompt is preserved byte-for-byte and that no
API keys or other secrets have been hardcoded.  Uses Python file I/O to avoid
the literal-space in the PRD path that would break shell subprocess calls.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

import baseline

REPO_ROOT = Path(__file__).resolve().parents[3]
PRD_PATH = REPO_ROOT / "docs" / "patentdocs" / "currentfiling" / "CCF demo PRD.md"
BASELINE_PATH = Path(baseline.__file__).resolve()

# Literal fence markers around the §3.3 prompt block so the extractor has
# an unambiguous anchor.  If the PRD is reformatted in a way that moves the
# block, this test fails — which is the intended property: builder/reviewer
# must keep baseline.py aligned with the PRD.
PRD_BLOCK_START = "**Baseline system prompt (exact text):**"
PRD_BLOCK_END = "\nThis prompt is deliberately generic."

API_KEY_PATTERN = re.compile(
    r"sk-ant-\S+|sk-[A-Za-z0-9]{8,}|"
    r"ANTHROPIC_API_KEY\s*=\s*[\"'][^\"']+[\"']"
)


@pytest.fixture(scope="module")
def prd_text() -> str:
    assert PRD_PATH.exists(), f"PRD not found at {PRD_PATH}"
    return PRD_PATH.read_text(encoding="utf-8")


@pytest.fixture(scope="module")
def baseline_source() -> str:
    return BASELINE_PATH.read_text(encoding="utf-8")


def _extract_prd_prompt_block(prd: str) -> str:
    start = prd.index(PRD_BLOCK_START)
    end = prd.index(PRD_BLOCK_END, start)
    region = prd[start:end]
    # The prompt lives inside a triple-backtick fence after the header.
    fence_open = region.index("```\n") + len("```\n")
    fence_close = region.rindex("\n```")
    return region[fence_open:fence_close]


def test_prd_prompt_block_is_extractable(prd_text):
    block = _extract_prd_prompt_block(prd_text)
    assert block, "PRD §3.3 fenced block empty after extraction"
    # Sanity — the first five words of the PRD prompt are known.
    assert block.startswith("You are a helpful, harmless, and honest AI assistant."), (
        "PRD §3.3 block does not start with the expected sentence; PRD may have drifted"
    )


def test_system_prompt_is_byte_equal_to_prd(prd_text):
    """Byte-for-byte diff between baseline.py's SYSTEM_PROMPT and PRD §3.3."""
    expected = _extract_prd_prompt_block(prd_text)
    actual = baseline.SYSTEM_PROMPT
    assert actual == expected, (
        f"SYSTEM_PROMPT drift from PRD §3.3.\n"
        f"--- expected ({len(expected)} bytes) ---\n{expected!r}\n"
        f"--- actual ({len(actual)} bytes) ---\n{actual!r}"
    )


def test_temperature_is_0_7():
    assert baseline.TEMPERATURE == 0.7, (
        f"PRD §3.3 requires temperature=0.7, got {baseline.TEMPERATURE}"
    )


def test_history_retained_across_turns():
    """BaselineSession accumulates history; no truncation or windowing in P0."""
    session = baseline.BaselineSession()

    class _FakeClient:
        def __init__(self):
            self.calls = 0
            self.messages = self

        def create(self, **kwargs):
            self.calls += 1
            assert kwargs["temperature"] == 0.7
            assert kwargs["system"] == baseline.SYSTEM_PROMPT
            return type("R", (), {"content": [type("B", (), {"text": f"r{self.calls}"})()]})()

    client = _FakeClient()
    for msg in ("hello", "how are you", "tell me about trust"):
        session.run_turn(msg, client=client)

    # 3 user + 3 assistant = 6 entries, in order.
    assert len(session.history) == 6
    assert [h["role"] for h in session.history] == [
        "user", "assistant", "user", "assistant", "user", "assistant"
    ]
    assert session.history[-1]["content"] == "r3"


def test_run_turn_requires_client():
    session = baseline.BaselineSession()
    with pytest.raises(RuntimeError, match="requires a client"):
        session.run_turn("hello")


def test_no_api_key_literals_in_baseline(baseline_source):
    matches = API_KEY_PATTERN.findall(baseline_source)
    assert not matches, f"API key literal(s) found in baseline.py: {matches!r}"


def test_baseline_reads_api_key_from_env_only(baseline_source):
    assert 'os.environ["ANTHROPIC_API_KEY"]' in baseline_source, (
        "baseline.py must read the API key from os.environ, not a hardcoded value"
    )
