//! Suppression Sync -- companion-to-core rule push.
//!
//! Coordinates three things:
//! 1. Drain classified entries from `StimulusLog` and feed to `SuppressionLearner`
//! 2. Apply learning results to `StartleProcessor`'s `SuppressionMap`
//! 3. Persist the `SuppressionMap` to SQLite and restore on startup
//!
//! # Contract Compliance
//! - **ARCH-001**: All learning logic lives in mbot-companion, not mbot-core
//! - **ARCH-002**: Brain is advisory; companion suggests rules, core enforces them
//! - **I-STRT-003**: SuppressionRule factors are clamped to [0.3, 1.0] by core

use mbot_core::nervous_system::stimulus::StimulusKind;
use mbot_core::nervous_system::suppression::{SuppressionMap, SuppressionRule};
use mbot_core::nervous_system::startle::StartleProcessor;
use mbot_core::nervous_system::stimulus_log::StimulusLog;
use super::suppression_learner::{SuppressionLearner, SuppressionLearnerConfig};

/// Coordinates draining the stimulus log, running the learner, and pushing
/// rule mutations into the core `StartleProcessor`.
pub struct SuppressionSync {
    learner: SuppressionLearner,
}

impl SuppressionSync {
    pub fn new() -> Self {
        Self {
            learner: SuppressionLearner::new(SuppressionLearnerConfig::default()),
        }
    }

    pub fn with_config(config: SuppressionLearnerConfig) -> Self {
        Self {
            learner: SuppressionLearner::new(config),
        }
    }

    /// Run one sync cycle: drain log -> learn -> apply to processor.
    /// Call this from the main loop periodically.
    pub fn sync(
        &mut self,
        stimulus_log: &mut StimulusLog,
        startle_processor: &mut StartleProcessor,
        current_tick: u64,
        curiosity_drive: f32,
        startle_sensitivity: f32,
    ) -> SyncResult {
        // 1. Drain classified entries from log
        let classified: Vec<_> = stimulus_log.drain_classified().collect();
        let ingested = classified.len();
        self.learner.ingest(&classified);

        // 2. Check if it's time to learn
        if !self.learner.should_learn(current_tick) {
            return SyncResult { ingested, rules_upserted: 0, rules_removed: 0 };
        }

        // 3. Run learning algorithm
        let result = self.learner.learn(current_tick, curiosity_drive, startle_sensitivity);

        // 4. Apply to processor's suppression map
        let rules_upserted = result.rules_to_upsert.len();
        let rules_removed = result.rules_to_remove.len();

        for rule in result.rules_to_upsert {
            startle_processor.suppression_map.upsert(rule);
        }
        for (kind, hash) in result.rules_to_remove {
            startle_processor.suppression_map.remove(kind, hash);
        }

        SyncResult { ingested, rules_upserted, rules_removed }
    }

    /// Save the current suppression map to SQLite.
    pub fn save_to_db(
        &self,
        conn: &rusqlite::Connection,
        map: &SuppressionMap,
    ) -> Result<(), String> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS suppression_rules (
                stimulus_kind TEXT NOT NULL,
                context_hash INTEGER NOT NULL,
                suppression_factor REAL NOT NULL,
                observation_count INTEGER NOT NULL,
                last_updated_tick INTEGER NOT NULL,
                PRIMARY KEY (stimulus_kind, context_hash)
            )",
        )
        .map_err(|e| format!("create table: {}", e))?;

        // Clear and reinsert (simple approach for a small table of at most 32 rows)
        conn.execute("DELETE FROM suppression_rules", [])
            .map_err(|e| format!("clear: {}", e))?;

        for rule in map.iter() {
            let kind_str = kind_to_str(rule.stimulus_kind);
            conn.execute(
                "INSERT INTO suppression_rules \
                 (stimulus_kind, context_hash, suppression_factor, observation_count, last_updated_tick) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    kind_str,
                    rule.context_hash,
                    rule.suppression_factor,
                    rule.observation_count,
                    rule.last_updated_tick,
                ],
            )
            .map_err(|e| format!("insert: {}", e))?;
        }

        Ok(())
    }

    /// Restore suppression map from SQLite.
    pub fn load_from_db(
        conn: &rusqlite::Connection,
        map: &mut SuppressionMap,
    ) -> Result<usize, String> {
        // Check if table exists
        let table_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='suppression_rules'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);

        if !table_exists {
            return Ok(0);
        }

        let mut stmt = conn
            .prepare(
                "SELECT stimulus_kind, context_hash, suppression_factor, \
                 observation_count, last_updated_tick FROM suppression_rules",
            )
            .map_err(|e| format!("prepare: {}", e))?;

        let mut count = 0;
        let rows = stmt
            .query_map([], |row| {
                let kind_str: String = row.get(0)?;
                let context_hash: u32 = row.get(1)?;
                let suppression_factor: f64 = row.get(2)?;
                let observation_count: u16 = row.get(3)?;
                let last_updated_tick: u64 = row.get(4)?;
                Ok((
                    kind_str,
                    context_hash,
                    suppression_factor as f32,
                    observation_count,
                    last_updated_tick,
                ))
            })
            .map_err(|e| format!("query: {}", e))?;

        for row in rows {
            let (kind_str, context_hash, suppression_factor, observation_count, last_updated_tick) =
                row.map_err(|e| format!("row: {}", e))?;

            let kind = match str_to_kind(&kind_str) {
                Some(k) => k,
                None => continue,
            };

            map.upsert(SuppressionRule {
                stimulus_kind: kind,
                context_hash,
                suppression_factor,
                observation_count,
                last_updated_tick,
            });
            count += 1;
        }

        Ok(count)
    }
}

impl Default for SuppressionSync {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a single sync cycle.
pub struct SyncResult {
    /// Number of classified entries drained from the stimulus log.
    pub ingested: usize,
    /// Number of suppression rules upserted into the map.
    pub rules_upserted: usize,
    /// Number of suppression rules removed from the map.
    pub rules_removed: usize,
}

// -- Serialisation helpers ------------------------------------------------

fn kind_to_str(kind: StimulusKind) -> &'static str {
    match kind {
        StimulusKind::LoudnessSpike => "LoudnessSpike",
        StimulusKind::BrightnessSpike => "BrightnessSpike",
        StimulusKind::ProximityRush => "ProximityRush",
        StimulusKind::ImpactShock => "ImpactShock",
        StimulusKind::OrientationFlip => "OrientationFlip",
    }
}

fn str_to_kind(s: &str) -> Option<StimulusKind> {
    match s {
        "LoudnessSpike" => Some(StimulusKind::LoudnessSpike),
        "BrightnessSpike" => Some(StimulusKind::BrightnessSpike),
        "ProximityRush" => Some(StimulusKind::ProximityRush),
        "ImpactShock" => Some(StimulusKind::ImpactShock),
        "OrientationFlip" => Some(StimulusKind::OrientationFlip),
        _ => None,
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use mbot_core::nervous_system::stimulus::StimulusEvent;
    use mbot_core::nervous_system::stimulus_log::{PostStimulusOutcome, StimulusLogEntry};

    /// Helper: create a classified log entry.
    fn make_entry(
        kind: StimulusKind,
        context_hash: u32,
        outcome: PostStimulusOutcome,
        tick: u64,
    ) -> StimulusLogEntry {
        StimulusLogEntry {
            stimulus: StimulusEvent {
                kind,
                magnitude: 0.5,
                tick,
            },
            context_hash,
            suppression_applied: 1.0,
            tension_delta: 0.2,
            post_stimulus_outcome: outcome,
        }
    }

    // ------------------------------------------------------------------
    // Test 1: Full sync cycle (drain -> learn -> apply)
    // ------------------------------------------------------------------
    #[test]
    fn test_full_sync_cycle() {
        let config = SuppressionLearnerConfig {
            min_observations: 3,
            benign_threshold: 0.8,
            harmful_threshold: 0.4,
            max_suppression: 0.7,
            relearn_interval_ticks: 0, // learn every cycle for testing
        };
        let mut sync = SuppressionSync::with_config(config);
        let mut log = StimulusLog::with_observation_window(5);
        let mut processor = StartleProcessor::new();

        // Insert 6 benign entries for (LoudnessSpike, 42)
        for i in 0..6 {
            log.log(make_entry(
                StimulusKind::LoudnessSpike,
                42,
                PostStimulusOutcome::Pending,
                i,
            ));
        }

        // Classify all as benign
        log.evaluate_pending(100, false, 0.2);

        // Run sync with curiosity=1.0 => effective_min_obs = ceil(3/1.0) = 3
        // 6 benign / 6 total = 100% benign ratio > 0.8 threshold => upsert
        let result = sync.sync(
            &mut log,
            &mut processor,
            500, // current_tick (>= relearn_interval_ticks=0)
            1.0, // curiosity_drive
            0.5, // startle_sensitivity
        );

        assert_eq!(result.ingested, 6, "Should drain 6 classified entries");
        assert_eq!(result.rules_upserted, 1, "Should upsert 1 rule");
        assert_eq!(result.rules_removed, 0, "Should remove 0 rules");

        // Verify the rule is now in the processor's suppression map
        let factor = processor.suppression_map.lookup(StimulusKind::LoudnessSpike, 42);
        assert!(
            factor < 1.0,
            "Suppression factor ({}) should be less than 1.0 (suppressed)",
            factor
        );
        assert!(
            factor >= 0.3,
            "Suppression factor ({}) must be >= 0.3 (I-STRT-003)",
            factor
        );
    }

    // ------------------------------------------------------------------
    // Test 2: Save and load round-trip via SQLite
    // ------------------------------------------------------------------
    #[test]
    fn test_save_and_load_sqlite() {
        let conn = rusqlite::Connection::open_in_memory()
            .expect("open in-memory db");

        // Create a map with two rules
        let mut original_map = SuppressionMap::new();
        original_map.upsert(SuppressionRule {
            stimulus_kind: StimulusKind::LoudnessSpike,
            context_hash: 42,
            suppression_factor: 0.5,
            observation_count: 10,
            last_updated_tick: 1000,
        });
        original_map.upsert(SuppressionRule {
            stimulus_kind: StimulusKind::ProximityRush,
            context_hash: 7,
            suppression_factor: 0.3,
            observation_count: 20,
            last_updated_tick: 2000,
        });

        // Save
        let sync = SuppressionSync::new();
        sync.save_to_db(&conn, &original_map)
            .expect("save should succeed");

        // Load into a fresh map
        let mut loaded_map = SuppressionMap::new();
        let count = SuppressionSync::load_from_db(&conn, &mut loaded_map)
            .expect("load should succeed");

        assert_eq!(count, 2, "Should load 2 rules");
        assert_eq!(loaded_map.len(), 2, "Map should have 2 rules");

        // Verify values round-tripped correctly
        assert!(
            (loaded_map.lookup(StimulusKind::LoudnessSpike, 42) - 0.5).abs() < 0.001,
            "LoudnessSpike/42 factor should be 0.5"
        );
        assert!(
            (loaded_map.lookup(StimulusKind::ProximityRush, 7) - 0.3).abs() < 0.001,
            "ProximityRush/7 factor should be 0.3"
        );

        // Other contexts still return 1.0 (no suppression)
        assert_eq!(
            loaded_map.lookup(StimulusKind::LoudnessSpike, 999),
            1.0,
            "Unknown context should return 1.0"
        );
    }

    // ------------------------------------------------------------------
    // Test 3: Empty log produces no changes
    // ------------------------------------------------------------------
    #[test]
    fn test_empty_log_no_changes() {
        let config = SuppressionLearnerConfig {
            relearn_interval_ticks: 0,
            ..Default::default()
        };
        let mut sync = SuppressionSync::with_config(config);
        let mut log = StimulusLog::new();
        let mut processor = StartleProcessor::new();

        let result = sync.sync(&mut log, &mut processor, 1000, 0.5, 0.5);

        assert_eq!(result.ingested, 0, "Empty log should drain 0 entries");
        assert_eq!(result.rules_upserted, 0, "No data => no rules upserted");
        assert_eq!(result.rules_removed, 0, "No data => no rules removed");
        assert!(processor.suppression_map.is_empty(), "Map should remain empty");
    }

    // ------------------------------------------------------------------
    // Test 4: load_from_db on empty database returns 0
    // ------------------------------------------------------------------
    #[test]
    fn test_load_from_empty_db() {
        let conn = rusqlite::Connection::open_in_memory()
            .expect("open in-memory db");
        let mut map = SuppressionMap::new();

        let count = SuppressionSync::load_from_db(&conn, &mut map)
            .expect("load should succeed even with no table");

        assert_eq!(count, 0, "No table => 0 rules loaded");
        assert!(map.is_empty(), "Map should remain empty");
    }

    // ------------------------------------------------------------------
    // Test 5: save_to_db overwrites previous data
    // ------------------------------------------------------------------
    #[test]
    fn test_save_overwrites_previous() {
        let conn = rusqlite::Connection::open_in_memory()
            .expect("open in-memory db");
        let sync = SuppressionSync::new();

        // First save: 2 rules
        let mut map1 = SuppressionMap::new();
        map1.upsert(SuppressionRule {
            stimulus_kind: StimulusKind::LoudnessSpike,
            context_hash: 1,
            suppression_factor: 0.5,
            observation_count: 5,
            last_updated_tick: 100,
        });
        map1.upsert(SuppressionRule {
            stimulus_kind: StimulusKind::BrightnessSpike,
            context_hash: 2,
            suppression_factor: 0.6,
            observation_count: 8,
            last_updated_tick: 200,
        });
        sync.save_to_db(&conn, &map1).expect("first save");

        // Second save: only 1 rule (different from before)
        let mut map2 = SuppressionMap::new();
        map2.upsert(SuppressionRule {
            stimulus_kind: StimulusKind::ImpactShock,
            context_hash: 99,
            suppression_factor: 0.4,
            observation_count: 12,
            last_updated_tick: 500,
        });
        sync.save_to_db(&conn, &map2).expect("second save");

        // Load should only contain the second save's data
        let mut loaded = SuppressionMap::new();
        let count = SuppressionSync::load_from_db(&conn, &mut loaded)
            .expect("load");

        assert_eq!(count, 1, "Should have 1 rule after overwrite");
        assert_eq!(
            loaded.lookup(StimulusKind::LoudnessSpike, 1),
            1.0,
            "Old rule should be gone"
        );
        assert!(
            (loaded.lookup(StimulusKind::ImpactShock, 99) - 0.4).abs() < 0.001,
            "New rule should be present"
        );
    }

    // ------------------------------------------------------------------
    // Test 6: Unknown kind in DB is gracefully skipped
    // ------------------------------------------------------------------
    #[test]
    fn test_unknown_kind_skipped() {
        let conn = rusqlite::Connection::open_in_memory()
            .expect("open in-memory db");

        // Manually create table and insert a row with unknown kind
        conn.execute_batch(
            "CREATE TABLE suppression_rules (
                stimulus_kind TEXT NOT NULL,
                context_hash INTEGER NOT NULL,
                suppression_factor REAL NOT NULL,
                observation_count INTEGER NOT NULL,
                last_updated_tick INTEGER NOT NULL,
                PRIMARY KEY (stimulus_kind, context_hash)
            )"
        ).expect("create table");

        conn.execute(
            "INSERT INTO suppression_rules VALUES ('UnknownThing', 1, 0.5, 5, 100)",
            [],
        ).expect("insert unknown kind");

        conn.execute(
            "INSERT INTO suppression_rules VALUES ('LoudnessSpike', 2, 0.4, 10, 200)",
            [],
        ).expect("insert valid kind");

        let mut map = SuppressionMap::new();
        let count = SuppressionSync::load_from_db(&conn, &mut map)
            .expect("load");

        assert_eq!(count, 1, "Only valid kind should be loaded");
        assert_eq!(map.len(), 1);
        assert!(
            (map.lookup(StimulusKind::LoudnessSpike, 2) - 0.4).abs() < 0.001,
            "Valid rule should load correctly"
        );
    }

    // ------------------------------------------------------------------
    // Test 7: Sync with pending-only entries does not trigger learning
    // ------------------------------------------------------------------
    #[test]
    fn test_pending_only_no_drain() {
        let config = SuppressionLearnerConfig {
            relearn_interval_ticks: 0,
            ..Default::default()
        };
        let mut sync = SuppressionSync::with_config(config);
        let mut log = StimulusLog::with_observation_window(1000); // very long window
        let mut processor = StartleProcessor::new();

        // Insert entries that will stay Pending (observation window not elapsed)
        for i in 0..10 {
            log.log(make_entry(
                StimulusKind::LoudnessSpike,
                42,
                PostStimulusOutcome::Pending,
                i,
            ));
        }

        // Do NOT call evaluate_pending, so entries remain Pending.
        // drain_classified only yields non-Pending entries.
        let result = sync.sync(&mut log, &mut processor, 1000, 0.5, 0.5);

        assert_eq!(result.ingested, 0, "Pending entries should not be drained");
        assert_eq!(result.rules_upserted, 0);
        assert_eq!(result.rules_removed, 0);
    }
}
