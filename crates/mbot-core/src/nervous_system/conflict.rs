//! Classification conflict resolution between reflexive and deliberative quadrants.
//!
//! When the core's real-time quadrant assessment (reflexive) diverges from the
//! companion's context-informed recommendation (deliberative), a conflict arises.
//! Conflicts produce visible hesitation behavior: slowed motors, amber LEDs, low hum.
//!
//! # Resolution rules
//!
//! - Same quadrant: no conflict, 0 hesitation.
//! - High tension (>0.6): reflexive wins (safety first).
//! - High coherence (>0.7): deliberative wins (trust experience).
//! - Otherwise: reflexive wins (default to caution).
//!
//! # Invariants
//!
//! - **I-CONF-001**: no_std compatible (uses only f32, u16, u32, u64, enums).
//! - **I-CONF-002**: High tension (>0.6) -> reflexive wins (safety first).
//! - **I-CONF-003**: High coherence (>0.7) -> deliberative wins (trust experience).
//! - **I-CONF-004**: Otherwise -> reflexive wins (default to caution).

use crate::coherence::SocialPhase;

/// Record of a conflict between reflexive and deliberative assessments.
#[derive(Clone, Debug)]
pub struct ConflictEvent {
    /// Tick at which the conflict occurred.
    pub tick: u64,
    /// Hash of the context key at the time of conflict.
    pub context_hash: u32,
    /// Quadrant from the core's real-time assessment.
    pub reflexive_quadrant: SocialPhase,
    /// Quadrant from the companion's context-informed recommendation.
    pub deliberative_quadrant: SocialPhase,
    /// Winning quadrant after resolution.
    pub resolution: SocialPhase,
    /// Number of ticks the hesitation behavior should last.
    pub hesitation_ticks: u16,
}

/// Resolves conflicts between reflexive (core) and deliberative (companion)
/// quadrant assessments.
#[derive(Clone, Debug)]
pub struct ConflictResolver {
    /// Base hesitation duration in ticks (default: 10 = 1 second at 10Hz).
    pub base_hesitation_ticks: u16,
}

impl ConflictResolver {
    pub fn new() -> Self {
        Self {
            base_hesitation_ticks: 10,
        }
    }

    /// Resolve a conflict between reflexive and deliberative quadrant assessments.
    ///
    /// Returns `(winning_quadrant, hesitation_ticks)`.
    /// If both quadrants agree, returns 0 hesitation (no conflict).
    ///
    /// # Resolution priority
    ///
    /// 1. Same quadrant -> no conflict, 0 hesitation.
    /// 2. `tension > 0.6` -> reflexive wins (I-CONF-002: safety first).
    /// 3. `context_coherence > 0.7` -> deliberative wins (I-CONF-003: trust experience).
    /// 4. Otherwise -> reflexive wins (I-CONF-004: default to caution).
    pub fn resolve(
        &self,
        reflexive: SocialPhase,
        deliberative: SocialPhase,
        context_coherence: f32,
        tension: f32,
    ) -> (SocialPhase, u16) {
        if reflexive == deliberative {
            return (reflexive, 0);
        }

        let hesitation = self.base_hesitation_ticks;

        let winner = if tension > 0.6 {
            reflexive // Safety first
        } else if context_coherence > 0.7 {
            deliberative // Trust experience
        } else {
            reflexive // Default to caution
        };

        (winner, hesitation)
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Hesitation behavior during conflict resolution.
///
/// When a conflict is detected, the robot's outputs are modulated to
/// produce a visible "thinking" or "uncertain" state: motors slow down,
/// LEDs shift to amber, and a low hum is emitted.
#[derive(Clone, Debug)]
pub struct HesitationBehavior {
    /// Motor speed multiplier during hesitation (0.3 = 30% speed).
    pub motor_scale: f32,
    /// LED color during hesitation [R, G, B].
    pub led_color: [u8; 3],
    /// Buzzer frequency during hesitation (Hz).
    pub buzzer_hz: u16,
}

impl HesitationBehavior {
    /// Standard hesitation behavior: slow motors, amber LEDs, low hum.
    pub fn standard() -> Self {
        Self {
            motor_scale: 0.3,
            led_color: [200, 150, 0], // warm amber
            buzzer_hz: 220,           // low A note
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_phases() -> [SocialPhase; 4] {
        [
            SocialPhase::ShyObserver,
            SocialPhase::StartledRetreat,
            SocialPhase::QuietlyBeloved,
            SocialPhase::ProtectiveGuardian,
        ]
    }

    #[test]
    fn test_no_conflict_same_quadrant() {
        let resolver = ConflictResolver::new();
        for phase in &all_phases() {
            let (winner, hesitation) = resolver.resolve(*phase, *phase, 0.5, 0.5);
            assert_eq!(winner, *phase);
            assert_eq!(hesitation, 0, "same quadrant should produce 0 hesitation");
        }
    }

    #[test]
    fn test_high_tension_reflexive_wins() {
        // I-CONF-002: tension > 0.6 -> reflexive wins
        let resolver = ConflictResolver::new();
        let (winner, hesitation) = resolver.resolve(
            SocialPhase::StartledRetreat,   // reflexive
            SocialPhase::QuietlyBeloved,     // deliberative
            0.9,  // high coherence (would normally favor deliberative)
            0.7,  // high tension -> reflexive overrides
        );
        assert_eq!(winner, SocialPhase::StartledRetreat);
        assert_eq!(hesitation, 10);
    }

    #[test]
    fn test_high_coherence_deliberative_wins() {
        // I-CONF-003: coherence > 0.7, tension <= 0.6 -> deliberative wins
        let resolver = ConflictResolver::new();
        let (winner, hesitation) = resolver.resolve(
            SocialPhase::ShyObserver,        // reflexive
            SocialPhase::QuietlyBeloved,     // deliberative
            0.8,  // high coherence -> trust experience
            0.3,  // low tension
        );
        assert_eq!(winner, SocialPhase::QuietlyBeloved);
        assert_eq!(hesitation, 10);
    }

    #[test]
    fn test_default_caution_reflexive() {
        // I-CONF-004: coherence <= 0.7, tension <= 0.6 -> reflexive wins
        let resolver = ConflictResolver::new();
        let (winner, hesitation) = resolver.resolve(
            SocialPhase::ShyObserver,        // reflexive
            SocialPhase::QuietlyBeloved,     // deliberative
            0.5,  // not high enough coherence
            0.4,  // not high tension either
        );
        assert_eq!(winner, SocialPhase::ShyObserver);
        assert_eq!(hesitation, 10);
    }

    #[test]
    fn test_hesitation_ticks_on_conflict() {
        // Any conflict (different quadrants) should produce base_hesitation_ticks
        let resolver = ConflictResolver::new();
        let (_, hesitation) = resolver.resolve(
            SocialPhase::ShyObserver,
            SocialPhase::StartledRetreat,
            0.5,
            0.5,
        );
        assert_eq!(hesitation, resolver.base_hesitation_ticks);
    }

    #[test]
    fn test_custom_hesitation_duration() {
        let mut resolver = ConflictResolver::new();
        resolver.base_hesitation_ticks = 20;

        let (_, hesitation) = resolver.resolve(
            SocialPhase::ShyObserver,
            SocialPhase::QuietlyBeloved,
            0.5,
            0.5,
        );
        assert_eq!(hesitation, 20);
    }

    #[test]
    fn test_all_quadrant_pairs() {
        // Every pair of different quadrants should produce hesitation > 0
        let resolver = ConflictResolver::new();
        let phases = all_phases();

        for &a in &phases {
            for &b in &phases {
                let (_, hesitation) = resolver.resolve(a, b, 0.5, 0.5);
                if a == b {
                    assert_eq!(hesitation, 0, "{:?} == {:?} should be 0", a, b);
                } else {
                    assert!(hesitation > 0, "{:?} != {:?} should produce hesitation", a, b);
                }
            }
        }
    }

    #[test]
    fn test_conflict_event_creation() {
        let resolver = ConflictResolver::new();
        let reflexive = SocialPhase::ShyObserver;
        let deliberative = SocialPhase::QuietlyBeloved;
        let (resolution, hesitation_ticks) = resolver.resolve(reflexive, deliberative, 0.5, 0.3);

        let event = ConflictEvent {
            tick: 42,
            context_hash: 0xDEADBEEF,
            reflexive_quadrant: reflexive,
            deliberative_quadrant: deliberative,
            resolution,
            hesitation_ticks,
        };

        assert_eq!(event.tick, 42);
        assert_eq!(event.context_hash, 0xDEADBEEF);
        assert_eq!(event.reflexive_quadrant, SocialPhase::ShyObserver);
        assert_eq!(event.deliberative_quadrant, SocialPhase::QuietlyBeloved);
        assert_eq!(event.resolution, SocialPhase::ShyObserver); // default caution
        assert!(event.hesitation_ticks > 0);
    }

    #[test]
    fn test_hesitation_behavior_standard() {
        let hb = HesitationBehavior::standard();
        assert!((hb.motor_scale - 0.3).abs() < f32::EPSILON);
        assert_eq!(hb.led_color, [200, 150, 0]);
        assert_eq!(hb.buzzer_hz, 220);
    }

    #[test]
    fn test_tension_boundary_exactly_0_6() {
        // tension == 0.6 is NOT > 0.6, so falls through to coherence check
        let resolver = ConflictResolver::new();
        let (winner, _) = resolver.resolve(
            SocialPhase::StartledRetreat,
            SocialPhase::QuietlyBeloved,
            0.8,  // high coherence
            0.6,  // exactly 0.6, NOT > 0.6
        );
        // Should fall through to coherence check -> deliberative wins
        assert_eq!(winner, SocialPhase::QuietlyBeloved);
    }

    #[test]
    fn test_coherence_boundary_exactly_0_7() {
        // coherence == 0.7 is NOT > 0.7, so falls through to default caution
        let resolver = ConflictResolver::new();
        let (winner, _) = resolver.resolve(
            SocialPhase::ShyObserver,
            SocialPhase::QuietlyBeloved,
            0.7,  // exactly 0.7, NOT > 0.7
            0.3,  // low tension
        );
        // Should fall through to default caution -> reflexive wins
        assert_eq!(winner, SocialPhase::ShyObserver);
    }
}
