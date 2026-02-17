//! Suppression Learner — reads classified StimulusLogEntry items and
//! generates, updates, or removes SuppressionRules.
//!
//! Lives in mbot-companion (std available, gated under `brain` feature).
//! Reads observation data from mbot-core's stimulus log and produces rule
//! change-sets that the caller pushes into mbot-core's SuppressionMap.
//!
//! # Contract Compliance
//! - **ARCH-001**: All learning logic lives in mbot-companion, not mbot-core
//! - **ARCH-002**: Brain is advisory; companion suggests rules, core enforces them
//! - **I-STRT-003**: SuppressionRule factors are clamped to [0.3, 1.0] by core

use std::collections::HashMap;

use mbot_core::nervous_system::stimulus::StimulusKind;
use mbot_core::nervous_system::stimulus_log::{PostStimulusOutcome, StimulusLogEntry};
use mbot_core::nervous_system::suppression::SuppressionRule;

/// Configuration for the suppression learner.
#[derive(Debug, Clone)]
pub struct SuppressionLearnerConfig {
    /// Minimum number of observations before generating a rule (default: 5)
    pub min_observations: u16,
    /// Benign ratio threshold above which we create/update a suppression rule (default: 0.8)
    pub benign_threshold: f32,
    /// Benign ratio threshold below which we remove a rule as harmful (default: 0.4)
    pub harmful_threshold: f32,
    /// Maximum suppression applied (factor floor), before personality modulation (default: 0.7)
    pub max_suppression: f32,
    /// Ticks between learning passes (default: 500)
    pub relearn_interval_ticks: u64,
}

impl Default for SuppressionLearnerConfig {
    fn default() -> Self {
        Self {
            min_observations: 5,
            benign_threshold: 0.8,
            harmful_threshold: 0.4,
            max_suppression: 0.7,
            relearn_interval_ticks: 500,
        }
    }
}

/// Accumulated observation statistics for a (StimulusKind, context_hash) pair.
#[derive(Debug, Clone, Default)]
struct ObservationStats {
    benign_count: u32,
    harmful_count: u32,
}

impl ObservationStats {
    fn total(&self) -> u32 {
        self.benign_count + self.harmful_count
    }

    fn benign_ratio(&self) -> f32 {
        let total = self.total();
        if total == 0 {
            return 0.0;
        }
        self.benign_count as f32 / total as f32
    }
}

/// The output of a learning pass -- rule mutations to apply to the core SuppressionMap.
#[derive(Debug, Clone)]
pub struct LearningResult {
    /// New or updated suppression rules to push into the core map.
    pub rules_to_upsert: Vec<SuppressionRule>,
    /// (StimulusKind, context_hash) pairs whose rules should be removed from the core map.
    pub rules_to_remove: Vec<(StimulusKind, u32)>,
}

/// Reads classified StimulusLogEntry items and generates/updates/removes
/// SuppressionRules based on accumulated benign vs. harmful observations.
///
/// # Usage
///
/// 1. Call `ingest()` each tick with newly classified entries from `StimulusLog::drain_classified()`.
/// 2. Call `should_learn(current_tick)` to check if a learning pass is due.
/// 3. Call `learn(current_tick, curiosity_drive, startle_sensitivity)` to produce a `LearningResult`.
/// 4. Apply the result to the core `SuppressionMap` via `upsert()` / `remove()`.
pub struct SuppressionLearner {
    config: SuppressionLearnerConfig,
    last_learn_tick: u64,
    /// Accumulated observations per (StimulusKind, context_hash).
    observations: HashMap<(StimulusKind, u32), ObservationStats>,
}

impl SuppressionLearner {
    /// Create a new learner with the given configuration.
    pub fn new(config: SuppressionLearnerConfig) -> Self {
        Self {
            config,
            last_learn_tick: 0,
            observations: HashMap::new(),
        }
    }

    /// Accumulate benign/harmful counts from classified log entries.
    ///
    /// Only entries with `Benign` or `Harmful` outcome are counted;
    /// `Pending` entries are silently skipped.
    pub fn ingest(&mut self, entries: &[StimulusLogEntry]) {
        for entry in entries {
            let key = (entry.stimulus.kind, entry.context_hash);
            let stats = self.observations.entry(key).or_default();
            match entry.post_stimulus_outcome {
                PostStimulusOutcome::Benign => stats.benign_count += 1,
                PostStimulusOutcome::Harmful => stats.harmful_count += 1,
                PostStimulusOutcome::Pending => {}
            }
        }
    }

    /// Returns true if enough ticks have elapsed since the last learning pass.
    pub fn should_learn(&self, current_tick: u64) -> bool {
        current_tick.saturating_sub(self.last_learn_tick) >= self.config.relearn_interval_ticks
    }

    /// Evaluate all accumulated observations and produce a set of rule mutations.
    ///
    /// Personality modulation:
    /// - `curiosity_drive` lowers the effective minimum observations (curious robots
    ///   learn faster from fewer samples).
    /// - `startle_sensitivity` reduces the effective maximum suppression (sensitive
    ///   robots retain more of their startle response).
    ///
    /// Updates `last_learn_tick` to `current_tick`.
    pub fn learn(
        &mut self,
        current_tick: u64,
        curiosity_drive: f32,
        startle_sensitivity: f32,
    ) -> LearningResult {
        self.last_learn_tick = current_tick;

        let mut result = LearningResult {
            rules_to_upsert: Vec::new(),
            rules_to_remove: Vec::new(),
        };

        // Personality-modulated effective thresholds.
        // Guard against division by zero: clamp curiosity_drive to [0.01, 1.0].
        let clamped_curiosity = curiosity_drive.clamp(0.01, 1.0);
        let effective_min_obs =
            (self.config.min_observations as f32 / clamped_curiosity).ceil() as u16;

        // Sensitive robots retain more startle: lower effective max suppression.
        let clamped_startle = startle_sensitivity.clamp(0.0, 1.0);
        let effective_max_suppression =
            self.config.max_suppression * (1.0 - clamped_startle * 0.3);

        for (&(kind, context_hash), stats) in &self.observations {
            // Skip pairs without enough observations.
            if stats.total() < effective_min_obs as u32 {
                continue;
            }

            let benign_ratio = stats.benign_ratio();

            if benign_ratio >= self.config.benign_threshold {
                // Mostly benign -- create or strengthen suppression rule.
                let raw_factor = 1.0 - (benign_ratio * effective_max_suppression);
                let factor = raw_factor.clamp(0.3, 1.0);

                result.rules_to_upsert.push(SuppressionRule {
                    stimulus_kind: kind,
                    context_hash,
                    suppression_factor: factor,
                    observation_count: stats.total().min(u16::MAX as u32) as u16,
                    last_updated_tick: current_tick,
                });
            } else if benign_ratio < self.config.harmful_threshold {
                // Mostly harmful -- remove any existing rule so the robot stays alert.
                result.rules_to_remove.push((kind, context_hash));
            }
            // Between harmful_threshold and benign_threshold: no action (inconclusive).
        }

        result
    }

    /// Read-only access to accumulated observation stats (useful for diagnostics).
    pub fn observation_count(&self, kind: StimulusKind, context_hash: u32) -> (u32, u32) {
        self.observations
            .get(&(kind, context_hash))
            .map(|s| (s.benign_count, s.harmful_count))
            .unwrap_or((0, 0))
    }

    /// Total number of tracked (kind, context_hash) pairs.
    pub fn tracked_pairs(&self) -> usize {
        self.observations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mbot_core::nervous_system::stimulus::StimulusEvent;

    /// Helper to create a classified log entry.
    fn make_entry(
        kind: StimulusKind,
        context_hash: u32,
        outcome: PostStimulusOutcome,
    ) -> StimulusLogEntry {
        StimulusLogEntry {
            stimulus: StimulusEvent {
                kind,
                magnitude: 0.5,
                tick: 100,
            },
            context_hash,
            suppression_applied: 1.0,
            tension_delta: 0.2,
            post_stimulus_outcome: outcome,
        }
    }

    /// Helper to create N benign entries for a given kind + context.
    fn benign_entries(kind: StimulusKind, hash: u32, count: usize) -> Vec<StimulusLogEntry> {
        (0..count)
            .map(|_| make_entry(kind, hash, PostStimulusOutcome::Benign))
            .collect()
    }

    /// Helper to create N harmful entries for a given kind + context.
    fn harmful_entries(kind: StimulusKind, hash: u32, count: usize) -> Vec<StimulusLogEntry> {
        (0..count)
            .map(|_| make_entry(kind, hash, PostStimulusOutcome::Harmful))
            .collect()
    }

    // -----------------------------------------------------------------------
    // Test 1: Generate rule from benign observations
    // -----------------------------------------------------------------------
    #[test]
    fn test_generate_rule_from_benign_observations() {
        let config = SuppressionLearnerConfig {
            min_observations: 5,
            benign_threshold: 0.8,
            harmful_threshold: 0.4,
            max_suppression: 0.7,
            relearn_interval_ticks: 500,
        };
        let mut learner = SuppressionLearner::new(config);

        // Ingest 10 benign entries (100% benign ratio, well above 0.8 threshold).
        let entries = benign_entries(StimulusKind::LoudnessSpike, 42, 10);
        learner.ingest(&entries);

        // Default personality: curiosity=0.5, startle=0.5
        let result = learner.learn(1000, 0.5, 0.5);

        assert_eq!(result.rules_to_upsert.len(), 1, "Should produce one upsert rule");
        assert!(result.rules_to_remove.is_empty(), "Should have no removals");

        let rule = &result.rules_to_upsert[0];
        assert_eq!(rule.stimulus_kind, StimulusKind::LoudnessSpike);
        assert_eq!(rule.context_hash, 42);
        assert!(
            rule.suppression_factor >= 0.3 && rule.suppression_factor <= 1.0,
            "Factor {} should be in [0.3, 1.0]",
            rule.suppression_factor
        );
        assert_eq!(rule.last_updated_tick, 1000);
    }

    // -----------------------------------------------------------------------
    // Test 2: Skip when insufficient observations
    // -----------------------------------------------------------------------
    #[test]
    fn test_skip_with_insufficient_observations() {
        let config = SuppressionLearnerConfig {
            min_observations: 5,
            ..Default::default()
        };
        let mut learner = SuppressionLearner::new(config);

        // Only 3 benign entries (below min_observations=5 even with curiosity modulation).
        let entries = benign_entries(StimulusKind::BrightnessSpike, 99, 3);
        learner.ingest(&entries);

        // curiosity_drive=0.5 => effective_min_obs = ceil(5/0.5) = 10
        // 3 < 10 => skip
        let result = learner.learn(1000, 0.5, 0.5);

        assert!(result.rules_to_upsert.is_empty(), "Should not produce rules with too few observations");
        assert!(result.rules_to_remove.is_empty(), "Should not produce removals either");
    }

    // -----------------------------------------------------------------------
    // Test 3: Remove rule when harmful
    // -----------------------------------------------------------------------
    #[test]
    fn test_remove_rule_when_harmful() {
        let config = SuppressionLearnerConfig {
            min_observations: 5,
            benign_threshold: 0.8,
            harmful_threshold: 0.4,
            max_suppression: 0.7,
            relearn_interval_ticks: 500,
        };
        let mut learner = SuppressionLearner::new(config);

        // 2 benign + 8 harmful = 20% benign ratio, well below 0.4 threshold.
        let mut entries = benign_entries(StimulusKind::ProximityRush, 7, 2);
        entries.extend(harmful_entries(StimulusKind::ProximityRush, 7, 8));
        learner.ingest(&entries);

        // curiosity=1.0 => effective_min_obs = ceil(5/1.0) = 5. We have 10 total => passes.
        let result = learner.learn(1000, 1.0, 0.5);

        assert!(result.rules_to_upsert.is_empty(), "Should not create rules for harmful stimuli");
        assert_eq!(result.rules_to_remove.len(), 1, "Should remove one rule");
        assert_eq!(result.rules_to_remove[0], (StimulusKind::ProximityRush, 7));
    }

    // -----------------------------------------------------------------------
    // Test 4: Curious robot learns faster (lower effective min_observations)
    // -----------------------------------------------------------------------
    #[test]
    fn test_curious_robot_learns_faster() {
        let config = SuppressionLearnerConfig {
            min_observations: 5,
            benign_threshold: 0.8,
            harmful_threshold: 0.4,
            max_suppression: 0.7,
            relearn_interval_ticks: 500,
        };

        // With curiosity_drive=1.0 => effective_min_obs = ceil(5/1.0) = 5
        // With curiosity_drive=0.5 => effective_min_obs = ceil(5/0.5) = 10
        // 6 observations should be enough for curious but not for neutral.

        // --- Curious robot (curiosity=1.0) ---
        let mut curious_learner = SuppressionLearner::new(config.clone());
        let entries = benign_entries(StimulusKind::LoudnessSpike, 42, 6);
        curious_learner.ingest(&entries);
        let curious_result = curious_learner.learn(1000, 1.0, 0.5);

        assert_eq!(
            curious_result.rules_to_upsert.len(),
            1,
            "Curious robot (curiosity=1.0) should learn from 6 observations"
        );

        // --- Neutral robot (curiosity=0.5) ---
        let mut neutral_learner = SuppressionLearner::new(config);
        let entries = benign_entries(StimulusKind::LoudnessSpike, 42, 6);
        neutral_learner.ingest(&entries);
        let neutral_result = neutral_learner.learn(1000, 0.5, 0.5);

        assert!(
            neutral_result.rules_to_upsert.is_empty(),
            "Neutral robot (curiosity=0.5) should NOT learn from 6 observations (needs 10)"
        );
    }

    // -----------------------------------------------------------------------
    // Test 5: Sensitive robot retains more startle (lower max_suppression)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sensitive_robot_retains_more_startle() {
        let config = SuppressionLearnerConfig {
            min_observations: 5,
            benign_threshold: 0.8,
            harmful_threshold: 0.4,
            max_suppression: 0.7,
            relearn_interval_ticks: 500,
        };

        // 10 benign entries, 100% benign ratio.
        let entries = benign_entries(StimulusKind::LoudnessSpike, 42, 10);

        // --- Low sensitivity (startle=0.0) ---
        let mut low_startle = SuppressionLearner::new(config.clone());
        low_startle.ingest(&entries);
        let low_result = low_startle.learn(1000, 1.0, 0.0);

        // --- High sensitivity (startle=1.0) ---
        let mut high_startle = SuppressionLearner::new(config);
        high_startle.ingest(&entries);
        let high_result = high_startle.learn(1000, 1.0, 1.0);

        let low_factor = low_result.rules_to_upsert[0].suppression_factor;
        let high_factor = high_result.rules_to_upsert[0].suppression_factor;

        // Sensitive robot should have a HIGHER suppression factor (less suppression,
        // retaining more of the startle response).
        assert!(
            high_factor > low_factor,
            "Sensitive robot factor ({}) should be higher than calm robot factor ({})",
            high_factor,
            low_factor
        );

        // Verify the math:
        // startle=0.0: effective_max_suppression = 0.7 * (1.0 - 0.0*0.3) = 0.7
        //   factor = 1.0 - (1.0 * 0.7) = 0.3
        // startle=1.0: effective_max_suppression = 0.7 * (1.0 - 1.0*0.3) = 0.49
        //   factor = 1.0 - (1.0 * 0.49) = 0.51
        assert!(
            (low_factor - 0.3).abs() < 0.01,
            "Low startle factor should be ~0.3, got {}",
            low_factor
        );
        assert!(
            (high_factor - 0.51).abs() < 0.01,
            "High startle factor should be ~0.51, got {}",
            high_factor
        );
    }

    // -----------------------------------------------------------------------
    // Test 6: Learning interval respected
    // -----------------------------------------------------------------------
    #[test]
    fn test_learning_interval_respected() {
        let config = SuppressionLearnerConfig {
            relearn_interval_ticks: 500,
            ..Default::default()
        };
        let learner = SuppressionLearner::new(config);

        // At tick 0, last_learn_tick=0, so should_learn(0) checks 0-0=0 < 500 => false
        assert!(
            !learner.should_learn(0),
            "Should not learn at tick 0 (interval not elapsed)"
        );
        assert!(
            !learner.should_learn(499),
            "Should not learn at tick 499 (interval not yet elapsed)"
        );
        assert!(
            learner.should_learn(500),
            "Should learn at tick 500 (interval elapsed)"
        );
        assert!(
            learner.should_learn(1000),
            "Should learn at tick 1000 (well past interval)"
        );
    }

    // -----------------------------------------------------------------------
    // Test 7: Learning resets the interval timer
    // -----------------------------------------------------------------------
    #[test]
    fn test_learn_resets_interval() {
        let config = SuppressionLearnerConfig {
            min_observations: 2,
            relearn_interval_ticks: 100,
            ..Default::default()
        };
        let mut learner = SuppressionLearner::new(config);

        assert!(learner.should_learn(100));

        // Perform a learn pass at tick 100.
        let _ = learner.learn(100, 0.5, 0.5);

        // Now should_learn should be false until tick 200.
        assert!(!learner.should_learn(150));
        assert!(learner.should_learn(200));
    }

    // -----------------------------------------------------------------------
    // Test 8: Pending entries are ignored during ingest
    // -----------------------------------------------------------------------
    #[test]
    fn test_pending_entries_ignored() {
        let config = SuppressionLearnerConfig::default();
        let mut learner = SuppressionLearner::new(config);

        let entries = vec![
            make_entry(StimulusKind::ImpactShock, 1, PostStimulusOutcome::Pending),
            make_entry(StimulusKind::ImpactShock, 1, PostStimulusOutcome::Pending),
        ];
        learner.ingest(&entries);

        let (benign, harmful) = learner.observation_count(StimulusKind::ImpactShock, 1);
        assert_eq!(benign, 0);
        assert_eq!(harmful, 0);
    }

    // -----------------------------------------------------------------------
    // Test 9: Multiple (kind, context) pairs tracked independently
    // -----------------------------------------------------------------------
    #[test]
    fn test_multiple_pairs_tracked_independently() {
        let config = SuppressionLearnerConfig {
            min_observations: 3,
            benign_threshold: 0.8,
            harmful_threshold: 0.4,
            max_suppression: 0.7,
            relearn_interval_ticks: 500,
        };
        let mut learner = SuppressionLearner::new(config);

        // Pair A: 5 benign (100% benign)
        let a = benign_entries(StimulusKind::LoudnessSpike, 10, 5);
        // Pair B: 5 harmful (0% benign)
        let b = harmful_entries(StimulusKind::BrightnessSpike, 20, 5);
        learner.ingest(&a);
        learner.ingest(&b);

        // curiosity=1.0 => effective_min_obs = 3
        let result = learner.learn(1000, 1.0, 0.5);

        assert_eq!(result.rules_to_upsert.len(), 1, "Should upsert one rule (benign pair)");
        assert_eq!(result.rules_to_upsert[0].stimulus_kind, StimulusKind::LoudnessSpike);

        assert_eq!(result.rules_to_remove.len(), 1, "Should remove one rule (harmful pair)");
        assert_eq!(result.rules_to_remove[0], (StimulusKind::BrightnessSpike, 20));
    }

    // -----------------------------------------------------------------------
    // Test 10: Inconclusive ratio produces no action
    // -----------------------------------------------------------------------
    #[test]
    fn test_inconclusive_ratio_no_action() {
        let config = SuppressionLearnerConfig {
            min_observations: 3,
            benign_threshold: 0.8,
            harmful_threshold: 0.4,
            max_suppression: 0.7,
            relearn_interval_ticks: 500,
        };
        let mut learner = SuppressionLearner::new(config);

        // 3 benign + 2 harmful = 60% benign, between 0.4 and 0.8 => inconclusive
        let mut entries = benign_entries(StimulusKind::OrientationFlip, 55, 3);
        entries.extend(harmful_entries(StimulusKind::OrientationFlip, 55, 2));
        learner.ingest(&entries);

        let result = learner.learn(1000, 1.0, 0.5);

        assert!(result.rules_to_upsert.is_empty(), "Inconclusive ratio should not upsert");
        assert!(result.rules_to_remove.is_empty(), "Inconclusive ratio should not remove");
    }
}
