//! Stimulus log — ring buffer with outcome classification.
//!
//! Invariants:
//! - I-STRT-001: no_std compatible, fixed-size arrays
//! - I-STRT-007: Observation window must be configurable
//! - I-STRT-LOG: Ring buffer never exceeds STIMULUS_LOG_SIZE entries

use super::stimulus::StimulusEvent;

/// Maximum number of log entries (ring buffer size).
pub const STIMULUS_LOG_SIZE: usize = 64;

/// Outcome of a stimulus event after the observation window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostStimulusOutcome {
    /// Not yet evaluated (within observation window)
    Pending,
    /// No collision/sustained-tension in the N ticks after stimulus
    Benign,
    /// Collision or sustained tension spike followed
    Harmful,
}

/// A logged stimulus event with its processing result and classified outcome.
#[derive(Clone, Copy, Debug)]
pub struct StimulusLogEntry {
    pub stimulus: StimulusEvent,
    pub context_hash: u32,
    pub suppression_applied: f32,
    pub tension_delta: f32,
    pub post_stimulus_outcome: PostStimulusOutcome,
}

/// Fixed-size ring buffer of stimulus log entries with outcome classification.
pub struct StimulusLog {
    entries: [Option<StimulusLogEntry>; STIMULUS_LOG_SIZE],
    write_idx: usize,
    /// Ticks to wait before classifying outcome (default: 50, ~5s at 10Hz)
    observation_window: u16,
}

impl StimulusLog {
    pub fn new() -> Self {
        Self::with_observation_window(50)
    }

    pub fn with_observation_window(window: u16) -> Self {
        Self {
            entries: [None; STIMULUS_LOG_SIZE],
            write_idx: 0,
            observation_window: window,
        }
    }

    /// Log a new stimulus entry. Overwrites oldest if buffer is full.
    pub fn log(&mut self, entry: StimulusLogEntry) {
        self.entries[self.write_idx] = Some(entry);
        self.write_idx = (self.write_idx + 1) % STIMULUS_LOG_SIZE;
    }

    /// Called each tick to evaluate pending entries.
    pub fn evaluate_pending(
        &mut self,
        current_tick: u64,
        had_collision: bool,
        tension: f32,
    ) {
        for entry in self.entries.iter_mut().flatten() {
            if entry.post_stimulus_outcome != PostStimulusOutcome::Pending {
                continue;
            }
            let ticks_since = current_tick.saturating_sub(entry.stimulus.tick);

            if had_collision || tension > 0.7 {
                entry.post_stimulus_outcome = PostStimulusOutcome::Harmful;
            } else if ticks_since >= self.observation_window as u64 {
                entry.post_stimulus_outcome = PostStimulusOutcome::Benign;
            }
        }
    }

    /// Yield entries with non-Pending outcomes for companion consumption.
    /// Classified entries are consumed (set to None).
    pub fn drain_classified(&mut self) -> DrainClassified<'_> {
        DrainClassified {
            entries: &mut self.entries,
            idx: 0,
        }
    }

    /// Count entries by outcome.
    pub fn count_by_outcome(&self, outcome: PostStimulusOutcome) -> usize {
        self.entries.iter().flatten()
            .filter(|e| e.post_stimulus_outcome == outcome)
            .count()
    }

    /// Total number of stored entries (including Pending).
    pub fn active_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_some()).count()
    }
}

impl Default for StimulusLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator that drains classified (non-Pending) entries from the log.
pub struct DrainClassified<'a> {
    entries: &'a mut [Option<StimulusLogEntry>; STIMULUS_LOG_SIZE],
    idx: usize,
}

impl<'a> Iterator for DrainClassified<'a> {
    type Item = StimulusLogEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < STIMULUS_LOG_SIZE {
            let i = self.idx;
            self.idx += 1;
            if let Some(entry) = &self.entries[i] {
                if entry.post_stimulus_outcome != PostStimulusOutcome::Pending {
                    return self.entries[i].take();
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nervous_system::stimulus::{StimulusEvent, StimulusKind};

    fn make_entry(tick: u64, hash: u32) -> StimulusLogEntry {
        StimulusLogEntry {
            stimulus: StimulusEvent {
                kind: StimulusKind::LoudnessSpike,
                magnitude: 0.5,
                tick,
            },
            context_hash: hash,
            suppression_applied: 1.0,
            tension_delta: 0.3,
            post_stimulus_outcome: PostStimulusOutcome::Pending,
        }
    }

    #[test]
    fn test_new_entry_is_pending() {
        let mut log = StimulusLog::new();
        log.log(make_entry(100, 42));
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Pending), 1);
    }

    #[test]
    fn test_benign_after_observation_window() {
        let mut log = StimulusLog::with_observation_window(50);
        log.log(make_entry(100, 42));
        // Not yet at window
        log.evaluate_pending(130, false, 0.3);
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Pending), 1);
        // At window
        log.evaluate_pending(150, false, 0.3);
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Benign), 1);
    }

    #[test]
    fn test_harmful_on_collision() {
        let mut log = StimulusLog::new();
        log.log(make_entry(100, 42));
        log.evaluate_pending(110, true, 0.3);
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Harmful), 1);
    }

    #[test]
    fn test_harmful_on_high_tension() {
        let mut log = StimulusLog::new();
        log.log(make_entry(100, 42));
        log.evaluate_pending(110, false, 0.8); // tension > 0.7
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Harmful), 1);
    }

    #[test]
    fn test_ring_buffer_wraps() {
        let mut log = StimulusLog::new();
        // Fill buffer
        for i in 0..STIMULUS_LOG_SIZE {
            log.log(make_entry(i as u64, 42));
        }
        assert_eq!(log.active_count(), STIMULUS_LOG_SIZE);

        // 65th entry overwrites oldest
        log.log(make_entry(999, 42));
        assert_eq!(log.active_count(), STIMULUS_LOG_SIZE);
    }

    #[test]
    fn test_drain_classified() {
        let mut log = StimulusLog::with_observation_window(10);
        // 3 entries at tick 100
        for _ in 0..3 {
            log.log(make_entry(100, 42));
        }
        // 2 entries at tick 200 (will remain pending)
        for _ in 0..2 {
            log.log(make_entry(200, 42));
        }
        // Classify entries at tick 100 as benign
        log.evaluate_pending(115, false, 0.3);
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Benign), 3);
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Pending), 2);

        // Drain classified
        let drained: Vec<_> = log.drain_classified().collect();
        assert_eq!(drained.len(), 3);
        assert!(drained.iter().all(|e| e.post_stimulus_outcome == PostStimulusOutcome::Benign));

        // Pending entries remain
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Pending), 2);
        assert_eq!(log.active_count(), 2);
    }

    #[test]
    fn test_already_classified_not_reclassified() {
        let mut log = StimulusLog::with_observation_window(10);
        log.log(make_entry(100, 42));
        // Classify as benign
        log.evaluate_pending(115, false, 0.3);
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Benign), 1);
        // Even with collision, already-classified entries don't change
        log.evaluate_pending(120, true, 0.9);
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Benign), 1);
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Harmful), 0);
    }

    #[test]
    fn test_custom_observation_window() {
        let mut log = StimulusLog::with_observation_window(100);
        log.log(make_entry(100, 42));
        // At 50 ticks, still pending
        log.evaluate_pending(150, false, 0.3);
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Pending), 1);
        // At 100 ticks, classified
        log.evaluate_pending(200, false, 0.3);
        assert_eq!(log.count_by_outcome(PostStimulusOutcome::Benign), 1);
    }
}
