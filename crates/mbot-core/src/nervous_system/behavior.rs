//! Observable behavior mapping — LED/motor/sound scaling by suppression factor.
//!
//! Maps startle results to concrete motor/LED/buzzer commands that scale
//! proportionally with how suppressed the startle response is.
//!
//! Invariants:
//! - I-STRT-001: no_std compatible
//! - I-BEHV-001: Unsuppressed (factor >= 0.9) produces full startle response
//! - I-BEHV-002: Partially suppressed (0.6..0.9) produces moderate response
//! - I-BEHV-003: Heavily suppressed (< 0.45) produces no visible response
//! - I-BEHV-004: Personality expressiveness modulates freeze/sound/LED intensity

use super::startle::StartleResult;

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
}
