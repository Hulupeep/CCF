//! Coherence Group Manager
//!
//! Manages the mapping from context hashes to coherence group IDs.
//! When min-cut partitions the relational graph, contexts in the same
//! partition share a single coherence accumulator. The group manager
//! translates individual context hashes to their group representative.
//!
//! # Contract Compliance
//! - **ARCH-001**: All grouping logic lives in mbot-companion, not mbot-core
//! - **INVARIANT-002**: Contexts within a group share coherence;
//!   contexts across groups remain independent
//! - **INVARIANT-008**: Atomic accumulator updates during merge

use std::collections::HashMap;

use super::mincut::Partition;

// ─── GroupManager ────────────────────────────────────────────────────

/// Manages the mapping from context hashes to coherence group IDs.
///
/// When min-cut partitions the relational graph, contexts in the same
/// partition share a single coherence accumulator. The group manager
/// translates individual context hashes to their group representative.
///
/// The group ID for each partition is the smallest context hash among
/// the partition's members. This provides a deterministic, stable
/// representative that does not depend on insertion order.
pub struct GroupManager {
    /// context_hash -> group_id (group_id is the smallest context_hash in the partition)
    group_map: HashMap<u32, u32>,
    /// group_id -> list of member context hashes
    groups: HashMap<u32, Vec<u32>>,
}

impl GroupManager {
    /// Create an empty manager with no grouping.
    pub fn new() -> Self {
        Self {
            group_map: HashMap::new(),
            groups: HashMap::new(),
        }
    }

    /// Update from min-cut partitions. Dissolves old groups and creates new ones.
    ///
    /// The group_id for each partition is the smallest context_hash in that partition.
    /// `graph_nodes` maps node indices (used by `Partition::nodes`) to context hashes.
    ///
    /// Partitions containing a single context are still tracked -- they form a
    /// trivial group of one. This ensures `resolve()` is well-defined for every
    /// context that appeared in the graph.
    pub fn update_from_partitions(&mut self, partitions: &[Partition], graph_nodes: &[u32]) {
        self.group_map.clear();
        self.groups.clear();

        for partition in partitions {
            // Collect the context hashes for this partition's node indices.
            let mut hashes: Vec<u32> = partition
                .nodes
                .iter()
                .filter_map(|&idx| graph_nodes.get(idx).copied())
                .collect();

            if hashes.is_empty() {
                continue;
            }

            // Sort so the smallest hash is deterministically chosen as group_id.
            hashes.sort_unstable();
            let group_id = hashes[0];

            for &h in &hashes {
                self.group_map.insert(h, group_id);
            }

            self.groups.insert(group_id, hashes);
        }
    }

    /// Resolve a context hash to its group representative.
    ///
    /// Returns the group_id (smallest hash in the partition) if the context
    /// is part of a group, or the original hash if it is not in any group.
    pub fn resolve(&self, context_hash: u32) -> u32 {
        self.group_map.get(&context_hash).copied().unwrap_or(context_hash)
    }

    /// Get all members of a group by group_id.
    ///
    /// Returns `None` if the group_id does not correspond to any known group.
    pub fn group_members(&self, group_id: u32) -> Option<&[u32]> {
        self.groups.get(&group_id).map(|v| v.as_slice())
    }

    /// Number of active groups.
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Total number of context hashes that belong to some group.
    pub fn grouped_context_count(&self) -> usize {
        self.group_map.len()
    }

    /// Whether a context hash is in a group (vs independent / ungrouped).
    pub fn is_grouped(&self, context_hash: u32) -> bool {
        self.group_map.contains_key(&context_hash)
    }

    /// Get the group map as `(context_hash, group_id)` pairs.
    ///
    /// Suitable for serialization into `DownwardMessage::CoherenceGroupMap`.
    pub fn as_pairs(&self) -> Vec<(u32, u32)> {
        self.group_map.iter().map(|(&ctx, &gid)| (ctx, gid)).collect()
    }

    /// Clear all groups, returning to the ungrouped state.
    pub fn clear(&mut self) {
        self.group_map.clear();
        self.groups.clear();
    }
}

impl Default for GroupManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Accumulator Merge ──────────────────────────────────────────────

/// Merge multiple accumulators into one using weighted average by interaction_count.
///
/// `values` is a slice of `(accumulator_value, interaction_count)` pairs.
///
/// Returns `(merged_value, total_interaction_count)` where:
/// - `merged_value = sum(value_i * count_i) / sum(count_i)`
/// - `total_interaction_count = sum(count_i)`
///
/// If the total interaction count is 0 (all accumulators are empty),
/// returns `(0.0, 0)`.
///
/// # INVARIANT-008
/// This function is a pure computation with no side effects, so it can
/// be called atomically before writing the merged result back.
pub fn merge_accumulators(values: &[(f32, u32)]) -> (f32, u32) {
    let total_count: u32 = values.iter().map(|&(_, c)| c).sum();

    if total_count == 0 {
        return (0.0, 0);
    }

    let weighted_sum: f64 = values
        .iter()
        .map(|&(v, c)| v as f64 * c as f64)
        .sum();

    let merged_value = (weighted_sum / total_count as f64) as f32;

    (merged_value, total_count)
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::mincut::Partition;

    // ------------------------------------------------------------------
    // Test 1: New GroupManager has no groups
    // ------------------------------------------------------------------
    #[test]
    fn test_new_group_manager_empty() {
        let gm = GroupManager::new();
        assert_eq!(gm.group_count(), 0);
        assert_eq!(gm.grouped_context_count(), 0);
        assert!(gm.as_pairs().is_empty());
    }

    // ------------------------------------------------------------------
    // Test 2: update_from_partitions creates correct groups
    // ------------------------------------------------------------------
    #[test]
    fn test_update_creates_groups() {
        let mut gm = GroupManager::new();
        // Graph nodes: index 0=100, 1=200, 2=50, 3=300
        let graph_nodes = vec![100, 200, 50, 300];

        // Two partitions: {0, 1} -> hashes {100, 200}, {2, 3} -> hashes {50, 300}
        let partitions = vec![
            Partition { nodes: vec![0, 1] },
            Partition { nodes: vec![2, 3] },
        ];

        gm.update_from_partitions(&partitions, &graph_nodes);

        assert_eq!(gm.group_count(), 2);
        assert_eq!(gm.grouped_context_count(), 4);

        // Group IDs should be the smallest hash in each partition.
        // Partition {100, 200} -> group_id = 100
        // Partition {50, 300} -> group_id = 50
        assert_eq!(gm.resolve(100), 100);
        assert_eq!(gm.resolve(200), 100);
        assert_eq!(gm.resolve(50), 50);
        assert_eq!(gm.resolve(300), 50);
    }

    // ------------------------------------------------------------------
    // Test 3: resolve returns group representative for grouped context
    // ------------------------------------------------------------------
    #[test]
    fn test_resolve_grouped() {
        let mut gm = GroupManager::new();
        let graph_nodes = vec![500, 100, 300];
        let partitions = vec![Partition { nodes: vec![0, 1, 2] }];

        gm.update_from_partitions(&partitions, &graph_nodes);

        // All three should resolve to the smallest: 100
        assert_eq!(gm.resolve(500), 100);
        assert_eq!(gm.resolve(100), 100);
        assert_eq!(gm.resolve(300), 100);
    }

    // ------------------------------------------------------------------
    // Test 4: resolve returns original hash for ungrouped context
    // ------------------------------------------------------------------
    #[test]
    fn test_resolve_ungrouped() {
        let gm = GroupManager::new();
        assert_eq!(gm.resolve(42), 42);
        assert_eq!(gm.resolve(999), 999);
    }

    // ------------------------------------------------------------------
    // Test 5: group_members returns correct member list
    // ------------------------------------------------------------------
    #[test]
    fn test_group_members() {
        let mut gm = GroupManager::new();
        let graph_nodes = vec![300, 100, 200];
        let partitions = vec![Partition { nodes: vec![0, 1, 2] }];

        gm.update_from_partitions(&partitions, &graph_nodes);

        // Group ID is 100 (smallest of 300, 100, 200).
        let members = gm.group_members(100).expect("group 100 should exist");
        // Members should be sorted: [100, 200, 300]
        assert_eq!(members, &[100, 200, 300]);

        // Non-existent group returns None.
        assert!(gm.group_members(42).is_none());
    }

    // ------------------------------------------------------------------
    // Test 6: group_count and grouped_context_count are correct
    // ------------------------------------------------------------------
    #[test]
    fn test_counts() {
        let mut gm = GroupManager::new();
        let graph_nodes = vec![10, 20, 30, 40, 50];
        let partitions = vec![
            Partition { nodes: vec![0, 1] },      // {10, 20}
            Partition { nodes: vec![2, 3, 4] },    // {30, 40, 50}
        ];

        gm.update_from_partitions(&partitions, &graph_nodes);

        assert_eq!(gm.group_count(), 2);
        assert_eq!(gm.grouped_context_count(), 5);
    }

    // ------------------------------------------------------------------
    // Test 7: update_from_partitions replaces old groups
    // ------------------------------------------------------------------
    #[test]
    fn test_update_replaces_old_groups() {
        let mut gm = GroupManager::new();

        // First grouping: {10, 20}
        let graph_nodes_1 = vec![10, 20];
        let partitions_1 = vec![Partition { nodes: vec![0, 1] }];
        gm.update_from_partitions(&partitions_1, &graph_nodes_1);

        assert_eq!(gm.group_count(), 1);
        assert_eq!(gm.resolve(10), 10);
        assert_eq!(gm.resolve(20), 10);

        // Second grouping: {30, 40}. Old {10, 20} should be gone.
        let graph_nodes_2 = vec![30, 40];
        let partitions_2 = vec![Partition { nodes: vec![0, 1] }];
        gm.update_from_partitions(&partitions_2, &graph_nodes_2);

        assert_eq!(gm.group_count(), 1);
        assert_eq!(gm.resolve(30), 30);
        assert_eq!(gm.resolve(40), 30);

        // Old contexts are no longer grouped.
        assert!(!gm.is_grouped(10));
        assert!(!gm.is_grouped(20));
        assert_eq!(gm.resolve(10), 10); // returns self, not old group
    }

    // ------------------------------------------------------------------
    // Test 8: clear removes all groups
    // ------------------------------------------------------------------
    #[test]
    fn test_clear() {
        let mut gm = GroupManager::new();
        let graph_nodes = vec![10, 20, 30];
        let partitions = vec![Partition { nodes: vec![0, 1, 2] }];
        gm.update_from_partitions(&partitions, &graph_nodes);

        assert_eq!(gm.group_count(), 1);
        assert_eq!(gm.grouped_context_count(), 3);

        gm.clear();

        assert_eq!(gm.group_count(), 0);
        assert_eq!(gm.grouped_context_count(), 0);
        assert!(!gm.is_grouped(10));
        assert_eq!(gm.resolve(10), 10);
    }

    // ------------------------------------------------------------------
    // Test 9: merge_accumulators weighted average correctness
    // ------------------------------------------------------------------
    #[test]
    fn test_merge_accumulators_weighted_average() {
        // Accumulator A: value=0.8, count=10
        // Accumulator B: value=0.2, count=30
        // Expected: (0.8*10 + 0.2*30) / (10+30) = (8 + 6) / 40 = 14/40 = 0.35
        let values = vec![(0.8, 10), (0.2, 30)];
        let (merged, total) = merge_accumulators(&values);

        assert_eq!(total, 40);
        assert!(
            (merged - 0.35).abs() < 1e-6,
            "expected 0.35, got {}",
            merged
        );
    }

    // ------------------------------------------------------------------
    // Test 10: merge_accumulators with zero counts returns (0.0, 0)
    // ------------------------------------------------------------------
    #[test]
    fn test_merge_accumulators_zero_counts() {
        let values = vec![(0.5, 0), (0.9, 0)];
        let (merged, total) = merge_accumulators(&values);

        assert_eq!(total, 0);
        assert_eq!(merged, 0.0);
    }

    // ------------------------------------------------------------------
    // Test 11: merge_accumulators with single accumulator returns it
    // ------------------------------------------------------------------
    #[test]
    fn test_merge_accumulators_single() {
        let values = vec![(0.75, 42)];
        let (merged, total) = merge_accumulators(&values);

        assert_eq!(total, 42);
        assert!(
            (merged - 0.75).abs() < 1e-6,
            "single accumulator should return its own value, got {}",
            merged
        );
    }

    // ------------------------------------------------------------------
    // Test 12: as_pairs returns correct context_hash -> group_id mapping
    // ------------------------------------------------------------------
    #[test]
    fn test_as_pairs() {
        let mut gm = GroupManager::new();
        let graph_nodes = vec![100, 200, 50];
        let partitions = vec![
            Partition { nodes: vec![0, 1] }, // {100, 200} -> group 100
            Partition { nodes: vec![2] },    // {50} -> group 50
        ];
        gm.update_from_partitions(&partitions, &graph_nodes);

        let mut pairs = gm.as_pairs();
        pairs.sort_by_key(|&(ctx, _)| ctx);

        // Expected: (50, 50), (100, 100), (200, 100)
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0], (50, 50));
        assert_eq!(pairs[1], (100, 100));
        assert_eq!(pairs[2], (200, 100));
    }

    // ------------------------------------------------------------------
    // Test 13: is_grouped returns true/false correctly
    // ------------------------------------------------------------------
    #[test]
    fn test_is_grouped() {
        let mut gm = GroupManager::new();
        let graph_nodes = vec![10, 20];
        let partitions = vec![Partition { nodes: vec![0, 1] }];
        gm.update_from_partitions(&partitions, &graph_nodes);

        assert!(gm.is_grouped(10));
        assert!(gm.is_grouped(20));
        assert!(!gm.is_grouped(30)); // not in any partition
    }

    // ------------------------------------------------------------------
    // Test 14: merge_accumulators with empty slice returns (0.0, 0)
    // ------------------------------------------------------------------
    #[test]
    fn test_merge_accumulators_empty() {
        let (merged, total) = merge_accumulators(&[]);
        assert_eq!(total, 0);
        assert_eq!(merged, 0.0);
    }

    // ------------------------------------------------------------------
    // Test 15: merge_accumulators with many accumulators
    // ------------------------------------------------------------------
    #[test]
    fn test_merge_accumulators_many() {
        // Three accumulators: (0.5, 10), (0.8, 20), (0.2, 10)
        // Weighted sum = 0.5*10 + 0.8*20 + 0.2*10 = 5 + 16 + 2 = 23
        // Total count = 40
        // Merged = 23/40 = 0.575
        let values = vec![(0.5, 10), (0.8, 20), (0.2, 10)];
        let (merged, total) = merge_accumulators(&values);

        assert_eq!(total, 40);
        assert!(
            (merged - 0.575).abs() < 1e-6,
            "expected 0.575, got {}",
            merged
        );
    }

    // ------------------------------------------------------------------
    // Test 16: Default trait works
    // ------------------------------------------------------------------
    #[test]
    fn test_default_trait() {
        let gm = GroupManager::default();
        assert_eq!(gm.group_count(), 0);
        assert_eq!(gm.grouped_context_count(), 0);
    }

    // ------------------------------------------------------------------
    // Test 17: update_from_partitions with out-of-bounds indices is safe
    // ------------------------------------------------------------------
    #[test]
    fn test_update_with_out_of_bounds_indices() {
        let mut gm = GroupManager::new();
        let graph_nodes = vec![10, 20];

        // Partition references index 5, which is out of bounds for graph_nodes.
        // The invalid index should be silently skipped.
        let partitions = vec![Partition { nodes: vec![0, 5] }];
        gm.update_from_partitions(&partitions, &graph_nodes);

        // Only hash 10 (index 0) should be in the group.
        assert_eq!(gm.group_count(), 1);
        assert_eq!(gm.grouped_context_count(), 1);
        assert!(gm.is_grouped(10));
        assert!(!gm.is_grouped(20));
    }

    // ------------------------------------------------------------------
    // Test 18: update with empty partitions clears state
    // ------------------------------------------------------------------
    #[test]
    fn test_update_empty_partitions() {
        let mut gm = GroupManager::new();

        // First, create some groups.
        let graph_nodes = vec![10, 20];
        let partitions = vec![Partition { nodes: vec![0, 1] }];
        gm.update_from_partitions(&partitions, &graph_nodes);
        assert_eq!(gm.group_count(), 1);

        // Now update with empty partitions.
        gm.update_from_partitions(&[], &graph_nodes);
        assert_eq!(gm.group_count(), 0);
        assert_eq!(gm.grouped_context_count(), 0);
    }
}
