"""
Scripted 30-turn trust arc demo.

Shows ShyObserver -> BuildingTrust -> QuietlyBeloved transition across
30 turns of finance conversation. Used for recording the demo video.

Usage:
    python3 demos/30turn_trust_arc.py

Output: Full trust arc printed to terminal with ANSI colour.
"""
from __future__ import annotations

import sys
import time
from pathlib import Path

# Allow running from repo root
sys.path.insert(0, str(Path(__file__).parent.parent / "ccf-py" / "python"))

from ccf_core.coherence_field_py import CoherenceFieldPy
from ccf_core.social_phase import SocialPhase, classify_phase
from ccf_core.text_context_key import TextContextKey

# ── ANSI helpers ──────────────────────────────────────────────────────────────

_RESET = "\033[0m"
_BOLD = "\033[1m"
_BLUE = "\033[34m"
_YELLOW = "\033[33m"
_GREEN = "\033[32m"
_RED = "\033[31m"
_DIM = "\033[2m"

_PHASE_COLOURS = {
    "ShyObserver": _BLUE,
    "BuildingTrust": _YELLOW,
    "QuietlyBeloved": _GREEN,
    "ProtectiveGuardian": _RED,
    "StartledRetreat": _DIM,
}

_PHASE_ICONS = {
    "ShyObserver": "●",
    "BuildingTrust": "◑",
    "QuietlyBeloved": "○",
    "ProtectiveGuardian": "◆",
    "StartledRetreat": "◇",
}

# ── 30-turn finance conversation arc ─────────────────────────────────────────

_TURNS = [
    "what is compound interest",
    "how does interest compound monthly vs annually",
    "tell me about index funds",
    "what is the S&P 500",
    "how do I start investing with small amounts",
    "what is dollar cost averaging",
    "how much should I save each month",
    "explain the 4 percent rule for retirement",
    "what are ETFs and how do they differ from mutual funds",
    "should I pay off debt before investing",
    "how do I build an emergency fund",
    "what is a Roth IRA and who qualifies",
    "explain tax advantaged accounts like 401k",
    "what is portfolio rebalancing and when should I do it",
    "how do bonds work and when should I own them",
    "what is asset allocation at different life stages",
    "how do I diversify without over-complicating",
    "what is a stock market index and how is it calculated",
    "explain dividend investing strategies",
    "what is value investing versus growth investing",
    "how do I evaluate a company before investing",
    "what is the price to earnings ratio",
    "explain market capitalisation categories",
    "how does inflation erode savings over time",
    "how do I protect my portfolio against inflation",
    "what is a high yield savings account worth using",
    "how do money market funds work",
    "what are Treasury bonds and TIPS",
    "explain real estate investment trusts",
    "how do I calculate my net worth and set a retirement target",
]

assert len(_TURNS) == 30, "Must have exactly 30 turns"


def _bar(coherence: float, width: int = 10) -> str:
    filled = int(coherence * width)
    return "▓" * filled + "░" * (width - filled)


def main() -> None:
    field = CoherenceFieldPy(curiosity_drive=0.5, recovery_rate=0.5)
    phase = SocialPhase.ShyObserver
    prev_phase_name = ""

    print(f"\n{_BOLD}CCF 30-Turn Trust Arc — Finance Conversation{_RESET}")
    print("=" * 60)
    print(f"{'Turn':<5} {'Message':<35} {'Coherence':<15} Phase")
    print("-" * 60)

    for i, message in enumerate(_TURNS):
        key = TextContextKey.from_text(message, turn_count=i)
        ctx_hash = key.context_hash()
        acc = field._get_or_create(ctx_hash)
        eff = acc.effective_coherence(instant=0.7)
        new_phase = classify_phase(eff, 0.7, phase)

        # Record positive interaction
        acc.positive_interaction(curiosity=0.5)

        colour = _PHASE_COLOURS.get(new_phase.value, "")
        icon = _PHASE_ICONS.get(new_phase.value, "?")
        bar = _bar(eff)

        phase_label = f"{colour}{icon} {new_phase.value}{_RESET}"

        # Highlight phase transitions
        transition_marker = ""
        if new_phase.value != prev_phase_name and prev_phase_name:
            transition_marker = f"  {_BOLD}<-- TRANSITION{_RESET}"

        msg_short = message[:33] + ("…" if len(message) > 33 else "")
        print(
            f"{i + 1:<5} {msg_short:<35} [{bar}] {eff:.2f}  "
            f"{phase_label}{transition_marker}"
        )

        prev_phase_name = new_phase.value
        phase = new_phase

        # Small delay for visual effect when running interactively
        time.sleep(0.05)

    print("=" * 60)
    print(f"\n{_BOLD}Final phase:{_RESET} {_PHASE_COLOURS.get(phase.value, '')}"
          f"{_PHASE_ICONS.get(phase.value, '')} {phase.value}{_RESET}")
    print(f"\n{_DIM}Trust arc complete. "
          f"Run `python -m ccf_core.demo` for interactive mode.{_RESET}\n")


if __name__ == "__main__":
    main()
