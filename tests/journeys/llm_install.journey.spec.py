"""
Journey Test: J-LLM-INSTALL
Tests that pip install ccf-core succeeds and the package is usable.

Specflow contract: J-LLM-INSTALL
Invariants: I-LLM-020, I-LLM-021, I-LLM-022, I-LLM-023, I-LLM-024
DOD: pip install ccf-core works on Python 3.9+ without requiring Rust toolchain.

Usage:
    # With the wheel installed:
    pytest tests/journeys/llm_install.journey.spec.py -v

    # Collect only (no ccf_core install needed):
    pytest tests/journeys/llm_install.journey.spec.py --collect-only
"""
import sys
import pytest


# ─── Helpers ────────────────────────────────────────────────────────────────

def _skip_if_not_installed():
    """Skip with a clear message if ccf_core is not installed."""
    try:
        import ccf_core  # noqa: F401
    except ImportError as e:
        pytest.skip(
            f"ccf_core not installed — run `pip install ccf-core` first.\n"
            f"ImportError: {e}"
        )


def _skip_if_rust_extension_missing():
    """Skip tests that require the compiled Rust extension (_ccf_core).

    The pure-Python package is always importable, but CoherenceField/ContextKey
    etc. are None until `maturin develop --features python-ffi` has been run.
    Run: cd ccf-py && maturin develop --features python-ffi
    """
    try:
        import ccf_core
        if ccf_core.CoherenceField is None:
            pytest.skip(
                "Rust extension not compiled — run:\n"
                "  cd ccf-py && maturin develop\n"
                "or install a pre-built wheel: pip install ccf-core"
            )
    except ImportError as e:
        pytest.skip(f"ccf_core not installed: {e}")


# ─── Journey: J-LLM-INSTALL ─────────────────────────────────────────────────


def test_python_version_compatible():
    """[data-testid: python-version-check] Python 3.9+ is required per I-LLM-020."""
    major, minor = sys.version_info[:2]
    assert (major, minor) >= (3, 9), (
        f"Python 3.9+ required per I-LLM-020, "
        f"but running {major}.{minor}"
    )
    print(f"[data-testid: python-version-check] Python {major}.{minor} OK")


def test_ccf_core_importable():
    """[data-testid: import-check] ccf_core imports without error."""
    _skip_if_not_installed()
    import ccf_core  # noqa: F401
    print("[data-testid: import-check] ccf_core imported successfully")


def test_ccf_core_version():
    """[data-testid: ccf-version] __version__ is set and is a valid semver string."""
    _skip_if_not_installed()
    import ccf_core
    assert hasattr(ccf_core, "__version__"), (
        "[data-testid: ccf-version] MISSING __version__ attribute"
    )
    version = ccf_core.__version__
    assert isinstance(version, str), (
        f"__version__ must be a string, got {type(version)}"
    )
    parts = version.split(".")
    assert len(parts) == 3, f"Expected semver X.Y.Z, got: {version!r}"
    assert all(p.isdigit() for p in parts), (
        f"Semver parts must be numeric, got: {version!r}"
    )
    print(f"[data-testid: ccf-version] {version}")


def test_coherence_field_importable():
    """[data-testid: import-check] CoherenceField symbol is exported from ccf_core."""
    _skip_if_rust_extension_missing()
    from ccf_core import CoherenceField
    assert CoherenceField is not None


def test_personality_importable():
    """[data-testid: import-check] Personality symbol is exported from ccf_core."""
    _skip_if_rust_extension_missing()
    from ccf_core import Personality
    assert Personality is not None


def test_social_phase_importable():
    """[data-testid: import-check] SocialPhase symbol is exported with expected variants."""
    _skip_if_rust_extension_missing()
    from ccf_core import SocialPhase
    assert hasattr(SocialPhase, "ShyObserver"), (
        "SocialPhase.ShyObserver missing"
    )
    assert hasattr(SocialPhase, "QuietlyBeloved"), (
        "SocialPhase.QuietlyBeloved missing"
    )
    assert hasattr(SocialPhase, "StartledRetreat"), (
        "SocialPhase.StartledRetreat missing"
    )
    assert hasattr(SocialPhase, "ProtectiveGuardian"), (
        "SocialPhase.ProtectiveGuardian missing"
    )


def test_context_key_importable():
    """[data-testid: import-check] ContextKey symbol is exported from ccf_core."""
    _skip_if_rust_extension_missing()
    from ccf_core import ContextKey
    assert ContextKey is not None


def test_coherence_field_instantiates():
    """[data-testid: install-success] CoherenceField() can be constructed without args."""
    _skip_if_rust_extension_missing()
    from ccf_core import CoherenceField
    field = CoherenceField()
    assert field is not None
    print("[data-testid: install-success] CoherenceField() instantiated OK")


def test_coherence_field_with_curiosity():
    """[data-testid: install-success] CoherenceField(curiosity_drive=0.5) works."""
    _skip_if_rust_extension_missing()
    from ccf_core import CoherenceField
    field = CoherenceField(curiosity_drive=0.5)
    assert field is not None


def test_context_key_instantiates():
    """[data-testid: install-success] ContextKey() can be constructed with defaults."""
    _skip_if_rust_extension_missing()
    from ccf_core import ContextKey
    ctx = ContextKey()
    assert ctx is not None


def test_context_key_with_args():
    """ContextKey accepts sensor parameters."""
    _skip_if_rust_extension_missing()
    from ccf_core import ContextKey
    ctx = ContextKey(
        light_level=0.7,
        sound_level=0.1,
        presence="static",
        accel_mag=1.0,
        motors_active=False,
        roll_deg=0.0,
        pitch_deg=0.0,
    )
    assert ctx is not None
    label = ctx.label()
    assert isinstance(label, str), f"label() must return str, got {type(label)}"
    assert len(label) > 0


def test_context_key_feature_vec():
    """ContextKey.feature_vec() returns a 6-element float tuple."""
    _skip_if_rust_extension_missing()
    from ccf_core import ContextKey
    ctx = ContextKey(light_level=0.7, sound_level=0.1)
    vec = ctx.feature_vec()
    assert len(vec) == 6, f"Expected 6 features, got {len(vec)}"
    assert all(0.0 <= v <= 1.0 for v in vec), (
        f"All feature values must be in [0, 1]: {vec}"
    )


def test_coherence_field_positive_interaction():
    """Positive interaction increases coherence above zero."""
    _skip_if_rust_extension_missing()
    from ccf_core import CoherenceField, ContextKey
    field = CoherenceField()
    ctx = ContextKey(light_level=0.6, sound_level=0.1)

    # Coherence starts at 0
    assert field.context_coherence(ctx) == 0.0

    # After positive interactions, coherence should grow
    for _ in range(10):
        field.positive_interaction(ctx)

    coherence = field.context_coherence(ctx)
    assert coherence > 0.0, f"Coherence should be > 0 after interactions: {coherence}"
    assert coherence <= 1.0, f"Coherence must be <= 1.0: {coherence}"
    print(f"[data-testid: install-success] Coherence after 10 interactions: {coherence:.4f}")


def test_coherence_field_effective_coherence():
    """CoherenceField.effective_coherence() respects the asymmetric gate."""
    _skip_if_rust_extension_missing()
    from ccf_core import CoherenceField, ContextKey

    field = CoherenceField()
    ctx = ContextKey(light_level=0.6, sound_level=0.1)

    # Unfamiliar context: effective_coherence should be <= context coherence
    eff = field.effective_coherence(0.9, ctx)
    ctx_coh = field.context_coherence(ctx)
    assert eff <= max(ctx_coh, 0.001), (
        f"Unfamiliar gate should cap at ctx coherence: eff={eff}, ctx={ctx_coh}"
    )


def test_personality_instantiates():
    """[data-testid: install-success] Personality() can be constructed."""
    _skip_if_rust_extension_missing()
    from ccf_core import Personality
    p = Personality()
    assert p is not None
    assert isinstance(p.curiosity_drive, float)
    assert 0.0 <= p.curiosity_drive <= 1.0
    print(f"[data-testid: install-success] Personality instantiated, curiosity_drive={p.curiosity_drive}")


def test_personality_defaults_are_neutral():
    """Default Personality has all parameters at 0.5 per I-PERS-003."""
    _skip_if_rust_extension_missing()
    from ccf_core import Personality
    p = Personality()
    assert abs(p.tension_baseline - 0.5) < 0.001
    assert abs(p.curiosity_drive - 0.5) < 0.001
    assert abs(p.recovery_speed - 0.5) < 0.001


def test_social_phase_classify_integration():
    """CoherenceField.classify_phase returns a SocialPhase."""
    _skip_if_rust_extension_missing()
    from ccf_core import CoherenceField, ContextKey, SocialPhase
    field = CoherenceField()
    ctx = ContextKey(light_level=0.5, sound_level=0.1)

    phase = field.classify_phase(
        instant_coherence=0.8,
        tension=0.2,
        context=ctx,
    )
    assert phase is not None
    # A new context with 0 accumulated coherence should be ShyObserver
    assert phase == SocialPhase.ShyObserver, (
        f"New context should be ShyObserver, got: {phase}"
    )
    print(f"[data-testid: install-success] classify_phase returned: {phase}")


def test_context_key_presence_variants():
    """ContextKey accepts all valid presence strings."""
    _skip_if_rust_extension_missing()
    from ccf_core import ContextKey
    for presence in ("absent", "static", "approaching", "retreating"):
        ctx = ContextKey(presence=presence)
        assert ctx is not None, f"ContextKey with presence={presence!r} failed"


def test_context_key_invalid_presence_raises():
    """ContextKey raises ValueError for invalid presence string."""
    _skip_if_rust_extension_missing()
    from ccf_core import ContextKey
    with pytest.raises(Exception):
        ContextKey(presence="invalid-value")


def test_i_llm_022_basic_workflow():
    """[data-testid: install-success] I-LLM-022: import ccf_core; ccf_core.CoherenceField() works."""
    _skip_if_rust_extension_missing()
    import ccf_core
    field = ccf_core.CoherenceField()
    assert field is not None
    print("[data-testid: install-success] I-LLM-022 satisfied: CoherenceField() works")
