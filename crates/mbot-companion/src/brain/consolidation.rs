//! Consolidation cycles -- offline review and recomputation.
//!
//! When the robot is idle for a configurable number of ticks with sufficient
//! energy and low tension, the companion runs a full review cycle:
//!
//! 1. Replay episodes through the relational graph (recomputation)
//! 2. Rebuild coherence groups via min-cut
//! 3. Compile habits into autonomous routines
//! 4. Review suppression rules
//! 5. Bundle all updates into `DownwardMessage`s for core
//!
//! # Contract Compliance
//! - **ARCH-001**: All consolidation logic lives in mbot-companion, not mbot-core
//! - **INVARIANT-007**: Consolidation is purely companion-side; core continues normal operation
//! - **INVARIANT-008**: Atomic updates between ticks (messages bundled together)

use mbot_core::nervous_system::suppression::SuppressionMap;

use super::downward::DownwardMessage;
use super::episodes::InteractionEpisode;
use super::group_manager::GroupManager;
use super::habits::HabitCompiler;
use super::recomputation::RecomputationScheduler;

// ─── ConsolidationTrigger ─────────────────────────────────────────

/// Conditions required to start a consolidation cycle.
#[derive(Clone, Debug)]
pub struct ConsolidationTrigger {
    /// How many idle ticks before consolidation triggers.
    pub idle_threshold: u64,
    /// Minimum energy to start consolidation.
    pub energy_threshold: f32,
    /// Maximum tension allowed (must be calm).
    pub tension_threshold: f32,
    /// Minimum ticks between consolidation cycles.
    pub cooldown_ticks: u64,
}

impl Default for ConsolidationTrigger {
    fn default() -> Self {
        Self {
            idle_threshold: 300,
            energy_threshold: 0.5,
            tension_threshold: 0.3,
            cooldown_ticks: 1000,
        }
    }
}

// ─── ConsolidationResult ──────────────────────────────────────────

/// Summary of what happened during a consolidation cycle.
#[derive(Clone, Debug)]
pub struct ConsolidationResult {
    /// Number of episodes replayed.
    pub episodes_reviewed: usize,
    /// Number of coherence groups after recomputation.
    pub groups_after: usize,
    /// Number of new routines compiled from habits.
    pub new_routines: usize,
    /// Number of suppression rules reviewed.
    pub rules_reviewed: usize,
    /// The tick when this consolidation ran.
    pub tick: u64,
}

// ─── ConsolidationEngine ──────────────────────────────────────────

/// Tracks the state of the consolidation engine.
///
/// Call `should_consolidate()` every tick from the main loop to check
/// whether the robot has been idle long enough to trigger a review.
/// When it returns `true`, call `consolidate()` to run the full cycle.
pub struct ConsolidationEngine {
    trigger: ConsolidationTrigger,
    /// How many consecutive ticks the robot has been idle.
    idle_ticks: u64,
    /// Whether a consolidation is currently in progress.
    is_consolidating: bool,
    /// Tick when the last consolidation completed.
    last_consolidation_tick: u64,
    /// Results from the last consolidation.
    last_result: Option<ConsolidationResult>,
}

impl ConsolidationEngine {
    /// Create a new engine with default trigger parameters.
    pub fn new() -> Self {
        Self {
            trigger: ConsolidationTrigger::default(),
            idle_ticks: 0,
            is_consolidating: false,
            last_consolidation_tick: 0,
            last_result: None,
        }
    }

    /// Create a new engine with custom trigger parameters.
    pub fn with_trigger(trigger: ConsolidationTrigger) -> Self {
        Self {
            trigger,
            idle_ticks: 0,
            is_consolidating: false,
            last_consolidation_tick: 0,
            last_result: None,
        }
    }

    /// Check if consolidation should trigger based on current state.
    ///
    /// Call this every tick from the main loop. Returns `true` if
    /// consolidation should start.
    ///
    /// Logic:
    /// 1. If tension > tension_threshold, reset idle counter (active interaction)
    /// 2. Otherwise, increment idle_ticks
    /// 3. Return true only when idle_threshold met AND energy sufficient
    ///    AND cooldown period elapsed
    pub fn should_consolidate(&mut self, tension: f32, energy: f32, tick: u64) -> bool {
        // Active interaction resets idle counter
        if tension > self.trigger.tension_threshold {
            self.idle_ticks = 0;
            return false;
        }

        self.idle_ticks += 1;

        // Check all three conditions
        self.idle_ticks >= self.trigger.idle_threshold
            && energy >= self.trigger.energy_threshold
            && tick.saturating_sub(self.last_consolidation_tick) >= self.trigger.cooldown_ticks
    }

    /// Run a full consolidation cycle.
    ///
    /// Returns `DownwardMessage`s to send to core and a `ConsolidationResult`
    /// summary. The messages are bundled atomically (INVARIANT-008):
    /// `ConsolidationState(true)`, group map, suppression map,
    /// `ConsolidationState(false)`.
    ///
    /// The `suppression_map` parameter should be the current map from the
    /// core's `StartleProcessor`. The `SuppressionLearner` does not hold
    /// its own map; the authoritative copy lives in core.
    pub fn consolidate(
        &mut self,
        episodes: &[InteractionEpisode],
        curiosity_drive: f32,
        recomputation: &mut RecomputationScheduler,
        group_manager: &mut GroupManager,
        habit_compiler: &mut HabitCompiler,
        suppression_map: &SuppressionMap,
        tick: u64,
    ) -> (Vec<DownwardMessage>, ConsolidationResult) {
        // 1. Mark consolidation in progress
        self.is_consolidating = true;

        // 2. Run recomputation: rebuild graph, run min-cut, update groups
        let groups_after = recomputation.recompute(episodes, curiosity_drive, group_manager, tick);

        // 3. Try to compile new habits into routines
        let new_routines = habit_compiler.try_compile(curiosity_drive);

        // 4. Get current suppression rules count
        let rules_reviewed = suppression_map.len();

        // 5. Build DownwardMessages (atomic bundle -- INVARIANT-008)
        let messages = vec![
            DownwardMessage::ConsolidationState(true),
            DownwardMessage::CoherenceGroupMap(group_manager.as_pairs()),
            DownwardMessage::SuppressionMapUpdate(suppression_map.clone()),
            DownwardMessage::ConsolidationState(false),
        ];

        // 6. Build result summary
        let result = ConsolidationResult {
            episodes_reviewed: episodes.len(),
            groups_after,
            new_routines: new_routines.len(),
            rules_reviewed,
            tick,
        };

        // 7. Update engine state
        self.is_consolidating = false;
        self.last_consolidation_tick = tick;
        self.idle_ticks = 0;
        self.last_result = Some(result.clone());

        (messages, result)
    }

    /// Whether a consolidation is currently in progress.
    pub fn is_consolidating(&self) -> bool {
        self.is_consolidating
    }

    /// Get the last consolidation result.
    pub fn last_result(&self) -> Option<&ConsolidationResult> {
        self.last_result.as_ref()
    }

    /// Reset idle counter (call when robot becomes active).
    pub fn reset_idle(&mut self) {
        self.idle_ticks = 0;
    }
}

impl Default for ConsolidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::episodes::{EpisodeOutcome, InteractionEpisode, TrajectoryVector};

    /// Helper: build an episode with a given context hash and trajectory.
    fn make_episode(ctx: u32, traj: TrajectoryVector) -> InteractionEpisode {
        InteractionEpisode {
            context_hash: ctx,
            start_tick: 0,
            end_tick: 10,
            outcome: EpisodeOutcome::Neutral,
            trajectory: traj,
        }
    }

    /// Build a set of episodes with two clearly separable clusters.
    fn make_two_cluster_episodes() -> Vec<InteractionEpisode> {
        let traj_a: TrajectoryVector =
            [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let traj_b: TrajectoryVector =
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0];

        let mut episodes = Vec::new();
        for _ in 0..3 {
            episodes.push(make_episode(10, traj_a));
            episodes.push(make_episode(20, traj_a));
        }
        for _ in 0..3 {
            episodes.push(make_episode(30, traj_b));
            episodes.push(make_episode(40, traj_b));
        }
        episodes
    }

    // ------------------------------------------------------------------
    // Test 1: New engine has correct defaults
    // ------------------------------------------------------------------
    #[test]
    fn test_new_engine_defaults() {
        let engine = ConsolidationEngine::new();
        assert_eq!(engine.trigger.idle_threshold, 300);
        assert!((engine.trigger.energy_threshold - 0.5).abs() < f32::EPSILON);
        assert!((engine.trigger.tension_threshold - 0.3).abs() < f32::EPSILON);
        assert_eq!(engine.trigger.cooldown_ticks, 1000);
        assert_eq!(engine.idle_ticks, 0);
        assert!(!engine.is_consolidating());
        assert!(engine.last_result().is_none());
    }

    // ------------------------------------------------------------------
    // Test 2: should_consolidate returns false when tension high
    // ------------------------------------------------------------------
    #[test]
    fn test_should_consolidate_false_when_tension_high() {
        let mut engine = ConsolidationEngine::new();

        // Tension above threshold (0.3) should prevent consolidation
        // even if we've been "idle" for many ticks
        for tick in 0..500 {
            let result = engine.should_consolidate(0.5, 0.8, tick);
            assert!(
                !result,
                "should not consolidate with tension=0.5 at tick {}",
                tick
            );
        }
    }

    // ------------------------------------------------------------------
    // Test 3: should_consolidate resets idle counter on high tension
    // ------------------------------------------------------------------
    #[test]
    fn test_should_consolidate_resets_idle_on_tension() {
        let trigger = ConsolidationTrigger {
            idle_threshold: 10,
            energy_threshold: 0.5,
            tension_threshold: 0.3,
            cooldown_ticks: 0,
        };
        let mut engine = ConsolidationEngine::with_trigger(trigger);

        // Accumulate 8 idle ticks (tension=0.1, below threshold)
        for tick in 0..8 {
            engine.should_consolidate(0.1, 0.8, tick);
        }
        assert_eq!(engine.idle_ticks, 8);

        // High tension resets counter
        engine.should_consolidate(0.5, 0.8, 8);
        assert_eq!(engine.idle_ticks, 0);

        // Need another 10 idle ticks to trigger
        for tick in 9..18 {
            let result = engine.should_consolidate(0.1, 0.8, tick);
            assert!(!result, "should not trigger before 10 idle ticks at tick {}", tick);
        }
        let result = engine.should_consolidate(0.1, 0.8, 18);
        assert!(result, "should trigger after 10 consecutive idle ticks");
    }

    // ------------------------------------------------------------------
    // Test 4: should_consolidate returns false before idle threshold
    // ------------------------------------------------------------------
    #[test]
    fn test_should_consolidate_false_before_threshold() {
        let trigger = ConsolidationTrigger {
            idle_threshold: 100,
            energy_threshold: 0.5,
            tension_threshold: 0.3,
            cooldown_ticks: 0,
        };
        let mut engine = ConsolidationEngine::with_trigger(trigger);

        for tick in 0..99 {
            let result = engine.should_consolidate(0.1, 0.8, tick);
            assert!(!result, "should not trigger before idle_threshold at tick {}", tick);
        }
    }

    // ------------------------------------------------------------------
    // Test 5: should_consolidate returns true at idle threshold
    // ------------------------------------------------------------------
    #[test]
    fn test_should_consolidate_true_at_threshold() {
        let trigger = ConsolidationTrigger {
            idle_threshold: 10,
            energy_threshold: 0.5,
            tension_threshold: 0.3,
            cooldown_ticks: 0,
        };
        let mut engine = ConsolidationEngine::with_trigger(trigger);

        // Accumulate 10 idle ticks
        for tick in 0..9 {
            engine.should_consolidate(0.1, 0.8, tick);
        }
        let result = engine.should_consolidate(0.1, 0.8, 9);
        assert!(result, "should trigger at idle_threshold with sufficient energy");
    }

    // ------------------------------------------------------------------
    // Test 6: should_consolidate returns false when energy too low
    // ------------------------------------------------------------------
    #[test]
    fn test_should_consolidate_false_when_energy_low() {
        let trigger = ConsolidationTrigger {
            idle_threshold: 5,
            energy_threshold: 0.5,
            tension_threshold: 0.3,
            cooldown_ticks: 0,
        };
        let mut engine = ConsolidationEngine::with_trigger(trigger);

        // Idle for plenty of ticks, but energy is too low
        for tick in 0..20 {
            let result = engine.should_consolidate(0.1, 0.3, tick);
            assert!(
                !result,
                "should not consolidate with energy=0.3 at tick {}",
                tick
            );
        }
    }

    // ------------------------------------------------------------------
    // Test 7: should_consolidate respects cooldown period
    // ------------------------------------------------------------------
    #[test]
    fn test_should_consolidate_respects_cooldown() {
        let trigger = ConsolidationTrigger {
            idle_threshold: 5,
            energy_threshold: 0.5,
            tension_threshold: 0.3,
            cooldown_ticks: 100,
        };
        let mut engine = ConsolidationEngine::with_trigger(trigger);

        // First consolidation: ticks start at 1000 so last_consolidation_tick(0)
        // cooldown is satisfied (1000 - 0 >= 100).
        for tick in 1000..1005 {
            engine.should_consolidate(0.1, 0.8, tick);
        }
        assert!(engine.should_consolidate(0.1, 0.8, 1005));

        // Simulate consolidation completing at tick 1010
        engine.last_consolidation_tick = 1010;
        engine.idle_ticks = 0;

        // Accumulate idle ticks again, but cooldown hasn't elapsed
        for tick in 1011..1025 {
            engine.should_consolidate(0.1, 0.8, tick);
        }
        // At tick 1025, idle_ticks >= 5 but tick - last_consolidation_tick = 15 < 100
        let result = engine.should_consolidate(0.1, 0.8, 1025);
        assert!(
            !result,
            "should not consolidate during cooldown (tick 1025 - last 1010 = 15 < 100)"
        );

        // Reset idle and wait for cooldown to elapse
        engine.idle_ticks = 0;
        for tick in 1105..1110 {
            engine.should_consolidate(0.1, 0.8, tick);
        }
        let result = engine.should_consolidate(0.1, 0.8, 1110);
        assert!(result, "should consolidate after cooldown elapsed");
    }

    // ------------------------------------------------------------------
    // Test 8: consolidate produces correct DownwardMessages
    // ------------------------------------------------------------------
    #[test]
    fn test_consolidate_produces_correct_messages() {
        let mut engine = ConsolidationEngine::new();
        let episodes = make_two_cluster_episodes();
        let mut recomp = RecomputationScheduler::new();
        let mut gm = GroupManager::new();
        let mut habits = HabitCompiler::new();
        let suppression_map = SuppressionMap::new();

        let (messages, _result) = engine.consolidate(
            &episodes,
            1.0,
            &mut recomp,
            &mut gm,
            &mut habits,
            &suppression_map,
            500,
        );

        // Should have exactly 4 messages
        assert_eq!(messages.len(), 4, "consolidate should produce 4 messages");

        // First message: ConsolidationState(true)
        match &messages[0] {
            DownwardMessage::ConsolidationState(active) => {
                assert!(active, "first message should be ConsolidationState(true)");
            }
            _ => panic!("first message should be ConsolidationState"),
        }

        // Second message: CoherenceGroupMap
        match &messages[1] {
            DownwardMessage::CoherenceGroupMap(_pairs) => {}
            _ => panic!("second message should be CoherenceGroupMap"),
        }

        // Third message: SuppressionMapUpdate
        match &messages[2] {
            DownwardMessage::SuppressionMapUpdate(_map) => {}
            _ => panic!("third message should be SuppressionMapUpdate"),
        }

        // Last message: ConsolidationState(false)
        match &messages[3] {
            DownwardMessage::ConsolidationState(active) => {
                assert!(!active, "last message should be ConsolidationState(false)");
            }
            _ => panic!("last message should be ConsolidationState"),
        }
    }

    // ------------------------------------------------------------------
    // Test 9: consolidate produces ConsolidationResult with correct counts
    // ------------------------------------------------------------------
    #[test]
    fn test_consolidate_result_counts() {
        let mut engine = ConsolidationEngine::new();
        let episodes = make_two_cluster_episodes();
        let mut recomp = RecomputationScheduler::new();
        let mut gm = GroupManager::new();
        let mut habits = HabitCompiler::new();
        let suppression_map = SuppressionMap::new();

        let (_messages, result) = engine.consolidate(
            &episodes,
            1.0,
            &mut recomp,
            &mut gm,
            &mut habits,
            &suppression_map,
            500,
        );

        assert_eq!(
            result.episodes_reviewed, 12,
            "should review all 12 episodes"
        );
        assert!(
            result.groups_after >= 2,
            "two orthogonal clusters should produce >= 2 groups, got {}",
            result.groups_after
        );
        assert_eq!(
            result.new_routines, 0,
            "no habits recorded, so no new routines"
        );
        assert_eq!(
            result.rules_reviewed, 0,
            "empty suppression map has 0 rules"
        );
        assert_eq!(result.tick, 500);
    }

    // ------------------------------------------------------------------
    // Test 10: is_consolidating is false after consolidate completes
    // ------------------------------------------------------------------
    #[test]
    fn test_is_consolidating_false_after_complete() {
        let mut engine = ConsolidationEngine::new();
        let episodes = make_two_cluster_episodes();
        let mut recomp = RecomputationScheduler::new();
        let mut gm = GroupManager::new();
        let mut habits = HabitCompiler::new();
        let suppression_map = SuppressionMap::new();

        assert!(!engine.is_consolidating(), "should not be consolidating initially");

        engine.consolidate(
            &episodes,
            1.0,
            &mut recomp,
            &mut gm,
            &mut habits,
            &suppression_map,
            100,
        );

        assert!(
            !engine.is_consolidating(),
            "should not be consolidating after consolidate() returns"
        );
    }

    // ------------------------------------------------------------------
    // Test 11: reset_idle clears the idle counter
    // ------------------------------------------------------------------
    #[test]
    fn test_reset_idle_clears_counter() {
        let trigger = ConsolidationTrigger {
            idle_threshold: 10,
            energy_threshold: 0.5,
            tension_threshold: 0.3,
            cooldown_ticks: 0,
        };
        let mut engine = ConsolidationEngine::with_trigger(trigger);

        // Accumulate some idle ticks
        for tick in 0..8 {
            engine.should_consolidate(0.1, 0.8, tick);
        }
        assert_eq!(engine.idle_ticks, 8);

        engine.reset_idle();
        assert_eq!(engine.idle_ticks, 0, "reset_idle should clear idle_ticks to 0");
    }

    // ------------------------------------------------------------------
    // Test 12: last_result returns the most recent consolidation summary
    // ------------------------------------------------------------------
    #[test]
    fn test_last_result_returns_most_recent() {
        let mut engine = ConsolidationEngine::new();
        let episodes = make_two_cluster_episodes();
        let mut recomp = RecomputationScheduler::new();
        let mut gm = GroupManager::new();
        let mut habits = HabitCompiler::new();
        let suppression_map = SuppressionMap::new();

        assert!(engine.last_result().is_none(), "no result before first consolidation");

        // First consolidation at tick 100
        engine.consolidate(
            &episodes,
            1.0,
            &mut recomp,
            &mut gm,
            &mut habits,
            &suppression_map,
            100,
        );

        let result1 = engine.last_result().expect("should have result after first consolidation");
        assert_eq!(result1.tick, 100);

        // Second consolidation at tick 500
        engine.consolidate(
            &episodes,
            1.0,
            &mut recomp,
            &mut gm,
            &mut habits,
            &suppression_map,
            500,
        );

        let result2 = engine.last_result().expect("should have result after second consolidation");
        assert_eq!(result2.tick, 500, "last_result should reflect the most recent consolidation");
    }

    // ------------------------------------------------------------------
    // Test 13: Default trait works
    // ------------------------------------------------------------------
    #[test]
    fn test_default_trait() {
        let engine = ConsolidationEngine::default();
        assert_eq!(engine.trigger.idle_threshold, 300);
        assert!((engine.trigger.energy_threshold - 0.5).abs() < f32::EPSILON);
        assert!(!engine.is_consolidating());
        assert!(engine.last_result().is_none());
    }

    // ------------------------------------------------------------------
    // Test 14: consolidate resets idle_ticks to 0
    // ------------------------------------------------------------------
    #[test]
    fn test_consolidate_resets_idle_ticks() {
        let trigger = ConsolidationTrigger {
            idle_threshold: 5,
            energy_threshold: 0.5,
            tension_threshold: 0.3,
            cooldown_ticks: 0,
        };
        let mut engine = ConsolidationEngine::with_trigger(trigger);

        // Accumulate some idle ticks
        for tick in 0..10 {
            engine.should_consolidate(0.1, 0.8, tick);
        }
        assert!(engine.idle_ticks > 0, "should have accumulated idle ticks");

        let episodes = make_two_cluster_episodes();
        let mut recomp = RecomputationScheduler::new();
        let mut gm = GroupManager::new();
        let mut habits = HabitCompiler::new();
        let suppression_map = SuppressionMap::new();

        engine.consolidate(
            &episodes,
            1.0,
            &mut recomp,
            &mut gm,
            &mut habits,
            &suppression_map,
            100,
        );

        assert_eq!(engine.idle_ticks, 0, "consolidate should reset idle_ticks to 0");
    }

    // ------------------------------------------------------------------
    // Test 15: consolidate updates last_consolidation_tick
    // ------------------------------------------------------------------
    #[test]
    fn test_consolidate_updates_last_tick() {
        let mut engine = ConsolidationEngine::new();
        assert_eq!(engine.last_consolidation_tick, 0);

        let episodes = make_two_cluster_episodes();
        let mut recomp = RecomputationScheduler::new();
        let mut gm = GroupManager::new();
        let mut habits = HabitCompiler::new();
        let suppression_map = SuppressionMap::new();

        engine.consolidate(
            &episodes,
            1.0,
            &mut recomp,
            &mut gm,
            &mut habits,
            &suppression_map,
            750,
        );

        assert_eq!(
            engine.last_consolidation_tick, 750,
            "last_consolidation_tick should be updated to the tick of the consolidation"
        );
    }

    // ------------------------------------------------------------------
    // Test 16: consolidate with empty episodes produces valid result
    // ------------------------------------------------------------------
    #[test]
    fn test_consolidate_empty_episodes() {
        let mut engine = ConsolidationEngine::new();
        let mut recomp = RecomputationScheduler::new();
        let mut gm = GroupManager::new();
        let mut habits = HabitCompiler::new();
        let suppression_map = SuppressionMap::new();

        let (messages, result) = engine.consolidate(
            &[],
            1.0,
            &mut recomp,
            &mut gm,
            &mut habits,
            &suppression_map,
            200,
        );

        assert_eq!(messages.len(), 4, "should still produce 4 messages");
        assert_eq!(result.episodes_reviewed, 0);
        assert_eq!(result.groups_after, 0);
        assert_eq!(result.new_routines, 0);
        assert_eq!(result.rules_reviewed, 0);
        assert_eq!(result.tick, 200);
    }
}
