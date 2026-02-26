//! Context-key trust-manifold graph with subpolynomial dynamic min-cut.
//!
//! Implements stories #40 (similarity graph) and #44 (trust-weighted edges).
//!
//! # Graph design
//!
//! **Graph A — World Shape (sensory similarity only)**
//! Used as the initial edge weight and as a fallback before trust is established.
//! Weight = `cosine_similarity(feature_vec_A, feature_vec_B)` ∈ [0.0, 1.0].
//! Answers: *how similar are these environments?*
//!
//! **Graph B — Trust Shape (patent-faithful)**
//! Activates once both endpoints have ≥ `MIN_TRUST_OBSERVATIONS` interactions.
//! Weight = `sim × tanh(coh_A × TRUST_SCALE) × tanh(coh_B × TRUST_SCALE)`.
//! Answers: *how similar AND how safe has this transition proven to be?*
//! The min-cut of Graph B drops when the robot repeatedly experiences a context
//! as unsafe — creating a "thin bridge" in the trust manifold.
//!
//! # Min-cut computation
//!
//! For the demo graph (2–20 nodes) the weighted min-cut proxy used here is
//! **minimum weighted degree**: the minimum sum of trust-weighted edge weights over
//! all registered contexts. This is the exact min-cut for 2-node graphs. For larger
//! graphs it is an upper bound (the single-vertex cut). It responds directly to
//! trust changes, which is the patent-relevant signal.
//!
//! `SubpolynomialMinCut` (arXiv:2512.13105) is the target algorithm per I-BNDRY-004.
//! Integration is deferred to issue #55: `insert_edge` and `build()` are O(n·λ_max)
//! expensive even for 2-node graphs at default config (λ_max=1000 → 6000 forests),
//! making them unusable in the tick loop or tests without a performance fix.
//!
//! # Invariants
//! - **I-BNDRY-001**: Min-cut is on the context-key graph (not the episode graph).
//! - **I-BNDRY-002**: Edge weight ∈ [0.0, 1.0] (Graph A or Graph B, clamped).
//! - **I-BNDRY-003**: Edges inserted only when similarity > EDGE_THRESHOLD (0.1).
//! - **I-BNDRY-004**: Target algorithm is `SubpolynomialMinCut` (arXiv:2512.13105).
//!   Current implementation uses minimum weighted degree proxy. Full integration
//!   blocked on issue #55 (performance fix for small graphs).
//! - **I-TRUST-001**: Trust component activates only when context has ≥ MIN_TRUST_OBSERVATIONS.
//! - **I-TRUST-002**: Trust change smoothness enforced upstream by CoherenceAccumulator gate.
//! - **I-TRUST-003**: Warm-start (ContextIndex / Graph A) and boundary (Graph B) are separate.

use std::collections::HashMap;

use mbot_core::coherence::ContextKey;

/// Minimum cosine similarity for an edge to be inserted (I-BNDRY-003).
const EDGE_THRESHOLD: f32 = 0.1;

/// Minimum positive interactions before the trust component activates (I-TRUST-001).
pub const MIN_TRUST_OBSERVATIONS: u32 = 50;

/// Scale applied to coherence inside tanh in `trust_weighted_edge`.
/// At coherence=1.0: tanh(2.0) ≈ 0.964.
/// At coherence=0.8: tanh(1.6) ≈ 0.922.
/// At coherence=0.15: tanh(0.30) ≈ 0.291.
const TRUST_SCALE: f32 = 2.0;

/// Per-context trust record.
#[derive(Clone, Copy, Default)]
struct TrustRecord {
    coherence: f32,
    observations: u32,
}

/// Maintains the context-key trust-manifold graph and its min-cut proxy.
///
/// Call flow:
/// 1. `report_context(&key)` — register a context once per novel key.
/// 2. `update_trust(&key, coherence, observations)` — update trust data and
///    re-weight all edges for that context. Call every tick from the main loop.
/// 3. `min_cut_value()` — returns the current weighted min-cut proxy.
pub struct ComfortZoneBoundary {
    /// Source-of-truth edge weights: canonical (min_hash, max_hash) → weight.
    edge_weights: HashMap<(u32, u32), f32>,
    /// Context hash → 6-dim feature vector (for edge recomputation).
    context_vecs: HashMap<u32, [f32; 6]>,
    /// Registered context hashes (subset of context_vecs; duplicated for fast count/novelty).
    key_to_vertex: HashMap<u32, u64>,
    /// Next vertex ID to assign.
    next_vertex_id: u64,
    /// Per-context trust record.
    context_trust: HashMap<u32, TrustRecord>,
    /// Cached min-cut proxy (minimum weighted degree over all contexts).
    current_min_cut: f32,
}

impl ComfortZoneBoundary {
    pub fn new() -> Self {
        Self {
            edge_weights: HashMap::new(),
            context_vecs: HashMap::new(),
            key_to_vertex: HashMap::new(),
            next_vertex_id: 0,
            context_trust: HashMap::new(),
            current_min_cut: 0.0,
        }
    }

    /// Register the current context key.
    ///
    /// Inserts a new node and computes trust-weighted edges to all existing nodes
    /// above `EDGE_THRESHOLD`. No-op for already-known keys.
    pub fn report_context(&mut self, key: &ContextKey) {
        let hash = key.context_hash_u32();
        if self.key_to_vertex.contains_key(&hash) {
            return;
        }

        let new_id = self.next_vertex_id;
        self.next_vertex_id += 1;
        let new_vec = key.to_feature_vec();
        let new_trust = self.context_trust.get(&hash).copied().unwrap_or_default();

        let existing: Vec<(u32, [f32; 6])> =
            self.context_vecs.iter().map(|(h, v)| (*h, *v)).collect();

        for (existing_hash, existing_vec) in existing {
            let sim = cosine_similarity(&new_vec, &existing_vec);
            if sim > EDGE_THRESHOLD {
                let existing_trust = self
                    .context_trust
                    .get(&existing_hash)
                    .copied()
                    .unwrap_or_default();
                let weight = trust_weighted_edge(
                    sim,
                    new_trust.coherence,
                    new_trust.observations,
                    existing_trust.coherence,
                    existing_trust.observations,
                );
                let canonical = (hash.min(existing_hash), hash.max(existing_hash));
                self.edge_weights.insert(canonical, weight);
            }
        }

        self.key_to_vertex.insert(hash, new_id);
        self.context_vecs.insert(hash, new_vec);
        self.recompute_min_cut_proxy();
    }

    /// Update trust record and re-weight all edges for this context.
    ///
    /// I-TRUST-001: trust component only activates when `observations >= MIN_TRUST_OBSERVATIONS`.
    pub fn update_trust(&mut self, key: &ContextKey, coherence: f32, observations: u32) {
        let hash = key.context_hash_u32();
        self.context_trust.insert(hash, TrustRecord { coherence, observations });

        if !self.key_to_vertex.contains_key(&hash) {
            return;
        }

        let my_vec = match self.context_vecs.get(&hash) {
            Some(v) => *v,
            None => return,
        };

        let pairs: Vec<(u32, u32)> = self
            .edge_weights
            .keys()
            .filter(|&&(h1, h2)| h1 == hash || h2 == hash)
            .copied()
            .collect();

        for (h1, h2) in pairs {
            let neighbor_hash = if h1 == hash { h2 } else { h1 };
            let neighbor_vec = match self.context_vecs.get(&neighbor_hash) {
                Some(v) => *v,
                None => continue,
            };
            let neighbor_trust = self
                .context_trust
                .get(&neighbor_hash)
                .copied()
                .unwrap_or_default();
            let sim = cosine_similarity(&my_vec, &neighbor_vec);
            let new_weight = trust_weighted_edge(
                sim,
                coherence,
                observations,
                neighbor_trust.coherence,
                neighbor_trust.observations,
            );
            self.edge_weights.insert((h1, h2), new_weight);
        }

        self.recompute_min_cut_proxy();
    }

    /// Weighted min-cut proxy (minimum weighted degree over all contexts).
    ///
    /// Returns 0.0 until at least two distinct context keys are registered.
    /// This is the exact min-cut for 2-node graphs and a conservative upper bound
    /// for larger graphs. Drops when a context's trust-weighted edge weights thin out.
    pub fn min_cut_value(&self) -> f32 {
        self.current_min_cut
    }

    /// Number of distinct context keys registered.
    pub fn context_count(&self) -> usize {
        self.key_to_vertex.len()
    }

    /// Returns true if this context key has never been registered (for warm-start).
    pub fn is_novel(&self, key: &ContextKey) -> bool {
        !self.key_to_vertex.contains_key(&key.context_hash_u32())
    }

    /// Recompute the min-cut proxy from current edge_weights.
    ///
    /// Minimum weighted degree: for each context, sum the trust-weighted edges
    /// incident to it. The minimum over all contexts is the proxy.
    fn recompute_min_cut_proxy(&mut self) {
        if self.key_to_vertex.len() < 2 {
            self.current_min_cut = 0.0;
            return;
        }

        // Sum edge weights per context.
        let mut weighted_degree: HashMap<u32, f32> = HashMap::new();
        for (&(h1, h2), &weight) in &self.edge_weights {
            *weighted_degree.entry(h1).or_insert(0.0) += weight;
            *weighted_degree.entry(h2).or_insert(0.0) += weight;
        }

        // Min over all registered contexts (isolated contexts contribute 0).
        let min_deg = self
            .key_to_vertex
            .keys()
            .map(|h| weighted_degree.get(h).copied().unwrap_or(0.0))
            .fold(f32::INFINITY, f32::min);

        self.current_min_cut = if min_deg.is_infinite() { 0.0 } else { min_deg };
    }
}

impl Default for ComfortZoneBoundary {
    fn default() -> Self {
        Self::new()
    }
}

/// Trust-weighted edge weight formula.
///
/// Returns `sim × tanh(coh_a × TRUST_SCALE) × tanh(coh_b × TRUST_SCALE)` when
/// both endpoints have sufficient observations (I-TRUST-001). Falls back to raw
/// cosine similarity (Graph A) before minimum evidence is established.
fn trust_weighted_edge(sim: f32, coh_a: f32, obs_a: u32, coh_b: f32, obs_b: u32) -> f32 {
    if obs_a >= MIN_TRUST_OBSERVATIONS && obs_b >= MIN_TRUST_OBSERVATIONS {
        let trust_a = (coh_a * TRUST_SCALE).tanh();
        let trust_b = (coh_b * TRUST_SCALE).tanh();
        (sim * trust_a * trust_b).clamp(0.0, 1.0)
    } else {
        sim // Graph A fallback (I-TRUST-001)
    }
}

/// Cosine similarity between two 6-dimensional feature vectors.
fn cosine_similarity(a: &[f32; 6], b: &[f32; 6]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a < 1e-6 || norm_b < 1e-6 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mbot_core::coherence::{
        BrightnessBand, MotionContext, NoiseBand, Orientation, PresenceSignature, TimePeriod,
    };

    fn make_key(b: BrightnessBand, n: NoiseBand) -> ContextKey {
        ContextKey {
            brightness: b,
            noise: n,
            presence: PresenceSignature::Static,
            motion: MotionContext::Stationary,
            orientation: Orientation::Upright,
            time_period: TimePeriod::Afternoon,
        }
    }

    // ---- structural tests --------------------------------------------------

    #[test]
    fn single_context_min_cut_is_zero() {
        let mut b = ComfortZoneBoundary::new();
        b.report_context(&make_key(BrightnessBand::Bright, NoiseBand::Quiet));
        assert_eq!(b.min_cut_value(), 0.0);
        assert_eq!(b.context_count(), 1);
    }

    #[test]
    fn two_similar_contexts_have_positive_min_cut() {
        let mut b = ComfortZoneBoundary::new();
        let key_a = make_key(BrightnessBand::Bright, NoiseBand::Quiet);
        let key_b = make_key(BrightnessBand::Bright, NoiseBand::Moderate);
        b.report_context(&key_a);
        b.report_context(&key_b);
        b.update_trust(&key_a, 0.80, MIN_TRUST_OBSERVATIONS);
        b.update_trust(&key_b, 0.80, MIN_TRUST_OBSERVATIONS);
        assert_eq!(b.context_count(), 2);
        assert!(
            b.min_cut_value() > 0.0,
            "two trusted similar contexts should have positive min-cut (got {})",
            b.min_cut_value()
        );
    }

    #[test]
    fn duplicate_report_is_noop() {
        let mut b = ComfortZoneBoundary::new();
        let key = make_key(BrightnessBand::Dim, NoiseBand::Quiet);
        b.report_context(&key);
        b.report_context(&key);
        assert_eq!(b.context_count(), 1);
    }

    #[test]
    fn cosine_identical_vectors_is_one() {
        let v = [1.0f32, 0.5, 0.33, 0.0, 0.0, 0.33];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_orthogonal_vectors_is_zero() {
        let a = [1.0f32, 0.0, 0.0, 0.0, 0.0, 0.0];
        let b = [0.0f32, 1.0, 0.0, 0.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn feature_vec_range() {
        let key = make_key(BrightnessBand::Dark, NoiseBand::Loud);
        let v = key.to_feature_vec();
        for x in &v {
            assert!(*x >= 0.0 && *x <= 1.0, "feature out of range: {}", x);
        }
    }

    // ---- trust-weighted edge tests (story #44) ----------------------------

    #[test]
    fn cold_weight_equals_cosine_similarity() {
        // I-TRUST-001: below MIN_TRUST_OBSERVATIONS weight = cosine sim (Graph A).
        let key_a = make_key(BrightnessBand::Bright, NoiseBand::Quiet);
        let key_b = make_key(BrightnessBand::Bright, NoiseBand::Moderate);
        let sim = cosine_similarity(&key_a.to_feature_vec(), &key_b.to_feature_vec());
        let w = trust_weighted_edge(sim, 0.8, 0, 0.8, 0);
        assert!((w - sim).abs() < 1e-6, "cold weight should equal cosine sim");
    }

    #[test]
    fn trusted_similar_contexts_have_thick_edge() {
        // coh=0.80, obs≥MIN → weight > 0.6.
        let key_a = make_key(BrightnessBand::Bright, NoiseBand::Quiet);
        let key_b = make_key(BrightnessBand::Bright, NoiseBand::Moderate);
        let sim = cosine_similarity(&key_a.to_feature_vec(), &key_b.to_feature_vec());
        let w = trust_weighted_edge(sim, 0.80, MIN_TRUST_OBSERVATIONS, 0.80, MIN_TRUST_OBSERVATIONS);
        assert!(w > 0.6, "well-trusted similar contexts: got {:.3}, sim={:.3}", w, sim);
    }

    #[test]
    fn dangerous_familiar_context_has_thin_edge() {
        // One trusted (0.80), one startled (0.15) → weight < 0.3.
        let key_a = make_key(BrightnessBand::Bright, NoiseBand::Quiet);
        let key_b = make_key(BrightnessBand::Bright, NoiseBand::Moderate);
        let sim = cosine_similarity(&key_a.to_feature_vec(), &key_b.to_feature_vec());
        let w =
            trust_weighted_edge(sim, 0.80, MIN_TRUST_OBSERVATIONS, 0.15, MIN_TRUST_OBSERVATIONS);
        assert!(w < 0.3, "familiar-but-dangerous context: got {:.3}, sim={:.3}", w, sim);
    }

    #[test]
    fn update_trust_changes_min_cut() {
        // Trust loss should reduce the min-cut proxy value.
        let mut b = ComfortZoneBoundary::new();
        let key_a = make_key(BrightnessBand::Bright, NoiseBand::Quiet);
        let key_b = make_key(BrightnessBand::Bright, NoiseBand::Moderate);

        b.report_context(&key_a);
        b.report_context(&key_b);
        b.update_trust(&key_a, 0.80, MIN_TRUST_OBSERVATIONS);
        b.update_trust(&key_b, 0.80, MIN_TRUST_OBSERVATIONS);
        let trusted_cut = b.min_cut_value();
        assert!(trusted_cut > 0.0, "trusted min_cut should be positive");

        // Context B experiences repeated startles.
        b.update_trust(&key_b, 0.10, MIN_TRUST_OBSERVATIONS);
        let degraded_cut = b.min_cut_value();

        assert!(
            degraded_cut < trusted_cut,
            "min_cut should drop when one context loses trust ({:.3} < {:.3})",
            degraded_cut,
            trusted_cut
        );
    }

    #[test]
    fn edge_weight_clamped_to_unit_interval() {
        // I-BNDRY-002: edge weight ∈ [0.0, 1.0].
        let w = trust_weighted_edge(1.0, 1.0, MIN_TRUST_OBSERVATIONS, 1.0, MIN_TRUST_OBSERVATIONS);
        assert!(w >= 0.0 && w <= 1.0, "edge weight out of [0,1]: {}", w);
    }
}
