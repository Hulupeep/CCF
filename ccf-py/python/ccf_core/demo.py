"""
ccf-core CLI demo — terminal chat with live trust dashboard.

Usage:
    python -m ccf_core.demo [--model llama3] [--state-file ./ccf_state.json]
                             [--no-dashboard] [--headless-turns N]

Invariants: I-LLM-060 through I-LLM-065
Journey: J-LLM-EARNED-TRUST
"""
from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path

# Allow running from repo root: python -m ccf_core.demo
sys.path.insert(0, str(Path(__file__).parent.parent))

from ccf_core.coherence_field_py import CoherenceFieldPy
from ccf_core.social_phase import SocialPhase, classify_phase, get_prompt_injection
from ccf_core.text_context_key import TextContextKey
from ccf_core.persistence import CcfPersistence

# ── ANSI colour codes ─────────────────────────────────────────────────────────

_RESET = "\033[0m"
_BOLD = "\033[1m"
_BLUE = "\033[34m"
_YELLOW = "\033[33m"
_GREEN = "\033[32m"
_RED = "\033[31m"
_DIM = "\033[2m"
_CYAN = "\033[36m"

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

# ── Scripted headless messages ────────────────────────────────────────────────

_HEADLESS_MESSAGES = [
    "what is compound interest",
    "tell me about index funds",
    "how do I start investing",
    "what is dollar cost averaging",
    "how much should I save each month",
    "explain the 4 percent rule",
    "what are ETFs",
    "should I pay off debt or invest",
    "how do I build an emergency fund",
    "what is a Roth IRA",
    "explain tax advantaged accounts",
    "what is portfolio rebalancing",
    "how do bonds work",
    "what is asset allocation",
    "how do I diversify investments",
    "what is a stock market index",
    "explain dividend investing",
    "what is value investing",
    "how do I evaluate a company",
    "what is a price to earnings ratio",
    "explain market capitalisation",
    "what is inflation and how does it affect savings",
    "how do I protect against inflation",
    "what is a high yield savings account",
    "how do money market funds work",
    "what are Treasury bonds",
    "explain real estate investment trusts",
    "how do I calculate net worth",
    "what is the difference between assets and liabilities",
    "how do I retire early",
]


# ── Headless runner ───────────────────────────────────────────────────────────

def run_headless(args: argparse.Namespace) -> None:
    """
    Run N scripted turns and print data-testid values to stdout.
    Used by journey tests to assert on dashboard state.
    """
    field = CoherenceFieldPy(curiosity_drive=0.5, recovery_rate=0.5)
    phase = SocialPhase.ShyObserver
    n = args.headless_turns

    for i in range(n):
        msg = _HEADLESS_MESSAGES[i % len(_HEADLESS_MESSAGES)]
        key = TextContextKey.from_text(msg, turn_count=i)
        ctx_hash = key.context_hash()
        acc = field._get_or_create(ctx_hash)
        eff = acc.effective_coherence(instant=0.7)
        phase = classify_phase(eff, 0.7, phase)
        acc.positive_interaction(curiosity=0.5)

        # Machine-readable output for test assertion
        print(f"[data-testid: phase-label] {phase.value}")
        print(f"[data-testid: coherence-pct] {eff:.4f}")
        print(f"[data-testid: ctx-hash-display] {ctx_hash:08x}")
        print(f"[data-testid: interaction-count] {i + 1}")


# ── CcfDemo class ─────────────────────────────────────────────────────────────

class CcfDemo:
    """Interactive CLI demo with live trust dashboard.

    Invariants: I-LLM-060 (dashboard after every turn),
                I-LLM-061 (visual phase distinction),
                I-LLM-062 (no GPU — Ollama CPU only),
                I-LLM-063 (single command launch),
                I-LLM-064 (80-column terminal),
                I-LLM-065 (--no-dashboard headless mode).
    """

    _COL_WIDE = 80
    _LEFT_WIDTH = 46   # left panel inner width (excluding borders)
    _RIGHT_WIDTH = 27  # right panel inner width (excluding borders)

    def __init__(
        self,
        model: str = "llama3",
        state_file: str = "./ccf_state.json",
        show_dashboard: bool = True,
    ) -> None:
        self.model = model
        self.state_file = state_file
        self.show_dashboard = show_dashboard

        # Load or create coherence field
        self.field = self._load_field()

        # Conversation history and counters
        self.conversation: list[dict] = []
        self.turn_count = 0
        self.last_save_time = time.time()

        # CCF state
        self.current_phase = SocialPhase.ShyObserver
        self.current_key: TextContextKey | None = None
        self.current_coherence = 0.0
        self._last_tick_ms = 0.0

    # ── Lifecycle ─────────────────────────────────────────────────────────

    def _load_field(self) -> CoherenceFieldPy:
        """Load field from persistence file, or create fresh."""
        accs, personality = CcfPersistence.load(self.state_file)
        if accs is None or personality is None:
            return CoherenceFieldPy(curiosity_drive=0.5, recovery_rate=0.5)

        field = CoherenceFieldPy(
            curiosity_drive=personality.get("curiosity", 0.5),
            recovery_rate=personality.get("recovery", 0.5),
        )
        from ccf_core.coherence_field_py import _Accumulator
        for ctx_hash, acc_data in accs.items():
            field._accumulators[ctx_hash] = _Accumulator.from_dict(acc_data)
        return field

    def _save(self) -> None:
        """Persist current field state atomically."""
        accs = {
            k: v.to_dict()
            for k, v in self.field._accumulators.items()
        }
        personality = {
            "curiosity": self.field.curiosity_drive,
            "recovery": self.field.recovery_rate,
        }
        CcfPersistence.save(accs, personality, self.state_file)
        self.last_save_time = time.time()

    def _save_and_exit(self) -> None:
        self._save()
        self._clear()
        print("State saved. Goodbye.")

    # ── Main loop ─────────────────────────────────────────────────────────

    def run(self) -> None:
        """Main interactive loop."""
        self._print_welcome()
        if self.show_dashboard:
            self._redraw()

        while True:
            try:
                if self.show_dashboard:
                    user_input = input("\n> ").strip()
                else:
                    user_input = input("> ").strip()
            except (EOFError, KeyboardInterrupt):
                print()
                self._save_and_exit()
                break

            if user_input.lower() == "q":
                self._save_and_exit()
                break
            elif user_input.lower() == "s":
                self._save()
                if self.show_dashboard:
                    self._redraw()
                else:
                    print("State saved.")
                continue
            elif user_input == "r":
                self._record_positive()
                continue
            elif user_input == "R":
                self._record_negative()
                continue
            elif not user_input:
                continue

            self._process_turn(user_input)

    # ── Turn processing ───────────────────────────────────────────────────

    def _process_turn(self, message: str) -> None:
        """Process one user turn: CCF tick -> LLM call -> dashboard update."""
        # 1. Derive TextContextKey
        key = TextContextKey.from_text(message, turn_count=self.turn_count)
        self.current_key = key

        # 2. CCF tick: update coherence
        tick_start = time.perf_counter()
        ctx_hash = key.context_hash()
        acc = self.field._get_or_create(ctx_hash)
        eff = acc.effective_coherence(instant=0.7)
        self.current_phase = classify_phase(eff, 0.7, self.current_phase)
        self.current_coherence = eff
        tick_ms = (time.perf_counter() - tick_start) * 1000
        self._last_tick_ms = tick_ms

        # 3. Get system prompt injection
        injection = get_prompt_injection(self.current_phase)

        # 4. Call LLM
        response_text = self._call_llm(message, injection)

        # 5. Update conversation history
        self.conversation.append({"role": "user", "content": message})
        self.conversation.append({"role": "assistant", "content": response_text})
        self.turn_count += 1

        # 6. Auto-record positive interaction (after response)
        acc.positive_interaction(curiosity=self.field.curiosity_drive)

        # 7. Auto-save every 5 turns
        if self.turn_count % 5 == 0:
            self._save()

        # 8. Redraw dashboard
        if self.show_dashboard:
            self._redraw(tick_ms)
        else:
            # Headless-friendly output (no dashboard)
            print(f"Assistant: {response_text[:200]}")

    def _record_positive(self) -> None:
        """Record a positive interaction for current context."""
        if self.current_key is None:
            print("No active context yet — say something first.")
            return
        acc = self.field._get_or_create(self.current_key.context_hash())
        acc.positive_interaction(curiosity=self.field.curiosity_drive)
        eff = acc.effective_coherence(instant=0.7)
        self.current_phase = classify_phase(eff, 0.7, self.current_phase)
        self.current_coherence = eff
        if self.show_dashboard:
            self._redraw(self._last_tick_ms)

    def _record_negative(self) -> None:
        """Record a negative interaction for current context."""
        if self.current_key is None:
            print("No active context yet — say something first.")
            return
        acc = self.field._get_or_create(self.current_key.context_hash())
        acc.negative_interaction(recovery=self.field.recovery_rate)
        eff = acc.effective_coherence(instant=0.7)
        self.current_phase = classify_phase(eff, 0.7, self.current_phase)
        self.current_coherence = eff
        if self.show_dashboard:
            self._redraw(self._last_tick_ms)

    # ── LLM call ─────────────────────────────────────────────────────────

    def _call_llm(self, message: str, injection: str) -> str:
        """Call Ollama. Returns demo response if Ollama not available."""
        try:
            import httpx  # type: ignore[import]
            messages: list[dict] = []
            if injection:
                messages.append({"role": "system", "content": injection})
            # Last 10 turns as context
            for turn in self.conversation[-10:]:
                messages.append(turn)
            messages.append({"role": "user", "content": message})

            resp = httpx.post(
                "http://localhost:11434/api/chat",
                json={"model": self.model, "messages": messages, "stream": False},
                timeout=60.0,
            )
            resp.raise_for_status()
            return resp.json()["message"]["content"]
        except Exception:
            # Fail-open: return a demo response so the demo always works
            return (
                f"[Demo mode — Ollama not available] "
                f"Simulated response for: \"{message[:60]}\". "
                f"Phase: {self.current_phase.value}, "
                f"Coherence: {self.current_coherence:.2f}."
            )

    # ── Rendering ─────────────────────────────────────────────────────────

    def _clear(self) -> None:
        """Clear terminal screen using ANSI escape (I-LLM-060)."""
        print("\033[H\033[J", end="", flush=True)

    def _print_welcome(self) -> None:
        print(f"{_BOLD}CCF CLI Demo{_RESET} — Contextual Coherence Fields")
        print(f"Model: {self.model}  |  State: {self.state_file}")
        print()

    def _redraw(self, tick_ms: float = 0.0) -> None:
        """
        Clear terminal and redraw conversation + dashboard (I-LLM-060).

        Uses ANSI clear-screen rather than curses — works in any 80-col terminal
        (I-LLM-064). No GPU required (I-LLM-062).
        """
        self._clear()
        self._print_header()
        self._print_body(tick_ms)
        self._print_footer()

    def _print_header(self) -> None:
        model_label = f"CCF Chat — {self.model}"[:34]
        # Pad to left panel width
        left = f" {model_label:<{self._LEFT_WIDTH - 1}}"
        right = f" {'Trust Dashboard':<{self._RIGHT_WIDTH - 1}}"
        print(f"{_BOLD}╔{'═' * self._LEFT_WIDTH}╦{'═' * self._RIGHT_WIDTH}╗{_RESET}")
        print(f"{_BOLD}║{_RESET}{left}{_BOLD}║{_RESET}{right}{_BOLD}║{_RESET}")
        print(f"{_BOLD}╠{'═' * self._LEFT_WIDTH}╬{'═' * self._RIGHT_WIDTH}╣{_RESET}")

    def _print_body(self, tick_ms: float) -> None:
        """
        Print conversation (left) and dashboard (right) side by side.
        Both panels render within 80 columns (I-LLM-064).
        """
        left_lines = self._build_conversation_lines()
        right_lines = self._build_dashboard_lines(tick_ms)

        # Pad both sides to the same height
        max_rows = max(len(left_lines), len(right_lines), 6)
        while len(left_lines) < max_rows:
            left_lines.append("")
        while len(right_lines) < max_rows:
            right_lines.append("")

        for l_text, r_text in zip(left_lines, right_lines):
            # Strip ANSI for width calculation
            l_plain = self._strip_ansi(l_text)
            r_plain = self._strip_ansi(r_text)
            l_pad = " " * max(0, self._LEFT_WIDTH - 1 - len(l_plain))
            r_pad = " " * max(0, self._RIGHT_WIDTH - 1 - len(r_plain))
            print(
                f"{_BOLD}║{_RESET} {l_text}{l_pad}"
                f"{_BOLD}║{_RESET} {r_text}{r_pad}"
                f"{_BOLD}║{_RESET}"
            )

        print(f"{_BOLD}╚{'═' * self._LEFT_WIDTH}╩{'═' * self._RIGHT_WIDTH}╝{_RESET}")

    def _build_conversation_lines(self) -> list[str]:
        """Build left-panel conversation lines, truncated to fit."""
        max_inner = self._LEFT_WIDTH - 2  # subtract " " prefix and trailing space
        lines = []

        # Show last few exchanges
        recent = self.conversation[-6:]  # up to 3 turns
        for turn in recent:
            role_label = "You" if turn["role"] == "user" else "AI "
            text = turn["content"].replace("\n", " ")
            prefix = f"{_BOLD}{role_label}:{_RESET} "
            # Available space after role label (no ANSI counted in width)
            avail = max_inner - 5  # "You: " or "AI:  "
            snippet = text[:avail] + ("…" if len(text) > avail else "")
            lines.append(f"{prefix}{snippet}")

        # Input prompt placeholder
        lines.append("")
        lines.append(f"{_DIM}> _{_RESET}")
        return lines

    def _build_dashboard_lines(self, tick_ms: float) -> list[str]:
        """Build right-panel dashboard lines (I-LLM-061)."""
        phase = self.current_phase
        coherence = self.current_coherence
        phase_name = phase.value

        colour = _PHASE_COLOURS.get(phase_name, "")
        icon = _PHASE_ICONS.get(phase_name, "?")

        bar_filled = int(coherence * 10)
        bar = "▓" * bar_filled + "░" * (10 - bar_filled)

        ctx_label = self.current_key.label() if self.current_key else "none"
        # Shorten for 27-col panel: strip topic: prefix, truncate
        ctx_short = ctx_label
        if ctx_short.startswith("topic:"):
            parts = ctx_short.split(":")
            # "finance:moderate" style
            ctx_short = ":".join(parts[2:]) if len(parts) >= 3 else ctx_short
        if len(ctx_short) > self._RIGHT_WIDTH - 12:
            ctx_short = ctx_short[: self._RIGHT_WIDTH - 15] + "…"

        save_secs = int(time.time() - self.last_save_time)
        if save_secs < 60:
            save_label = f"{save_secs}s ago"
        else:
            save_label = f"{save_secs // 60}m ago"

        lines = [
            f"Phase:  {colour}{icon} {phase_name}{_RESET}",
            f"Coh:    [{bar}] {coherence:.2f}",
            f"Ctx:    {ctx_short}",
            f"[data-testid: tick-latency-ms] {tick_ms:.1f}ms",
            f"[data-testid: interaction-count] Turns: {self.turn_count}",
            f"[data-testid: save-indicator] {save_label}",
        ]
        return lines

    def _print_footer(self) -> None:
        ctrl = (
            f"{_DIM}[Enter]{_RESET} send  "
            f"{_DIM}[r]{_RESET} positive  "
            f"{_DIM}[R]{_RESET} negative  "
            f"{_DIM}[s]{_RESET} save  "
            f"{_DIM}[q]{_RESET} quit"
        )
        print(ctrl)

    @staticmethod
    def _strip_ansi(text: str) -> str:
        """Remove ANSI escape codes for width measurement."""
        import re
        return re.sub(r"\033\[[0-9;]*m", "", text)


# ── Argument parsing ──────────────────────────────────────────────────────────

def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="CCF CLI demo — terminal chat with live trust dashboard",
        prog="ccf-demo",
    )
    parser.add_argument(
        "--model",
        default="llama3",
        help="Ollama model name (default: llama3)",
    )
    parser.add_argument(
        "--state-file",
        default="./ccf_state.json",
        help="Path to JSON state file (default: ./ccf_state.json)",
    )
    parser.add_argument(
        "--no-dashboard",
        action="store_true",
        help="Disable dashboard — headless-friendly text output (I-LLM-065)",
    )
    parser.add_argument(
        "--headless-turns",
        type=int,
        default=0,
        metavar="N",
        help="Run N scripted turns in CI headless mode, print data-testid output",
    )
    return parser.parse_args(argv)


# ── Entry point ───────────────────────────────────────────────────────────────

def main(argv: list[str] | None = None) -> None:
    """Entry point for `python -m ccf_core.demo` and `ccf-demo` console script."""
    args = parse_args(argv)

    if args.headless_turns:
        # Headless mode: run N scripted turns and emit data-testid output (I-LLM-065)
        run_headless(args)
        return

    demo = CcfDemo(
        model=args.model,
        state_file=args.state_file,
        show_dashboard=not args.no_dashboard,
    )
    demo.run()


if __name__ == "__main__":
    main()
