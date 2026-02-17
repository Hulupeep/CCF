//! Observable behavior mapping — LED/motor/sound scaling by suppression factor,
//! plus per-quadrant motor/LED/sound behavioral profiles.
//!
//! Maps startle results to concrete motor/LED/buzzer commands that scale
//! proportionally with how suppressed the startle response is.
//!
//! Also provides [`QuadrantBehavior`] which maps each [`SocialPhase`] to
//! concrete motor speed, LED color/pulse, and sound mode parameters,
//! modulated by permeability and personality expressiveness.
//!
//! Invariants:
//! - I-STRT-001: no_std compatible
//! - I-BEHV-001: Unsuppressed (factor >= 0.9) produces full startle response
//! - I-BEHV-002: Partially suppressed (0.6..0.9) produces moderate response
//! - I-BEHV-003: Heavily suppressed (< 0.45) produces no visible response
//! - I-BEHV-004: Personality expressiveness modulates freeze/sound/LED intensity
//! - I-BEHV-005: QuadrantBehavior is no_std compatible (no HashMap, String, Vec)
//! - I-BEHV-006: Permeability=0 produces minimal expression across all quadrants
//! - I-BEHV-007: Personality expressiveness further modulates quadrant output

use super::startle::StartleResult;
use crate::coherence::SocialPhase;

/// Startle behavior response scaled by suppression factor.
#[derive(Clone, Debug)]
pub struct StartleBehavior {
    /// Override motor command: (left, right). None = keep current.
    pub motor_override: Option<(i8, i8)>,
    /// Override LED color: [R, G, B]. None = keep current.
    pub led_override: Option<[u8; 3]>,
    /// Override buzzer frequency. None = keep current.
    pub buzzer_override: Option<u16>,
    /// Number of ticks to freeze motor output (0 = no freeze).
    pub freeze_ticks: u32,
}

impl StartleBehavior {
    /// Compute the observable behavior from a startle result.
    ///
    /// `movement_expressiveness` and `sound_expressiveness` are personality
    /// parameters in [0.0, 1.0].
    ///
    /// # Behavior tiers
    ///
    /// | Factor | Motors | LEDs | Sound | Pause |
    /// |--------|--------|------|-------|-------|
    /// | >= 0.9 | Reverse (-60, -60) | Red flash [255,0,0] | 880 Hz chirp | ~25 ticks |
    /// | 0.6..0.9 | None (decelerate externally) | Amber [255,180,0] | 330 Hz soft | ~5 ticks |
    /// | 0.45..0.6 | None | Subtle yellow (if expressive) | None | 0 |
    /// | < 0.45 | None | None | None | 0 |
    pub fn from_startle(
        result: &StartleResult,
        movement_expressiveness: f32,
        sound_expressiveness: f32,
    ) -> Self {
        let factor = result.suppression_factor;

        if factor >= 0.9 {
            // Unsuppressed: full startle response
            Self {
                motor_override: Some((-60, -60)), // reverse
                led_override: Some([255, 0, 0]),  // red flash
                buzzer_override: Some(880),       // surprise chirp
                freeze_ticks: (25.0 * movement_expressiveness).max(10.0) as u32,
            }
        } else if factor >= 0.6 {
            // Partially suppressed: moderate response
            Self {
                motor_override: None, // slight deceleration handled by caller
                led_override: Some([255, 180, 0]), // amber flicker
                buzzer_override: if sound_expressiveness > 0.3 {
                    Some(330) // soft low tone
                } else {
                    None
                },
                freeze_ticks: (5.0 * movement_expressiveness).max(2.0) as u32,
            }
        } else if factor >= 0.45 {
            // Mostly suppressed: subtle response
            Self {
                motor_override: None,
                led_override: if movement_expressiveness > 0.5 {
                    Some([100, 100, 50]) // very subtle yellow pulse
                } else {
                    None
                },
                buzzer_override: None,
                freeze_ticks: 0,
            }
        } else {
            // Heavily suppressed: no visible response
            Self {
                motor_override: None,
                led_override: None,
                buzzer_override: None,
                freeze_ticks: 0,
            }
        }
    }

    /// Returns true if this behavior has any visible effect.
    pub fn is_visible(&self) -> bool {
        self.motor_override.is_some()
            || self.led_override.is_some()
            || self.buzzer_override.is_some()
            || self.freeze_ticks > 0
    }
}

// ─── Per-Quadrant Behavioral Profiles ────────────────────────────────

/// Sound output mode for quadrant behavior.
///
/// Ordered from quietest to loudest. The `quadrant_behavior()` function
/// may downgrade the mode when permeability or sound expressiveness is low.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundMode {
    /// No sound output.
    Silent,
    /// A faint, occasional tone (used in ShyObserver on novelty).
    SoftTone,
    /// Continuous low hum (ambient awareness).
    Hum,
    /// Expressive R2-D2-style beeps and chirps.
    R2D2Tones,
    /// Alert/warning hum for protective or startled states.
    WarningHum,
}

impl SoundMode {
    /// Returns a numeric rank for ordering: higher = louder/more expressive.
    fn rank(self) -> u8 {
        match self {
            SoundMode::Silent => 0,
            SoundMode::SoftTone => 1,
            SoundMode::Hum => 2,
            SoundMode::R2D2Tones => 3,
            SoundMode::WarningHum => 4,
        }
    }

    /// Downgrade the sound mode by one rank toward Silent.
    fn downgrade(self) -> Self {
        match self {
            SoundMode::WarningHum => SoundMode::R2D2Tones,
            SoundMode::R2D2Tones => SoundMode::Hum,
            SoundMode::Hum => SoundMode::SoftTone,
            SoundMode::SoftTone => SoundMode::Silent,
            SoundMode::Silent => SoundMode::Silent,
        }
    }
}

/// Per-quadrant behavioral profile: motor speed, LED color/pulse, and sound.
///
/// Created by [`quadrant_behavior()`] which maps a [`SocialPhase`] to
/// concrete output parameters, modulated by permeability and personality
/// expressiveness values.
#[derive(Clone, Debug)]
pub struct QuadrantBehavior {
    /// Motor speed scaling factor [0.0, 1.0] applied to base motor commands.
    pub motor_speed_factor: f32,
    /// RGB base color for LEDs.
    pub led_color: [u8; 3],
    /// Ticks per LED pulse cycle. 0 = steady (no pulsing).
    pub led_pulse_period: u16,
    /// LED brightness scaling [0.0, 1.0].
    pub led_brightness: f32,
    /// Sound output mode.
    pub sound_mode: SoundMode,
}

/// Compute per-quadrant behavioral profile from social phase, permeability,
/// and personality expressiveness parameters.
///
/// This is the main entry point for translating the robot's relational state
/// into concrete motor/LED/sound behavior.
///
/// # Arguments
///
/// * `phase` - The current social phase (from `SocialPhase::classify()`).
/// * `permeability` - Output permeability [0.0, 1.0] (from `coherence::permeability()`).
/// * `movement_expr` - Personality movement expressiveness [0.0, 1.0].
/// * `sound_expr` - Personality sound expressiveness [0.0, 1.0].
/// * `light_expr` - Personality light expressiveness [0.0, 1.0].
///
/// # Per-quadrant base profiles
///
/// | Phase | Motor | LED Color | Pulse | Sound |
/// |-------|-------|-----------|-------|-------|
/// | ShyObserver | 0.3 | [100,120,180] dim blue-white | 60 ticks | Silent (SoftTone on novelty) |
/// | StartledRetreat | 1.0 (reverse) | [220,30,20] red | 10 ticks fast | WarningHum |
/// | QuietlyBeloved | 1.0 | [80,160,220] warm rich | 0 (steady) | R2D2Tones |
/// | ProtectiveGuardian | 0.7 | [200,150,0] amber | 0 (steady) | WarningHum |
///
/// # Modulation
///
/// - `motor_speed_factor` is scaled by `permeability * movement_expr`
/// - `led_brightness` is scaled by `permeability * light_expr`
/// - Sound mode is downgraded when `permeability * sound_expr < 0.3`
pub fn quadrant_behavior(
    phase: SocialPhase,
    permeability: f32,
    movement_expr: f32,
    sound_expr: f32,
    light_expr: f32,
) -> QuadrantBehavior {
    // Base profiles per quadrant (before modulation)
    let (base_motor, base_led, base_pulse, base_brightness, base_sound) = match phase {
        SocialPhase::ShyObserver => (
            0.3_f32,
            [100_u8, 120, 180],   // dim blue-white
            60_u16,                // slow pulse
            0.4_f32,              // dim
            SoundMode::Silent,    // quiet by default (SoftTone only on novelty)
        ),
        SocialPhase::StartledRetreat => (
            1.0,
            [220, 30, 20],        // red alert
            10,                    // fast pulse
            0.9,                   // bright warning
            SoundMode::WarningHum,
        ),
        SocialPhase::QuietlyBeloved => (
            1.0,
            [80, 160, 220],       // warm rich blue
            0,                     // steady (no pulse)
            1.0,                   // full brightness
            SoundMode::R2D2Tones,
        ),
        SocialPhase::ProtectiveGuardian => (
            0.7,
            [200, 150, 0],        // amber
            0,                     // steady
            0.8,                   // fairly bright
            SoundMode::WarningHum,
        ),
    };

    // Modulate motor speed by permeability and personality movement expressiveness
    let motor_speed_factor = (base_motor * permeability * movement_expr).clamp(0.0, 1.0);

    // Modulate LED brightness by permeability and personality light expressiveness
    let led_brightness = (base_brightness * permeability * light_expr).clamp(0.0, 1.0);

    // Downgrade sound mode when combined sound expression is too low
    let sound_factor = permeability * sound_expr;
    let sound_mode = if sound_factor < 0.3 {
        base_sound.downgrade()
    } else {
        base_sound
    };

    QuadrantBehavior {
        motor_speed_factor,
        led_color: base_led,
        led_pulse_period: base_pulse,
        led_brightness,
        sound_mode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nervous_system::stimulus::{StimulusEvent, StimulusKind};

    /// Helper: build a StartleResult with a given suppression factor.
    fn result_with_factor(factor: f32) -> StartleResult {
        StartleResult {
            tension_delta: 0.5 * factor,
            suppressed: factor < 0.8,
            suppression_factor: factor,
            stimulus: StimulusEvent {
                kind: StimulusKind::LoudnessSpike,
                magnitude: 0.8,
                tick: 1,
            },
        }
    }

    // ---------------------------------------------------------------
    // I-BEHV-001: Unsuppressed (factor >= 0.9) produces full response
    // ---------------------------------------------------------------

    #[test]
    fn unsuppressed_full_response() {
        let result = result_with_factor(1.0);
        let behavior = StartleBehavior::from_startle(&result, 0.5, 0.5);

        assert_eq!(behavior.motor_override, Some((-60, -60)), "must reverse");
        assert_eq!(behavior.led_override, Some([255, 0, 0]), "must flash red");
        assert_eq!(behavior.buzzer_override, Some(880), "must chirp at 880 Hz");
        assert!(behavior.freeze_ticks >= 10, "must freeze for at least 10 ticks");
        assert!(behavior.is_visible());
    }

    #[test]
    fn unsuppressed_at_boundary() {
        let result = result_with_factor(0.9);
        let behavior = StartleBehavior::from_startle(&result, 0.5, 0.5);

        assert_eq!(behavior.motor_override, Some((-60, -60)));
        assert_eq!(behavior.led_override, Some([255, 0, 0]));
        assert_eq!(behavior.buzzer_override, Some(880));
    }

    // ---------------------------------------------------------------
    // I-BEHV-002: Partially suppressed (0.6..0.9) produces moderate
    // ---------------------------------------------------------------

    #[test]
    fn partially_suppressed_moderate_response() {
        let result = result_with_factor(0.6);
        let behavior = StartleBehavior::from_startle(&result, 0.5, 0.5);

        assert_eq!(behavior.motor_override, None, "no motor override for partial");
        assert_eq!(
            behavior.led_override,
            Some([255, 180, 0]),
            "must flash amber"
        );
        assert_eq!(
            behavior.buzzer_override,
            Some(330),
            "soft tone when sound_expressiveness > 0.3"
        );
        assert!(behavior.freeze_ticks >= 2, "must have short freeze");
        assert!(behavior.is_visible());
    }

    #[test]
    fn partially_suppressed_quiet_personality() {
        let result = result_with_factor(0.7);
        // sound_expressiveness = 0.2, below threshold of 0.3
        let behavior = StartleBehavior::from_startle(&result, 0.5, 0.2);

        assert_eq!(behavior.led_override, Some([255, 180, 0]));
        assert_eq!(
            behavior.buzzer_override, None,
            "no buzzer when sound_expressiveness <= 0.3"
        );
    }

    // ---------------------------------------------------------------
    // I-BEHV-003: Heavily suppressed (< 0.45) produces no visible
    // ---------------------------------------------------------------

    #[test]
    fn heavily_suppressed_no_response() {
        let result = result_with_factor(0.3);
        let behavior = StartleBehavior::from_startle(&result, 0.5, 0.5);

        assert_eq!(behavior.motor_override, None);
        assert_eq!(behavior.led_override, None);
        assert_eq!(behavior.buzzer_override, None);
        assert_eq!(behavior.freeze_ticks, 0);
        assert!(!behavior.is_visible());
    }

    #[test]
    fn heavily_suppressed_at_floor() {
        let result = result_with_factor(0.3); // minimum per STRT-003
        let behavior = StartleBehavior::from_startle(&result, 1.0, 1.0);

        assert!(!behavior.is_visible(), "must be invisible even with max expressiveness");
    }

    // ---------------------------------------------------------------
    // I-BEHV-004: Personality modulates behavior
    // ---------------------------------------------------------------

    #[test]
    fn personality_modulates_freeze_ticks() {
        let result = result_with_factor(1.0);

        let expressive = StartleBehavior::from_startle(&result, 1.0, 0.5);
        let reserved = StartleBehavior::from_startle(&result, 0.2, 0.5);

        // 25.0 * 1.0 = 25 ticks vs 25.0 * 0.2 = 5.0, clamped to max(10) = 10
        assert!(
            expressive.freeze_ticks > reserved.freeze_ticks,
            "expressive personality should freeze longer: {} vs {}",
            expressive.freeze_ticks,
            reserved.freeze_ticks,
        );
        assert_eq!(expressive.freeze_ticks, 25);
        assert_eq!(reserved.freeze_ticks, 10); // max(5.0, 10.0) = 10
    }

    #[test]
    fn personality_modulates_subtle_led() {
        let result = result_with_factor(0.5); // mostly-suppressed tier

        let expressive = StartleBehavior::from_startle(&result, 0.8, 0.5);
        let reserved = StartleBehavior::from_startle(&result, 0.3, 0.5);

        assert_eq!(
            expressive.led_override,
            Some([100, 100, 50]),
            "expressive personality gets subtle LED at mostly-suppressed"
        );
        assert_eq!(
            reserved.led_override, None,
            "reserved personality gets no LED at mostly-suppressed"
        );
    }

    #[test]
    fn personality_modulates_partial_freeze() {
        let result = result_with_factor(0.7);

        let high = StartleBehavior::from_startle(&result, 1.0, 0.5);
        let low = StartleBehavior::from_startle(&result, 0.1, 0.5);

        // 5.0 * 1.0 = 5 ticks vs 5.0 * 0.1 = 0.5, clamped to max(2) = 2
        assert_eq!(high.freeze_ticks, 5);
        assert_eq!(low.freeze_ticks, 2);
    }

    // ===============================================================
    // Per-Quadrant Behavioral Profiles (Issue #27)
    // ===============================================================

    // ---------------------------------------------------------------
    // ShyObserver: slow motors, dim LEDs, silent sound
    // ---------------------------------------------------------------

    #[test]
    fn shy_observer_slow_motors_dim_leds_silent() {
        let qb = quadrant_behavior(
            SocialPhase::ShyObserver,
            0.5,  // moderate permeability
            0.5,  // neutral movement expressiveness
            0.5,  // neutral sound expressiveness
            0.5,  // neutral light expressiveness
        );

        // Motor should be slow: base 0.3 * 0.5 * 0.5 = 0.075
        assert!(
            qb.motor_speed_factor < 0.15,
            "ShyObserver motors should be slow: {}",
            qb.motor_speed_factor
        );

        // LED should be dim
        assert!(
            qb.led_brightness < 0.15,
            "ShyObserver LEDs should be dim: {}",
            qb.led_brightness
        );

        // LED color should be blue-white
        assert_eq!(qb.led_color, [100, 120, 180]);

        // Pulse should be slow (60 ticks)
        assert_eq!(qb.led_pulse_period, 60);

        // Sound: Silent base, downgraded stays Silent since 0.5*0.5=0.25 < 0.3
        assert_eq!(qb.sound_mode, SoundMode::Silent);
    }

    // ---------------------------------------------------------------
    // QuietlyBeloved: full motors, bright LEDs, R2D2 tones
    // ---------------------------------------------------------------

    #[test]
    fn quietly_beloved_full_motors_bright_leds_r2d2() {
        let qb = quadrant_behavior(
            SocialPhase::QuietlyBeloved,
            0.9,  // high permeability
            0.8,  // expressive movement
            0.8,  // expressive sound
            0.8,  // expressive light
        );

        // Motor should be high: base 1.0 * 0.9 * 0.8 = 0.72
        assert!(
            qb.motor_speed_factor > 0.5,
            "QuietlyBeloved motors should be full: {}",
            qb.motor_speed_factor
        );

        // LED should be bright
        assert!(
            qb.led_brightness > 0.5,
            "QuietlyBeloved LEDs should be bright: {}",
            qb.led_brightness
        );

        // LED color should be warm rich blue
        assert_eq!(qb.led_color, [80, 160, 220]);

        // No pulse (steady)
        assert_eq!(qb.led_pulse_period, 0);

        // R2D2 tones (0.9 * 0.8 = 0.72 >= 0.3 so not downgraded)
        assert_eq!(qb.sound_mode, SoundMode::R2D2Tones);
    }

    // ---------------------------------------------------------------
    // ProtectiveGuardian: alert motors, amber LEDs, warning
    // ---------------------------------------------------------------

    #[test]
    fn protective_guardian_alert_motors_amber_leds_warning() {
        let qb = quadrant_behavior(
            SocialPhase::ProtectiveGuardian,
            0.5,  // moderate permeability
            0.7,  // fairly expressive movement
            0.7,  // fairly expressive sound
            0.7,  // fairly expressive light
        );

        // Motor: base 0.7 * 0.5 * 0.7 = 0.245
        assert!(
            qb.motor_speed_factor > 0.2,
            "ProtectiveGuardian motors should be alert: {}",
            qb.motor_speed_factor
        );
        assert!(
            qb.motor_speed_factor < 0.5,
            "ProtectiveGuardian motors should be moderate, not full: {}",
            qb.motor_speed_factor
        );

        // LED color should be amber
        assert_eq!(qb.led_color, [200, 150, 0]);

        // Steady (no pulse)
        assert_eq!(qb.led_pulse_period, 0);

        // WarningHum (0.5 * 0.7 = 0.35 >= 0.3, not downgraded)
        assert_eq!(qb.sound_mode, SoundMode::WarningHum);
    }

    // ---------------------------------------------------------------
    // StartledRetreat: reverse motors, red LEDs, fast pulse
    // ---------------------------------------------------------------

    #[test]
    fn startled_retreat_reverse_motors_red_leds_fast_pulse() {
        let qb = quadrant_behavior(
            SocialPhase::StartledRetreat,
            0.1,  // low permeability (reflexive)
            0.5,  // neutral
            0.5,  // neutral
            0.5,  // neutral
        );

        // Motor: base 1.0 * 0.1 * 0.5 = 0.05 (permeability is very low in SR)
        // Note: the motor_speed_factor indicates *intensity* of the retreat,
        // scaled down by low permeability
        assert!(
            qb.motor_speed_factor <= 0.1,
            "StartledRetreat at low permeability should have low motor factor: {}",
            qb.motor_speed_factor
        );

        // LED color should be red
        assert_eq!(qb.led_color, [220, 30, 20]);

        // Fast pulse
        assert_eq!(qb.led_pulse_period, 10);

        // Sound: WarningHum downgraded to R2D2Tones when 0.1 * 0.5 = 0.05 < 0.3
        assert_eq!(qb.sound_mode, SoundMode::R2D2Tones);
    }

    // ---------------------------------------------------------------
    // I-BEHV-006: Permeability=0 produces minimal expression
    // ---------------------------------------------------------------

    #[test]
    fn permeability_zero_minimal_expression_all_quadrants() {
        let phases = [
            SocialPhase::ShyObserver,
            SocialPhase::StartledRetreat,
            SocialPhase::QuietlyBeloved,
            SocialPhase::ProtectiveGuardian,
        ];

        for phase in &phases {
            let qb = quadrant_behavior(*phase, 0.0, 1.0, 1.0, 1.0);

            assert!(
                qb.motor_speed_factor < f32::EPSILON,
                "{:?}: motor should be ~0 at permeability=0, got {}",
                phase, qb.motor_speed_factor
            );
            assert!(
                qb.led_brightness < f32::EPSILON,
                "{:?}: LED brightness should be ~0 at permeability=0, got {}",
                phase, qb.led_brightness
            );
            // Sound should be downgraded (0.0 * 1.0 = 0.0 < 0.3)
            let base_sound = match phase {
                SocialPhase::ShyObserver => SoundMode::Silent,
                SocialPhase::StartledRetreat => SoundMode::WarningHum,
                SocialPhase::QuietlyBeloved => SoundMode::R2D2Tones,
                SocialPhase::ProtectiveGuardian => SoundMode::WarningHum,
            };
            assert_eq!(
                qb.sound_mode,
                base_sound.downgrade(),
                "{:?}: sound should be downgraded at permeability=0",
                phase
            );
        }
    }

    // ---------------------------------------------------------------
    // I-BEHV-007: Personality expressiveness modulates output
    // ---------------------------------------------------------------

    #[test]
    fn personality_expressiveness_modulates_motor() {
        let high = quadrant_behavior(SocialPhase::QuietlyBeloved, 0.8, 1.0, 0.5, 0.5);
        let low = quadrant_behavior(SocialPhase::QuietlyBeloved, 0.8, 0.2, 0.5, 0.5);

        assert!(
            high.motor_speed_factor > low.motor_speed_factor,
            "High movement expressiveness should produce faster motors: {} vs {}",
            high.motor_speed_factor, low.motor_speed_factor
        );
    }

    #[test]
    fn personality_expressiveness_modulates_led_brightness() {
        let high = quadrant_behavior(SocialPhase::QuietlyBeloved, 0.8, 0.5, 0.5, 1.0);
        let low = quadrant_behavior(SocialPhase::QuietlyBeloved, 0.8, 0.5, 0.5, 0.2);

        assert!(
            high.led_brightness > low.led_brightness,
            "High light expressiveness should produce brighter LEDs: {} vs {}",
            high.led_brightness, low.led_brightness
        );
    }

    #[test]
    fn personality_expressiveness_modulates_sound() {
        // With high sound expressiveness: 0.5 * 0.8 = 0.4 >= 0.3 -> no downgrade
        let high = quadrant_behavior(SocialPhase::QuietlyBeloved, 0.5, 0.5, 0.8, 0.5);
        assert_eq!(high.sound_mode, SoundMode::R2D2Tones);

        // With low sound expressiveness: 0.5 * 0.2 = 0.1 < 0.3 -> downgrade
        let low = quadrant_behavior(SocialPhase::QuietlyBeloved, 0.5, 0.5, 0.2, 0.5);
        assert_eq!(low.sound_mode, SoundMode::Hum);
    }

    #[test]
    fn sound_mode_downgrade_chain() {
        // Verify the full downgrade chain: WarningHum -> R2D2 -> Hum -> SoftTone -> Silent -> Silent
        assert_eq!(SoundMode::WarningHum.downgrade(), SoundMode::R2D2Tones);
        assert_eq!(SoundMode::R2D2Tones.downgrade(), SoundMode::Hum);
        assert_eq!(SoundMode::Hum.downgrade(), SoundMode::SoftTone);
        assert_eq!(SoundMode::SoftTone.downgrade(), SoundMode::Silent);
        assert_eq!(SoundMode::Silent.downgrade(), SoundMode::Silent);
    }

    #[test]
    fn sound_mode_rank_ordering() {
        // Verify rank ordering: Silent < SoftTone < Hum < R2D2Tones < WarningHum
        assert!(SoundMode::Silent.rank() < SoundMode::SoftTone.rank());
        assert!(SoundMode::SoftTone.rank() < SoundMode::Hum.rank());
        assert!(SoundMode::Hum.rank() < SoundMode::R2D2Tones.rank());
        assert!(SoundMode::R2D2Tones.rank() < SoundMode::WarningHum.rank());
    }

    #[test]
    fn quadrant_behavior_values_bounded() {
        // All quadrants with extreme inputs should produce bounded outputs
        let phases = [
            SocialPhase::ShyObserver,
            SocialPhase::StartledRetreat,
            SocialPhase::QuietlyBeloved,
            SocialPhase::ProtectiveGuardian,
        ];

        for phase in &phases {
            for perm in &[0.0_f32, 0.5, 1.0] {
                for expr in &[0.0_f32, 0.5, 1.0] {
                    let qb = quadrant_behavior(*phase, *perm, *expr, *expr, *expr);
                    assert!(
                        qb.motor_speed_factor >= 0.0 && qb.motor_speed_factor <= 1.0,
                        "{:?} perm={} expr={}: motor out of bounds: {}",
                        phase, perm, expr, qb.motor_speed_factor
                    );
                    assert!(
                        qb.led_brightness >= 0.0 && qb.led_brightness <= 1.0,
                        "{:?} perm={} expr={}: brightness out of bounds: {}",
                        phase, perm, expr, qb.led_brightness
                    );
                }
            }
        }
    }
}
