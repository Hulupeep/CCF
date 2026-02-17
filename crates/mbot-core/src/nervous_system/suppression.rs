//! Suppression rules and map — fixed-size LRU rule store.
//!
//! Invariants:
//! - I-STRT-001: no_std compatible, fixed-size array
//! - I-STRT-002: Max 32 rules, LRU eviction
//! - I-STRT-003: suppression_factor clamped to [0.3, 1.0]
//! - I-STRT-004: Keyed by (StimulusKind, context_hash)

use super::stimulus::StimulusKind;

/// Maximum number of suppression rules stored simultaneously.
pub const MAX_SUPPRESSION_RULES: usize = 32;

/// Minimum suppression factor — always retain 30% residual response (STRT-003).
pub const MIN_SUPPRESSION_FACTOR: f32 = 0.3;

/// A learned suppression rule for a specific stimulus in a specific context.
#[derive(Clone, Copy, Debug)]
pub struct SuppressionRule {
    pub stimulus_kind: StimulusKind,
    pub context_hash: u32,
    /// Suppression factor [0.3, 1.0]. Lower = more suppressed. Never below 0.3 (STRT-003).
    pub suppression_factor: f32,
    pub observation_count: u16,
    pub last_updated_tick: u64,
}

/// Fixed-size, no_std compatible map of suppression rules with LRU eviction.
#[derive(Clone, Debug)]
pub struct SuppressionMap {
    rules: [Option<SuppressionRule>; MAX_SUPPRESSION_RULES],
    len: usize,
}

impl SuppressionMap {
    pub fn new() -> Self {
        Self {
            rules: [None; MAX_SUPPRESSION_RULES],
            len: 0,
        }
    }

    /// Look up suppression factor for a stimulus in a context.
    /// Returns 1.0 (no suppression) if no matching rule exists.
    pub fn lookup(&self, kind: StimulusKind, context_hash: u32) -> f32 {
        for i in 0..self.len {
            if let Some(rule) = &self.rules[i] {
                if rule.stimulus_kind == kind && rule.context_hash == context_hash {
                    return rule.suppression_factor;
                }
            }
        }
        1.0
    }

    /// Replace or insert a rule. LRU eviction by last_updated_tick if full.
    /// Clamps suppression_factor to [MIN_SUPPRESSION_FACTOR, 1.0] (STRT-003).
    pub fn upsert(&mut self, mut rule: SuppressionRule) {
        // Enforce STRT-003: never below 0.3
        if rule.suppression_factor < MIN_SUPPRESSION_FACTOR {
            rule.suppression_factor = MIN_SUPPRESSION_FACTOR;
        }
        if rule.suppression_factor > 1.0 {
            rule.suppression_factor = 1.0;
        }

        // Check if rule for this (kind, context_hash) already exists
        for i in 0..self.len {
            if let Some(existing) = &mut self.rules[i] {
                if existing.stimulus_kind == rule.stimulus_kind
                    && existing.context_hash == rule.context_hash
                {
                    *existing = rule;
                    return;
                }
            }
        }

        // Insert into empty slot
        if self.len < MAX_SUPPRESSION_RULES {
            self.rules[self.len] = Some(rule);
            self.len += 1;
            return;
        }

        // Evict oldest by last_updated_tick
        let mut oldest_idx = 0;
        let mut oldest_tick = u64::MAX;
        for i in 0..self.len {
            if let Some(r) = &self.rules[i] {
                if r.last_updated_tick < oldest_tick {
                    oldest_tick = r.last_updated_tick;
                    oldest_idx = i;
                }
            }
        }
        self.rules[oldest_idx] = Some(rule);
    }

    /// Remove a suppression rule for a specific (kind, context_hash).
    pub fn remove(&mut self, kind: StimulusKind, context_hash: u32) {
        for i in 0..self.len {
            if let Some(rule) = &self.rules[i] {
                if rule.stimulus_kind == kind && rule.context_hash == context_hash {
                    // Move last element into this slot to keep array compact
                    if i < self.len - 1 {
                        self.rules[i] = self.rules[self.len - 1].take();
                    } else {
                        self.rules[i] = None;
                    }
                    self.len -= 1;
                    return;
                }
            }
        }
    }

    /// Number of active rules.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterate over all active rules.
    pub fn iter(&self) -> impl Iterator<Item = &SuppressionRule> {
        self.rules[..self.len].iter().filter_map(|r| r.as_ref())
    }
}

impl Default for SuppressionMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rule(kind: StimulusKind, hash: u32, factor: f32, tick: u64) -> SuppressionRule {
        SuppressionRule {
            stimulus_kind: kind,
            context_hash: hash,
            suppression_factor: factor,
            observation_count: 5,
            last_updated_tick: tick,
        }
    }

    #[test]
    fn test_lookup_unknown_returns_1() {
        let map = SuppressionMap::new();
        assert_eq!(map.lookup(StimulusKind::LoudnessSpike, 42), 1.0);
    }

    #[test]
    fn test_lookup_known_returns_factor() {
        let mut map = SuppressionMap::new();
        map.upsert(make_rule(StimulusKind::LoudnessSpike, 42, 0.4, 100));
        assert_eq!(map.lookup(StimulusKind::LoudnessSpike, 42), 0.4);
    }

    #[test]
    fn test_context_specific_isolation() {
        let mut map = SuppressionMap::new();
        map.upsert(make_rule(StimulusKind::LoudnessSpike, 42, 0.4, 100));
        // Different context → no suppression
        assert_eq!(map.lookup(StimulusKind::LoudnessSpike, 99), 1.0);
    }

    #[test]
    fn test_kind_specific_isolation() {
        let mut map = SuppressionMap::new();
        map.upsert(make_rule(StimulusKind::LoudnessSpike, 42, 0.4, 100));
        // Same context, different kind → no suppression
        assert_eq!(map.lookup(StimulusKind::BrightnessSpike, 42), 1.0);
    }

    #[test]
    fn test_upsert_updates_existing() {
        let mut map = SuppressionMap::new();
        map.upsert(make_rule(StimulusKind::LoudnessSpike, 42, 0.6, 100));
        assert_eq!(map.lookup(StimulusKind::LoudnessSpike, 42), 0.6);
        map.upsert(make_rule(StimulusKind::LoudnessSpike, 42, 0.4, 200));
        assert_eq!(map.lookup(StimulusKind::LoudnessSpike, 42), 0.4);
        assert_eq!(map.len(), 1); // didn't add a second entry
    }

    #[test]
    fn test_lru_eviction_when_full() {
        let mut map = SuppressionMap::new();
        // Fill with 32 rules, tick = i (so rule 0 is oldest)
        for i in 0..MAX_SUPPRESSION_RULES {
            map.upsert(make_rule(StimulusKind::LoudnessSpike, i as u32, 0.5, i as u64));
        }
        assert_eq!(map.len(), MAX_SUPPRESSION_RULES);

        // Insert 33rd rule — should evict the one with tick=0 (context_hash=0)
        map.upsert(make_rule(StimulusKind::BrightnessSpike, 999, 0.4, 100));
        assert_eq!(map.len(), MAX_SUPPRESSION_RULES);

        // Evicted rule should be gone
        assert_eq!(map.lookup(StimulusKind::LoudnessSpike, 0), 1.0);
        // New rule should be present
        assert_eq!(map.lookup(StimulusKind::BrightnessSpike, 999), 0.4);
    }

    #[test]
    fn test_remove() {
        let mut map = SuppressionMap::new();
        map.upsert(make_rule(StimulusKind::LoudnessSpike, 42, 0.4, 100));
        assert_eq!(map.len(), 1);
        map.remove(StimulusKind::LoudnessSpike, 42);
        assert_eq!(map.len(), 0);
        assert_eq!(map.lookup(StimulusKind::LoudnessSpike, 42), 1.0);
    }

    #[test]
    fn test_remove_nonexistent_noop() {
        let mut map = SuppressionMap::new();
        map.upsert(make_rule(StimulusKind::LoudnessSpike, 42, 0.4, 100));
        map.remove(StimulusKind::BrightnessSpike, 42); // different kind
        assert_eq!(map.len(), 1); // unchanged
    }

    #[test]
    fn test_suppression_factor_clamped_low() {
        let mut map = SuppressionMap::new();
        // Try to set factor below 0.3 (STRT-003)
        map.upsert(make_rule(StimulusKind::LoudnessSpike, 42, 0.1, 100));
        assert_eq!(map.lookup(StimulusKind::LoudnessSpike, 42), MIN_SUPPRESSION_FACTOR);
    }

    #[test]
    fn test_suppression_factor_clamped_high() {
        let mut map = SuppressionMap::new();
        map.upsert(make_rule(StimulusKind::LoudnessSpike, 42, 1.5, 100));
        assert_eq!(map.lookup(StimulusKind::LoudnessSpike, 42), 1.0);
    }
}
