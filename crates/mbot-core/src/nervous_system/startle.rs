//! Startle processor — attenuated tension computation.
//!
//! Invariants:
//! - I-STRT-001: no_std compatible
//! - I-STRT-003: Suppression factor from map (never below 0.3)
//! - I-STRT-005: Personality startle_sensitivity scales base delta

use super::stimulus::StimulusEvent;
use super::suppression::SuppressionMap;

/// Result of processing a stimulus through the startle pipeline.
#[derive(Clone, Copy, Debug)]
pub struct StartleResult {
    /// Attenuated tension delta to add to current tension.
    pub tension_delta: f32,
    /// Whether this stimulus was meaningfully suppressed (factor < 0.8).
    pub suppressed: bool,
    /// The suppression factor that was applied [0.3, 1.0].
    pub suppression_factor: f32,
    /// The original stimulus event.
    pub stimulus: StimulusEvent,
}

/// Computes attenuated tension deltas from stimulus events.
pub struct StartleProcessor {
    pub suppression_map: SuppressionMap,
}

impl StartleProcessor {
    pub fn new() -> Self {
        Self {
            suppression_map: SuppressionMap::new(),
        }
    }

    /// Compute the tension delta from a stimulus event, after applying suppression.
    ///
    /// `startle_sensitivity` is from the robot's personality [0.0, 1.0+].
    pub fn process_stimulus(
        &self,
        event: &StimulusEvent,
        context_hash: u32,
        startle_sensitivity: f32,
    ) -> StartleResult {
        let suppression = self.suppression_map.lookup(event.kind, context_hash);

        // Base tension delta from stimulus magnitude, scaled by personality
        let base_delta = event.magnitude * startle_sensitivity;

        // Apply suppression
        let attenuated_delta = base_delta * suppression;

        StartleResult {
            tension_delta: attenuated_delta,
            suppressed: suppression < 0.8,
            suppression_factor: suppression,
            stimulus: *event,
        }
    }
}

impl Default for StartleProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nervous_system::stimulus::{StimulusEvent, StimulusKind};
    use crate::nervous_system::suppression::SuppressionRule;

    fn event(kind: StimulusKind, magnitude: f32) -> StimulusEvent {
        StimulusEvent { kind, magnitude, tick: 1 }
    }

    #[test]
    fn test_unsuppressed_full_delta() {
        let proc = StartleProcessor::new();
        let result = proc.process_stimulus(
            &event(StimulusKind::LoudnessSpike, 0.8),
            42,
            0.7,
        );
        // 0.8 * 0.7 * 1.0 = 0.56
        assert!((result.tension_delta - 0.56).abs() < 0.001);
        assert!(!result.suppressed);
        assert_eq!(result.suppression_factor, 1.0);
    }

    #[test]
    fn test_suppressed_reduced_delta() {
        let mut proc = StartleProcessor::new();
        proc.suppression_map.upsert(SuppressionRule {
            stimulus_kind: StimulusKind::LoudnessSpike,
            context_hash: 42,
            suppression_factor: 0.3,
            observation_count: 10,
            last_updated_tick: 100,
        });
        let result = proc.process_stimulus(
            &event(StimulusKind::LoudnessSpike, 0.8),
            42,
            0.7,
        );
        // 0.8 * 0.7 * 0.3 = 0.168
        assert!((result.tension_delta - 0.168).abs() < 0.001);
        assert!(result.suppressed);
        assert_eq!(result.suppression_factor, 0.3);
    }

    #[test]
    fn test_different_context_no_suppression() {
        let mut proc = StartleProcessor::new();
        proc.suppression_map.upsert(SuppressionRule {
            stimulus_kind: StimulusKind::LoudnessSpike,
            context_hash: 42,
            suppression_factor: 0.3,
            observation_count: 10,
            last_updated_tick: 100,
        });
        let result = proc.process_stimulus(
            &event(StimulusKind::LoudnessSpike, 0.8),
            99, // different context
            0.7,
        );
        // 0.8 * 0.7 * 1.0 = 0.56 (no rule for context 99)
        assert!((result.tension_delta - 0.56).abs() < 0.001);
        assert!(!result.suppressed);
    }

    #[test]
    fn test_personality_modulates_delta() {
        let proc = StartleProcessor::new();
        let high = proc.process_stimulus(
            &event(StimulusKind::LoudnessSpike, 0.8),
            42,
            1.0,
        );
        let low = proc.process_stimulus(
            &event(StimulusKind::LoudnessSpike, 0.8),
            42,
            0.3,
        );
        assert!((high.tension_delta - 0.8).abs() < 0.001);
        assert!((low.tension_delta - 0.24).abs() < 0.001);
        assert!(high.tension_delta > low.tension_delta);
    }
}
