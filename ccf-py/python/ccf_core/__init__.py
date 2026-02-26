"""
ccf-core: Contextual Coherence Fields — earned relational trust for AI systems.

Patent pending: US Provisional Application 63/988,438 (priority date 23 Feb 2026).

Key types:
    CoherenceField  — The main trust accumulator. Maps context keys to earned coherence.
    ContextKey      — Composite sensor fingerprint identifying a relational situation.
    SocialPhase     — Behavioral quadrant from (coherence x tension) space.
    Personality     — Configurable behavior profile with bounded parameters.

Quick start:
    >>> from ccf_core import CoherenceField, ContextKey, SocialPhase
    >>> field = CoherenceField(curiosity_drive=0.5)
    >>> ctx = ContextKey(light_level=0.7, sound_level=0.1, presence="static")
    >>> field.positive_interaction(ctx)
    >>> phase = field.classify_phase(0.8, 0.2, ctx)
    >>> print(phase)
    SocialPhase.ShyObserver
"""

try:
    from ccf_core._ccf_core import (
        CoherenceField,
        ContextKey,
        SocialPhase,
        Personality,
    )
except ModuleNotFoundError:
    # Rust extension not built yet (e.g. pure-Python dev environment).
    # TextContextKey and other pure-Python modules remain available.
    CoherenceField = None  # type: ignore[assignment,misc]
    ContextKey = None  # type: ignore[assignment,misc]
    SocialPhase = None  # type: ignore[assignment,misc]
    Personality = None  # type: ignore[assignment,misc]

try:
    from ccf_core.__version__ import __version__
except ModuleNotFoundError:
    __version__ = "0.0.0+unbuilt"

from ccf_core.text_context_key import (
    TextContextKey,
    ConversationDepth,
    EmotionalRegister,
    TimeOfDay,
    SessionPhase,
)

from ccf_core.coherence_field_py import CoherenceFieldPy
from ccf_core.social_phase import (
    SocialPhase as SocialPhasePy,
    classify_phase,
    get_system_prompt,
    DEFAULT_PHASE_TEMPLATES,
)
from ccf_core.ollama_middleware import CcfOllama

__all__ = [
    "CoherenceField",
    "ContextKey",
    "SocialPhase",
    "Personality",
    "__version__",
    # TextContextKey — 5-dimensional LLM context vocabulary (I-LLM-010)
    "TextContextKey",
    "ConversationDepth",
    "EmotionalRegister",
    "TimeOfDay",
    "SessionPhase",
    # Pure-Python CoherenceField fallback (I-LLM-030, I-LLM-032)
    "CoherenceFieldPy",
    # SocialPhase (pure-Python) + helpers (I-LLM-030)
    "SocialPhasePy",
    "classify_phase",
    "get_system_prompt",
    "DEFAULT_PHASE_TEMPLATES",
    # Ollama middleware (I-LLM-030)
    "CcfOllama",
]
