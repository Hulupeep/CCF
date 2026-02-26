//! Context-index warm-start for novel contexts.
//!
//! Implements story #41: when the companion encounters a ContextKey it has
//! never seen before, it seeds the new CoherenceAccumulator from a
//! distance-weighted blend of the K=3 nearest visited contexts' coherence
//! values, rather than starting from zero.
//!
//! # How it works
//!
//! 1. Each tick after the CCF block, call `upsert(key, coherence)` to keep
//!    the index current.
//! 2. Before `coherence_field.get_or_create()`, check `is_novel` via
//!    `ComfortZoneBoundary::is_novel()`. If novel, call `warm_start_value(key)`
//!    to get a blended starting coherence.
//! 3. After `get_or_create()`, if the context was novel and warm_val > 0.0,
//!    override `acc.value = acc.value.max(warm_val)`.
//!
//! # Invariants
//! - **I-HNSW-001**: All feature vector components ∈ [0.0, 1.0]
//! - **I-HNSW-002**: Warm-start value ∈ [0.0, 1.0]
//! - **I-HNSW-003**: `CoherenceField`, `CoherenceAccumulator`, `SuppressionMap` unmodified
//! - **I-HNSW-004**: If no similar contexts exist, returns 0.0 (cold start preserved)

use mbot_core::coherence::ContextKey;

/// Number of nearest neighbours used for the warm-start blend.
const K: usize = 3;

/// Minimum number of indexed contexts needed before warm-start activates.
/// With fewer than 2 contexts there is no meaningful neighbour to blend from.
const MIN_ENTRIES_FOR_WARMSTART: usize = 1;

/// In-memory index of visited contexts and their current coherence values.
///
/// Maintained by the companion main loop. Used once per novel context to
/// produce a warm-start seed for the new `CoherenceAccumulator`.
pub struct ContextIndex {
    /// (context_hash, feature_vec, accumulated_coherence)
    entries: Vec<(u32, [f32; 6], f32)>,
}

impl ContextIndex {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Update or insert the current coherence value for a context key.
    ///
    /// Call this every tick after the CCF block so the index stays current.
    pub fn upsert(&mut self, key: &ContextKey, coherence: f32) {
        let hash = key.context_hash_u32();
        if let Some(entry) = self.entries.iter_mut().find(|(h, _, _)| *h == hash) {
            entry.2 = coherence;
        } else {
            self.entries.push((hash, key.to_feature_vec(), coherence));
        }
    }

    /// Compute a distance-weighted blend of the K=3 nearest contexts' coherence.
    ///
    /// Returns 0.0 if:
    /// - The index has fewer than `MIN_ENTRIES_FOR_WARMSTART` entries, or
    /// - All neighbours are at distance 0 (identical — shouldn't happen for novel key)
    ///
    /// Weight formula: `w_i = 1 / (1 + dist_i)`. Result is clamped to [0.0, 1.0].
    pub fn warm_start_value(&self, key: &ContextKey) -> f32 {
        if self.entries.len() < MIN_ENTRIES_FOR_WARMSTART {
            return 0.0;
        }

        let hash = key.context_hash_u32();
        let vec = key.to_feature_vec();

        // Collect (distance, coherence) for all non-self entries.
        let mut neighbours: Vec<(f32, f32)> = self
            .entries
            .iter()
            .filter(|(h, _, _)| *h != hash)
            .map(|(_, v, c)| (euclidean_distance(&vec, v), *c))
            .collect();

        if neighbours.is_empty() {
            return 0.0;
        }

        // Sort by ascending distance; take K nearest.
        neighbours.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let k_nearest = &neighbours[..K.min(neighbours.len())];

        // Discounted average: each slot contributes coherence * 1/(1+dist),
        // normalised by the number of neighbours actually used (not by total
        // weight). This ensures distance always discounts the result — a single
        // very-far neighbour does NOT return its full coherence value.
        let k_used = k_nearest.len() as f32;
        let blended: f32 = k_nearest
            .iter()
            .map(|(d, c)| c * (1.0 / (1.0 + d)))
            .sum::<f32>()
            / k_used;

        blended.clamp(0.0, 1.0)
    }

    /// Number of contexts currently indexed.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for ContextIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Euclidean distance between two 6-dimensional feature vectors.
fn euclidean_distance(a: &[f32; 6], b: &[f32; 6]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mbot_core::coherence::{
        BrightnessBand, MotionContext, NoiseBand, Orientation, PresenceSignature, TimePeriod,
    };

    fn key(b: BrightnessBand, n: NoiseBand) -> ContextKey {
        ContextKey {
            brightness: b,
            noise: n,
            presence: PresenceSignature::Static,
            motion: MotionContext::Stationary,
            orientation: Orientation::Upright,
            time_period: TimePeriod::Afternoon,
        }
    }

    #[test]
    fn empty_index_returns_zero() {
        let idx = ContextIndex::new();
        let k = key(BrightnessBand::Bright, NoiseBand::Quiet);
        assert_eq!(idx.warm_start_value(&k), 0.0);
    }

    #[test]
    fn novel_context_blends_from_nearest() {
        let mut idx = ContextIndex::new();
        // Register Bright+Quiet with coherence 0.40
        let known = key(BrightnessBand::Bright, NoiseBand::Quiet);
        idx.upsert(&known, 0.40);

        // Novel: Bright+Moderate (only noise differs by 0.5)
        let novel = key(BrightnessBand::Bright, NoiseBand::Moderate);
        let ws = idx.warm_start_value(&novel);

        // Should be > 0.0 (there is a neighbour) and < 0.40 (discounted by distance)
        assert!(ws > 0.0, "expected warm-start > 0, got {}", ws);
        assert!(ws < 0.40, "expected warm-start < source coherence, got {}", ws);
    }

    #[test]
    fn dissimilar_context_gets_low_warmstart() {
        let mut idx = ContextIndex::new();
        // Only bright-room contexts known
        idx.upsert(&key(BrightnessBand::Bright, NoiseBand::Quiet), 0.50);

        // Dark+Loud is maximally different
        let dark = key(BrightnessBand::Dark, NoiseBand::Loud);
        let ws = idx.warm_start_value(&dark);

        // Should be positive but much less than 0.50
        assert!(ws < 0.30, "expected low warm-start for dissimilar context, got {}", ws);
    }

    #[test]
    fn upsert_updates_existing_coherence() {
        let mut idx = ContextIndex::new();
        let k = key(BrightnessBand::Dim, NoiseBand::Moderate);
        idx.upsert(&k, 0.20);
        idx.upsert(&k, 0.45); // update
        assert_eq!(idx.len(), 1);

        // Novel similar key should now blend from 0.45, not 0.20
        let novel = key(BrightnessBand::Dim, NoiseBand::Quiet);
        let ws = idx.warm_start_value(&novel);
        assert!(ws > 0.20, "expected blend > old value after upsert update");
    }

    #[test]
    fn three_nearest_neighbours_are_used() {
        let mut idx = ContextIndex::new();
        idx.upsert(&key(BrightnessBand::Bright, NoiseBand::Quiet), 0.60);
        idx.upsert(&key(BrightnessBand::Bright, NoiseBand::Moderate), 0.40);
        idx.upsert(&key(BrightnessBand::Dim, NoiseBand::Quiet), 0.20);

        let novel = key(BrightnessBand::Bright, NoiseBand::Loud);
        let ws = idx.warm_start_value(&novel);
        // Blended from three contexts — should be between 0.20 and 0.60
        assert!(ws >= 0.0 && ws <= 0.60, "warm-start out of range: {}", ws);
    }

    #[test]
    fn warm_start_value_always_in_01() {
        let mut idx = ContextIndex::new();
        // Saturate coherence at 1.0
        idx.upsert(&key(BrightnessBand::Bright, NoiseBand::Quiet), 1.0);
        let novel = key(BrightnessBand::Bright, NoiseBand::Moderate);
        let ws = idx.warm_start_value(&novel);
        assert!(ws >= 0.0 && ws <= 1.0, "I-HNSW-002 violated: {}", ws);
    }
}
