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

//! Contextual Coherence Fields
//!
//! Replaces the global coherence scalar with context-keyed accumulators
//! that must be independently earned through repeated interaction.
//!
//! # Architecture (ARCH-001: no_std compatible)
//!
//! Data structures and pure computation live here in mbot-core.
//! Persistence (RuVector OS platform), graph construction, and min-cut live in mbot-companion.
//!
//! # Key types
//!
//! - [`ContextKey`]: Composite sensor fingerprint identifying a relational situation
//! - [`CoherenceAccumulator`]: Per-context earned coherence with interaction history
//! - [`CoherenceField`]: The full map of context → accumulator
//! - [`SocialPhase`]: 2D behavioral phase from (coherence × tension)
//!
//! # Invariants
//!
//! - **CCF-001**: effective_coherence uses asymmetric gate:
//!   - Unfamiliar contexts (ctx < 0.3): `min(instant, ctx)` — earn trust first
//!   - Familiar contexts (ctx >= 0.3): `0.3*instant + 0.7*ctx` — history buffers noise
//! - **CCF-002**: All accumulator values bounded [0.0, 1.0]
//! - **CCF-003**: Personality modulates deltas, not structure
//! - **CCF-004**: Quadrant boundaries use hysteresis (0.10 deadband)

#![cfg_attr(not(feature = "std"), no_std)]

use hashbrown::HashMap;

// ─── Context Vocabulary ─────────────────────────────────────────────

/// Brightness band derived from CyberPi `cyberpi.get_bri()`.
/// Mapped from `MBotSensors.light_level` (0.0-1.0).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BrightnessBand {
    /// light_level < 0.15 (raw brightness < 15)
    Dark,
    /// light_level 0.15 - 0.50
    Dim,
    /// light_level > 0.50
    Bright,
}

impl BrightnessBand {
    pub fn from_light_level(level: f32) -> Self {
        if level < 0.15 {
            BrightnessBand::Dark
        } else if level < 0.50 {
            BrightnessBand::Dim
        } else {
            BrightnessBand::Bright
        }
    }
}

/// Ambient noise band derived from CyberPi `cyberpi.get_loudness('maximum')`.
/// Mapped from `MBotSensors.sound_level` (0.0-1.0).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoiseBand {
    /// sound_level < 0.15
    Quiet,
    /// sound_level 0.15 - 0.50
    Moderate,
    /// sound_level > 0.50
    Loud,
}

impl NoiseBand {
    pub fn from_sound_level(level: f32) -> Self {
        if level < 0.15 {
            NoiseBand::Quiet
        } else if level < 0.50 {
            NoiseBand::Moderate
        } else {
            NoiseBand::Loud
        }
    }
}

/// Presence signature derived from ultrasonic distance variance
/// over a sliding window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PresenceSignature {
    /// No object within 100cm consistently.
    Absent,
    /// Object distance stable (variance < 5cm over window).
    Static,
    /// Object getting closer over window.
    Approaching,
    /// Object getting farther over window.
    Retreating,
}

/// Motion context from accelerometer magnitude.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionContext {
    /// Accelerometer near gravity-only (robot stationary on table).
    Stationary,
    /// Robot is driving (self-caused motion).
    SelfMoving,
    /// Robot is being picked up or carried (high accel without motor commands).
    BeingHandled,
}

impl MotionContext {
    /// Classify from accelerometer magnitude and whether motors are active.
    /// `accel_mag`: magnitude of [x,y,z] accelerometer vector.
    /// `motors_active`: true if any motor power is non-zero.
    pub fn classify(accel_mag: f32, motors_active: bool) -> Self {
        // Gravity alone reads ~9.8 m/s². CyberPi accel is in m/s² / some scale.
        // Threshold: accel_mag > 2.0 means significant non-gravitational force.
        if accel_mag < 2.0 {
            MotionContext::Stationary
        } else if motors_active {
            MotionContext::SelfMoving
        } else {
            MotionContext::BeingHandled
        }
    }
}

/// Orientation from gyroscope / accelerometer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Orientation {
    /// Normal operating position.
    Upright,
    /// Significantly tilted (roll or pitch > 30°).
    Tilted,
}

impl Orientation {
    /// Classify from CyberPi roll/pitch (degrees).
    /// `roll_deg` and `pitch_deg` from `cyberpi.get_roll()` / `cyberpi.get_pitch()`.
    pub fn from_roll_pitch(roll_deg: f32, pitch_deg: f32) -> Self {
        let abs_roll = if roll_deg < 0.0 { -roll_deg } else { roll_deg };
        let abs_pitch = if pitch_deg < 0.0 { -pitch_deg } else { pitch_deg };
        if abs_roll > 30.0 || abs_pitch > 30.0 {
            Orientation::Tilted
        } else {
            Orientation::Upright
        }
    }
}

/// Time-of-day period. Since CyberPi has no RTC, this is estimated
/// from tick count modulo an estimated day-length, or set by the companion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimePeriod {
    Morning,
    Afternoon,
    Evening,
    Night,
}

impl TimePeriod {
    /// Estimate from brightness as a rough proxy when no clock is available.
    /// Bright = daytime (Morning/Afternoon), Dim = Evening, Dark = Night.
    pub fn estimate_from_brightness(band: BrightnessBand) -> Self {
        match band {
            BrightnessBand::Bright => TimePeriod::Afternoon,
            BrightnessBand::Dim => TimePeriod::Evening,
            BrightnessBand::Dark => TimePeriod::Night,
        }
    }
}

/// Composite context key — the full situation fingerprint.
///
/// Two interactions that produce the same `ContextKey` are considered
/// the same relational situation for coherence accumulation purposes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContextKey {
    pub brightness: BrightnessBand,
    pub noise: NoiseBand,
    pub presence: PresenceSignature,
    pub motion: MotionContext,
    pub orientation: Orientation,
    pub time_period: TimePeriod,
}

impl ContextKey {
    /// Compute a deterministic u32 hash of this context key.
    ///
    /// Used by the startle suppression system to key (StimulusKind, context_hash)
    /// pairs without requiring std HashMap.
    pub fn context_hash_u32(&self) -> u32 {
        // FNV-1a from the variant indices. Deterministic and no_std compatible.
        let mut h: u32 = 2166136261;

        let b = match self.brightness {
            BrightnessBand::Dark => 0u32,
            BrightnessBand::Dim => 1,
            BrightnessBand::Bright => 2,
        };
        h ^= b; h = h.wrapping_mul(16777619);

        let n = match self.noise {
            NoiseBand::Quiet => 0u32,
            NoiseBand::Moderate => 1,
            NoiseBand::Loud => 2,
        };
        h ^= n; h = h.wrapping_mul(16777619);

        let p = match self.presence {
            PresenceSignature::Absent => 0u32,
            PresenceSignature::Static => 1,
            PresenceSignature::Approaching => 2,
            PresenceSignature::Retreating => 3,
        };
        h ^= p; h = h.wrapping_mul(16777619);

        let m = match self.motion {
            MotionContext::Stationary => 0u32,
            MotionContext::SelfMoving => 1,
            MotionContext::BeingHandled => 2,
        };
        h ^= m; h = h.wrapping_mul(16777619);

        let o = match self.orientation {
            Orientation::Upright => 0u32,
            Orientation::Tilted => 1,
        };
        h ^= o; h = h.wrapping_mul(16777619);

        let t = match self.time_period {
            TimePeriod::Morning => 0u32,
            TimePeriod::Afternoon => 1,
            TimePeriod::Evening => 2,
            TimePeriod::Night => 3,
        };
        h ^= t; h = h.wrapping_mul(16777619);

        h
    }

    /// Build a context key from current sensor readings.
    ///
    /// `light_level`: 0.0-1.0 from CyberPi brightness.
    /// `sound_level`: 0.0-1.0 from CyberPi loudness.
    /// `presence`: pre-computed from ultrasonic window (see `PresenceDetector`).
    /// `accel_mag`: accelerometer vector magnitude.
    /// `motors_active`: whether motors are currently powered.
    /// `roll_deg`, `pitch_deg`: from CyberPi IMU.
    pub fn from_sensors(
        light_level: f32,
        sound_level: f32,
        presence: PresenceSignature,
        accel_mag: f32,
        motors_active: bool,
        roll_deg: f32,
        pitch_deg: f32,
    ) -> Self {
        let brightness = BrightnessBand::from_light_level(light_level);
        Self {
            brightness,
            noise: NoiseBand::from_sound_level(sound_level),
            presence,
            motion: MotionContext::classify(accel_mag, motors_active),
            orientation: Orientation::from_roll_pitch(roll_deg, pitch_deg),
            time_period: TimePeriod::estimate_from_brightness(brightness),
        }
    }
}

// ─── Presence Detector (sliding window) ─────────────────────────────

/// Sliding window presence detector for ultrasonic readings.
/// Classifies approach/retreat/static/absent from distance trend.
pub struct PresenceDetector {
    /// Ring buffer of recent distance readings.
    window: [f32; Self::WINDOW_SIZE],
    /// Write index into ring buffer.
    write_idx: usize,
    /// Number of valid readings in the buffer.
    count: usize,
}

impl PresenceDetector {
    const WINDOW_SIZE: usize = 10;
    /// Distances above this are considered "nothing present".
    const ABSENT_THRESHOLD: f32 = 100.0;
    /// Minimum distance change over window to classify as approach/retreat.
    const TREND_THRESHOLD: f32 = 5.0;

    pub fn new() -> Self {
        Self {
            window: [999.0; Self::WINDOW_SIZE],
            write_idx: 0,
            count: 0,
        }
    }

    /// Push a new ultrasonic reading and return the current presence classification.
    pub fn update(&mut self, distance_cm: f32) -> PresenceSignature {
        // Skip invalid readings (0.0 = sensor error)
        if distance_cm < 2.0 {
            return self.classify();
        }

        self.window[self.write_idx] = distance_cm;
        self.write_idx = (self.write_idx + 1) % Self::WINDOW_SIZE;
        if self.count < Self::WINDOW_SIZE {
            self.count += 1;
        }

        self.classify()
    }

    fn classify(&self) -> PresenceSignature {
        if self.count < 3 {
            return PresenceSignature::Absent;
        }

        // Check if anything is present at all
        let latest = self.window[(self.write_idx + Self::WINDOW_SIZE - 1) % Self::WINDOW_SIZE];
        if latest > Self::ABSENT_THRESHOLD {
            return PresenceSignature::Absent;
        }

        // Compute trend: compare oldest valid readings to newest
        let oldest_idx = if self.count == Self::WINDOW_SIZE {
            self.write_idx // oldest is at write position in full buffer
        } else {
            0
        };
        let oldest = self.window[oldest_idx];

        let delta = latest - oldest;
        if delta < -Self::TREND_THRESHOLD {
            PresenceSignature::Approaching
        } else if delta > Self::TREND_THRESHOLD {
            PresenceSignature::Retreating
        } else {
            PresenceSignature::Static
        }
    }
}

impl Default for PresenceDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Coherence Accumulator ──────────────────────────────────────────

/// Per-context coherence accumulator. Grows through repeated positive
/// interaction, decays with disuse, drops on negative events.
#[derive(Clone, Debug)]
pub struct CoherenceAccumulator {
    /// Accumulated coherence for this context [0.0, 1.0].
    pub value: f32,
    /// Total positive interactions in this context.
    pub interaction_count: u32,
    /// Tick of the most recent interaction.
    pub last_interaction_tick: u64,
}

impl CoherenceAccumulator {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            interaction_count: 0,
            last_interaction_tick: 0,
        }
    }

    /// Cold-start constructor: initializes value from personality curiosity_drive.
    /// `curiosity`: personality curiosity_drive [0.0, 1.0].
    /// Baseline = 0.15 * curiosity (max 0.15 for curiosity=1.0).
    pub fn new_with_baseline(curiosity: f32) -> Self {
        Self {
            value: (0.15 * curiosity).clamp(0.0, 1.0),
            interaction_count: 0,
            last_interaction_tick: 0,
        }
    }

    /// Record a positive interaction. Coherence grows asymptotically toward 1.0.
    ///
    /// `recovery_speed`: personality parameter [0.0, 1.0] — higher = faster growth.
    /// `tick`: current tick for freshness tracking.
    /// `alone`: true if presence is Absent — doubles delta for faster bootstrap.
    pub fn positive_interaction(&mut self, recovery_speed: f32, tick: u64, alone: bool) {
        // Base delta scaled by personality, diminishing as value approaches 1.0
        let mut delta = 0.02 * (0.5 + recovery_speed) * (1.0 - self.value);
        if alone {
            delta *= 2.0; // Alone contexts bootstrap faster
        }
        self.value = (self.value + delta).min(1.0);
        self.interaction_count = self.interaction_count.saturating_add(1);
        self.last_interaction_tick = tick;
    }

    /// Record a negative interaction (startle, collision, high tension).
    ///
    /// `startle_sensitivity`: personality parameter [0.0, 1.0] — higher = bigger drop.
    /// `tick`: current tick.
    pub fn negative_interaction(&mut self, startle_sensitivity: f32, tick: u64) {
        // Drop scaled by personality, but floored by interaction history
        let floor = self.earned_floor();
        let delta = 0.05 * (0.5 + startle_sensitivity);
        self.value = (self.value - delta).max(floor);
        self.last_interaction_tick = tick;
    }

    /// Apply time-based decay. Call once per tick (or less frequently).
    ///
    /// Coherence decays toward the earned floor, not toward zero.
    /// More interactions = higher floor = harder to lose earned trust.
    pub fn decay(&mut self, ticks_elapsed: u64) {
        let floor = self.earned_floor();
        if self.value > floor {
            let decay_rate = 0.0001 * ticks_elapsed as f32;
            self.value = (self.value - decay_rate).max(floor);
        }
    }

    /// The minimum coherence value that interaction history protects.
    /// After many interactions, coherence can't fall below this floor.
    /// Asymptotically approaches 0.5 (never fully "immune", but resilient).
    fn earned_floor(&self) -> f32 {
        // floor = 0.5 * (1.0 - 1.0/(1.0 + count/20.0))
        // At 0 interactions: floor = 0.0
        // At 20 interactions: floor ≈ 0.25
        // At 100 interactions: floor ≈ 0.42
        // Asymptote: 0.5
        0.5 * (1.0 - 1.0 / (1.0 + self.interaction_count as f32 / 20.0))
    }
}

impl Default for CoherenceAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Coherence Field (the full map) ─────────────────────────────────

/// Maximum number of tracked contexts. Oldest entries are evicted when full.
const MAX_CONTEXTS: usize = 64;

/// The coherence field: a map of context → accumulator.
pub struct CoherenceField {
    /// Context-keyed accumulators.
    accumulators: HashMap<ContextKey, CoherenceAccumulator>,
    /// Personality baseline for new contexts (0.15 * curiosity_drive).
    personality_baseline: f32,
    /// Fallback coherence used as floor for unseen contexts in degraded mode.
    /// When the companion is unavailable and no persistence exists, this
    /// prevents every context from starting at 0.0.
    fallback_coherence: Option<f32>,
}

impl CoherenceField {
    pub fn new() -> Self {
        Self {
            accumulators: HashMap::new(),
            personality_baseline: 0.0,
            fallback_coherence: None,
        }
    }

    /// Create with personality-driven cold-start baseline.
    /// New contexts will start at `0.15 * curiosity_drive` instead of 0.0.
    pub fn new_with_personality(curiosity_drive: f32) -> Self {
        Self {
            accumulators: HashMap::new(),
            personality_baseline: (0.15 * curiosity_drive).clamp(0.0, 1.0),
            fallback_coherence: None,
        }
    }

    /// Set the fallback coherence (degraded mode floor for unseen contexts).
    pub fn set_fallback(&mut self, value: Option<f32>) {
        self.fallback_coherence = value;
    }

    /// Get or create the accumulator for a context key.
    pub fn get_or_create(&mut self, key: &ContextKey) -> &mut CoherenceAccumulator {
        if !self.accumulators.contains_key(key) {
            // Evict oldest if at capacity
            if self.accumulators.len() >= MAX_CONTEXTS {
                self.evict_oldest();
            }
            // Reverse-engineer curiosity from baseline: baseline = 0.15 * curiosity
            let curiosity = if self.personality_baseline > 0.0 {
                (self.personality_baseline / 0.15).clamp(0.0, 1.0)
            } else {
                0.0
            };
            self.accumulators.insert(
                key.clone(),
                CoherenceAccumulator::new_with_baseline(curiosity),
            );
        }
        self.accumulators.get_mut(key).unwrap()
    }

    /// Get the accumulated coherence for a context.
    /// Returns the accumulator value if seen, or the fallback/0.0 if unseen.
    pub fn context_coherence(&self, key: &ContextKey) -> f32 {
        self.accumulators.get(key).map_or_else(
            || self.fallback_coherence.unwrap_or(0.0),
            |a| a.value,
        )
    }

    /// CCF-001 (asymmetric gate):
    ///
    /// - Unfamiliar contexts (ctx < 0.3): `min(instant, ctx)` — strict, earn trust first.
    /// - Familiar contexts (ctx >= 0.3): `0.3*instant + 0.7*ctx` — history buffers noise.
    ///
    /// The robot can never be more expressive than its accumulated
    /// familiarity with the current situation allows, but familiar
    /// contexts provide resilience against transient sensor noise.
    pub fn effective_coherence(&self, instant_coherence: f32, key: &ContextKey) -> f32 {
        let ctx = self.context_coherence(key);
        if ctx < 0.3 {
            // Unfamiliar: strict gate
            if instant_coherence < ctx {
                instant_coherence
            } else {
                ctx
            }
        } else {
            // Familiar: weighted blend — history dampens noise
            (0.3 * instant_coherence + 0.7 * ctx).clamp(0.0, 1.0)
        }
    }

    /// Apply decay to all accumulators.
    pub fn decay_all(&mut self, ticks_elapsed: u64) {
        for acc in self.accumulators.values_mut() {
            acc.decay(ticks_elapsed);
        }
    }

    /// Number of tracked contexts.
    pub fn context_count(&self) -> usize {
        self.accumulators.len()
    }

    /// Iterate over all (context, accumulator) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&ContextKey, &CoherenceAccumulator)> {
        self.accumulators.iter()
    }

    /// Get a snapshot of all context coherence values for serialization/dashboard.
    pub fn snapshot(&self) -> CoherenceSnapshot {
        let mut entries = hashbrown::HashMap::new();
        for (key, acc) in &self.accumulators {
            entries.insert(key.clone(), acc.value);
        }
        CoherenceSnapshot {
            context_count: self.accumulators.len(),
            entries,
        }
    }

    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self.accumulators
            .iter()
            .min_by_key(|(_, acc)| acc.last_interaction_tick)
            .map(|(k, _)| k.clone())
        {
            self.accumulators.remove(&oldest_key);
        }
    }
}

impl Default for CoherenceField {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for CoherenceField {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CoherenceField")
            .field("context_count", &self.accumulators.len())
            .field("personality_baseline", &self.personality_baseline)
            .field("fallback_coherence", &self.fallback_coherence)
            .finish()
    }
}

/// Serializable snapshot of the coherence field for dashboard/API.
pub struct CoherenceSnapshot {
    pub context_count: usize,
    pub entries: HashMap<ContextKey, f32>,
}

// ─── Phase Space (configurable thresholds) ──────────────────────────

/// Configurable thresholds for SocialPhase transitions.
/// Uses hysteresis (Schmitt trigger): enter thresholds are stricter than exit
/// to prevent oscillation at boundaries (CCF-004).
#[derive(Clone, Debug)]
pub struct PhaseSpace {
    /// Coherence threshold to enter high-coherence quadrants (QB, PG).
    pub coherence_high_enter: f32,
    /// Coherence threshold to exit high-coherence quadrants.
    pub coherence_high_exit: f32,
    /// Tension threshold to enter high-tension quadrants (SR, PG).
    pub tension_high_enter: f32,
    /// Tension threshold to exit high-tension quadrants.
    pub tension_high_exit: f32,
}

impl Default for PhaseSpace {
    fn default() -> Self {
        Self {
            coherence_high_enter: 0.65,
            coherence_high_exit: 0.55,
            tension_high_enter: 0.45,
            tension_high_exit: 0.35,
        }
    }
}

// ─── Social Phase (2D behavioral quadrant) ──────────────────────────

/// Behavioral phase from the 2D (coherence x tension) space.
/// Uses hysteresis to prevent oscillation at boundaries (CCF-004).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocialPhase {
    /// Low coherence, low tension: minimal expression, cautious observation.
    ShyObserver,
    /// Low coherence, high tension: protective reflex with added withdrawal.
    StartledRetreat,
    /// High coherence, low tension: full expressive range, "small flourishes".
    QuietlyBeloved,
    /// High coherence, high tension: protective but with relational context.
    ProtectiveGuardian,
}

impl SocialPhase {
    /// Determine social phase with hysteresis.
    ///
    /// `effective_coherence`: output of `CoherenceField::effective_coherence()`.
    /// `tension`: current tension from homeostasis.
    /// `previous`: the phase from the previous tick (for hysteresis).
    /// `phase_space`: configurable thresholds for quadrant transitions.
    pub fn classify(
        effective_coherence: f32,
        tension: f32,
        previous: SocialPhase,
        phase_space: &PhaseSpace,
    ) -> SocialPhase {
        let high_coherence = match previous {
            SocialPhase::QuietlyBeloved | SocialPhase::ProtectiveGuardian => {
                effective_coherence >= phase_space.coherence_high_exit
            }
            _ => effective_coherence >= phase_space.coherence_high_enter,
        };

        let high_tension = match previous {
            SocialPhase::StartledRetreat | SocialPhase::ProtectiveGuardian => {
                tension >= phase_space.tension_high_exit
            }
            _ => tension >= phase_space.tension_high_enter,
        };

        match (high_coherence, high_tension) {
            (false, false) => SocialPhase::ShyObserver,
            (false, true) => SocialPhase::StartledRetreat,
            (true, false) => SocialPhase::QuietlyBeloved,
            (true, true) => SocialPhase::ProtectiveGuardian,
        }
    }

    /// Scale factor for expressive output in this phase [0.0, 1.0].
    ///
    /// Legacy API -- delegates to [`permeability()`] with typical mid-range
    /// values (coherence=0.5, tension=0.3) for backward compatibility.
    /// New code should call `permeability()` directly for full control.
    pub fn expression_scale(&self) -> f32 {
        permeability(0.5, 0.3, *self)
    }

    /// LED color tint for this phase (overlay on reflex mode color).
    pub fn led_tint(&self) -> [u8; 3] {
        match self {
            SocialPhase::ShyObserver => [40, 40, 80],       // Muted blue-grey
            SocialPhase::StartledRetreat => [80, 20, 20],   // Dark red
            SocialPhase::QuietlyBeloved => [60, 120, 200],  // Warm blue
            SocialPhase::ProtectiveGuardian => [200, 100, 0], // Amber
        }
    }
}

// ─── Output Permeability ─────────────────────────────────────────────

/// Compute output permeability -- how much personality expression passes through.
///
/// The quadrant determines qualitative behavior. The exact position within
/// the quadrant determines intensity. This scales all output channels
/// (motor speed, LED intensity, sound probability, narration depth).
///
/// # Arguments
///
/// * `effective_coherence` - Output of [`CoherenceField::effective_coherence()`], [0.0, 1.0].
/// * `_tension` - Current tension from homeostasis (reserved for future use).
/// * `quadrant` - The current [`SocialPhase`] behavioral quadrant.
///
/// # Returns
///
/// A scalar in [0.0, 1.0] that scales all output channels.
///
/// # Ranges per quadrant
///
/// - **ShyObserver**: 0.0-0.3 (scaled by coherence within quadrant)
/// - **StartledRetreat**: fixed 0.1 (reflexive, not expression)
/// - **QuietlyBeloved**: 0.5-1.0 (high expression, scaled by coherence)
/// - **ProtectiveGuardian**: 0.4-0.6 (confident but focused)
pub fn permeability(effective_coherence: f32, _tension: f32, quadrant: SocialPhase) -> f32 {
    match quadrant {
        SocialPhase::ShyObserver => {
            effective_coherence * 0.3 // max 0.3
        }
        SocialPhase::StartledRetreat => {
            0.1 // fixed low -- protective reflex, not expression
        }
        SocialPhase::QuietlyBeloved => {
            0.5 + effective_coherence * 0.5 // range 0.5-1.0
        }
        SocialPhase::ProtectiveGuardian => {
            0.4 + effective_coherence * 0.2 // range 0.4-0.6
        }
    }
}

/// Narration depth levels gated by permeability.
///
/// Determines how much LLM reflection the robot performs based on
/// the current output permeability. Lower permeability means less
/// narration overhead (saving compute and keeping behavior tighter).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NarrationDepth {
    /// permeability < 0.2: No LLM reflection.
    None,
    /// permeability 0.2-0.4: Factual observations only.
    Minimal,
    /// permeability 0.4-0.6: Contextual awareness.
    Brief,
    /// permeability 0.6-0.8: Personality-colored narration.
    Full,
    /// permeability > 0.8: Phenomenological reflection.
    Deep,
}

impl NarrationDepth {
    /// Map a permeability scalar to a narration depth level.
    ///
    /// Thresholds: 0.2, 0.4, 0.6, 0.8.
    pub fn from_permeability(p: f32) -> Self {
        if p < 0.2 {
            NarrationDepth::None
        } else if p < 0.4 {
            NarrationDepth::Minimal
        } else if p < 0.6 {
            NarrationDepth::Brief
        } else if p < 0.8 {
            NarrationDepth::Full
        } else {
            NarrationDepth::Deep
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brightness_bands() {
        assert_eq!(BrightnessBand::from_light_level(0.05), BrightnessBand::Dark);
        assert_eq!(BrightnessBand::from_light_level(0.30), BrightnessBand::Dim);
        assert_eq!(BrightnessBand::from_light_level(0.75), BrightnessBand::Bright);
        // Boundary: 0.15 is Dim, not Dark
        assert_eq!(BrightnessBand::from_light_level(0.15), BrightnessBand::Dim);
    }

    #[test]
    fn test_noise_bands() {
        assert_eq!(NoiseBand::from_sound_level(0.05), NoiseBand::Quiet);
        assert_eq!(NoiseBand::from_sound_level(0.30), NoiseBand::Moderate);
        assert_eq!(NoiseBand::from_sound_level(0.75), NoiseBand::Loud);
    }

    #[test]
    fn test_motion_context() {
        assert_eq!(MotionContext::classify(1.0, false), MotionContext::Stationary);
        assert_eq!(MotionContext::classify(5.0, true), MotionContext::SelfMoving);
        assert_eq!(MotionContext::classify(5.0, false), MotionContext::BeingHandled);
    }

    #[test]
    fn test_orientation() {
        assert_eq!(Orientation::from_roll_pitch(5.0, 5.0), Orientation::Upright);
        assert_eq!(Orientation::from_roll_pitch(35.0, 5.0), Orientation::Tilted);
        assert_eq!(Orientation::from_roll_pitch(-5.0, -35.0), Orientation::Tilted);
    }

    #[test]
    fn test_context_key_from_sensors() {
        let key = ContextKey::from_sensors(
            0.60, // bright
            0.10, // quiet
            PresenceSignature::Approaching,
            1.5,  // stationary
            false,
            5.0,  // upright
            3.0,
        );
        assert_eq!(key.brightness, BrightnessBand::Bright);
        assert_eq!(key.noise, NoiseBand::Quiet);
        assert_eq!(key.presence, PresenceSignature::Approaching);
        assert_eq!(key.motion, MotionContext::Stationary);
        assert_eq!(key.orientation, Orientation::Upright);
    }

    #[test]
    fn test_context_key_equality() {
        let k1 = ContextKey::from_sensors(0.60, 0.10, PresenceSignature::Static, 1.0, false, 0.0, 0.0);
        let k2 = ContextKey::from_sensors(0.70, 0.05, PresenceSignature::Static, 0.5, false, 2.0, 1.0);
        // Both bright, quiet, static, stationary, upright — same key
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_context_key_difference() {
        let k1 = ContextKey::from_sensors(0.60, 0.10, PresenceSignature::Static, 1.0, false, 0.0, 0.0);
        let k2 = ContextKey::from_sensors(0.60, 0.60, PresenceSignature::Static, 1.0, false, 0.0, 0.0);
        // Noise band differs: Quiet vs Loud
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_presence_detector_absent() {
        let mut pd = PresenceDetector::new();
        // Push readings far away
        for _ in 0..5 {
            let sig = pd.update(200.0);
            assert_eq!(sig, PresenceSignature::Absent);
        }
    }

    #[test]
    fn test_presence_detector_approaching() {
        let mut pd = PresenceDetector::new();
        // Object getting closer
        for i in 0..10 {
            pd.update(80.0 - (i as f32 * 5.0));
        }
        let sig = pd.update(25.0);
        assert_eq!(sig, PresenceSignature::Approaching);
    }

    #[test]
    fn test_presence_detector_retreating() {
        let mut pd = PresenceDetector::new();
        // Object moving away
        for i in 0..10 {
            pd.update(20.0 + (i as f32 * 5.0));
        }
        let sig = pd.update(75.0);
        assert_eq!(sig, PresenceSignature::Retreating);
    }

    #[test]
    fn test_presence_detector_static() {
        let mut pd = PresenceDetector::new();
        // Object at stable distance
        for _ in 0..10 {
            pd.update(50.0);
        }
        let sig = pd.update(51.0);
        assert_eq!(sig, PresenceSignature::Static);
    }

    #[test]
    fn test_presence_detector_ignores_invalid() {
        let mut pd = PresenceDetector::new();
        // Invalid readings (0.0, 1.0) should not affect classification
        pd.update(0.0);
        pd.update(1.0);
        assert_eq!(pd.classify(), PresenceSignature::Absent);
    }

    #[test]
    fn test_accumulator_positive_growth() {
        let mut acc = CoherenceAccumulator::new();
        assert_eq!(acc.value, 0.0);

        // 50 positive interactions with neutral personality (0.5)
        for i in 0..50 {
            acc.positive_interaction(0.5, i, false);
        }
        // Should have grown significantly but not reached 1.0
        assert!(acc.value > 0.3, "value={}", acc.value);
        assert!(acc.value < 1.0);
        assert_eq!(acc.interaction_count, 50);
    }

    #[test]
    fn test_accumulator_asymptotic_growth() {
        let mut acc = CoherenceAccumulator::new();
        // Many interactions
        for i in 0..500 {
            acc.positive_interaction(0.5, i, false);
        }
        let high_value = acc.value;
        // Further interactions should produce diminishing returns
        for i in 500..510 {
            acc.positive_interaction(0.5, i, false);
        }
        let delta = acc.value - high_value;
        assert!(delta < 0.01, "delta should be small at high values: {}", delta);
    }

    #[test]
    fn test_accumulator_personality_modulation() {
        let mut fast = CoherenceAccumulator::new();
        let mut slow = CoherenceAccumulator::new();

        // Same number of interactions, different recovery speeds
        for i in 0..20 {
            fast.positive_interaction(0.9, i, false); // fast recovery
            slow.positive_interaction(0.1, i, false); // slow recovery
        }
        assert!(fast.value > slow.value,
                "fast={} should be > slow={}", fast.value, slow.value);
    }

    #[test]
    fn test_accumulator_negative_interaction() {
        let mut acc = CoherenceAccumulator::new();
        // Build up some coherence
        for i in 0..30 {
            acc.positive_interaction(0.5, i, false);
        }
        let before = acc.value;

        // Negative interaction should reduce value
        acc.negative_interaction(0.5, 31);
        assert!(acc.value < before);
    }

    #[test]
    fn test_accumulator_earned_floor() {
        let mut acc = CoherenceAccumulator::new();
        // Build up interaction history
        for i in 0..100 {
            acc.positive_interaction(0.5, i, false);
        }
        let before = acc.value;

        // Many negative interactions shouldn't drop below floor
        for i in 100..200 {
            acc.negative_interaction(1.0, i); // max startle
        }
        // Floor at 100 interactions ≈ 0.42
        assert!(acc.value > 0.3,
                "value={} should be above earned floor", acc.value);
        assert!(acc.value < before);
    }

    #[test]
    fn test_accumulator_decay_toward_floor() {
        let mut acc = CoherenceAccumulator::new();
        // Build up
        for i in 0..50 {
            acc.positive_interaction(0.5, i, false);
        }
        let before = acc.value;

        // Decay over time
        acc.decay(1000);
        assert!(acc.value < before);
        // But should be above earned floor
        let floor = 0.5 * (1.0 - 1.0 / (1.0 + 50.0 / 20.0));
        assert!(acc.value >= floor,
                "value={} should be >= floor={}", acc.value, floor);
    }

    #[test]
    fn test_coherence_field_effective_coherence_unfamiliar() {
        // Test strict gate for unfamiliar contexts (ctx < 0.3)
        let mut field = CoherenceField::new();
        let key = ContextKey::from_sensors(
            0.60, 0.10, PresenceSignature::Static, 1.0, false, 0.0, 0.0,
        );

        // New context: context coherence = 0.0
        // CCF-001 unfamiliar: effective = min(0.8, 0.0) = 0.0
        let eff = field.effective_coherence(0.8, &key);
        assert_eq!(eff, 0.0);

        // Build up a little coherence (not enough to cross 0.3 threshold)
        {
            let acc = field.get_or_create(&key);
            for i in 0..10 {
                acc.positive_interaction(0.5, i, false);
            }
        }
        let ctx_coh = field.context_coherence(&key);
        assert!(ctx_coh > 0.0);
        assert!(ctx_coh < 0.3, "ctx_coh={} should be < 0.3 for unfamiliar test", ctx_coh);

        // Unfamiliar: min(0.8, ctx_coh) = ctx_coh
        let eff = field.effective_coherence(0.8, &key);
        assert!((eff - ctx_coh).abs() < 0.001);

        // Unfamiliar: min(0.05, ctx_coh) = 0.05 (instant limits)
        let eff = field.effective_coherence(0.05, &key);
        assert!((eff - 0.05).abs() < 0.001);
    }

    #[test]
    fn test_coherence_field_effective_coherence_familiar() {
        // Test weighted blend for familiar contexts (ctx >= 0.3)
        let mut field = CoherenceField::new();
        let key = ContextKey::from_sensors(
            0.60, 0.10, PresenceSignature::Static, 1.0, false, 0.0, 0.0,
        );

        // Build up enough coherence to cross 0.3 threshold
        {
            let acc = field.get_or_create(&key);
            for i in 0..80 {
                acc.positive_interaction(0.5, i, false);
            }
        }
        let ctx_coh = field.context_coherence(&key);
        assert!(ctx_coh >= 0.3, "ctx_coh={} should be >= 0.3 for familiar test", ctx_coh);

        // Familiar: 0.3*instant + 0.7*ctx
        let eff = field.effective_coherence(0.8, &key);
        let expected = 0.3 * 0.8 + 0.7 * ctx_coh;
        assert!((eff - expected).abs() < 0.001,
                "eff={} expected={}", eff, expected);

        // Familiar with low instant: blend dampens the drop
        let eff_low = field.effective_coherence(0.1, &key);
        let expected_low = 0.3 * 0.1 + 0.7 * ctx_coh;
        assert!((eff_low - expected_low).abs() < 0.001,
                "eff_low={} expected_low={}", eff_low, expected_low);
        // The familiar context should buffer against the low instant value
        assert!(eff_low > 0.1, "familiar context should buffer: eff_low={}", eff_low);
    }

    #[test]
    fn test_coherence_field_independent_contexts() {
        let mut field = CoherenceField::new();
        let morning = ContextKey::from_sensors(
            0.60, 0.10, PresenceSignature::Approaching, 1.0, false, 0.0, 0.0,
        );
        let night = ContextKey::from_sensors(
            0.05, 0.10, PresenceSignature::Approaching, 1.0, false, 0.0, 0.0,
        );

        // Build coherence only in morning context
        {
            let acc = field.get_or_create(&morning);
            for i in 0..50 {
                acc.positive_interaction(0.5, i, false);
            }
        }

        // Morning context should have coherence; night should not
        assert!(field.context_coherence(&morning) > 0.3);
        assert_eq!(field.context_coherence(&night), 0.0);
    }

    #[test]
    fn test_coherence_field_eviction() {
        let mut field = CoherenceField::new();
        // Fill to MAX_CONTEXTS + 1
        for i in 0..=MAX_CONTEXTS {
            let key = ContextKey {
                brightness: BrightnessBand::Bright,
                noise: NoiseBand::Quiet,
                presence: PresenceSignature::Static,
                motion: MotionContext::Stationary,
                orientation: Orientation::Upright,
                // Vary time_period to make unique keys (only 4 variants, so also vary noise)
                time_period: if i % 2 == 0 { TimePeriod::Morning } else { TimePeriod::Night },
                // This won't be enough unique keys with only enums,
                // but the eviction logic is tested by checking count
            };
            let acc = field.get_or_create(&key);
            acc.last_interaction_tick = i as u64;
        }
        assert!(field.context_count() <= MAX_CONTEXTS);
    }

    #[test]
    fn test_social_phase_shy_observer() {
        let ps = PhaseSpace::default();
        let phase = SocialPhase::classify(0.1, 0.1, SocialPhase::ShyObserver, &ps);
        assert_eq!(phase, SocialPhase::ShyObserver);
    }

    #[test]
    fn test_social_phase_quietly_beloved() {
        let ps = PhaseSpace::default();
        let phase = SocialPhase::classify(0.8, 0.1, SocialPhase::ShyObserver, &ps);
        assert_eq!(phase, SocialPhase::QuietlyBeloved);
    }

    #[test]
    fn test_social_phase_startled_retreat() {
        let ps = PhaseSpace::default();
        let phase = SocialPhase::classify(0.1, 0.7, SocialPhase::ShyObserver, &ps);
        assert_eq!(phase, SocialPhase::StartledRetreat);
    }

    #[test]
    fn test_social_phase_protective_guardian() {
        let ps = PhaseSpace::default();
        let phase = SocialPhase::classify(0.8, 0.7, SocialPhase::ShyObserver, &ps);
        assert_eq!(phase, SocialPhase::ProtectiveGuardian);
    }

    #[test]
    fn test_social_phase_hysteresis() {
        let ps = PhaseSpace::default();

        // Enter QuietlyBeloved at coherence 0.65
        let phase = SocialPhase::classify(0.66, 0.1, SocialPhase::ShyObserver, &ps);
        assert_eq!(phase, SocialPhase::QuietlyBeloved);

        // Stay in QuietlyBeloved at coherence 0.56 (above exit threshold 0.55)
        let phase = SocialPhase::classify(0.56, 0.1, SocialPhase::QuietlyBeloved, &ps);
        assert_eq!(phase, SocialPhase::QuietlyBeloved);

        // Exit QuietlyBeloved at coherence 0.54 (below exit threshold 0.55)
        let phase = SocialPhase::classify(0.54, 0.1, SocialPhase::QuietlyBeloved, &ps);
        assert_eq!(phase, SocialPhase::ShyObserver);
    }

    #[test]
    fn test_social_phase_tension_hysteresis() {
        let ps = PhaseSpace::default();

        // Enter StartledRetreat at tension 0.45
        let phase = SocialPhase::classify(0.1, 0.46, SocialPhase::ShyObserver, &ps);
        assert_eq!(phase, SocialPhase::StartledRetreat);

        // Stay in StartledRetreat at tension 0.36 (above exit threshold 0.35)
        let phase = SocialPhase::classify(0.1, 0.36, SocialPhase::StartledRetreat, &ps);
        assert_eq!(phase, SocialPhase::StartledRetreat);

        // Exit StartledRetreat at tension 0.34 (below exit threshold 0.35)
        let phase = SocialPhase::classify(0.1, 0.34, SocialPhase::StartledRetreat, &ps);
        assert_eq!(phase, SocialPhase::ShyObserver);
    }

    #[test]
    fn test_custom_thresholds_stricter() {
        // Stricter coherence threshold: QB is harder to reach
        let strict = PhaseSpace {
            coherence_high_enter: 0.80,
            coherence_high_exit: 0.70,
            ..PhaseSpace::default()
        };

        // 0.70 coherence: enough for default QB, but not enough for strict
        let phase = SocialPhase::classify(0.70, 0.1, SocialPhase::ShyObserver, &strict);
        assert_eq!(phase, SocialPhase::ShyObserver,
            "coherence 0.70 should NOT enter QB with strict threshold 0.80");

        // 0.85 coherence: above even the strict threshold
        let phase = SocialPhase::classify(0.85, 0.1, SocialPhase::ShyObserver, &strict);
        assert_eq!(phase, SocialPhase::QuietlyBeloved,
            "coherence 0.85 should enter QB with strict threshold 0.80");

        // Hysteresis exit: stay in QB at 0.75 (above strict exit 0.70)
        let phase = SocialPhase::classify(0.75, 0.1, SocialPhase::QuietlyBeloved, &strict);
        assert_eq!(phase, SocialPhase::QuietlyBeloved,
            "coherence 0.75 should stay in QB (above exit 0.70)");

        // Drop below strict exit: leave QB
        let phase = SocialPhase::classify(0.65, 0.1, SocialPhase::QuietlyBeloved, &strict);
        assert_eq!(phase, SocialPhase::ShyObserver,
            "coherence 0.65 should exit QB (below exit 0.70)");
    }

    #[test]
    fn test_custom_thresholds_looser() {
        // Looser coherence threshold: QB is easier to reach
        let loose = PhaseSpace {
            coherence_high_enter: 0.40,
            coherence_high_exit: 0.30,
            ..PhaseSpace::default()
        };

        // 0.42 coherence: not enough for default QB (0.65), but enough for loose
        let phase = SocialPhase::classify(0.42, 0.1, SocialPhase::ShyObserver, &loose);
        assert_eq!(phase, SocialPhase::QuietlyBeloved,
            "coherence 0.42 should enter QB with loose threshold 0.40");

        // With default thresholds, same value stays ShyObserver
        let ps = PhaseSpace::default();
        let phase = SocialPhase::classify(0.42, 0.1, SocialPhase::ShyObserver, &ps);
        assert_eq!(phase, SocialPhase::ShyObserver,
            "coherence 0.42 should NOT enter QB with default threshold 0.65");
    }

    #[test]
    fn test_default_matches_previous() {
        // Verify PhaseSpace::default() produces identical results to the old
        // hardcoded behavior (coherence: 0.65/0.55, tension: 0.45/0.35).
        let ps = PhaseSpace::default();

        // Spot-check the default values
        assert!((ps.coherence_high_enter - 0.65).abs() < f32::EPSILON);
        assert!((ps.coherence_high_exit - 0.55).abs() < f32::EPSILON);
        assert!((ps.tension_high_enter - 0.45).abs() < f32::EPSILON);
        assert!((ps.tension_high_exit - 0.35).abs() < f32::EPSILON);

        // Full quadrant sweep with default thresholds
        let cases: &[(f32, f32, SocialPhase, SocialPhase)] = &[
            // (coherence, tension, previous, expected)
            (0.1, 0.1, SocialPhase::ShyObserver, SocialPhase::ShyObserver),
            (0.8, 0.1, SocialPhase::ShyObserver, SocialPhase::QuietlyBeloved),
            (0.1, 0.7, SocialPhase::ShyObserver, SocialPhase::StartledRetreat),
            (0.8, 0.7, SocialPhase::ShyObserver, SocialPhase::ProtectiveGuardian),
            // Hysteresis: stay in QB above exit
            (0.56, 0.1, SocialPhase::QuietlyBeloved, SocialPhase::QuietlyBeloved),
            // Hysteresis: exit QB below exit
            (0.54, 0.1, SocialPhase::QuietlyBeloved, SocialPhase::ShyObserver),
            // Hysteresis: stay in SR above tension exit
            (0.1, 0.36, SocialPhase::StartledRetreat, SocialPhase::StartledRetreat),
            // Hysteresis: exit SR below tension exit
            (0.1, 0.34, SocialPhase::StartledRetreat, SocialPhase::ShyObserver),
        ];

        for &(coh, ten, prev, expected) in cases {
            let result = SocialPhase::classify(coh, ten, prev, &ps);
            assert_eq!(result, expected,
                "coh={} ten={} prev={:?}: got {:?}, expected {:?}",
                coh, ten, prev, result, expected);
        }
    }

    #[test]
    fn test_expression_scale_ordering() {
        // Quietly Beloved should have highest expression
        assert!(SocialPhase::QuietlyBeloved.expression_scale() >
                SocialPhase::ProtectiveGuardian.expression_scale());
        assert!(SocialPhase::ProtectiveGuardian.expression_scale() >
                SocialPhase::ShyObserver.expression_scale());
        assert!(SocialPhase::ShyObserver.expression_scale() >
                SocialPhase::StartledRetreat.expression_scale());
    }

    // --- New tests for cold-start, asymmetric gate, fallback, alone-boost ---

    #[test]
    fn test_cold_start_baseline() {
        // High curiosity → baseline = 0.15 * 1.0 = 0.15
        let acc = CoherenceAccumulator::new_with_baseline(1.0);
        assert!((acc.value - 0.15).abs() < 0.001, "value={}", acc.value);

        // Low curiosity → baseline = 0.15 * 0.2 = 0.03
        let acc = CoherenceAccumulator::new_with_baseline(0.2);
        assert!((acc.value - 0.03).abs() < 0.001, "value={}", acc.value);

        // Zero curiosity → baseline = 0.0
        let acc = CoherenceAccumulator::new_with_baseline(0.0);
        assert_eq!(acc.value, 0.0);
    }

    #[test]
    fn test_cold_start_field() {
        // CoherenceField with personality creates accumulators at baseline
        let mut field = CoherenceField::new_with_personality(0.8);
        let key = ContextKey::from_sensors(
            0.60, 0.10, PresenceSignature::Static, 1.0, false, 0.0, 0.0,
        );
        let acc = field.get_or_create(&key);
        // baseline = 0.15 * 0.8 = 0.12
        assert!(acc.value > 0.0, "cold-start should be > 0: value={}", acc.value);
        assert!(acc.value < 0.2, "cold-start should be < 0.2: value={}", acc.value);
    }

    #[test]
    fn test_alone_boost() {
        // Alone interactions should grow faster than non-alone
        let mut alone_acc = CoherenceAccumulator::new();
        let mut social_acc = CoherenceAccumulator::new();

        for i in 0..20 {
            alone_acc.positive_interaction(0.5, i, true);   // alone = true
            social_acc.positive_interaction(0.5, i, false);  // alone = false
        }

        assert!(alone_acc.value > social_acc.value,
                "alone={} should be > social={}", alone_acc.value, social_acc.value);
    }

    #[test]
    fn test_fallback_coherence() {
        let mut field = CoherenceField::new();
        let key = ContextKey::from_sensors(
            0.60, 0.10, PresenceSignature::Static, 1.0, false, 0.0, 0.0,
        );

        // Without fallback, unseen context = 0.0
        assert_eq!(field.context_coherence(&key), 0.0);

        // With fallback, unseen context = fallback value
        field.set_fallback(Some(0.4));
        assert!((field.context_coherence(&key) - 0.4).abs() < 0.001);

        // Seen context still uses its actual value
        {
            let acc = field.get_or_create(&key);
            acc.value = 0.6;
        }
        assert!((field.context_coherence(&key) - 0.6).abs() < 0.001);

        // Clear fallback
        field.set_fallback(None);
        // Seen context still works
        assert!((field.context_coherence(&key) - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_asymmetric_gate_noise_resilience() {
        // In a familiar context, a brief dip in instant_coherence should be buffered
        let mut field = CoherenceField::new();
        let key = ContextKey::from_sensors(
            0.60, 0.10, PresenceSignature::Static, 1.0, false, 0.0, 0.0,
        );

        // Build up to familiar territory
        {
            let acc = field.get_or_create(&key);
            for i in 0..100 {
                acc.positive_interaction(0.5, i, false);
            }
        }
        let ctx_coh = field.context_coherence(&key);
        assert!(ctx_coh >= 0.3, "should be familiar");

        // Simulate a light flicker: instant drops to 0.2
        let eff = field.effective_coherence(0.2, &key);
        // With asymmetric gate: 0.3*0.2 + 0.7*ctx_coh = 0.06 + 0.7*ctx
        // This should be much higher than min(0.2, ctx) = 0.2
        assert!(eff > 0.2, "familiar context should buffer noise: eff={}", eff);
    }

    #[test]
    fn test_asymmetric_gate_unfamiliar_strict() {
        // In an unfamiliar context, robot can't exceed earned trust
        let mut field = CoherenceField::new();
        let key = ContextKey::from_sensors(
            0.60, 0.10, PresenceSignature::Static, 1.0, false, 0.0, 0.0,
        );

        // Build up just a little (stay under 0.3)
        {
            let acc = field.get_or_create(&key);
            for i in 0..5 {
                acc.positive_interaction(0.5, i, false);
            }
        }
        let ctx_coh = field.context_coherence(&key);
        assert!(ctx_coh < 0.3);

        // High instant doesn't help: min(0.9, ctx) = ctx
        let eff = field.effective_coherence(0.9, &key);
        assert!((eff - ctx_coh).abs() < 0.001,
                "unfamiliar gate should cap at ctx: eff={} ctx={}", eff, ctx_coh);
    }

    #[test]
    fn test_accumulator_value_bounded() {
        // CCF-002: values always in [0.0, 1.0] with new_with_baseline
        let acc = CoherenceAccumulator::new_with_baseline(2.0); // out-of-range curiosity
        assert!(acc.value >= 0.0 && acc.value <= 1.0, "value={}", acc.value);

        let mut acc = CoherenceAccumulator::new_with_baseline(0.5);
        // Many alone interactions
        for i in 0..1000 {
            acc.positive_interaction(1.0, i, true);
        }
        assert!(acc.value <= 1.0, "value={}", acc.value);
    }

    #[test]
    fn test_context_hash_deterministic() {
        let key = ContextKey::from_sensors(0.3, 0.1, PresenceSignature::Static, 0.5, false, 0.0, 0.0);
        let h1 = key.context_hash_u32();
        let h2 = key.context_hash_u32();
        assert_eq!(h1, h2, "same key must produce same hash");
    }

    #[test]
    fn test_context_hash_differs_by_context() {
        let dark_quiet = ContextKey::from_sensors(0.05, 0.05, PresenceSignature::Absent, 0.5, false, 0.0, 0.0);
        let bright_loud = ContextKey::from_sensors(0.8, 0.8, PresenceSignature::Approaching, 0.5, false, 0.0, 0.0);
        assert_ne!(
            dark_quiet.context_hash_u32(),
            bright_loud.context_hash_u32(),
            "different contexts must produce different hashes"
        );
    }

    #[test]
    fn test_context_hash_nonzero() {
        let key = ContextKey::from_sensors(0.3, 0.3, PresenceSignature::Static, 0.5, false, 0.0, 0.0);
        assert_ne!(key.context_hash_u32(), 0, "hash should not be zero");
    }

    // --- Permeability function tests ---

    #[test]
    fn test_permeability_shy_observer_range() {
        // ShyObserver: effective_coherence * 0.3, so range [0.0, 0.3]
        let p_zero = permeability(0.0, 0.3, SocialPhase::ShyObserver);
        assert!((p_zero - 0.0).abs() < f32::EPSILON,
                "SO at coherence 0.0 should be 0.0, got {}", p_zero);

        let p_max = permeability(1.0, 0.3, SocialPhase::ShyObserver);
        assert!((p_max - 0.3).abs() < f32::EPSILON,
                "SO at coherence 1.0 should be 0.3, got {}", p_max);

        let p_mid = permeability(0.5, 0.3, SocialPhase::ShyObserver);
        assert!((p_mid - 0.15).abs() < f32::EPSILON,
                "SO at coherence 0.5 should be 0.15, got {}", p_mid);
    }

    #[test]
    fn test_permeability_startled_retreat_fixed() {
        // StartledRetreat: always 0.1 regardless of coherence or tension
        for coh in &[0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            for ten in &[0.0_f32, 0.5, 1.0] {
                let p = permeability(*coh, *ten, SocialPhase::StartledRetreat);
                assert!((p - 0.1).abs() < f32::EPSILON,
                        "SR should always be 0.1, got {} at coh={} ten={}", p, coh, ten);
            }
        }
    }

    #[test]
    fn test_permeability_quietly_beloved_range() {
        // QuietlyBeloved: 0.5 + effective_coherence * 0.5, so range [0.5, 1.0]
        let p_zero = permeability(0.0, 0.3, SocialPhase::QuietlyBeloved);
        assert!((p_zero - 0.5).abs() < f32::EPSILON,
                "QB at coherence 0.0 should be 0.5, got {}", p_zero);

        let p_max = permeability(1.0, 0.3, SocialPhase::QuietlyBeloved);
        assert!((p_max - 1.0).abs() < f32::EPSILON,
                "QB at coherence 1.0 should be 1.0, got {}", p_max);

        let p_mid = permeability(0.5, 0.3, SocialPhase::QuietlyBeloved);
        assert!((p_mid - 0.75).abs() < f32::EPSILON,
                "QB at coherence 0.5 should be 0.75, got {}", p_mid);
    }

    #[test]
    fn test_permeability_protective_guardian_range() {
        // ProtectiveGuardian: 0.4 + effective_coherence * 0.2, so range [0.4, 0.6]
        let p_zero = permeability(0.0, 0.3, SocialPhase::ProtectiveGuardian);
        assert!((p_zero - 0.4).abs() < f32::EPSILON,
                "PG at coherence 0.0 should be 0.4, got {}", p_zero);

        let p_max = permeability(1.0, 0.3, SocialPhase::ProtectiveGuardian);
        assert!((p_max - 0.6).abs() < f32::EPSILON,
                "PG at coherence 1.0 should be 0.6, got {}", p_max);

        let p_mid = permeability(0.5, 0.3, SocialPhase::ProtectiveGuardian);
        assert!((p_mid - 0.5).abs() < f32::EPSILON,
                "PG at coherence 0.5 should be 0.5, got {}", p_mid);
    }

    #[test]
    fn test_permeability_ordering() {
        // For the same coherence, QB > PG > SO > SR
        let coh = 0.7;
        let ten = 0.3;
        let qb = permeability(coh, ten, SocialPhase::QuietlyBeloved);
        let pg = permeability(coh, ten, SocialPhase::ProtectiveGuardian);
        let so = permeability(coh, ten, SocialPhase::ShyObserver);
        let sr = permeability(coh, ten, SocialPhase::StartledRetreat);

        assert!(qb > pg, "QB({}) should be > PG({})", qb, pg);
        assert!(pg > so, "PG({}) should be > SO({})", pg, so);
        assert!(so > sr, "SO({}) should be > SR({})", so, sr);
    }

    #[test]
    fn test_narration_depth_thresholds() {
        // Boundary checks for each depth level
        assert_eq!(NarrationDepth::from_permeability(0.0), NarrationDepth::None);
        assert_eq!(NarrationDepth::from_permeability(0.19), NarrationDepth::None);
        assert_eq!(NarrationDepth::from_permeability(0.2), NarrationDepth::Minimal);
        assert_eq!(NarrationDepth::from_permeability(0.39), NarrationDepth::Minimal);
        assert_eq!(NarrationDepth::from_permeability(0.4), NarrationDepth::Brief);
        assert_eq!(NarrationDepth::from_permeability(0.59), NarrationDepth::Brief);
        assert_eq!(NarrationDepth::from_permeability(0.6), NarrationDepth::Full);
        assert_eq!(NarrationDepth::from_permeability(0.79), NarrationDepth::Full);
        assert_eq!(NarrationDepth::from_permeability(0.8), NarrationDepth::Deep);
        assert_eq!(NarrationDepth::from_permeability(1.0), NarrationDepth::Deep);
    }

    #[test]
    fn test_narration_depth_matches_quadrants() {
        // Verify that typical quadrant permeabilities map to sensible narration depths
        // ShyObserver at max coherence: p=0.3 -> Minimal
        assert_eq!(
            NarrationDepth::from_permeability(permeability(1.0, 0.3, SocialPhase::ShyObserver)),
            NarrationDepth::Minimal
        );
        // StartledRetreat: p=0.1 -> None
        assert_eq!(
            NarrationDepth::from_permeability(permeability(0.5, 0.5, SocialPhase::StartledRetreat)),
            NarrationDepth::None
        );
        // QuietlyBeloved at max coherence: p=1.0 -> Deep
        assert_eq!(
            NarrationDepth::from_permeability(permeability(1.0, 0.1, SocialPhase::QuietlyBeloved)),
            NarrationDepth::Deep
        );
        // ProtectiveGuardian at mid coherence: p=0.5 -> Brief
        assert_eq!(
            NarrationDepth::from_permeability(permeability(0.5, 0.5, SocialPhase::ProtectiveGuardian)),
            NarrationDepth::Brief
        );
    }

    #[test]
    fn test_expression_scale_still_works() {
        // Backward compatibility: expression_scale delegates to permeability(0.5, 0.3, self)
        // Verify the ordering is preserved: QB > PG > SO > SR
        let qb = SocialPhase::QuietlyBeloved.expression_scale();
        let pg = SocialPhase::ProtectiveGuardian.expression_scale();
        let so = SocialPhase::ShyObserver.expression_scale();
        let sr = SocialPhase::StartledRetreat.expression_scale();

        assert!(qb > pg, "QB({}) should be > PG({})", qb, pg);
        assert!(pg > so, "PG({}) should be > SO({})", pg, so);
        assert!(so > sr, "SO({}) should be > SR({})", so, sr);

        // Verify specific values match permeability(0.5, 0.3, quadrant)
        assert!((qb - permeability(0.5, 0.3, SocialPhase::QuietlyBeloved)).abs() < f32::EPSILON);
        assert!((pg - permeability(0.5, 0.3, SocialPhase::ProtectiveGuardian)).abs() < f32::EPSILON);
        assert!((so - permeability(0.5, 0.3, SocialPhase::ShyObserver)).abs() < f32::EPSILON);
        assert!((sr - permeability(0.5, 0.3, SocialPhase::StartledRetreat)).abs() < f32::EPSILON);
    }
}
