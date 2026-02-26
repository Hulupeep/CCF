# ccf-core — Python Package

Python bindings for **Contextual Coherence Fields (CCF)** — earned relational trust for AI systems.

**Patent pending:** US Provisional Application 63/988,438 (priority date 23 Feb 2026).

## Install

```bash
pip install ccf-core
```

No Rust toolchain required. Pre-built wheels available for:
- Linux x86_64 / aarch64
- macOS x86_64 / ARM64 (Apple Silicon)
- Windows x86_64

Requires Python 3.9+.

## Quick Start

```python
from ccf_core import CoherenceField, ContextKey, SocialPhase, Personality

# Create a coherence field with a curious personality baseline
field = CoherenceField(curiosity_drive=0.5)

# Build a context from current sensor readings
ctx = ContextKey(
    light_level=0.7,    # bright room
    sound_level=0.1,    # quiet
    presence="static",  # someone nearby, not moving
)

# Record a positive interaction
field.positive_interaction(ctx, recovery_speed=0.5)

# Get the accumulated coherence for this context
coherence = field.context_coherence(ctx)
print(f"Coherence: {coherence:.4f}")

# Classify behavioral phase
phase = field.classify_phase(
    instant_coherence=coherence,
    tension=0.2,
    context=ctx,
)
print(f"Social phase: {phase}")  # SocialPhase.ShyObserver
```

## Key Types

| Type | Description |
|------|-------------|
| `CoherenceField` | The main trust accumulator. Maps context keys to earned coherence [0, 1]. |
| `ContextKey` | Composite sensor fingerprint. Two identical situations map to the same key. |
| `SocialPhase` | Behavioral quadrant: ShyObserver, StartledRetreat, QuietlyBeloved, ProtectiveGuardian. |
| `Personality` | Configurable behavior profile. All parameters bounded to [0.0, 1.0]. |

## License

Business Source License 1.1 (BUSL-1.1). See LICENSE for details.
