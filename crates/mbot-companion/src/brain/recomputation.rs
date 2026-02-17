//! Periodic min-cut recomputation & personality modulation of cut threshold.
//!
//! Schedules periodic recomputation of the relational graph's min-cut
//! partitions and modulates the max_cut_weight threshold by personality
//! curiosity_drive.  Higher curiosity lowers the threshold, producing
//! finer-grained coherence groups.
//!
//! # Contract Compliance
//! - **ARCH-001**: All recomputation logic lives in mbot-companion, not mbot-core
//! - **INVARIANT-004**: Personality modulates dynamics (cut threshold), not structure

use super::episodes::InteractionEpisode;
use super::group_manager::GroupManager;
use super::mincut::min_cut_n_way;
use super::relational_graph::RelationalGraph;

// ─── RecomputationScheduler ─────────────────────────────────────────

/// Schedules periodic min-cut recomputation and modulates the cut
/// threshold using the robot's curiosity_drive personality parameter.
pub struct RecomputationScheduler {
    /// Base interval multiplier: recompute every N * num_contexts episodes.
    pub episodes_per_context: u32,
    /// Base max_cut_weight before personality modulation.
    pub base_max_cut_weight: f32,
    /// Similarity threshold for graph building.
    pub similarity_threshold: f32,
    /// Minimum episodes per context for graph inclusion.
    pub min_episodes_per_context: usize,
    /// Number of episodes since last recomputation.
    episodes_since_last: u32,
    /// Number of observed unique contexts (updated during recomputation).
    observed_contexts: u32,
    /// Last recomputation tick.
    last_recomputation_tick: u64,
}

impl RecomputationScheduler {
    /// Create a new scheduler with default parameters.
    pub fn new() -> Self {
        Self {
            episodes_per_context: 10,
            base_max_cut_weight: 0.3,
            similarity_threshold: 0.3,
            min_episodes_per_context: 3,
            episodes_since_last: 0,
            observed_contexts: 0,
            last_recomputation_tick: 0,
        }
    }

    /// Record that an episode was completed. Returns true if
    /// recomputation should trigger.
    pub fn record_episode(&mut self) -> bool {
        self.episodes_since_last += 1;
        self.episodes_since_last >= self.recomputation_threshold()
    }

    /// Get the recomputation threshold: episodes_per_context * observed_contexts.
    /// Minimum of 10 episodes to avoid recomputing with very few data points.
    pub fn recomputation_threshold(&self) -> u32 {
        let dynamic = self.episodes_per_context.saturating_mul(self.observed_contexts);
        dynamic.max(10)
    }

    /// Compute the effective max_cut_weight modulated by personality curiosity_drive.
    ///
    /// Higher curiosity produces a lower threshold, which causes the n-way
    /// min-cut to produce more groups (finer partitioning). Lower curiosity
    /// produces a higher threshold, keeping groups larger.
    ///
    /// Formula: `effective = base_max_cut_weight / curiosity_drive.max(0.1)`
    ///
    /// The curiosity_drive floor of 0.1 prevents division by zero.
    pub fn effective_max_cut_weight(&self, curiosity_drive: f32) -> f32 {
        let clamped_curiosity = curiosity_drive.max(0.1);
        self.base_max_cut_weight / clamped_curiosity
    }

    /// Run full recomputation: build graph, run min-cut, update groups.
    ///
    /// Returns the number of groups produced.
    pub fn recompute(
        &mut self,
        episodes: &[InteractionEpisode],
        curiosity_drive: f32,
        group_manager: &mut GroupManager,
        tick: u64,
    ) -> usize {
        // 1. Build a RelationalGraph from the episodes.
        let graph = RelationalGraph::build(
            episodes,
            self.min_episodes_per_context,
            self.similarity_threshold,
        );

        // 2. Update observed_contexts from the graph's node count.
        self.observed_contexts = graph.node_count() as u32;

        // 3. Compute effective_max_cut_weight with curiosity modulation.
        let effective_threshold = self.effective_max_cut_weight(curiosity_drive);

        // 4. Run min_cut_n_way to get partitions.
        let partitions = min_cut_n_way(&graph, effective_threshold);

        // 5. Update group_manager with new partitions.
        group_manager.update_from_partitions(&partitions, &graph.nodes);

        // 6. Reset episodes_since_last to 0.
        self.episodes_since_last = 0;

        // 7. Store last_recomputation_tick.
        self.last_recomputation_tick = tick;

        // 8. Return number of groups.
        group_manager.group_count()
    }

    /// Episodes accumulated since last recomputation.
    pub fn episodes_since_last(&self) -> u32 {
        self.episodes_since_last
    }

    /// When was the last recomputation (tick).
    pub fn last_recomputation_tick(&self) -> u64 {
        self.last_recomputation_tick
    }
}

impl Default for RecomputationScheduler {
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
    /// Cluster A (contexts 10, 20): trajectory concentrated in dims 0-1
    /// Cluster B (contexts 30, 40): trajectory concentrated in dims 6-7
    /// Each context has 3+ episodes (meets min_episodes_per_context=3).
    fn make_two_cluster_episodes() -> Vec<InteractionEpisode> {
        let traj_a: TrajectoryVector = [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let traj_b: TrajectoryVector = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0];

        let mut episodes = Vec::new();
        // 3 episodes each for contexts 10 and 20 (cluster A)
        for _ in 0..3 {
            episodes.push(make_episode(10, traj_a));
            episodes.push(make_episode(20, traj_a));
        }
        // 3 episodes each for contexts 30 and 40 (cluster B)
        for _ in 0..3 {
            episodes.push(make_episode(30, traj_b));
            episodes.push(make_episode(40, traj_b));
        }
        episodes
    }

    // ------------------------------------------------------------------
    // Test 1: New scheduler has correct defaults
    // ------------------------------------------------------------------
    #[test]
    fn test_new_scheduler_defaults() {
        let s = RecomputationScheduler::new();
        assert_eq!(s.episodes_per_context, 10);
        assert!((s.base_max_cut_weight - 0.3).abs() < 1e-6);
        assert!((s.similarity_threshold - 0.3).abs() < 1e-6);
        assert_eq!(s.min_episodes_per_context, 3);
        assert_eq!(s.episodes_since_last(), 0);
        assert_eq!(s.last_recomputation_tick(), 0);
    }

    // ------------------------------------------------------------------
    // Test 2: record_episode increments counter
    // ------------------------------------------------------------------
    #[test]
    fn test_record_episode_increments_counter() {
        let mut s = RecomputationScheduler::new();
        assert_eq!(s.episodes_since_last(), 0);

        s.record_episode();
        assert_eq!(s.episodes_since_last(), 1);

        s.record_episode();
        assert_eq!(s.episodes_since_last(), 2);

        s.record_episode();
        assert_eq!(s.episodes_since_last(), 3);
    }

    // ------------------------------------------------------------------
    // Test 3: record_episode returns true when threshold reached
    // ------------------------------------------------------------------
    #[test]
    fn test_record_episode_triggers_at_threshold() {
        let mut s = RecomputationScheduler::new();
        // With observed_contexts=0, threshold = max(10*0, 10) = 10.
        for i in 0..9 {
            let triggered = s.record_episode();
            assert!(
                !triggered,
                "should not trigger at episode {} (threshold=10)",
                i + 1
            );
        }
        // 10th episode should trigger.
        let triggered = s.record_episode();
        assert!(triggered, "should trigger at episode 10 (threshold=10)");
    }

    // ------------------------------------------------------------------
    // Test 4: recomputation_threshold scales with observed_contexts
    // ------------------------------------------------------------------
    #[test]
    fn test_threshold_scales_with_contexts() {
        let mut s = RecomputationScheduler::new();

        // Initially 0 contexts -> threshold = max(10*0, 10) = 10.
        assert_eq!(s.recomputation_threshold(), 10);

        // Simulate observing 5 contexts.
        s.observed_contexts = 5;
        // threshold = max(10*5, 10) = 50
        assert_eq!(s.recomputation_threshold(), 50);

        // Simulate observing 20 contexts.
        s.observed_contexts = 20;
        // threshold = max(10*20, 10) = 200
        assert_eq!(s.recomputation_threshold(), 200);
    }

    // ------------------------------------------------------------------
    // Test 5: recomputation_threshold has minimum of 10
    // ------------------------------------------------------------------
    #[test]
    fn test_threshold_minimum_10() {
        let mut s = RecomputationScheduler::new();
        s.episodes_per_context = 2;
        s.observed_contexts = 3;
        // threshold = max(2*3, 10) = max(6, 10) = 10
        assert_eq!(s.recomputation_threshold(), 10);

        s.observed_contexts = 0;
        // threshold = max(2*0, 10) = 10
        assert_eq!(s.recomputation_threshold(), 10);
    }

    // ------------------------------------------------------------------
    // Test 6: effective_max_cut_weight with curiosity=1.0 equals base
    // ------------------------------------------------------------------
    #[test]
    fn test_effective_weight_curiosity_1() {
        let s = RecomputationScheduler::new();
        let w = s.effective_max_cut_weight(1.0);
        assert!(
            (w - 0.3).abs() < 1e-6,
            "curiosity=1.0 should return base (0.3), got {}",
            w
        );
    }

    // ------------------------------------------------------------------
    // Test 7: effective_max_cut_weight with curiosity=1.5 is lower (more groups)
    // ------------------------------------------------------------------
    #[test]
    fn test_effective_weight_high_curiosity_lower() {
        let s = RecomputationScheduler::new();
        let w = s.effective_max_cut_weight(1.5);
        // 0.3 / 1.5 = 0.2
        assert!(
            (w - 0.2).abs() < 1e-6,
            "curiosity=1.5 should return 0.2, got {}",
            w
        );
        assert!(
            w < s.base_max_cut_weight,
            "higher curiosity should produce lower threshold"
        );
    }

    // ------------------------------------------------------------------
    // Test 8: effective_max_cut_weight with curiosity=0.5 is higher (fewer groups)
    // ------------------------------------------------------------------
    #[test]
    fn test_effective_weight_low_curiosity_higher() {
        let s = RecomputationScheduler::new();
        let w = s.effective_max_cut_weight(0.5);
        // 0.3 / 0.5 = 0.6
        assert!(
            (w - 0.6).abs() < 1e-6,
            "curiosity=0.5 should return 0.6, got {}",
            w
        );
        assert!(
            w > s.base_max_cut_weight,
            "lower curiosity should produce higher threshold"
        );
    }

    // ------------------------------------------------------------------
    // Test 9: effective_max_cut_weight clamps curiosity minimum to 0.1
    // ------------------------------------------------------------------
    #[test]
    fn test_effective_weight_clamps_curiosity_floor() {
        let s = RecomputationScheduler::new();

        // curiosity = 0.0 should be clamped to 0.1.
        let w_zero = s.effective_max_cut_weight(0.0);
        let w_neg = s.effective_max_cut_weight(-5.0);
        let w_floor = s.effective_max_cut_weight(0.1);

        // All should produce 0.3 / 0.1 = 3.0.
        assert!(
            (w_zero - 3.0).abs() < 1e-6,
            "curiosity=0.0 should clamp to 0.1 -> 3.0, got {}",
            w_zero
        );
        assert!(
            (w_neg - 3.0).abs() < 1e-6,
            "curiosity=-5.0 should clamp to 0.1 -> 3.0, got {}",
            w_neg
        );
        assert!(
            (w_floor - 3.0).abs() < 1e-6,
            "curiosity=0.1 should produce 3.0, got {}",
            w_floor
        );
    }

    // ------------------------------------------------------------------
    // Test 10: recompute produces correct group count
    // ------------------------------------------------------------------
    #[test]
    fn test_recompute_produces_groups() {
        let mut s = RecomputationScheduler::new();
        let mut gm = GroupManager::new();
        let episodes = make_two_cluster_episodes();

        let group_count = s.recompute(&episodes, 1.0, &mut gm, 100);

        // With 4 contexts in 2 orthogonal clusters and base threshold 0.3,
        // the min-cut should separate them into at least 2 groups.
        assert!(
            group_count >= 2,
            "two orthogonal clusters should produce >= 2 groups, got {}",
            group_count
        );
        assert_eq!(gm.group_count(), group_count);
    }

    // ------------------------------------------------------------------
    // Test 11: recompute resets episodes_since_last
    // ------------------------------------------------------------------
    #[test]
    fn test_recompute_resets_counter() {
        let mut s = RecomputationScheduler::new();
        let mut gm = GroupManager::new();
        let episodes = make_two_cluster_episodes();

        // Accumulate some episodes.
        for _ in 0..5 {
            s.record_episode();
        }
        assert_eq!(s.episodes_since_last(), 5);

        // Recompute should reset.
        s.recompute(&episodes, 1.0, &mut gm, 200);
        assert_eq!(s.episodes_since_last(), 0);
    }

    // ------------------------------------------------------------------
    // Test 12: recompute updates last_recomputation_tick
    // ------------------------------------------------------------------
    #[test]
    fn test_recompute_updates_tick() {
        let mut s = RecomputationScheduler::new();
        let mut gm = GroupManager::new();
        let episodes = make_two_cluster_episodes();

        assert_eq!(s.last_recomputation_tick(), 0);

        s.recompute(&episodes, 1.0, &mut gm, 500);
        assert_eq!(s.last_recomputation_tick(), 500);

        s.recompute(&episodes, 1.0, &mut gm, 1000);
        assert_eq!(s.last_recomputation_tick(), 1000);
    }

    // ------------------------------------------------------------------
    // Test 13: High curiosity produces more groups than low curiosity
    //          with same data
    // ------------------------------------------------------------------
    #[test]
    fn test_high_curiosity_more_groups_than_low() {
        // Build episodes where contexts form a single tight cluster
        // that can be split if the threshold is low enough.
        // Use 4 contexts with very similar (but not identical) trajectories.
        let traj_a: TrajectoryVector = [1.0, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let traj_b: TrajectoryVector = [0.9, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let traj_c: TrajectoryVector = [0.8, 0.8, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let traj_d: TrajectoryVector = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0];

        let mut episodes = Vec::new();
        for _ in 0..3 {
            episodes.push(make_episode(10, traj_a));
            episodes.push(make_episode(20, traj_b));
            episodes.push(make_episode(30, traj_c));
            episodes.push(make_episode(40, traj_d));
        }

        // Low curiosity (0.3) -> high threshold (0.3/0.3 = 1.0) -> fewer groups.
        let mut s_low = RecomputationScheduler::new();
        let mut gm_low = GroupManager::new();
        let groups_low = s_low.recompute(&episodes, 0.3, &mut gm_low, 1);

        // High curiosity (3.0) -> low threshold (0.3/3.0 = 0.1) -> more groups.
        let mut s_high = RecomputationScheduler::new();
        let mut gm_high = GroupManager::new();
        let groups_high = s_high.recompute(&episodes, 3.0, &mut gm_high, 1);

        assert!(
            groups_high >= groups_low,
            "high curiosity ({} groups) should produce >= groups than low curiosity ({} groups)",
            groups_high,
            groups_low
        );
    }

    // ------------------------------------------------------------------
    // Test 14: recompute with empty episodes produces 0 groups
    // ------------------------------------------------------------------
    #[test]
    fn test_recompute_empty_episodes() {
        let mut s = RecomputationScheduler::new();
        let mut gm = GroupManager::new();

        let group_count = s.recompute(&[], 1.0, &mut gm, 50);

        assert_eq!(group_count, 0, "empty episodes should produce 0 groups");
        assert_eq!(s.observed_contexts, 0);
        assert_eq!(s.episodes_since_last(), 0);
        assert_eq!(s.last_recomputation_tick(), 50);
    }

    // ------------------------------------------------------------------
    // Test 15: Default trait works
    // ------------------------------------------------------------------
    #[test]
    fn test_default_trait() {
        let s = RecomputationScheduler::default();
        assert_eq!(s.episodes_per_context, 10);
        assert!((s.base_max_cut_weight - 0.3).abs() < 1e-6);
    }

    // ------------------------------------------------------------------
    // Test 16: recompute updates observed_contexts from graph node count
    // ------------------------------------------------------------------
    #[test]
    fn test_recompute_updates_observed_contexts() {
        let mut s = RecomputationScheduler::new();
        let mut gm = GroupManager::new();
        let episodes = make_two_cluster_episodes();

        assert_eq!(s.observed_contexts, 0);

        s.recompute(&episodes, 1.0, &mut gm, 100);

        // make_two_cluster_episodes has 4 unique contexts, each with 3 episodes.
        // With min_episodes_per_context=3, all 4 should qualify as nodes.
        assert_eq!(
            s.observed_contexts, 4,
            "should observe 4 contexts from the episode data"
        );
    }
}
