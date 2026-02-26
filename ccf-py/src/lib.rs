/*
 * Notice of Provisional Patent Filing:
 * The methods and algorithms implemented in this file (specifically relating to
 * Contextual Coherence Fields and relational coherence accumulation) are the
 * subject of a United States Provisional Patent Application (63/988,438)
 * filed on February 23, 2026.
 *
 * This source code is licensed under the Business Source License 1.1.
 * See LICENSE and PATENTS.md in the root directory for full details.
 */

//! Python bindings for CCF (Contextual Coherence Fields).
//!
//! This crate wraps `mbot-core` and exposes a clean Python API via PyO3.
//!
//! Invariants:
//! - I-LLM-022: `import ccf_core; ccf_core.CoherenceField()` works
//! - I-LLM-023: maturin build is deterministic
//! - I-LLM-024: Wheel size < 5MB

use pyo3::prelude::*;

use mbot_core::coherence::{
    CoherenceField as RustCoherenceField,
    CoherenceAccumulator as RustCoherenceAccumulator,
    ContextKey as RustContextKey,
    BrightnessBand, NoiseBand, PresenceSignature, MotionContext, Orientation, TimePeriod,
    SocialPhase as RustSocialPhase,
    PhaseSpace,
};
use mbot_core::personality::Personality as RustPersonality;

// ─── SocialPhase ────────────────────────────────────────────────────────────

/// Behavioral phase from the 2D (coherence x tension) space.
///
/// - ShyObserver: Low coherence, low tension — minimal expression.
/// - StartledRetreat: Low coherence, high tension — protective withdrawal.
/// - QuietlyBeloved: High coherence, low tension — full expressive range.
/// - ProtectiveGuardian: High coherence, high tension — confident protection.
#[pyclass(name = "SocialPhase")]
#[derive(Clone, Debug)]
pub struct PySocialPhase {
    inner: RustSocialPhase,
}

#[pymethods]
impl PySocialPhase {
    #[classattr]
    #[allow(non_snake_case)]
    fn ShyObserver(py: Python<'_>) -> PyObject {
        PySocialPhase { inner: RustSocialPhase::ShyObserver }.into_py(py)
    }

    #[classattr]
    #[allow(non_snake_case)]
    fn StartledRetreat(py: Python<'_>) -> PyObject {
        PySocialPhase { inner: RustSocialPhase::StartledRetreat }.into_py(py)
    }

    #[classattr]
    #[allow(non_snake_case)]
    fn QuietlyBeloved(py: Python<'_>) -> PyObject {
        PySocialPhase { inner: RustSocialPhase::QuietlyBeloved }.into_py(py)
    }

    #[classattr]
    #[allow(non_snake_case)]
    fn ProtectiveGuardian(py: Python<'_>) -> PyObject {
        PySocialPhase { inner: RustSocialPhase::ProtectiveGuardian }.into_py(py)
    }

    /// Scale factor for expressive output in this phase [0.0, 1.0].
    fn expression_scale(&self) -> f32 {
        self.inner.expression_scale()
    }

    /// LED color tint for this phase as [R, G, B].
    fn led_tint(&self) -> [u8; 3] {
        self.inner.led_tint()
    }

    fn __repr__(&self) -> String {
        format!("SocialPhase.{:?}", self.inner)
    }

    fn __eq__(&self, other: &PySocialPhase) -> bool {
        self.inner == other.inner
    }
}

// ─── ContextKey ─────────────────────────────────────────────────────────────

/// Composite context key — the full situation fingerprint.
///
/// Two interactions that produce the same ContextKey are considered
/// the same relational situation for coherence accumulation.
#[pyclass(name = "ContextKey")]
#[derive(Clone, Debug)]
pub struct PyContextKey {
    inner: RustContextKey,
}

#[pymethods]
impl PyContextKey {
    /// Build a context key from raw sensor values.
    ///
    /// Args:
    ///     light_level: 0.0-1.0 ambient brightness.
    ///     sound_level: 0.0-1.0 ambient noise.
    ///     presence: "absent", "static", "approaching", or "retreating".
    ///     accel_mag: accelerometer vector magnitude.
    ///     motors_active: whether motors are currently powered.
    ///     roll_deg: roll angle in degrees.
    ///     pitch_deg: pitch angle in degrees.
    #[new]
    #[pyo3(signature = (
        light_level = 0.5,
        sound_level = 0.1,
        presence = "static",
        accel_mag = 1.0,
        motors_active = false,
        roll_deg = 0.0,
        pitch_deg = 0.0
    ))]
    fn new(
        light_level: f32,
        sound_level: f32,
        presence: &str,
        accel_mag: f32,
        motors_active: bool,
        roll_deg: f32,
        pitch_deg: f32,
    ) -> PyResult<Self> {
        let presence_sig = match presence {
            "absent" => PresenceSignature::Absent,
            "static" => PresenceSignature::Static,
            "approaching" => PresenceSignature::Approaching,
            "retreating" => PresenceSignature::Retreating,
            other => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid presence value: '{}'. Must be one of: absent, static, approaching, retreating",
                    other
                )));
            }
        };

        let inner = RustContextKey::from_sensors(
            light_level,
            sound_level,
            presence_sig,
            accel_mag,
            motors_active,
            roll_deg,
            pitch_deg,
        );

        Ok(PyContextKey { inner })
    }

    /// Human-readable label, e.g. "Bright · Quiet · Static".
    fn label(&self) -> String {
        self.inner.to_label()
    }

    /// Encode as a 6-dimensional float feature vector [0.0, 1.0].
    fn feature_vec(&self) -> [f32; 6] {
        self.inner.to_feature_vec()
    }

    /// Deterministic u32 hash of this context key.
    fn context_hash(&self) -> u32 {
        self.inner.context_hash_u32()
    }

    fn __repr__(&self) -> String {
        format!("ContextKey({})", self.inner.to_label())
    }

    fn __eq__(&self, other: &PyContextKey) -> bool {
        self.inner == other.inner
    }

    fn __hash__(&self) -> u32 {
        self.inner.context_hash_u32()
    }
}

// ─── CoherenceField ─────────────────────────────────────────────────────────

/// The coherence field: a map of context → accumulated trust.
///
/// Implements CCF (Contextual Coherence Fields): earned relational trust
/// that must be independently built in each distinct sensor context.
///
/// Patent pending: US Provisional Application 63/988,438 (priority date 23 Feb 2026).
///
/// Example:
///     >>> from ccf_core import CoherenceField, ContextKey
///     >>> field = CoherenceField()
///     >>> ctx = ContextKey(light_level=0.7, sound_level=0.1)
///     >>> field.positive_interaction(ctx)
///     >>> print(field.context_coherence(ctx))
#[pyclass(name = "CoherenceField")]
pub struct PyCoherenceField {
    inner: RustCoherenceField,
    tick: u64,
}

#[pymethods]
impl PyCoherenceField {
    /// Create a new coherence field.
    ///
    /// Args:
    ///     curiosity_drive: Personality curiosity [0.0, 1.0]. Higher values give
    ///         new contexts a small head-start (cold-start baseline). Default: 0.0.
    #[new]
    #[pyo3(signature = (curiosity_drive = 0.0))]
    fn new(curiosity_drive: f32) -> Self {
        let inner = if curiosity_drive > 0.0 {
            RustCoherenceField::new_with_personality(curiosity_drive)
        } else {
            RustCoherenceField::new()
        };
        PyCoherenceField { inner, tick: 0 }
    }

    /// Record a positive interaction for the given context.
    ///
    /// Args:
    ///     context: The ContextKey identifying the current situation.
    ///     recovery_speed: Personality parameter [0.0, 1.0]. Default: 0.5.
    ///     alone: True if no presence detected (bootstraps faster). Default: False.
    #[pyo3(signature = (context, recovery_speed = 0.5, alone = false))]
    fn positive_interaction(
        &mut self,
        context: &PyContextKey,
        recovery_speed: f32,
        alone: bool,
    ) {
        self.tick += 1;
        let acc = self.inner.get_or_create(&context.inner);
        acc.positive_interaction(recovery_speed, self.tick, alone);
    }

    /// Record a negative interaction (startle, collision, high tension).
    ///
    /// Args:
    ///     context: The ContextKey identifying the current situation.
    ///     startle_sensitivity: Personality parameter [0.0, 1.0]. Default: 0.5.
    #[pyo3(signature = (context, startle_sensitivity = 0.5))]
    fn negative_interaction(&mut self, context: &PyContextKey, startle_sensitivity: f32) {
        self.tick += 1;
        let acc = self.inner.get_or_create(&context.inner);
        acc.negative_interaction(startle_sensitivity, self.tick);
    }

    /// Get the accumulated coherence for a context [0.0, 1.0].
    ///
    /// Returns 0.0 for unseen contexts (or fallback if set).
    fn context_coherence(&self, context: &PyContextKey) -> f32 {
        self.inner.context_coherence(&context.inner)
    }

    /// Compute effective coherence using the asymmetric gate (CCF-001).
    ///
    /// - Unfamiliar contexts (ctx < 0.3): min(instant, ctx) — earn trust first.
    /// - Familiar contexts (ctx >= 0.3): 0.3*instant + 0.7*ctx — history buffers noise.
    ///
    /// Args:
    ///     instant_coherence: Current sensor-derived coherence [0.0, 1.0].
    ///     context: The ContextKey identifying the current situation.
    fn effective_coherence(&self, instant_coherence: f32, context: &PyContextKey) -> f32 {
        self.inner.effective_coherence(instant_coherence, &context.inner)
    }

    /// Number of positive interactions recorded for a context (0 if unseen).
    fn interaction_count(&self, context: &PyContextKey) -> u32 {
        self.inner.context_interaction_count(&context.inner)
    }

    /// Number of tracked contexts.
    fn context_count(&self) -> usize {
        self.inner.context_count()
    }

    /// Apply time-based decay to all accumulators.
    ///
    /// Args:
    ///     ticks_elapsed: Number of ticks since last decay call. Default: 1.
    #[pyo3(signature = (ticks_elapsed = 1))]
    fn decay_all(&mut self, ticks_elapsed: u64) {
        self.inner.decay_all(ticks_elapsed);
    }

    /// Set fallback coherence for unseen contexts (degraded mode).
    ///
    /// When the companion is unavailable, prevents every unseen context
    /// from starting at 0.0. Pass None to clear.
    fn set_fallback(&mut self, value: Option<f32>) {
        self.inner.set_fallback(value);
    }

    /// Classify the current social phase given instant coherence and tension.
    ///
    /// Returns a SocialPhase value.
    ///
    /// Args:
    ///     instant_coherence: Current sensor-derived coherence [0.0, 1.0].
    ///     tension: Current tension from homeostasis [0.0, 1.0].
    ///     context: The ContextKey identifying the current situation.
    ///     previous_phase: Previous SocialPhase for hysteresis. Default: ShyObserver.
    #[pyo3(signature = (instant_coherence, tension, context, previous_phase = None))]
    fn classify_phase(
        &self,
        instant_coherence: f32,
        tension: f32,
        context: &PyContextKey,
        previous_phase: Option<&PySocialPhase>,
    ) -> PySocialPhase {
        let eff = self.inner.effective_coherence(instant_coherence, &context.inner);
        let previous = previous_phase
            .map(|p| p.inner)
            .unwrap_or(RustSocialPhase::ShyObserver);
        let phase_space = PhaseSpace::default();
        let phase = RustSocialPhase::classify(eff, tension, previous, &phase_space);
        PySocialPhase { inner: phase }
    }

    fn __repr__(&self) -> String {
        format!(
            "CoherenceField(contexts={}, tick={})",
            self.inner.context_count(),
            self.tick
        )
    }
}

// ─── Personality ────────────────────────────────────────────────────────────

/// Personality configuration for mBot2.
///
/// Defines how the robot behaves, reacts to stimuli, and expresses its internal
/// state. All parameters are bounded to [0.0, 1.0].
///
/// Example:
///     >>> from ccf_core import Personality
///     >>> p = Personality()
///     >>> print(p.curiosity_drive)
///     0.5
#[pyclass(name = "Personality")]
pub struct PyPersonality {
    inner: RustPersonality,
}

#[pymethods]
impl PyPersonality {
    /// Create a Personality with default neutral values (all 0.5).
    #[new]
    fn new() -> Self {
        PyPersonality {
            inner: RustPersonality::default(),
        }
    }

    #[getter]
    fn id(&self) -> &str {
        &self.inner.id
    }

    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    #[getter]
    fn tension_baseline(&self) -> f32 {
        self.inner.tension_baseline()
    }

    #[getter]
    fn coherence_baseline(&self) -> f32 {
        self.inner.coherence_baseline()
    }

    #[getter]
    fn energy_baseline(&self) -> f32 {
        self.inner.energy_baseline()
    }

    #[getter]
    fn startle_sensitivity(&self) -> f32 {
        self.inner.startle_sensitivity()
    }

    #[getter]
    fn recovery_speed(&self) -> f32 {
        self.inner.recovery_speed()
    }

    #[getter]
    fn curiosity_drive(&self) -> f32 {
        self.inner.curiosity_drive()
    }

    #[getter]
    fn movement_expressiveness(&self) -> f32 {
        self.inner.movement_expressiveness()
    }

    #[getter]
    fn sound_expressiveness(&self) -> f32 {
        self.inner.sound_expressiveness()
    }

    #[getter]
    fn light_expressiveness(&self) -> f32 {
        self.inner.light_expressiveness()
    }

    fn validate(&self) -> PyResult<()> {
        self.inner
            .validate()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!(
            "Personality(id='{}', name='{}', curiosity={:.2})",
            self.inner.id,
            self.inner.name,
            self.inner.curiosity_drive()
        )
    }
}

// ─── Module ─────────────────────────────────────────────────────────────────

/// CCF (Contextual Coherence Fields) — earned relational trust for AI systems.
///
/// Patent pending: US Provisional Application 63/988,438 (priority date 23 Feb 2026).
///
/// Key types:
///   - CoherenceField: The main trust accumulator map.
///   - ContextKey: Composite sensor fingerprint for a relational situation.
///   - SocialPhase: Behavioral quadrant from (coherence x tension).
///   - Personality: Configurable behavior profile.
#[pymodule]
fn _ccf_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCoherenceField>()?;
    m.add_class::<PyContextKey>()?;
    m.add_class::<PySocialPhase>()?;
    m.add_class::<PyPersonality>()?;
    Ok(())
}
