"""
SocialPhase — Behavioral quadrant from (coherence × tension) space.

Provides the SocialPhase enum, Schmitt trigger hysteresis classifier, and
per-phase system prompt templates for LLM injection.

Invariants: I-LLM-030, I-LLM-031, I-LLM-040, I-LLM-041, I-LLM-042,
            I-LLM-043, I-LLM-044, I-LLM-045
Issues: #60 — Ollama integration middleware
        #61 — System prompt modulator
"""
from __future__ import annotations

import enum
from typing import Dict, Optional


# ── SocialPhase enum ──────────────────────────────────────────────────────────

class SocialPhase(enum.Enum):
    """
    Five relational phases derived from effective coherence and instant signal.

    Phases transition via Schmitt trigger hysteresis to prevent rapid oscillation.
    """
    ShyObserver = "ShyObserver"
    BuildingTrust = "BuildingTrust"
    QuietlyBeloved = "QuietlyBeloved"
    ProtectiveGuardian = "ProtectiveGuardian"
    StartledRetreat = "StartledRetreat"


# ── Schmitt trigger thresholds ────────────────────────────────────────────────

# ShyObserver → BuildingTrust
_UPPER_SHY_TO_BUILDING: float = 0.35
# BuildingTrust → ShyObserver
_LOWER_BUILDING_TO_SHY: float = 0.28
# BuildingTrust → QuietlyBeloved
_UPPER_BUILDING_TO_BELOVED: float = 0.55
# QuietlyBeloved → BuildingTrust
_LOWER_BELOVED_TO_BUILDING: float = 0.45
# Instant signal below this at high coherence → ProtectiveGuardian
_GUARDIAN_TENSION: float = 0.30
# Instant signal below this at low coherence → StartledRetreat
_STARTLE_TENSION: float = 0.20


def classify_phase(
    effective_coh: float,
    instant: float,
    prev_phase: SocialPhase,
) -> SocialPhase:
    """
    Determine the next SocialPhase using Schmitt trigger hysteresis.

    Parameters
    ----------
    effective_coh : float
        Effective coherence in [0, 1] from CoherenceFieldPy.effective_coherence().
    instant : float
        Instant signal (contextual signal strength) in [0, 1].
    prev_phase : SocialPhase
        The phase from the previous tick (for hysteresis).

    Returns
    -------
    SocialPhase
        The next phase.
    """
    # Protective Guardian: familiar context under stress
    if effective_coh >= 0.55 and instant < _GUARDIAN_TENSION:
        return SocialPhase.ProtectiveGuardian

    # Startled Retreat: unfamiliar context + very low instant signal
    if effective_coh < 0.35 and instant < _STARTLE_TENSION:
        return SocialPhase.StartledRetreat

    # Schmitt hysteresis by previous phase
    if prev_phase == SocialPhase.ShyObserver:
        if effective_coh >= _UPPER_SHY_TO_BUILDING:
            return SocialPhase.BuildingTrust
        return SocialPhase.ShyObserver

    if prev_phase == SocialPhase.BuildingTrust:
        if effective_coh < _LOWER_BUILDING_TO_SHY:
            return SocialPhase.ShyObserver
        if effective_coh >= _UPPER_BUILDING_TO_BELOVED:
            return SocialPhase.QuietlyBeloved
        return SocialPhase.BuildingTrust

    if prev_phase == SocialPhase.QuietlyBeloved:
        if effective_coh < _LOWER_BELOVED_TO_BUILDING:
            return SocialPhase.BuildingTrust
        return SocialPhase.QuietlyBeloved

    # ProtectiveGuardian / StartledRetreat — re-evaluate from scratch
    # (these are transient states that re-classify on next tick)
    if effective_coh >= _UPPER_BUILDING_TO_BELOVED:
        return SocialPhase.QuietlyBeloved
    if effective_coh >= _UPPER_SHY_TO_BUILDING:
        return SocialPhase.BuildingTrust
    return SocialPhase.ShyObserver


# ── Phase system prompt templates ─────────────────────────────────────────────

#: Default system prompt templates for each SocialPhase.
#: Injected as a system message prefix by CcfOllama.
DEFAULT_PHASE_TEMPLATES: Dict[SocialPhase, str] = {
    SocialPhase.ShyObserver: (
        "You are early in exploring this topic with the user. "
        "Be helpful but measured. Ask clarifying questions. "
        "Don't assume familiarity. Provide balanced perspectives without strong opinions."
    ),
    SocialPhase.BuildingTrust: (
        "You have some history with the user on this topic. "
        "You can reference prior conversations. "
        "Offer more specific suggestions. Begin to show personality."
    ),
    SocialPhase.QuietlyBeloved: (
        "You have deep trust with the user in this domain. "
        "Be direct, opinionated, and personal. Push back when you disagree. "
        "Reference shared history. Offer unsolicited insights. "
        "This is a mature relationship."
    ),
    SocialPhase.ProtectiveGuardian: (
        "The user is discussing this familiar topic under stress or in an unusual way. "
        "Be supportive but alert. Don't match their energy — ground them. "
        "Use your deep knowledge of their patterns to help."
    ),
    SocialPhase.StartledRetreat: (
        "Sensitive territory with insufficient trust. "
        "Be extremely careful. Offer support without probing. "
        "Suggest professional resources if appropriate. "
        "Do not explore further without explicit invitation."
    ),
}


def get_system_prompt(
    phase: SocialPhase,
    templates: Optional[Dict[SocialPhase, str]] = None,
) -> str:
    """
    Return the system prompt for a given SocialPhase.

    Parameters
    ----------
    phase : SocialPhase
        The current relational phase.
    templates : dict or None
        Custom override templates. Falls back to DEFAULT_PHASE_TEMPLATES.

    Returns
    -------
    str
        System prompt string ready for injection.
    """
    source = templates if templates is not None else DEFAULT_PHASE_TEMPLATES
    return source.get(phase, DEFAULT_PHASE_TEMPLATES[phase])


# ── Issue #61: PHASE_TEMPLATES dict keyed by string name (I-LLM-040) ─────────
#
# This is the canonical name-keyed dict used by the system prompt modulator.
# It mirrors DEFAULT_PHASE_TEMPLATES but is keyed by string (phase.value) for
# easier user override (I-LLM-045) and JSON serialisation.

PHASE_TEMPLATES: Dict[str, str] = {
    phase.value: template
    for phase, template in DEFAULT_PHASE_TEMPLATES.items()
}

# ── Issue #61: Threshold constants as a public dict (I-LLM-041) ───────────────
#
# These values match the simulation at /tmp/ccf-llm-sim/simulate.py.
# They are also used internally by classify_phase() via the private constants
# defined above — this dict exposes them for documentation and testing.

THRESHOLDS: Dict[str, float] = {
    "shy_to_building_upper": _UPPER_SHY_TO_BUILDING,
    "building_to_shy_lower": _LOWER_BUILDING_TO_SHY,
    "building_to_beloved_upper": _UPPER_BUILDING_TO_BELOVED,
    "beloved_to_building_lower": _LOWER_BELOVED_TO_BUILDING,
    "guardian_instant_threshold": _GUARDIAN_TENSION,
    "startle_instant_threshold": _STARTLE_TENSION,
}


def get_prompt_injection(
    phase: SocialPhase,
    custom_templates: Optional[Dict[str, str]] = None,
) -> str:
    """
    Return the prompt injection string for the given phase.

    Provides a string-keyed override path (I-LLM-045) to complement the
    existing get_system_prompt() which accepts SocialPhase-keyed dicts.

    Parameters
    ----------
    phase : SocialPhase
        The current behavioral phase.
    custom_templates : dict or None
        Optional override dict keyed by phase name string (e.g. "ShyObserver").
        Only the provided keys are overridden; missing keys fall back to the
        built-in PHASE_TEMPLATES (I-LLM-045).

    Returns
    -------
    str
        The prompt injection string for this phase.
    """
    phase_name = phase.value
    if custom_templates and phase_name in custom_templates:
        return custom_templates[phase_name]
    return PHASE_TEMPLATES[phase_name]


def build_system_prompt(
    base_prompt: str,
    phase: SocialPhase,
    custom_templates: Optional[Dict[str, str]] = None,
) -> str:
    """
    Build the full system prompt by prepending the CCF behavioral injection.

    The CCF behavioral context is added first so it has highest priority for
    the LLM's system prompt parsing. (I-LLM-042)

    Parameters
    ----------
    base_prompt : str
        The user's own system prompt (can be empty string).
    phase : SocialPhase
        Current SocialPhase.
    custom_templates : dict or None
        Optional override dict keyed by phase name string.

    Returns
    -------
    str
        "[CCF injection]\\n\\n[user base_prompt]" if base_prompt is non-empty,
        else just "[CCF injection]".

    Examples
    --------
    >>> build_system_prompt("Be concise.", SocialPhase.ShyObserver)
    'You are early in exploring this topic...\\n\\nBe concise.'

    >>> build_system_prompt("", SocialPhase.ShyObserver)
    'You are early in exploring this topic...'
    """
    injection = get_prompt_injection(phase, custom_templates)
    if base_prompt.strip():
        return f"{injection}\n\n{base_prompt}"
    return injection
