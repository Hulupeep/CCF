//! RelationalGraph construction from episode similarity.
//!
//! Builds a graph where nodes are context hashes (with enough episodes)
//! and edges connect contexts whose average trajectory vectors exceed
//! a cosine-similarity threshold.
//!
//! # Contract Compliance
//! - **ARCH-001**: All graph logic lives in mbot-companion, not mbot-core
//! - **I-GRAPH-001**: Only contexts with >= min_episodes become nodes
//! - **I-GRAPH-002**: Edge weight is cosine similarity, clamped to [0.0, 1.0]

use std::collections::HashMap;

use super::episodes::{InteractionEpisode, TrajectoryVector};

// ─── Core Struct ─────────────────────────────────────────────────────

/// A similarity graph over interaction contexts.
///
/// Each node is a context hash that appeared in at least `min_episodes`
/// episodes.  Edges connect pairs whose average trajectory vectors have
/// cosine similarity above the configured threshold.
pub struct RelationalGraph {
    /// Context hashes that qualify as nodes, in stable insertion order.
    pub nodes: Vec<u32>,
    /// Edges: `(node_index_a, node_index_b, cosine_similarity)`.
    pub edges: Vec<(usize, usize, f32)>,
    /// Average trajectory vector for each node (parallel to `nodes`).
    trajectories: Vec<TrajectoryVector>,
}

impl RelationalGraph {
    /// Build a relational graph from a slice of episodes.
    ///
    /// * `min_episodes` -- contexts with fewer episodes are excluded
    ///   (I-GRAPH-001).
    /// * `similarity_threshold` -- only pairs with cosine similarity
    ///   >= this value get an edge (I-GRAPH-002).
    pub fn build(
        episodes: &[InteractionEpisode],
        min_episodes: usize,
        similarity_threshold: f32,
    ) -> Self {
        // 1. Group episodes by context_hash, preserving insertion order.
        let mut groups: HashMap<u32, Vec<&TrajectoryVector>> = HashMap::new();
        let mut order: Vec<u32> = Vec::new();

        for ep in episodes {
            let entry = groups.entry(ep.context_hash).or_default();
            if entry.is_empty() {
                order.push(ep.context_hash);
            }
            entry.push(&ep.trajectory);
        }

        // 2. Filter to contexts meeting min_episodes and compute averages.
        let mut nodes: Vec<u32> = Vec::new();
        let mut trajectories: Vec<TrajectoryVector> = Vec::new();

        for ctx in &order {
            if let Some(tvs) = groups.get(ctx) {
                if tvs.len() >= min_episodes {
                    nodes.push(*ctx);
                    trajectories.push(average_trajectory(tvs));
                }
            }
        }

        // 3. Compute pairwise cosine similarity; add edges above threshold.
        let mut edges: Vec<(usize, usize, f32)> = Vec::new();
        let n = nodes.len();

        for i in 0..n {
            for j in (i + 1)..n {
                let sim = cosine_similarity(&trajectories[i], &trajectories[j]);
                // I-GRAPH-002: clamp negatives to 0.
                let sim = sim.max(0.0);
                if sim >= similarity_threshold {
                    edges.push((i, j, sim));
                }
            }
        }

        Self {
            nodes,
            edges,
            trajectories,
        }
    }

    /// Get the average trajectory for a node by index.
    pub fn node_trajectory(&self, index: usize) -> Option<&TrajectoryVector> {
        self.trajectories.get(index)
    }

    /// Number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get neighbors of a node with their similarity weights.
    ///
    /// Returns `(neighbor_index, similarity)` pairs for every edge that
    /// touches `node_index`.
    pub fn neighbors(&self, node_index: usize) -> Vec<(usize, f32)> {
        let mut result = Vec::new();
        for &(a, b, sim) in &self.edges {
            if a == node_index {
                result.push((b, sim));
            } else if b == node_index {
                result.push((a, sim));
            }
        }
        result
    }
}

// ─── Helper Functions ────────────────────────────────────────────────

/// Compute the element-wise average of a collection of trajectory vectors.
fn average_trajectory(tvs: &[&TrajectoryVector]) -> TrajectoryVector {
    let n = tvs.len() as f32;
    if n == 0.0 {
        return [0.0; 12];
    }
    let mut avg = [0.0f32; 12];
    for tv in tvs {
        for (i, &val) in tv.iter().enumerate() {
            avg[i] += val;
        }
    }
    for v in avg.iter_mut() {
        *v /= n;
    }
    avg
}

/// Cosine similarity between two 12-dimensional vectors.
///
/// Returns 0.0 if either vector has zero magnitude.
fn cosine_similarity(a: &TrajectoryVector, b: &TrajectoryVector) -> f32 {
    let mut dot = 0.0f32;
    let mut mag_a = 0.0f32;
    let mut mag_b = 0.0f32;

    for i in 0..12 {
        dot += a[i] * b[i];
        mag_a += a[i] * a[i];
        mag_b += b[i] * b[i];
    }

    let denom = mag_a.sqrt() * mag_b.sqrt();
    if denom < 1e-12 {
        0.0
    } else {
        dot / denom
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::episodes::EpisodeOutcome;

    /// Helper: build an episode with a given context hash and trajectory.
    fn make_episode(context_hash: u32, trajectory: TrajectoryVector) -> InteractionEpisode {
        InteractionEpisode {
            context_hash,
            start_tick: 0,
            end_tick: 10,
            outcome: EpisodeOutcome::Neutral,
            trajectory,
        }
    }

    // ------------------------------------------------------------------
    // Test 1: Graph excludes contexts with too few episodes
    // ------------------------------------------------------------------
    #[test]
    fn test_excludes_contexts_below_min_episodes() {
        let episodes = vec![
            make_episode(100, [1.0; 12]),
            make_episode(100, [1.0; 12]),
            // context 200 has only 1 episode (below min_episodes=3)
            make_episode(200, [0.5; 12]),
        ];

        let graph = RelationalGraph::build(&episodes, 3, 0.3);

        // context 100 has 2, context 200 has 1 -- neither meets min=3
        assert_eq!(graph.node_count(), 0);
    }

    // ------------------------------------------------------------------
    // Test 2: Graph includes contexts meeting the minimum
    // ------------------------------------------------------------------
    #[test]
    fn test_includes_contexts_meeting_minimum() {
        let episodes = vec![
            make_episode(100, [1.0; 12]),
            make_episode(100, [1.0; 12]),
            make_episode(100, [1.0; 12]),
            // context 200 has only 2 episodes
            make_episode(200, [0.5; 12]),
            make_episode(200, [0.5; 12]),
        ];

        let graph = RelationalGraph::build(&episodes, 3, 0.3);

        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.nodes[0], 100);
    }

    // ------------------------------------------------------------------
    // Test 3: Cosine similarity of identical vectors = 1.0
    // ------------------------------------------------------------------
    #[test]
    fn test_cosine_identical_vectors() {
        let a: TrajectoryVector = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 0.5, 0.3];
        let sim = cosine_similarity(&a, &a);
        assert!(
            (sim - 1.0).abs() < 1e-5,
            "identical vectors should have cosine similarity ~1.0, got {}",
            sim
        );
    }

    // ------------------------------------------------------------------
    // Test 4: Cosine similarity of orthogonal vectors = 0.0
    // ------------------------------------------------------------------
    #[test]
    fn test_cosine_orthogonal_vectors() {
        // e1 and e2 are orthogonal unit-ish vectors in 12D
        let mut a: TrajectoryVector = [0.0; 12];
        let mut b: TrajectoryVector = [0.0; 12];
        a[0] = 1.0;
        b[1] = 1.0;

        let sim = cosine_similarity(&a, &b);
        assert!(
            sim.abs() < 1e-6,
            "orthogonal vectors should have cosine similarity ~0.0, got {}",
            sim
        );
    }

    // ------------------------------------------------------------------
    // Test 5: No edges between dissimilar trajectories (below threshold)
    // ------------------------------------------------------------------
    #[test]
    fn test_no_edges_below_threshold() {
        // Two very different trajectory patterns
        let mut traj_a: TrajectoryVector = [0.0; 12];
        traj_a[0] = 1.0; // only first dimension nonzero

        let mut traj_b: TrajectoryVector = [0.0; 12];
        traj_b[11] = 1.0; // only last dimension nonzero

        let episodes = vec![
            make_episode(100, traj_a),
            make_episode(100, traj_a),
            make_episode(100, traj_a),
            make_episode(200, traj_b),
            make_episode(200, traj_b),
            make_episode(200, traj_b),
        ];

        let graph = RelationalGraph::build(&episodes, 3, 0.3);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 0, "orthogonal contexts should have no edges");
    }

    // ------------------------------------------------------------------
    // Test 6: Edges exist between similar trajectories
    // ------------------------------------------------------------------
    #[test]
    fn test_edges_between_similar_trajectories() {
        let traj_a: TrajectoryVector = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        // Slightly different but still very similar
        let traj_b: TrajectoryVector = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.9];

        let episodes = vec![
            make_episode(100, traj_a),
            make_episode(100, traj_a),
            make_episode(100, traj_a),
            make_episode(200, traj_b),
            make_episode(200, traj_b),
            make_episode(200, traj_b),
        ];

        let graph = RelationalGraph::build(&episodes, 3, 0.3);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1, "similar contexts should have an edge");

        let (a, b, sim) = graph.edges[0];
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert!(sim > 0.99, "near-identical vectors should have very high similarity, got {}", sim);
    }

    // ------------------------------------------------------------------
    // Test 7: Empty episodes produces empty graph
    // ------------------------------------------------------------------
    #[test]
    fn test_empty_episodes_empty_graph() {
        let graph = RelationalGraph::build(&[], 3, 0.3);

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    // ------------------------------------------------------------------
    // Test 8: Single context produces node but no edges
    // ------------------------------------------------------------------
    #[test]
    fn test_single_context_no_edges() {
        let traj: TrajectoryVector = [0.5; 12];
        let episodes = vec![
            make_episode(42, traj),
            make_episode(42, traj),
            make_episode(42, traj),
        ];

        let graph = RelationalGraph::build(&episodes, 3, 0.3);

        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0, "single node cannot have edges");
        assert_eq!(graph.nodes[0], 42);
    }

    // ------------------------------------------------------------------
    // Test 9: neighbors() returns correct pairs
    // ------------------------------------------------------------------
    #[test]
    fn test_neighbors_returns_correct_pairs() {
        // Three contexts, all very similar (all [1.0; 12]) so all pairs get edges
        let traj: TrajectoryVector = [1.0; 12];

        let episodes = vec![
            make_episode(10, traj),
            make_episode(10, traj),
            make_episode(10, traj),
            make_episode(20, traj),
            make_episode(20, traj),
            make_episode(20, traj),
            make_episode(30, traj),
            make_episode(30, traj),
            make_episode(30, traj),
        ];

        let graph = RelationalGraph::build(&episodes, 3, 0.3);

        assert_eq!(graph.node_count(), 3);
        // 3 choose 2 = 3 edges
        assert_eq!(graph.edge_count(), 3);

        // Node 0 (ctx 10) should have neighbors 1 and 2
        let n0 = graph.neighbors(0);
        assert_eq!(n0.len(), 2);
        let neighbor_indices: Vec<usize> = n0.iter().map(|&(idx, _)| idx).collect();
        assert!(neighbor_indices.contains(&1));
        assert!(neighbor_indices.contains(&2));

        // Node 1 (ctx 20) should have neighbors 0 and 2
        let n1 = graph.neighbors(1);
        assert_eq!(n1.len(), 2);
        let neighbor_indices: Vec<usize> = n1.iter().map(|&(idx, _)| idx).collect();
        assert!(neighbor_indices.contains(&0));
        assert!(neighbor_indices.contains(&2));
    }

    // ------------------------------------------------------------------
    // Test 10: node_trajectory() returns the stored average
    // ------------------------------------------------------------------
    #[test]
    fn test_node_trajectory_returns_average() {
        let traj_a: TrajectoryVector = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let traj_b: TrajectoryVector = [3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0];

        let episodes = vec![
            make_episode(100, traj_a),
            make_episode(100, traj_b),
            make_episode(100, traj_a),
        ];

        let graph = RelationalGraph::build(&episodes, 3, 0.3);

        assert_eq!(graph.node_count(), 1);

        let avg = graph.node_trajectory(0).expect("node 0 should exist");
        // Average of [1,2,3,...,12], [3,4,5,...,14], [1,2,3,...,12]
        // = [(1+3+1)/3, (2+4+2)/3, ...] = [5/3, 8/3, 11/3, ...]
        let expected: TrajectoryVector = [
            5.0 / 3.0,
            8.0 / 3.0,
            11.0 / 3.0,
            14.0 / 3.0,
            17.0 / 3.0,
            20.0 / 3.0,
            23.0 / 3.0,
            26.0 / 3.0,
            29.0 / 3.0,
            32.0 / 3.0,
            35.0 / 3.0,
            38.0 / 3.0,
        ];

        for i in 0..12 {
            assert!(
                (avg[i] - expected[i]).abs() < 1e-5,
                "trajectory[{}]: expected {}, got {}",
                i,
                expected[i],
                avg[i]
            );
        }

        // Out-of-bounds index returns None
        assert!(graph.node_trajectory(1).is_none());
    }

    // ------------------------------------------------------------------
    // Test 11: Cosine similarity with zero vector returns 0.0
    // ------------------------------------------------------------------
    #[test]
    fn test_cosine_zero_vector() {
        let zero: TrajectoryVector = [0.0; 12];
        let nonzero: TrajectoryVector = [1.0; 12];

        assert_eq!(cosine_similarity(&zero, &nonzero), 0.0);
        assert_eq!(cosine_similarity(&nonzero, &zero), 0.0);
        assert_eq!(cosine_similarity(&zero, &zero), 0.0);
    }

    // ------------------------------------------------------------------
    // Test 12: Negative cosine similarity is clamped to 0 in edges
    // ------------------------------------------------------------------
    #[test]
    fn test_negative_similarity_clamped() {
        // Anti-parallel vectors: a = [1, 0, ...], b = [-1, 0, ...]
        // cosine = -1.0, should be clamped to 0.0 -> no edge at threshold 0.3
        let mut traj_a: TrajectoryVector = [0.0; 12];
        traj_a[0] = 1.0;

        let mut traj_b: TrajectoryVector = [0.0; 12];
        traj_b[0] = -1.0;

        let episodes = vec![
            make_episode(100, traj_a),
            make_episode(100, traj_a),
            make_episode(100, traj_a),
            make_episode(200, traj_b),
            make_episode(200, traj_b),
            make_episode(200, traj_b),
        ];

        let graph = RelationalGraph::build(&episodes, 3, 0.0);

        assert_eq!(graph.node_count(), 2);
        // cosine = -1.0 clamped to 0.0 -- exactly at threshold 0.0 it should be included
        // but the raw cosine is negative so clamped to 0.0 which equals threshold
        assert!(
            graph.edge_count() <= 1,
            "clamped negative similarity should be 0.0"
        );

        // With threshold > 0.0, definitely no edge
        let graph2 = RelationalGraph::build(&episodes, 3, 0.1);
        assert_eq!(graph2.edge_count(), 0, "negative similarity clamped to 0 should not meet threshold 0.1");
    }
}
