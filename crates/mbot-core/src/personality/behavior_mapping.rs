//! Behavior Mapping - Maps personality parameters to robot behaviors
//!
//! This module provides the bridge between personality configuration and the
//! nervous system, ensuring that personality influences behavior in predictable,
//! deterministic ways.
//!
//! # Invariants
//! - **I-PERS-004:** Personality parameters must smoothly influence nervous system, not override it
//! - **I-PERS-005:** Behavior must emerge from personality + nervous system, not be scripted
//! - **I-PERS-006:** Transitions must be gradual, never jarring

#![cfg_attr(feature = "no_std", allow(unused_imports))]

#[cfg(not(feature = "std"))]
extern crate alloc;

use super::Personality;

/// Personality influence on nervous system behavior
///
/// Contains multipliers and modifiers that the personality applies to
/// the homeostasis calculations and output generation.
#[derive(Debug, Clone, PartialEq)]
pub struct PersonalityInfluence {
    // === Baseline targets for homeostasis ===
    /// Target tension value that homeostasis gravitates toward (0.0-1.0)
    pub tension_target: f32,
    /// Target coherence value (0.0-1.0)
    pub coherence_target: f32,
    /// Target energy level (0.0-1.0)
    pub energy_target: f32,

    // === Stimulus processing multipliers ===
    /// Multiplier for incoming stimulus intensity (0.5-1.5)
    pub stimulus_multiplier: f32,
    /// How quickly tension recovers to baseline (0.5-1.5)
    pub recovery_rate: f32,
    /// How strongly novelty attracts attention (0.0-1.0)
    pub curiosity_multiplier: f32,

    // === Output expression scaling ===
    /// Scale factor for movement amplitude (0.0-1.0)
    pub movement_scale: f32,
    /// Scale factor for sound volume/frequency (0.0-1.0)
    pub sound_scale: f32,
    /// Scale factor for LED brightness (0.0-1.0)
    pub light_scale: f32,
}

impl Default for PersonalityInfluence {
    /// Creates a neutral influence with no modifications (all targets at 0.5, all scales at 1.0)
    fn default() -> Self {
        Self {
            tension_target: 0.5,
            coherence_target: 0.5,
            energy_target: 0.5,
            stimulus_multiplier: 1.0,
            recovery_rate: 1.0,
            curiosity_multiplier: 0.5,
            movement_scale: 1.0,
            sound_scale: 1.0,
            light_scale: 1.0,
        }
    }
}

/// Personality mapper with smooth transition support
///
/// This struct manages the application of personality to the nervous system,
/// including smooth transitions between different personalities.
#[derive(Debug, Clone)]
pub struct PersonalityMapper {
    /// Current personality influence being applied
    current_influence: PersonalityInfluence,
    /// Starting influence (when transition began)
    start_influence: Option<PersonalityInfluence>,
    /// Target influence (if transitioning)
    target_influence: Option<PersonalityInfluence>,
    /// Transition progress (0.0-1.0), None if not transitioning
    transition_progress: Option<f32>,
    /// Transition speed (delta per tick)
    transition_speed: f32,
}

impl PersonalityMapper {
    /// Creates a new mapper with default (neutral) influence
    pub fn new() -> Self {
        Self {
            current_influence: PersonalityInfluence::default(),
            start_influence: None,
            target_influence: None,
            transition_progress: None,
            transition_speed: 0.01, // 100 ticks for full transition
        }
    }

    /// Creates a mapper with a specific personality applied immediately
    pub fn with_personality(personality: &Personality) -> Self {
        Self {
            current_influence: Self::calculate_influence_static(personality),
            start_influence: None,
            target_influence: None,
            transition_progress: None,
            transition_speed: 0.01,
        }
    }

    /// Calculates influence parameters from a personality (static, deterministic)
    ///
    /// Maps personality parameters to behavior multipliers:
    /// - Baselines → homeostasis targets
    /// - Reactivity → stimulus processing
    /// - Expressiveness → output scaling
    pub fn calculate_influence(personality: &Personality) -> PersonalityInfluence {
        Self::calculate_influence_static(personality)
    }

    fn calculate_influence_static(personality: &Personality) -> PersonalityInfluence {
        // === Baseline targets ===
        // Use personality baselines directly as homeostasis gravitational centers
        let tension_target = personality.tension_baseline();
        let coherence_target = personality.coherence_baseline();
        let energy_target = personality.energy_baseline();

        // === Reactivity multipliers ===
        // High startle_sensitivity → more responsive to stimuli (0.5 to 1.5 range)
        let stimulus_multiplier = 0.5 + personality.startle_sensitivity();

        // High recovery_speed → faster return to baseline (0.5 to 1.5 range)
        let recovery_rate = 0.5 + personality.recovery_speed();

        // Curiosity drive maps directly
        let curiosity_multiplier = personality.curiosity_drive();

        // === Expression scaling ===
        // Expressiveness parameters directly scale outputs
        let movement_scale = personality.movement_expressiveness();
        let sound_scale = personality.sound_expressiveness();
        let light_scale = personality.light_expressiveness();

        PersonalityInfluence {
            tension_target,
            coherence_target,
            energy_target,
            stimulus_multiplier,
            recovery_rate,
            curiosity_multiplier,
            movement_scale,
            sound_scale,
            light_scale,
        }
    }

    /// Initiates a smooth transition to a new personality
    ///
    /// The transition will occur gradually over approximately `duration_ticks` calls to
    /// `tick_transition()`.
    pub fn transition_to(&mut self, target: &Personality, duration_ticks: u32) {
        let target_influence = Self::calculate_influence_static(target);
        self.start_influence = Some(self.current_influence.clone());
        self.target_influence = Some(target_influence);
        self.transition_progress = Some(0.0);
        self.transition_speed = if duration_ticks > 0 {
            1.0 / duration_ticks as f32
        } else {
            1.0 // Instant transition
        };
    }

    /// Updates transition state and returns current influence
    ///
    /// Call this each tick to advance the transition. When transition completes,
    /// this automatically clears the transition state.
    pub fn tick_transition(&mut self) -> &PersonalityInfluence {
        if let (Some(ref start), Some(ref target), Some(progress)) =
            (&self.start_influence, &self.target_influence, self.transition_progress) {

            // Advance progress first
            let new_progress = (progress + self.transition_speed).min(1.0);

            if new_progress >= 1.0 {
                // Transition complete - apply final target exactly
                self.current_influence = target.clone();
                self.start_influence = None;
                self.target_influence = None;
                self.transition_progress = None;
            } else {
                // Interpolate between START and target (not current!)
                self.current_influence = self.interpolate_influence(
                    start,
                    target,
                    new_progress,
                );
                self.transition_progress = Some(new_progress);
            }
        }

        &self.current_influence
    }

    /// Gets the current influence without updating transition
    pub fn current_influence(&self) -> &PersonalityInfluence {
        &self.current_influence
    }

    /// Returns true if currently transitioning between personalities
    pub fn is_transitioning(&self) -> bool {
        self.transition_progress.is_some()
    }

    /// Returns transition progress (0.0-1.0), or None if not transitioning
    pub fn transition_progress(&self) -> Option<f32> {
        self.transition_progress
    }

    /// Linearly interpolates between two influences
    fn interpolate_influence(
        &self,
        from: &PersonalityInfluence,
        to: &PersonalityInfluence,
        t: f32,
    ) -> PersonalityInfluence {
        let t = t.clamp(0.0, 1.0);
        let lerp = |a: f32, b: f32| a + (b - a) * t;

        PersonalityInfluence {
            tension_target: lerp(from.tension_target, to.tension_target),
            coherence_target: lerp(from.coherence_target, to.coherence_target),
            energy_target: lerp(from.energy_target, to.energy_target),
            stimulus_multiplier: lerp(from.stimulus_multiplier, to.stimulus_multiplier),
            recovery_rate: lerp(from.recovery_rate, to.recovery_rate),
            curiosity_multiplier: lerp(from.curiosity_multiplier, to.curiosity_multiplier),
            movement_scale: lerp(from.movement_scale, to.movement_scale),
            sound_scale: lerp(from.sound_scale, to.sound_scale),
            light_scale: lerp(from.light_scale, to.light_scale),
        }
    }

    /// Sets the current personality immediately (no transition)
    pub fn set_personality_immediate(&mut self, personality: &Personality) {
        self.current_influence = Self::calculate_influence_static(personality);
        self.start_influence = None;
        self.target_influence = None;
        self.transition_progress = None;
    }
}

impl Default for PersonalityMapper {
    fn default() -> Self {
        Self::new()
    }
}

// === Helper Functions for MBotBrain Integration ===

/// Applies personality influence to raw tension value
///
/// This modifies incoming stimulus based on startle_sensitivity and
/// applies gravitational pull toward the personality's tension baseline.
#[inline]
pub fn apply_tension_influence(
    raw_tension: f32,
    current_tension: f32,
    influence: &PersonalityInfluence,
    gravity_strength: f32,
) -> f32 {
    // Scale incoming stimulus
    let modified_stimulus = raw_tension * influence.stimulus_multiplier;

    // Apply gravitational pull toward baseline
    let baseline_pull = (influence.tension_target - current_tension) * gravity_strength;

    // Combine (but clamp to valid range)
    (current_tension + modified_stimulus + baseline_pull).clamp(0.0, 1.0)
}

/// Applies personality influence to coherence calculation
#[inline]
pub fn apply_coherence_influence(
    raw_coherence: f32,
    current_coherence: f32,
    influence: &PersonalityInfluence,
    gravity_strength: f32,
) -> f32 {
    // Apply gravitational pull toward baseline
    let baseline_pull = (influence.coherence_target - current_coherence) * gravity_strength;

    (raw_coherence + baseline_pull).clamp(0.0, 1.0)
}

/// Applies personality influence to energy level
#[inline]
pub fn apply_energy_influence(
    current_energy: f32,
    influence: &PersonalityInfluence,
    recovery_delta: f32,
    depletion_delta: f32,
) -> f32 {
    // Scale recovery by personality's recovery rate
    let scaled_recovery = recovery_delta * influence.recovery_rate;

    // Apply gravitational pull toward energy baseline
    let baseline_pull = (influence.energy_target - current_energy) * 0.001;

    (current_energy + scaled_recovery - depletion_delta + baseline_pull).clamp(0.0, 1.0)
}

/// Applies personality influence to curiosity calculation
#[inline]
pub fn apply_curiosity_influence(
    raw_curiosity: f32,
    influence: &PersonalityInfluence,
) -> f32 {
    (raw_curiosity * influence.curiosity_multiplier).clamp(0.0, 1.0)
}

/// Scales motor commands by personality's movement expressiveness
#[inline]
pub fn scale_motor_output(left: i8, right: i8, influence: &PersonalityInfluence) -> (i8, i8) {
    let left_scaled = (left as f32 * influence.movement_scale) as i8;
    let right_scaled = (right as f32 * influence.movement_scale) as i8;
    (left_scaled.clamp(-100, 100), right_scaled.clamp(-100, 100))
}

/// Scales buzzer frequency by personality's sound expressiveness
#[inline]
pub fn scale_sound_output(buzzer_hz: u16, influence: &PersonalityInfluence) -> u16 {
    if buzzer_hz == 0 {
        0
    } else {
        let base_freq = buzzer_hz as f32;
        // Scale volume/intensity by moving frequency (higher = louder perception)
        let scaled = base_freq * (0.5 + influence.sound_scale * 0.5);
        scaled.clamp(100.0, 5000.0) as u16
    }
}

/// Scales LED brightness by personality's light expressiveness
#[inline]
pub fn scale_light_output(color: [u8; 3], influence: &PersonalityInfluence) -> [u8; 3] {
    [
        (color[0] as f32 * influence.light_scale) as u8,
        (color[1] as f32 * influence.light_scale) as u8,
        (color[2] as f32 * influence.light_scale) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personality::PersonalityPreset;

    // === I-PERS-004: Personality influences but doesn't override nervous system ===

    #[test]
    fn test_personality_influences_not_overrides() {
        let timid = PersonalityPreset::Timid.to_personality();
        let influence = PersonalityMapper::calculate_influence(&timid);

        // Timid has high startle sensitivity, so stimulus_multiplier should be > 1.0
        assert!(influence.stimulus_multiplier > 1.0);
        assert!(influence.stimulus_multiplier < 2.0); // But not extreme

        // Low recovery speed
        assert!(influence.recovery_rate < 1.0);

        // But values should still be reasonable
        assert!(influence.tension_target >= 0.0 && influence.tension_target <= 1.0);
    }

    #[test]
    fn test_tension_influence_respects_bounds() {
        let mut personality = Personality::default();
        personality.set_tension_baseline(0.8).unwrap();
        personality.set_startle_sensitivity(0.9).unwrap();

        let influence = PersonalityMapper::calculate_influence(&personality);

        // Even with high stimulus, should stay in bounds
        let raw_tension = 1.0; // Max stimulus
        let current = 0.5;
        let result = apply_tension_influence(raw_tension, current, &influence, 0.01);

        assert!(result >= 0.0 && result <= 1.0);
    }

    #[test]
    fn test_motor_output_scaling() {
        let mut personality = Personality::default();
        personality.set_movement_expressiveness(0.2).unwrap(); // Very subdued

        let influence = PersonalityMapper::calculate_influence(&personality);

        let (left, right) = scale_motor_output(100, 100, &influence);

        // Should be scaled down
        assert!(left < 100);
        assert!(right < 100);
        // But still in valid range
        assert!(left >= -100 && left <= 100);
        assert!(right >= -100 && right <= 100);
    }

    // === I-PERS-005: Behavior emerges from personality + nervous system ===

    #[test]
    fn test_different_personalities_produce_different_influence() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let excitable = PersonalityPreset::Excitable.to_personality();

        let mellow_influence = PersonalityMapper::calculate_influence(&mellow);
        let excitable_influence = PersonalityMapper::calculate_influence(&excitable);

        // Should be noticeably different
        assert!(
            (mellow_influence.tension_target - excitable_influence.tension_target).abs() > 0.1
        );
        assert!(
            (mellow_influence.stimulus_multiplier - excitable_influence.stimulus_multiplier).abs() > 0.1
        );
        assert!(
            (mellow_influence.movement_scale - excitable_influence.movement_scale).abs() > 0.1
        );
    }

    #[test]
    fn test_curiosity_drive_affects_behavior() {
        let mut low_curiosity = Personality::default();
        low_curiosity.set_curiosity_drive(0.1).unwrap();

        let mut high_curiosity = Personality::default();
        high_curiosity.set_curiosity_drive(0.9).unwrap();

        let low_influence = PersonalityMapper::calculate_influence(&low_curiosity);
        let high_influence = PersonalityMapper::calculate_influence(&high_curiosity);

        let raw_curiosity = 0.5;

        let low_result = apply_curiosity_influence(raw_curiosity, &low_influence);
        let high_result = apply_curiosity_influence(raw_curiosity, &high_influence);

        assert!(high_result > low_result);
    }

    // === I-PERS-006: Transitions must be gradual ===

    #[test]
    fn test_smooth_transition() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let excitable = PersonalityPreset::Excitable.to_personality();

        let mut mapper = PersonalityMapper::with_personality(&mellow);
        mapper.transition_to(&excitable, 100); // 100 tick transition

        let start_tension = mapper.current_influence().tension_target;

        // Advance 50 ticks
        for _ in 0..50 {
            mapper.tick_transition();
        }

        let mid_tension = mapper.current_influence().tension_target;

        // Should have moved partway
        assert!(mid_tension > start_tension);
        assert!(mid_tension < excitable.tension_baseline());

        // Complete transition (need 51 more ticks to reach 100% + 1 to finalize)
        for _ in 0..51 {
            mapper.tick_transition();
        }

        let end_tension = mapper.current_influence().tension_target;

        // Should now match target
        assert!((end_tension - excitable.tension_baseline()).abs() < 0.01);
        assert!(!mapper.is_transitioning());
    }

    #[test]
    fn test_no_sudden_jumps_in_transition() {
        let zen = PersonalityPreset::Zen.to_personality();
        let excitable = PersonalityPreset::Excitable.to_personality();

        // Zen tension_baseline: 0.1, Excitable: 0.6
        // Difference: 0.5, over 50 ticks = 0.01 per tick
        let mut mapper = PersonalityMapper::with_personality(&zen);
        mapper.transition_to(&excitable, 50);

        let mut prev_tension = mapper.current_influence().tension_target;

        for _ in 0..50 {
            mapper.tick_transition();
            let curr_tension = mapper.current_influence().tension_target;

            // Change per tick should be approximately 1/50 of total difference
            // Total difference is 0.5, so max delta should be ~0.01 + small epsilon
            let delta = (curr_tension - prev_tension).abs();
            assert!(delta < 0.03, "Jump too large: {}", delta);

            prev_tension = curr_tension;
        }
    }

    #[test]
    fn test_transition_completes() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let curious = PersonalityPreset::Curious.to_personality();

        let mut mapper = PersonalityMapper::with_personality(&mellow);
        mapper.transition_to(&curious, 10);

        assert!(mapper.is_transitioning());

        for _ in 0..20 {
            mapper.tick_transition();
        }

        assert!(!mapper.is_transitioning());
        assert!(mapper.transition_progress().is_none());
    }

    // === Determinism Tests (I-PERS-002) ===

    #[test]
    fn test_calculate_influence_is_deterministic() {
        let personality = PersonalityPreset::Curious.to_personality();

        let influence1 = PersonalityMapper::calculate_influence(&personality);
        let influence2 = PersonalityMapper::calculate_influence(&personality);

        // Should be exactly identical
        assert_eq!(influence1, influence2);
    }

    #[test]
    fn test_same_inputs_produce_same_outputs() {
        let personality = Personality::default();
        let influence = PersonalityMapper::calculate_influence(&personality);

        let result1 = apply_tension_influence(0.5, 0.3, &influence, 0.01);
        let result2 = apply_tension_influence(0.5, 0.3, &influence, 0.01);

        assert!((result1 - result2).abs() < 0.0001);
    }

    // === Baseline Gravity Tests ===

    #[test]
    fn test_tension_gravitates_toward_baseline() {
        let mut personality = Personality::default();
        personality.set_tension_baseline(0.7).unwrap();

        let influence = PersonalityMapper::calculate_influence(&personality);

        let current_tension = 0.3; // Below baseline
        let raw_stimulus = 0.0; // No external stimulus

        let result = apply_tension_influence(raw_stimulus, current_tension, &influence, 0.1);

        // Should be pulled upward toward 0.7
        assert!(result > current_tension);
    }

    #[test]
    fn test_energy_gravitates_toward_baseline() {
        let mut personality = Personality::default();
        personality.set_energy_baseline(0.8).unwrap();

        let influence = PersonalityMapper::calculate_influence(&personality);

        let current_energy = 0.4;

        let result = apply_energy_influence(current_energy, &influence, 0.0, 0.0);

        // Should be pulled toward 0.8
        assert!(result > current_energy);
    }

    // === Expression Scaling Tests ===

    #[test]
    fn test_light_output_scaling() {
        let mut personality = Personality::default();
        personality.set_light_expressiveness(0.5).unwrap();

        let influence = PersonalityMapper::calculate_influence(&personality);

        let color = [200, 150, 100];
        let scaled = scale_light_output(color, &influence);

        // Should be dimmed
        assert!(scaled[0] < color[0]);
        assert!(scaled[1] < color[1]);
        assert!(scaled[2] < color[2]);
    }

    #[test]
    fn test_sound_output_scaling() {
        let mut personality = Personality::default();
        personality.set_sound_expressiveness(0.2).unwrap(); // Very quiet

        let influence = PersonalityMapper::calculate_influence(&personality);

        let freq = scale_sound_output(440, &influence);

        // Should be less intense (lower frequency)
        assert!(freq < 440);
    }

    #[test]
    fn test_sound_output_zero_stays_zero() {
        let personality = Personality::default();
        let influence = PersonalityMapper::calculate_influence(&personality);

        let freq = scale_sound_output(0, &influence);

        assert_eq!(freq, 0);
    }

    // === Edge Cases ===

    #[test]
    fn test_instant_transition() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let excitable = PersonalityPreset::Excitable.to_personality();

        let mut mapper = PersonalityMapper::with_personality(&mellow);
        mapper.transition_to(&excitable, 0); // Instant

        mapper.tick_transition();

        // Should complete immediately
        assert!(!mapper.is_transitioning());
        let tension = mapper.current_influence().tension_target;
        assert!((tension - excitable.tension_baseline()).abs() < 0.01);
    }

    #[test]
    fn test_set_immediate_cancels_transition() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let excitable = PersonalityPreset::Excitable.to_personality();
        let zen = PersonalityPreset::Zen.to_personality();

        let mut mapper = PersonalityMapper::with_personality(&mellow);
        mapper.transition_to(&excitable, 100);

        assert!(mapper.is_transitioning());

        mapper.set_personality_immediate(&zen);

        assert!(!mapper.is_transitioning());
        let tension = mapper.current_influence().tension_target;
        assert!((tension - zen.tension_baseline()).abs() < 0.01);
    }

    #[test]
    fn test_default_influence_is_neutral() {
        let influence = PersonalityInfluence::default();

        assert_eq!(influence.tension_target, 0.5);
        assert_eq!(influence.coherence_target, 0.5);
        assert_eq!(influence.energy_target, 0.5);
        assert_eq!(influence.stimulus_multiplier, 1.0);
        assert_eq!(influence.recovery_rate, 1.0);
    }
}
