//! Stoer-Wagner minimum cut for partitioning a RelationalGraph.
//!
//! Implements the classic O(V*E + V^2 log V) Stoer-Wagner algorithm for
//! finding the global minimum s-t cut in an undirected weighted graph,
//! then extends it to n-way recursive bisection for coherence grouping.
//!
//! # Contract Compliance
//! - **ARCH-001**: All min-cut logic lives in mbot-companion, not mbot-core
//! - **I-MINCUT-001**: No orphan nodes -- every node appears in exactly one partition
//! - **I-MINCUT-002**: Cut weight is globally minimal (Stoer-Wagner guarantee)

use super::relational_graph::RelationalGraph;

// ─── Public Types ───────────────────────────────────────────────────

/// Result of a single min-cut operation.
#[derive(Clone, Debug)]
pub struct MinCutResult {
    /// Partition A (node indices into original graph).
    pub partition_a: Vec<usize>,
    /// Partition B.
    pub partition_b: Vec<usize>,
    /// Weight of the minimum cut.
    pub cut_weight: f32,
}

/// A partition (coherence group) with its member node indices.
#[derive(Clone, Debug)]
pub struct Partition {
    pub nodes: Vec<usize>,
}

/// An edge that crosses a partition boundary.
#[derive(Clone, Debug)]
pub struct BleedingEdge {
    pub node_a: usize,
    pub node_b: usize,
    pub partition_a: usize,
    pub partition_b: usize,
    pub weight: f32,
}

// ─── Internal Adjacency Representation ──────────────────────────────

/// Internal graph representation for the Stoer-Wagner algorithm.
///
/// Uses an adjacency matrix so that edge-weight merges are O(V).
struct SWGraph {
    /// Number of original vertices.
    n: usize,
    /// Adjacency matrix: `adj[i][j]` = total edge weight between super-vertices i and j.
    adj: Vec<Vec<f32>>,
    /// Whether a super-vertex has been merged away.
    merged: Vec<bool>,
    /// `mapping[i]` = list of original node indices that have been merged into super-vertex i.
    mapping: Vec<Vec<usize>>,
}

impl SWGraph {
    /// Build the internal representation from a `RelationalGraph`.
    fn from_relational(graph: &RelationalGraph) -> Self {
        let n = graph.node_count();
        let mut adj = vec![vec![0.0f32; n]; n];

        for &(a, b, w) in &graph.edges {
            adj[a][b] = w;
            adj[b][a] = w;
        }

        let mapping: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();

        Self {
            n,
            adj,
            merged: vec![false; n],
            mapping,
        }
    }

    /// Build from a sub-graph defined by a subset of original nodes.
    ///
    /// `sub_nodes` are indices into the original `RelationalGraph`.
    fn from_subgraph(graph: &RelationalGraph, sub_nodes: &[usize]) -> Self {
        let n = sub_nodes.len();
        let mut adj = vec![vec![0.0f32; n]; n];

        // Map original index -> local index in sub_nodes.
        let mut local_index = std::collections::HashMap::new();
        for (local, &original) in sub_nodes.iter().enumerate() {
            local_index.insert(original, local);
        }

        for &(a, b, w) in &graph.edges {
            if let (Some(&la), Some(&lb)) = (local_index.get(&a), local_index.get(&b)) {
                adj[la][lb] = w;
                adj[lb][la] = w;
            }
        }

        let mapping: Vec<Vec<usize>> = sub_nodes.iter().map(|&orig| vec![orig]).collect();

        Self {
            n,
            adj,
            merged: vec![false; n],
            mapping,
        }
    }

    /// Number of non-merged vertices.
    fn active_count(&self) -> usize {
        self.merged.iter().filter(|&&m| !m).count()
    }

    /// Run one phase of Stoer-Wagner: grow set A by adding the most tightly
    /// connected vertex at each step.
    ///
    /// Returns `(s, t, cut_of_the_phase)` where s and t are the last two
    /// vertices added, and `cut_of_the_phase` is the total weight of edges
    /// from t to all other active vertices.
    fn minimum_cut_phase(&self) -> (usize, usize, f32) {
        let active: Vec<usize> = (0..self.n).filter(|&i| !self.merged[i]).collect();

        // `in_a[i]` = true if vertex i is already in the growing set A.
        let mut in_a = vec![false; self.n];
        // `w[i]` = total edge weight from vertex i to all vertices currently in A.
        let mut w = vec![0.0f32; self.n];

        let mut prev = active[0];
        let mut last = active[0];

        // Add the first active vertex to A.
        in_a[active[0]] = true;
        for &v in &active {
            if v != active[0] {
                w[v] = self.adj[active[0]][v];
            }
        }

        // Add remaining active vertices one at a time.
        for _ in 1..active.len() {
            // Find the vertex not in A with maximum w.
            let mut best = usize::MAX;
            let mut best_w = -1.0f32;
            for &v in &active {
                if !in_a[v] && w[v] > best_w {
                    best = v;
                    best_w = w[v];
                }
            }

            prev = last;
            last = best;
            in_a[best] = true;

            // Update w for all vertices not yet in A.
            for &v in &active {
                if !in_a[v] {
                    w[v] += self.adj[best][v];
                }
            }
        }

        // cut_of_the_phase = w[last] at the time it was added,
        // which is the total weight of edges from `last` to all other active vertices.
        // We recalculate it directly for clarity:
        let cut_weight: f32 = active
            .iter()
            .filter(|&&v| v != last)
            .map(|&v| self.adj[last][v])
            .sum();

        (prev, last, cut_weight)
    }

    /// Merge vertex `t` into vertex `s`: combine their edge weights and mapping.
    fn merge(&mut self, s: usize, t: usize) {
        self.merged[t] = true;

        for i in 0..self.n {
            if i != s && i != t && !self.merged[i] {
                self.adj[s][i] += self.adj[t][i];
                self.adj[i][s] = self.adj[s][i];
            }
        }

        // Move t's original node mapping into s.
        let t_mapping = std::mem::take(&mut self.mapping[t]);
        self.mapping[s].extend(t_mapping);
    }
}

// ─── Public API ─────────────────────────────────────────────────────

/// Find the global minimum 2-way cut using Stoer-Wagner.
///
/// # Edge Cases
/// - Empty graph (0 nodes): returns two empty partitions with cut_weight = 0.0
/// - Single node: partition_a contains that node, partition_b is empty, cut_weight = 0.0
pub fn stoer_wagner_min_cut(graph: &RelationalGraph) -> MinCutResult {
    let n = graph.node_count();

    if n == 0 {
        return MinCutResult {
            partition_a: vec![],
            partition_b: vec![],
            cut_weight: 0.0,
        };
    }

    if n == 1 {
        return MinCutResult {
            partition_a: vec![0],
            partition_b: vec![],
            cut_weight: 0.0,
        };
    }

    let mut sw = SWGraph::from_relational(graph);
    let mut best_cut_weight = f32::MAX;
    let mut best_partition: Vec<usize> = Vec::new();

    // Run V-1 phases.
    while sw.active_count() > 1 {
        let (s, t, cut_weight) = sw.minimum_cut_phase();

        if cut_weight < best_cut_weight {
            best_cut_weight = cut_weight;
            // The partition on the t-side is the set of original nodes merged into t.
            best_partition = sw.mapping[t].clone();
        }

        sw.merge(s, t);
    }

    // Build the two-sided partition from best_partition.
    let partition_b_set: std::collections::HashSet<usize> =
        best_partition.iter().copied().collect();
    let partition_a: Vec<usize> = (0..n).filter(|i| !partition_b_set.contains(i)).collect();

    MinCutResult {
        partition_a,
        partition_b: best_partition,
        cut_weight: best_cut_weight,
    }
}

/// Recursively bisect the graph until no cut weight exceeds `max_cut_weight`.
///
/// Returns partitions where each partition's internal connectivity exceeds
/// `max_cut_weight` (i.e., it cannot be further split without cutting a
/// heavily-connected boundary).
///
/// # Edge Cases
/// - Empty graph (0 nodes): returns empty Vec
/// - Single node: returns one partition containing that node
/// - Fully disconnected: each node becomes its own partition
pub fn min_cut_n_way(graph: &RelationalGraph, max_cut_weight: f32) -> Vec<Partition> {
    let n = graph.node_count();

    if n == 0 {
        return vec![];
    }

    if n == 1 {
        return vec![Partition { nodes: vec![0] }];
    }

    // Check if the graph is fully disconnected (no edges).
    if graph.edge_count() == 0 {
        return (0..n).map(|i| Partition { nodes: vec![i] }).collect();
    }

    // Start recursive bisection with the full set of node indices.
    let all_nodes: Vec<usize> = (0..n).collect();
    let mut result = Vec::new();
    recursive_bisect(graph, &all_nodes, max_cut_weight, &mut result);
    result
}

/// Recursive helper: given a subset of original-graph node indices, either
/// accept them as an indivisible partition or bisect further.
fn recursive_bisect(
    graph: &RelationalGraph,
    nodes: &[usize],
    max_cut_weight: f32,
    out: &mut Vec<Partition>,
) {
    if nodes.len() <= 1 {
        if !nodes.is_empty() {
            out.push(Partition {
                nodes: nodes.to_vec(),
            });
        }
        return;
    }

    // Build a sub-graph restricted to these nodes and run Stoer-Wagner on it.
    let mut sw = SWGraph::from_subgraph(graph, nodes);

    // Check if the sub-graph has any edges at all.
    let has_edges = nodes.iter().enumerate().any(|(i, _)| {
        nodes.iter().enumerate().any(|(j, _)| j > i && sw.adj[i][j] > 0.0)
    });

    if !has_edges {
        // Fully disconnected sub-graph: each node is its own partition.
        for &node in nodes {
            out.push(Partition { nodes: vec![node] });
        }
        return;
    }

    let mut best_cut_weight = f32::MAX;
    let mut best_partition: Vec<usize> = Vec::new(); // local indices of t-side

    while sw.active_count() > 1 {
        let (s, t, cut_weight) = sw.minimum_cut_phase();

        if cut_weight < best_cut_weight {
            best_cut_weight = cut_weight;
            best_partition = sw.mapping[t].clone();
        }

        sw.merge(s, t);
    }

    if best_cut_weight > max_cut_weight {
        // Indivisible: the internal connectivity is too strong to split.
        out.push(Partition {
            nodes: nodes.to_vec(),
        });
        return;
    }

    // Split into two groups based on the best partition.
    // `best_partition` contains local indices; convert back to original indices.
    let b_set: std::collections::HashSet<usize> = best_partition.iter().copied().collect();
    let side_a: Vec<usize> = (0..nodes.len())
        .filter(|i| !b_set.contains(i))
        .map(|i| nodes[i])
        .collect();
    let side_b: Vec<usize> = best_partition.iter().map(|&i| nodes[i]).collect();

    // Recurse on both halves.
    recursive_bisect(graph, &side_a, max_cut_weight, out);
    recursive_bisect(graph, &side_b, max_cut_weight, out);
}

/// Find all edges that cross partition boundaries.
///
/// Scans every edge in the graph and reports those whose endpoints fall
/// in different partitions.
///
/// # Edge Cases
/// - Single partition: returns empty Vec (no cross-partition edges possible)
/// - Empty partitions list: returns empty Vec
pub fn find_bleeding_edges(graph: &RelationalGraph, partitions: &[Partition]) -> Vec<BleedingEdge> {
    if partitions.len() <= 1 {
        return vec![];
    }

    // Build a lookup: original node index -> partition index.
    let n = graph.node_count();
    let mut node_to_partition = vec![usize::MAX; n];
    for (p_idx, partition) in partitions.iter().enumerate() {
        for &node in &partition.nodes {
            if node < n {
                node_to_partition[node] = p_idx;
            }
        }
    }

    let mut bleeding = Vec::new();
    for &(a, b, w) in &graph.edges {
        let pa = node_to_partition[a];
        let pb = node_to_partition[b];
        if pa != pb && pa != usize::MAX && pb != usize::MAX {
            bleeding.push(BleedingEdge {
                node_a: a,
                node_b: b,
                partition_a: pa,
                partition_b: pb,
                weight: w,
            });
        }
    }

    bleeding
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

    /// Build a graph with two clear clusters:
    /// Cluster {0, 1} with strong internal edges and Cluster {2, 3} with strong
    /// internal edges, connected by weak cross-cluster edges.
    fn make_two_cluster_graph() -> RelationalGraph {
        // Cluster A: nodes 0 and 1 share trajectory [1,1,0,0,0,0,0,0,0,0,0,0]
        let traj_a: TrajectoryVector = [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        // Cluster B: nodes 2 and 3 share trajectory [0,0,1,1,0,0,0,0,0,0,0,0]
        let traj_b: TrajectoryVector = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        // Node 1 also has a slight component toward cluster B for a weak cross-edge
        let traj_a_bridge: TrajectoryVector = [1.0, 1.0, 0.1, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        // Node 2 also has a slight component toward cluster A for a weak cross-edge
        let traj_b_bridge: TrajectoryVector = [0.1, 0.1, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let episodes = vec![
            make_episode(0, traj_a),
            make_episode(1, traj_a_bridge),
            make_episode(2, traj_b_bridge),
            make_episode(3, traj_b),
        ];

        RelationalGraph::build(&episodes, 1, 0.01)
    }

    // ------------------------------------------------------------------
    // Test 1: Empty graph produces trivial result
    // ------------------------------------------------------------------
    #[test]
    fn test_empty_graph_trivial_result() {
        let graph = RelationalGraph::build(&[], 1, 0.0);
        let result = stoer_wagner_min_cut(&graph);

        assert!(result.partition_a.is_empty());
        assert!(result.partition_b.is_empty());
        assert_eq!(result.cut_weight, 0.0);
    }

    // ------------------------------------------------------------------
    // Test 2: Single node returns partition with that node
    // ------------------------------------------------------------------
    #[test]
    fn test_single_node_single_partition() {
        let episodes = vec![make_episode(42, [1.0; 12])];
        let graph = RelationalGraph::build(&episodes, 1, 0.0);

        assert_eq!(graph.node_count(), 1);

        let result = stoer_wagner_min_cut(&graph);
        assert_eq!(result.partition_a, vec![0]);
        assert!(result.partition_b.is_empty());
        assert_eq!(result.cut_weight, 0.0);
    }

    // ------------------------------------------------------------------
    // Test 3: Two-node graph with one edge produces correct cut
    // ------------------------------------------------------------------
    #[test]
    fn test_two_node_graph_correct_cut() {
        // Two nodes with identical trajectories -> edge with sim ~1.0
        let traj: TrajectoryVector = [1.0; 12];
        let episodes = vec![make_episode(10, traj), make_episode(20, traj)];
        let graph = RelationalGraph::build(&episodes, 1, 0.0);

        assert_eq!(graph.node_count(), 2);
        assert!(graph.edge_count() > 0);

        let result = stoer_wagner_min_cut(&graph);

        // One partition should have node 0, the other node 1 (or vice versa).
        let total_nodes = result.partition_a.len() + result.partition_b.len();
        assert_eq!(total_nodes, 2, "all nodes must be partitioned");
        assert!(result.cut_weight > 0.0, "connected graph should have positive cut");
    }

    // ------------------------------------------------------------------
    // Test 4: 4-node graph with clear cluster structure
    // ------------------------------------------------------------------
    #[test]
    fn test_four_node_two_clusters() {
        let graph = make_two_cluster_graph();

        assert_eq!(graph.node_count(), 4);

        let result = stoer_wagner_min_cut(&graph);

        // The min cut should separate the two clusters.
        let total = result.partition_a.len() + result.partition_b.len();
        assert_eq!(total, 4, "I-MINCUT-001: all 4 nodes must be in a partition");

        // The min cut should have the two clusters on opposite sides.
        // Either partition_a = {0,1} and partition_b = {2,3} or vice versa.
        let mut a_sorted = result.partition_a.clone();
        let mut b_sorted = result.partition_b.clone();
        a_sorted.sort();
        b_sorted.sort();

        let is_clean_split = (a_sorted == vec![0, 1] && b_sorted == vec![2, 3])
            || (a_sorted == vec![2, 3] && b_sorted == vec![0, 1]);

        assert!(
            is_clean_split,
            "Expected clusters {{0,1}} and {{2,3}}, got A={:?} B={:?}",
            a_sorted, b_sorted
        );
    }

    // ------------------------------------------------------------------
    // Test 5: Fully connected 3-node graph finds correct minimum cut
    // ------------------------------------------------------------------
    #[test]
    fn test_fully_connected_three_nodes() {
        // Three nodes all with very similar trajectories -> fully connected.
        let traj: TrajectoryVector = [1.0; 12];
        let episodes = vec![
            make_episode(10, traj),
            make_episode(20, traj),
            make_episode(30, traj),
        ];
        let graph = RelationalGraph::build(&episodes, 1, 0.0);

        assert_eq!(graph.node_count(), 3);

        let result = stoer_wagner_min_cut(&graph);

        let total = result.partition_a.len() + result.partition_b.len();
        assert_eq!(total, 3, "I-MINCUT-001: all nodes must be partitioned");

        // In a fully connected graph with equal weights, the min cut isolates
        // one node (cut = sum of its two edges) rather than splitting 2-1 with
        // higher total. With identical weights w, cutting 1 node costs 2w
        // while cutting 2-1 also costs 2w. Either is valid.
        assert!(
            result.cut_weight > 0.0,
            "fully connected graph must have positive cut weight"
        );
    }

    // ------------------------------------------------------------------
    // Test 6: min_cut_n_way with threshold separates weak connections
    // ------------------------------------------------------------------
    #[test]
    fn test_n_way_separates_weak_connections() {
        let graph = make_two_cluster_graph();

        // Use a threshold that is above the cross-cluster edge weight
        // but below the intra-cluster weight. The cross-cluster similarity
        // for our bridge trajectories is relatively low.
        let partitions = min_cut_n_way(&graph, 0.5);

        // Should separate into at least 2 partitions.
        assert!(
            partitions.len() >= 2,
            "weak connections should be split, got {} partitions",
            partitions.len()
        );

        // I-MINCUT-001: every node must appear exactly once.
        let mut all_nodes: Vec<usize> = partitions.iter().flat_map(|p| p.nodes.iter().copied()).collect();
        all_nodes.sort();
        all_nodes.dedup();
        assert_eq!(all_nodes.len(), graph.node_count(), "I-MINCUT-001: no orphan nodes");
    }

    // ------------------------------------------------------------------
    // Test 7: min_cut_n_way returns single partition when graph is tight
    // ------------------------------------------------------------------
    #[test]
    fn test_n_way_single_partition_tight_graph() {
        // Three nodes with identical trajectories -> very high similarity.
        // Each edge has weight ~1.0. The minimum cut isolates one node and
        // costs ~2.0 (two edges). If max_cut_weight is below that, the
        // graph is indivisible.
        let traj: TrajectoryVector = [1.0; 12];
        let episodes = vec![
            make_episode(10, traj),
            make_episode(20, traj),
            make_episode(30, traj),
        ];
        let graph = RelationalGraph::build(&episodes, 1, 0.0);

        // Threshold below the min-cut cost: no split is cheap enough.
        let partitions = min_cut_n_way(&graph, 0.5);

        assert_eq!(
            partitions.len(),
            1,
            "tight graph with low threshold should remain as one partition"
        );
        assert_eq!(partitions[0].nodes.len(), 3);
    }

    // ------------------------------------------------------------------
    // Test 8: find_bleeding_edges identifies cross-partition edges
    // ------------------------------------------------------------------
    #[test]
    fn test_find_bleeding_edges_cross_partition() {
        let graph = make_two_cluster_graph();

        // Manually define partitions: {0,1} and {2,3}.
        let partitions = vec![
            Partition { nodes: vec![0, 1] },
            Partition { nodes: vec![2, 3] },
        ];

        let bleeding = find_bleeding_edges(&graph, &partitions);

        // There should be at least one cross-cluster edge.
        assert!(
            !bleeding.is_empty(),
            "should find cross-partition edges between the two clusters"
        );

        // Every bleeding edge should connect different partitions.
        for edge in &bleeding {
            assert_ne!(
                edge.partition_a, edge.partition_b,
                "bleeding edge should cross partition boundary"
            );
        }
    }

    // ------------------------------------------------------------------
    // Test 9: find_bleeding_edges returns empty for single partition
    // ------------------------------------------------------------------
    #[test]
    fn test_find_bleeding_edges_single_partition_empty() {
        let traj: TrajectoryVector = [1.0; 12];
        let episodes = vec![
            make_episode(10, traj),
            make_episode(20, traj),
        ];
        let graph = RelationalGraph::build(&episodes, 1, 0.0);

        let partitions = vec![Partition { nodes: vec![0, 1] }];
        let bleeding = find_bleeding_edges(&graph, &partitions);

        assert!(
            bleeding.is_empty(),
            "single partition should have no bleeding edges"
        );
    }

    // ------------------------------------------------------------------
    // Test 10: Disconnected components become separate partitions
    // ------------------------------------------------------------------
    #[test]
    fn test_disconnected_components_separate() {
        // Create two pairs of nodes with no edges between them.
        // Pair 1: orthogonal to Pair 2.
        let mut traj_a: TrajectoryVector = [0.0; 12];
        traj_a[0] = 1.0;
        let mut traj_b: TrajectoryVector = [0.0; 12];
        traj_b[6] = 1.0;

        let episodes = vec![
            make_episode(10, traj_a),
            make_episode(20, traj_a),
            make_episode(30, traj_b),
            make_episode(40, traj_b),
        ];

        // High threshold so only very similar nodes get edges.
        let graph = RelationalGraph::build(&episodes, 1, 0.99);

        // Nodes 0,1 are identical (sim=1.0 -> edge), nodes 2,3 are identical (sim=1.0 -> edge).
        // But 0-2, 0-3, 1-2, 1-3 are orthogonal (sim=0.0 -> no edge).
        assert_eq!(graph.node_count(), 4);

        let partitions = min_cut_n_way(&graph, 0.5);

        // Should separate into at least 2 groups.
        assert!(
            partitions.len() >= 2,
            "disconnected components should become separate partitions, got {} partitions",
            partitions.len()
        );

        // I-MINCUT-001: no orphan nodes.
        let mut all_nodes: Vec<usize> = partitions.iter().flat_map(|p| p.nodes.iter().copied()).collect();
        all_nodes.sort();
        all_nodes.dedup();
        assert_eq!(all_nodes.len(), 4, "I-MINCUT-001: all 4 nodes must appear");
    }

    // ------------------------------------------------------------------
    // Test 11: No orphan nodes invariant (I-MINCUT-001)
    // ------------------------------------------------------------------
    #[test]
    fn test_no_orphan_nodes_invariant() {
        // Build a non-trivial graph and verify the invariant.
        let traj_a: TrajectoryVector = [1.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let traj_b: TrajectoryVector = [0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let traj_c: TrajectoryVector = [0.0, 0.0, 1.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let traj_d: TrajectoryVector = [0.0, 0.0, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let traj_e: TrajectoryVector = [0.0, 0.0, 0.0, 0.0, 1.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let episodes = vec![
            make_episode(10, traj_a),
            make_episode(20, traj_b),
            make_episode(30, traj_c),
            make_episode(40, traj_d),
            make_episode(50, traj_e),
        ];

        let graph = RelationalGraph::build(&episodes, 1, 0.0);
        let n = graph.node_count();

        // Test stoer_wagner_min_cut invariant
        let result = stoer_wagner_min_cut(&graph);
        let mut all_sw: Vec<usize> = result.partition_a.iter().chain(result.partition_b.iter()).copied().collect();
        all_sw.sort();
        all_sw.dedup();
        assert_eq!(
            all_sw.len(),
            n,
            "I-MINCUT-001: stoer_wagner must account for all {} nodes, got {:?}",
            n,
            all_sw
        );

        // Test min_cut_n_way invariant
        let partitions = min_cut_n_way(&graph, 0.3);
        let mut all_nway: Vec<usize> = partitions.iter().flat_map(|p| p.nodes.iter().copied()).collect();
        all_nway.sort();
        let unique_count = {
            let mut deduped = all_nway.clone();
            deduped.dedup();
            deduped.len()
        };
        assert_eq!(
            unique_count, n,
            "I-MINCUT-001: min_cut_n_way must have all {} nodes exactly once, got {:?}",
            n, all_nway
        );
        // Also check no duplicates.
        assert_eq!(
            all_nway.len(), unique_count,
            "I-MINCUT-001: no node should appear in multiple partitions"
        );
    }

    // ------------------------------------------------------------------
    // Test 12: find_bleeding_edges with empty partitions
    // ------------------------------------------------------------------
    #[test]
    fn test_find_bleeding_edges_empty_partitions() {
        let graph = RelationalGraph::build(&[], 1, 0.0);
        let bleeding = find_bleeding_edges(&graph, &[]);
        assert!(bleeding.is_empty());
    }

    // ------------------------------------------------------------------
    // Test 13: min_cut_n_way with fully disconnected graph
    // ------------------------------------------------------------------
    #[test]
    fn test_n_way_fully_disconnected() {
        // Each node has a unique orthogonal trajectory, high threshold -> no edges.
        let mut traj_a: TrajectoryVector = [0.0; 12];
        traj_a[0] = 1.0;
        let mut traj_b: TrajectoryVector = [0.0; 12];
        traj_b[1] = 1.0;
        let mut traj_c: TrajectoryVector = [0.0; 12];
        traj_c[2] = 1.0;

        let episodes = vec![
            make_episode(10, traj_a),
            make_episode(20, traj_b),
            make_episode(30, traj_c),
        ];

        let graph = RelationalGraph::build(&episodes, 1, 0.5);

        // With threshold 0.5, orthogonal vectors (sim=0) won't get edges.
        assert_eq!(graph.edge_count(), 0);

        let partitions = min_cut_n_way(&graph, 0.1);

        assert_eq!(
            partitions.len(),
            3,
            "fully disconnected graph: each node should be its own partition"
        );

        for p in &partitions {
            assert_eq!(p.nodes.len(), 1);
        }
    }
}
