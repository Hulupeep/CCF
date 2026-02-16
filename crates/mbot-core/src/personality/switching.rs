//! Personality Switching - Runtime personality changes with smooth transitions
//!
//! This module provides the `PersonalitySwitcher` for managing personality changes
//! at runtime, with smooth transitions and LED animations.
//!
//! # Invariants
//! - **I-PERS-010:** Personality switch must complete within specified duration
//! - **I-PERS-011:** Mid-action switch must not cause crashes or undefined behavior
//! - **I-PERS-012:** Transition must interpolate smoothly between all parameters
//! - **I-PERS-006:** Transitions must be gradual, never jarring

#![cfg_attr(feature = "no_std", allow(unused_imports))]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

use super::{Personality, PersonalityMapper, PersonalityInfluence};

/// Easing function for smooth transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    /// Constant rate transition
    Linear,
    /// Accelerate from zero velocity
    EaseIn,
    /// Decelerate to zero velocity
    EaseOut,
    /// Accelerate then decelerate
    EaseInOut,
}

impl Easing {
    /// Applies the easing function to a linear progress value (0.0-1.0)
    #[inline]
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => t * (2.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
        }
    }
}

/// Configuration for personality transition
#[derive(Debug, Clone)]
pub struct TransitionConfig {
    /// Duration in milliseconds (default 3000)
    pub duration_ms: u64,
    /// Easing function to use
    pub easing: Easing,
    /// Whether to show LED animation during transition
    pub led_animation: bool,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            duration_ms: 3000,
            easing: Easing::EaseInOut,
            led_animation: true,
        }
    }
}

/// Event type for transition callbacks
#[derive(Debug, Clone)]
pub enum TransitionEvent {
    /// Transition started from one personality to another
    Started {
        from: String,
        to: String,
    },
    /// Transition completed
    Completed {
        personality: String,
    },
    /// Transition was cancelled
    Cancelled {
        at_progress: f32,
    },
}

/// Personality switcher with smooth transition support
///
/// Manages the active personality and handles smooth transitions between
/// different personalities at runtime.
///
/// # Example
///
/// ```
/// use mbot_core::personality::{PersonalitySwitcher, PersonalityPreset, TransitionConfig};
///
/// let mut switcher = PersonalitySwitcher::new(PersonalityPreset::Mellow.to_personality());
///
/// // Switch to a new personality
/// let config = TransitionConfig::default();
/// switcher.switch_to(PersonalityPreset::Excitable.to_personality(), config);
///
/// // Update each tick (call this in your main loop)
/// while switcher.is_transitioning() {
///     let influence = switcher.update(16); // 16ms delta (~60fps)
///     // Use influence to affect robot behavior
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PersonalitySwitcher {
    /// Current active personality
    current: Personality,
    /// Target personality (if transitioning)
    target: Option<Personality>,
    /// Personality mapper handling the actual parameter interpolation
    mapper: PersonalityMapper,
    /// Transition configuration
    transition_config: TransitionConfig,
    /// Elapsed time in transition (milliseconds)
    elapsed_ms: u64,
    /// Whether a transition is active
    transitioning: bool,
    /// Source personality name (for events)
    from_name: String,
}

impl PersonalitySwitcher {
    /// Creates a new switcher with the given initial personality
    pub fn new(initial: Personality) -> Self {
        let from_name = initial.name.clone();
        Self {
            current: initial.clone(),
            target: None,
            mapper: PersonalityMapper::with_personality(&initial),
            transition_config: TransitionConfig::default(),
            elapsed_ms: 0,
            transitioning: false,
            from_name,
        }
    }

    /// Initiates a switch to a new personality
    ///
    /// This will start a smooth transition from the current personality to the
    /// target personality. The transition will occur over the duration specified
    /// in the config.
    ///
    /// # Arguments
    /// * `personality` - The target personality to switch to
    /// * `config` - Transition configuration (duration, easing, animation)
    ///
    /// # Returns
    /// A `TransitionEvent::Started` event
    pub fn switch_to(&mut self, personality: Personality, config: TransitionConfig) -> TransitionEvent {
        let from_name = self.current.name.clone();
        let to_name = personality.name.clone();

        // Calculate duration in ticks (assuming ~60Hz update rate)
        let duration_ticks = (config.duration_ms as f32 / 16.0) as u32;

        self.target = Some(personality.clone());
        self.transition_config = config;
        self.elapsed_ms = 0;
        self.transitioning = true;
        self.from_name = from_name.clone();

        // Start the transition in the mapper
        self.mapper.transition_to(&personality, duration_ticks);

        TransitionEvent::Started {
            from: from_name,
            to: to_name,
        }
    }

    /// Updates the transition state
    ///
    /// Call this each frame/tick to advance the transition. Returns the current
    /// personality influence to be applied to the nervous system.
    ///
    /// # Arguments
    /// * `delta_ms` - Time elapsed since last update in milliseconds
    ///
    /// # Returns
    /// The current `PersonalityInfluence` to apply to the robot's behavior
    pub fn update(&mut self, delta_ms: u64) -> &PersonalityInfluence {
        if self.transitioning {
            self.elapsed_ms += delta_ms;

            // Update the mapper which handles the actual interpolation
            self.mapper.tick_transition();

            // Check if transition should complete (both time-based and mapper-based)
            if self.elapsed_ms >= self.transition_config.duration_ms || !self.mapper.is_transitioning() {
                self.complete_transition();
            }
        }

        self.mapper.current_influence()
    }

    /// Gets the current personality
    pub fn get_current(&self) -> &Personality {
        &self.current
    }

    /// Returns true if currently transitioning between personalities
    pub fn is_transitioning(&self) -> bool {
        self.transitioning
    }

    /// Gets the transition progress (0.0-1.0), or 0.0 if not transitioning
    pub fn get_progress(&self) -> f32 {
        if !self.transitioning {
            return 0.0;
        }

        let linear_progress = (self.elapsed_ms as f32 / self.transition_config.duration_ms as f32).min(1.0);
        self.transition_config.easing.apply(linear_progress)
    }

    /// Cancels the current transition
    ///
    /// The robot will remain at the current interpolated state (not jump back
    /// to the source personality).
    ///
    /// # Returns
    /// A `TransitionEvent::Cancelled` event with the progress at cancellation
    pub fn cancel_transition(&mut self) -> TransitionEvent {
        let progress = self.get_progress();

        // Freeze current state in the mapper by creating a personality from current influence
        // This ensures we stay exactly at the current interpolated values
        let current_influence = self.mapper.current_influence().clone();

        // Create a synthetic personality that matches current influence
        let mut frozen = self.current.clone();
        frozen.name = format!("{} (transitioning)", frozen.name);

        // Clear transition state
        self.transitioning = false;
        self.target = None;
        self.elapsed_ms = 0;

        TransitionEvent::Cancelled {
            at_progress: progress,
        }
    }

    /// Gets the current personality influence without updating transition
    pub fn current_influence(&self) -> &PersonalityInfluence {
        self.mapper.current_influence()
    }

    /// Calculates LED color for transition animation
    ///
    /// Returns a blended color that represents the transition progress
    /// between the source and target personality icons/colors.
    pub fn transition_led_color(&self) -> [u8; 3] {
        if !self.transitioning || !self.transition_config.led_animation {
            // Default to blue when not transitioning
            return [0, 100, 255];
        }

        let progress = self.get_progress();

        // Create a pulsing effect during transition
        let pulse = (progress * core::f32::consts::PI * 4.0).sin().abs();

        // Blend from cyan to magenta during transition for visual feedback
        let r = (pulse * 255.0) as u8;
        let g = (100.0 * (1.0 - progress)) as u8;
        let b = (255.0 * (1.0 - progress) + pulse * 128.0) as u8;

        [r, g, b]
    }

    /// Completes the current transition
    fn complete_transition(&mut self) {
        if let Some(target) = self.target.take() {
            self.current = target;
            self.from_name = self.current.name.clone();
        }
        self.transitioning = false;
        self.elapsed_ms = 0;
    }

    /// Sets the current personality immediately without transition
    ///
    /// This bypasses the transition system and instantly applies the new personality.
    /// Use this for initialization or when you need immediate changes.
    pub fn set_immediate(&mut self, personality: Personality) {
        self.current = personality.clone();
        self.from_name = self.current.name.clone();
        self.target = None;
        self.transitioning = false;
        self.elapsed_ms = 0;
        self.mapper.set_personality_immediate(&personality);
    }

    /// Gets time remaining in transition (milliseconds), or 0 if not transitioning
    pub fn time_remaining_ms(&self) -> u64 {
        if !self.transitioning {
            return 0;
        }
        self.transition_config.duration_ms.saturating_sub(self.elapsed_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personality::PersonalityPreset;

    // === I-PERS-010: Personality switch must complete within specified duration ===

    #[test]
    fn test_transition_completes_within_duration() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let excitable = PersonalityPreset::Excitable.to_personality();

        let mut switcher = PersonalitySwitcher::new(mellow);

        let config = TransitionConfig {
            duration_ms: 1000,
            easing: Easing::Linear,
            led_animation: true,
        };

        switcher.switch_to(excitable.clone(), config);

        assert!(switcher.is_transitioning());

        // Simulate updates at 60fps (16ms per frame)
        let mut total_time = 0;
        while switcher.is_transitioning() && total_time < 2000 {
            switcher.update(16);
            total_time += 16;
        }

        // Should complete within 1000ms + some tolerance for frame timing
        assert!(total_time <= 1100, "Transition took {}ms, expected ~1000ms", total_time);
        assert!(!switcher.is_transitioning());
        assert_eq!(switcher.get_current().name, excitable.name);
    }

    #[test]
    fn test_transition_duration_honored() {
        let zen = PersonalityPreset::Zen.to_personality();
        let playful = PersonalityPreset::Playful.to_personality();

        let mut switcher = PersonalitySwitcher::new(zen);

        let config = TransitionConfig {
            duration_ms: 3000,
            ..Default::default()
        };

        switcher.switch_to(playful, config);

        // After 1500ms (halfway), should still be transitioning
        for _ in 0..93 {  // 93 * 16ms ≈ 1488ms
            switcher.update(16);
        }

        assert!(switcher.is_transitioning());
        let progress = switcher.get_progress();
        assert!(progress > 0.4 && progress < 0.6, "Progress should be near 0.5, got {}", progress);

        // Complete the transition
        for _ in 0..100 {
            switcher.update(16);
        }

        assert!(!switcher.is_transitioning());
    }

    // === I-PERS-011: Mid-action switch must not cause crashes ===

    #[test]
    fn test_rapid_switching_no_crash() {
        let personalities = [
            PersonalityPreset::Mellow.to_personality(),
            PersonalityPreset::Excitable.to_personality(),
            PersonalityPreset::Zen.to_personality(),
            PersonalityPreset::Curious.to_personality(),
            PersonalityPreset::Timid.to_personality(),
        ];

        let mut switcher = PersonalitySwitcher::new(personalities[0].clone());

        let config = TransitionConfig {
            duration_ms: 500,
            ..Default::default()
        };

        // Rapidly switch 10 times
        for i in 0..10 {
            let target = personalities[i % personalities.len()].clone();
            switcher.switch_to(target, config.clone());

            // Advance a bit but don't complete
            switcher.update(50);
        }

        // Should not crash and should be in valid state
        assert!(switcher.get_progress() >= 0.0 && switcher.get_progress() <= 1.0);
    }

    #[test]
    fn test_switch_during_transition() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let excitable = PersonalityPreset::Excitable.to_personality();
        let zen = PersonalityPreset::Zen.to_personality();

        let mut switcher = PersonalitySwitcher::new(mellow);

        let config = TransitionConfig {
            duration_ms: 1000,
            ..Default::default()
        };

        // Start first transition
        switcher.switch_to(excitable, config.clone());

        // Advance partway
        for _ in 0..30 {  // ~480ms
            switcher.update(16);
        }

        // Start new transition mid-way
        switcher.switch_to(zen.clone(), config);

        assert!(switcher.is_transitioning());

        // Complete the new transition
        for _ in 0..70 {
            switcher.update(16);
        }

        assert!(!switcher.is_transitioning());
        assert_eq!(switcher.get_current().name, zen.name);
    }

    // === I-PERS-012: Transition must interpolate smoothly ===

    #[test]
    fn test_smooth_parameter_interpolation() {
        let mellow = PersonalityPreset::Mellow.to_personality(); // tension_baseline: 0.2
        let anxious = PersonalityPreset::Anxious.to_personality(); // tension_baseline: 0.8

        let mut switcher = PersonalitySwitcher::new(mellow);

        let config = TransitionConfig {
            duration_ms: 1000,
            easing: Easing::Linear,
            led_animation: false,
        };

        switcher.switch_to(anxious, config);

        let mut prev_tension = switcher.current_influence().tension_target;
        let mut values = Vec::new();

        // Sample values throughout transition
        for _ in 0..62 {  // ~992ms
            switcher.update(16);
            let curr_tension = switcher.current_influence().tension_target;
            values.push(curr_tension);

            // Should always increase (or stay same)
            assert!(curr_tension >= prev_tension - 0.001,
                "Tension decreased: {} -> {}", prev_tension, curr_tension);

            prev_tension = curr_tension;
        }

        // Should have sampled multiple distinct values
        assert!(values.len() > 10);

        // First value should be near mellow baseline
        assert!((values[0] - 0.2).abs() < 0.1);

        // Last value should be near anxious baseline
        assert!((values[values.len() - 1] - 0.8).abs() < 0.1);
    }

    #[test]
    fn test_easing_functions() {
        // Linear
        assert!((Easing::Linear.apply(0.0) - 0.0).abs() < 0.001);
        assert!((Easing::Linear.apply(0.5) - 0.5).abs() < 0.001);
        assert!((Easing::Linear.apply(1.0) - 1.0).abs() < 0.001);

        // EaseIn (should start slow)
        let ease_in_half = Easing::EaseIn.apply(0.5);
        assert!(ease_in_half < 0.5, "EaseIn should be < 0.5 at t=0.5");

        // EaseOut (should end slow)
        let ease_out_half = Easing::EaseOut.apply(0.5);
        assert!(ease_out_half > 0.5, "EaseOut should be > 0.5 at t=0.5");

        // EaseInOut (should be symmetric)
        let ease_inout_quarter = Easing::EaseInOut.apply(0.25);
        let ease_inout_three_quarters = Easing::EaseInOut.apply(0.75);
        assert!((ease_inout_quarter + ease_inout_three_quarters - 1.0).abs() < 0.1);
    }

    // === I-PERS-006: Transitions must be gradual, never jarring ===

    #[test]
    fn test_no_sudden_jumps() {
        let zen = PersonalityPreset::Zen.to_personality();
        let energetic = PersonalityPreset::Energetic.to_personality();

        let mut switcher = PersonalitySwitcher::new(zen);

        let config = TransitionConfig {
            duration_ms: 500,
            easing: Easing::Linear,
            led_animation: false,
        };

        switcher.switch_to(energetic, config);

        let mut prev_values = (
            switcher.current_influence().tension_target,
            switcher.current_influence().energy_target,
            switcher.current_influence().movement_scale,
        );

        for _ in 0..31 {  // ~496ms
            switcher.update(16);
            let curr_values = (
                switcher.current_influence().tension_target,
                switcher.current_influence().energy_target,
                switcher.current_influence().movement_scale,
            );

            // Check that no single frame has a huge jump
            let tension_delta = (curr_values.0 - prev_values.0).abs();
            let energy_delta = (curr_values.1 - prev_values.1).abs();
            let movement_delta = (curr_values.2 - prev_values.2).abs();

            assert!(tension_delta < 0.1, "Tension jumped by {}", tension_delta);
            assert!(energy_delta < 0.1, "Energy jumped by {}", energy_delta);
            assert!(movement_delta < 0.1, "Movement jumped by {}", movement_delta);

            prev_values = curr_values;
        }
    }

    // === Additional Tests ===

    #[test]
    fn test_cancel_transition() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let excitable = PersonalityPreset::Excitable.to_personality();

        let mut switcher = PersonalitySwitcher::new(mellow);

        let config = TransitionConfig {
            duration_ms: 1000,
            ..Default::default()
        };

        switcher.switch_to(excitable, config);

        // Advance partway
        for _ in 0..30 {
            switcher.update(16);
        }

        let progress_at_cancel = switcher.get_progress();
        let influence_at_cancel = switcher.current_influence().tension_target;

        // Cancel
        let event = switcher.cancel_transition();

        match event {
            TransitionEvent::Cancelled { at_progress } => {
                assert!((at_progress - progress_at_cancel).abs() < 0.01);
            }
            _ => panic!("Expected Cancelled event"),
        }

        assert!(!switcher.is_transitioning());

        // Should stay at current state, not jump
        let influence_after = switcher.current_influence().tension_target;
        assert!((influence_after - influence_at_cancel).abs() < 0.001);
    }

    #[test]
    fn test_set_immediate() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let excitable = PersonalityPreset::Excitable.to_personality();
        let zen = PersonalityPreset::Zen.to_personality();

        let mut switcher = PersonalitySwitcher::new(mellow);

        let config = TransitionConfig::default();
        switcher.switch_to(excitable, config);

        assert!(switcher.is_transitioning());

        // Set immediate should cancel transition
        switcher.set_immediate(zen.clone());

        assert!(!switcher.is_transitioning());
        assert_eq!(switcher.get_current().name, zen.name);

        // Influence should match zen exactly
        let tension = switcher.current_influence().tension_target;
        assert!((tension - zen.tension_baseline()).abs() < 0.01);
    }

    #[test]
    fn test_get_current_personality() {
        let curious = PersonalityPreset::Curious.to_personality();
        let switcher = PersonalitySwitcher::new(curious.clone());

        assert_eq!(switcher.get_current().id, curious.id);
        assert_eq!(switcher.get_current().name, curious.name);
    }

    #[test]
    fn test_transition_led_color_changes() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let excitable = PersonalityPreset::Excitable.to_personality();

        let mut switcher = PersonalitySwitcher::new(mellow);

        let config = TransitionConfig {
            duration_ms: 1000,
            led_animation: true,
            ..Default::default()
        };

        switcher.switch_to(excitable, config);

        let color1 = switcher.transition_led_color();

        // Advance
        for _ in 0..30 {
            switcher.update(16);
        }

        let color2 = switcher.transition_led_color();

        // Colors should be different during transition
        assert!(color1 != color2, "LED color should change during transition");
    }

    #[test]
    fn test_progress_returns_zero_when_not_transitioning() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let switcher = PersonalitySwitcher::new(mellow);

        assert_eq!(switcher.get_progress(), 0.0);
        assert!(!switcher.is_transitioning());
    }

    #[test]
    fn test_time_remaining() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let excitable = PersonalityPreset::Excitable.to_personality();

        let mut switcher = PersonalitySwitcher::new(mellow);

        let config = TransitionConfig {
            duration_ms: 1000,
            ..Default::default()
        };

        switcher.switch_to(excitable, config);

        let remaining1 = switcher.time_remaining_ms();
        assert!(remaining1 > 900 && remaining1 <= 1000);

        // Advance 500ms
        for _ in 0..31 {  // ~496ms
            switcher.update(16);
        }

        let remaining2 = switcher.time_remaining_ms();
        assert!(remaining2 > 400 && remaining2 < 600, "Remaining: {}", remaining2);

        // Complete
        for _ in 0..35 {
            switcher.update(16);
        }

        assert_eq!(switcher.time_remaining_ms(), 0);
    }

    #[test]
    fn test_transition_event_started() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        let excitable = PersonalityPreset::Excitable.to_personality();

        let mut switcher = PersonalitySwitcher::new(mellow.clone());

        let event = switcher.switch_to(excitable.clone(), TransitionConfig::default());

        match event {
            TransitionEvent::Started { from, to } => {
                assert_eq!(from, mellow.name);
                assert_eq!(to, excitable.name);
            }
            _ => panic!("Expected Started event"),
        }
    }

    #[test]
    fn test_easing_bounds() {
        // All easing functions should stay in [0, 1] range
        for &t in &[0.0, 0.25, 0.5, 0.75, 1.0, 1.5, -0.5] {
            let linear = Easing::Linear.apply(t);
            let ease_in = Easing::EaseIn.apply(t);
            let ease_out = Easing::EaseOut.apply(t);
            let ease_inout = Easing::EaseInOut.apply(t);

            assert!(linear >= 0.0 && linear <= 1.0);
            assert!(ease_in >= 0.0 && ease_in <= 1.0);
            assert!(ease_out >= 0.0 && ease_out <= 1.0);
            assert!(ease_inout >= 0.0 && ease_inout <= 1.0);
        }
    }
}
