//! Coherence accumulator persistence -- save/load across restarts.
//!
//! Stores CoherenceField accumulators to SQLite so the robot remembers
//! which contexts are familiar after a restart.
//!
//! # Contract Compliance
//! - **ARCH-001**: All persistence logic lives in mbot-companion, not mbot-core
//! - **CCF-002**: All loaded accumulator values are clamped to [0.0, 1.0]

use mbot_core::coherence::{
    BrightnessBand, CoherenceField, ContextKey, MotionContext, NoiseBand,
    Orientation, PresenceSignature, TimePeriod,
};

/// Create the `coherence_accumulators` table if it does not exist.
pub fn create_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS coherence_accumulators (
            context_hash_u32 INTEGER PRIMARY KEY,
            brightness TEXT NOT NULL,
            noise TEXT NOT NULL,
            presence TEXT NOT NULL,
            motion TEXT NOT NULL,
            orientation TEXT NOT NULL,
            time_period TEXT NOT NULL,
            value REAL NOT NULL,
            interaction_count INTEGER NOT NULL,
            last_interaction_tick INTEGER NOT NULL
        )",
    )
    .map_err(|e| format!("create coherence_accumulators table: {}", e))
}

/// Save all accumulators from a `CoherenceField` to SQLite.
///
/// Uses DELETE + INSERT (same pattern as suppression_sync) since the
/// accumulator table is small (max 64 rows).
///
/// Returns the number of rows written.
pub fn save_to_db(
    conn: &rusqlite::Connection,
    field: &CoherenceField,
) -> Result<usize, String> {
    create_table(conn)?;

    conn.execute("DELETE FROM coherence_accumulators", [])
        .map_err(|e| format!("clear coherence_accumulators: {}", e))?;

    let mut count = 0;
    for (key, acc) in field.iter() {
        let hash = key.context_hash_u32();
        conn.execute(
            "INSERT INTO coherence_accumulators \
             (context_hash_u32, brightness, noise, presence, motion, \
              orientation, time_period, value, interaction_count, last_interaction_tick) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                hash,
                brightness_to_str(key.brightness),
                noise_to_str(key.noise),
                presence_to_str(key.presence),
                motion_to_str(key.motion),
                orientation_to_str(key.orientation),
                time_period_to_str(key.time_period),
                acc.value as f64,
                acc.interaction_count,
                acc.last_interaction_tick,
            ],
        )
        .map_err(|e| format!("insert accumulator (hash={}): {}", hash, e))?;
        count += 1;
    }

    Ok(count)
}

/// Load accumulators from SQLite into a `CoherenceField`.
///
/// Reconstructs `ContextKey` from the stored text columns and overwrites
/// the accumulator's value, interaction_count, and last_interaction_tick.
/// Rows with unrecognised enum variants are silently skipped.
///
/// Returns the number of accumulators loaded.
pub fn load_from_db(
    conn: &rusqlite::Connection,
    field: &mut CoherenceField,
) -> Result<usize, String> {
    // Check if table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master \
             WHERE type='table' AND name='coherence_accumulators'",
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
            "SELECT brightness, noise, presence, motion, orientation, \
             time_period, value, interaction_count, last_interaction_tick \
             FROM coherence_accumulators",
        )
        .map_err(|e| format!("prepare coherence_accumulators query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            let brightness: String = row.get(0)?;
            let noise: String = row.get(1)?;
            let presence: String = row.get(2)?;
            let motion: String = row.get(3)?;
            let orientation: String = row.get(4)?;
            let time_period: String = row.get(5)?;
            let value: f64 = row.get(6)?;
            let interaction_count: u32 = row.get(7)?;
            let last_interaction_tick: u64 = row.get(8)?;
            Ok((
                brightness,
                noise,
                presence,
                motion,
                orientation,
                time_period,
                value,
                interaction_count,
                last_interaction_tick,
            ))
        })
        .map_err(|e| format!("query coherence_accumulators: {}", e))?;

    let mut count = 0;
    for row in rows {
        let (
            brightness_str,
            noise_str,
            presence_str,
            motion_str,
            orientation_str,
            time_period_str,
            value,
            interaction_count,
            last_interaction_tick,
        ) = row.map_err(|e| format!("row read: {}", e))?;

        // Reconstruct ContextKey from text. Skip if any variant is unknown.
        let brightness = match str_to_brightness(&brightness_str) {
            Some(v) => v,
            None => continue,
        };
        let noise = match str_to_noise(&noise_str) {
            Some(v) => v,
            None => continue,
        };
        let presence = match str_to_presence(&presence_str) {
            Some(v) => v,
            None => continue,
        };
        let motion = match str_to_motion(&motion_str) {
            Some(v) => v,
            None => continue,
        };
        let orientation = match str_to_orientation(&orientation_str) {
            Some(v) => v,
            None => continue,
        };
        let time_period = match str_to_time_period(&time_period_str) {
            Some(v) => v,
            None => continue,
        };

        let key = ContextKey {
            brightness,
            noise,
            presence,
            motion,
            orientation,
            time_period,
        };

        // get_or_create inserts with personality baseline; we then overwrite
        // with the persisted values.
        let acc = field.get_or_create(&key);
        acc.value = (value as f32).clamp(0.0, 1.0); // CCF-002
        acc.interaction_count = interaction_count;
        acc.last_interaction_tick = last_interaction_tick;

        count += 1;
    }

    Ok(count)
}

// -- Serialisation helpers (enum <-> text) --------------------------------

fn brightness_to_str(b: BrightnessBand) -> &'static str {
    match b {
        BrightnessBand::Dark => "Dark",
        BrightnessBand::Dim => "Dim",
        BrightnessBand::Bright => "Bright",
    }
}

fn str_to_brightness(s: &str) -> Option<BrightnessBand> {
    match s {
        "Dark" => Some(BrightnessBand::Dark),
        "Dim" => Some(BrightnessBand::Dim),
        "Bright" => Some(BrightnessBand::Bright),
        _ => None,
    }
}

fn noise_to_str(n: NoiseBand) -> &'static str {
    match n {
        NoiseBand::Quiet => "Quiet",
        NoiseBand::Moderate => "Moderate",
        NoiseBand::Loud => "Loud",
    }
}

fn str_to_noise(s: &str) -> Option<NoiseBand> {
    match s {
        "Quiet" => Some(NoiseBand::Quiet),
        "Moderate" => Some(NoiseBand::Moderate),
        "Loud" => Some(NoiseBand::Loud),
        _ => None,
    }
}

fn presence_to_str(p: PresenceSignature) -> &'static str {
    match p {
        PresenceSignature::Absent => "Absent",
        PresenceSignature::Static => "Static",
        PresenceSignature::Approaching => "Approaching",
        PresenceSignature::Retreating => "Retreating",
    }
}

fn str_to_presence(s: &str) -> Option<PresenceSignature> {
    match s {
        "Absent" => Some(PresenceSignature::Absent),
        "Static" => Some(PresenceSignature::Static),
        "Approaching" => Some(PresenceSignature::Approaching),
        "Retreating" => Some(PresenceSignature::Retreating),
        _ => None,
    }
}

fn motion_to_str(m: MotionContext) -> &'static str {
    match m {
        MotionContext::Stationary => "Stationary",
        MotionContext::SelfMoving => "SelfMoving",
        MotionContext::BeingHandled => "BeingHandled",
    }
}

fn str_to_motion(s: &str) -> Option<MotionContext> {
    match s {
        "Stationary" => Some(MotionContext::Stationary),
        "SelfMoving" => Some(MotionContext::SelfMoving),
        "BeingHandled" => Some(MotionContext::BeingHandled),
        _ => None,
    }
}

fn orientation_to_str(o: Orientation) -> &'static str {
    match o {
        Orientation::Upright => "Upright",
        Orientation::Tilted => "Tilted",
    }
}

fn str_to_orientation(s: &str) -> Option<Orientation> {
    match s {
        "Upright" => Some(Orientation::Upright),
        "Tilted" => Some(Orientation::Tilted),
        _ => None,
    }
}

fn time_period_to_str(t: TimePeriod) -> &'static str {
    match t {
        TimePeriod::Morning => "Morning",
        TimePeriod::Afternoon => "Afternoon",
        TimePeriod::Evening => "Evening",
        TimePeriod::Night => "Night",
    }
}

fn str_to_time_period(s: &str) -> Option<TimePeriod> {
    match s {
        "Morning" => Some(TimePeriod::Morning),
        "Afternoon" => Some(TimePeriod::Afternoon),
        "Evening" => Some(TimePeriod::Evening),
        "Night" => Some(TimePeriod::Night),
        _ => None,
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use mbot_core::coherence::CoherenceField;

    /// Helper: build a ContextKey with specific enum values.
    fn make_key(
        brightness: BrightnessBand,
        noise: NoiseBand,
        presence: PresenceSignature,
    ) -> ContextKey {
        ContextKey {
            brightness,
            noise,
            presence,
            motion: MotionContext::Stationary,
            orientation: Orientation::Upright,
            time_period: TimePeriod::Afternoon,
        }
    }

    // ------------------------------------------------------------------
    // Test 1: Round-trip -- save accumulators, load them back, values match
    // ------------------------------------------------------------------
    #[test]
    fn test_round_trip() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");

        let mut field = CoherenceField::new();

        // Populate two contexts with distinct values
        let key1 = make_key(BrightnessBand::Bright, NoiseBand::Quiet, PresenceSignature::Static);
        {
            let acc = field.get_or_create(&key1);
            acc.value = 0.75;
            acc.interaction_count = 42;
            acc.last_interaction_tick = 1000;
        }

        let key2 = make_key(BrightnessBand::Dark, NoiseBand::Loud, PresenceSignature::Approaching);
        {
            let acc = field.get_or_create(&key2);
            acc.value = 0.30;
            acc.interaction_count = 10;
            acc.last_interaction_tick = 500;
        }

        // Save
        let saved = save_to_db(&conn, &field).expect("save should succeed");
        assert_eq!(saved, 2, "Should save 2 accumulators");

        // Load into a fresh field
        let mut loaded_field = CoherenceField::new();
        let loaded = load_from_db(&conn, &mut loaded_field).expect("load should succeed");
        assert_eq!(loaded, 2, "Should load 2 accumulators");
        assert_eq!(loaded_field.context_count(), 2);

        // Verify key1 values
        let ctx1 = loaded_field.context_coherence(&key1);
        assert!(
            (ctx1 - 0.75).abs() < 0.001,
            "key1 value should be 0.75, got {}",
            ctx1
        );

        // Verify key2 values
        let ctx2 = loaded_field.context_coherence(&key2);
        assert!(
            (ctx2 - 0.30).abs() < 0.001,
            "key2 value should be 0.30, got {}",
            ctx2
        );

        // Verify interaction counts survived by checking the accumulator directly
        // (use iter to find the right entry)
        for (key, acc) in loaded_field.iter() {
            if *key == key1 {
                assert_eq!(acc.interaction_count, 42);
                assert_eq!(acc.last_interaction_tick, 1000);
            } else if *key == key2 {
                assert_eq!(acc.interaction_count, 10);
                assert_eq!(acc.last_interaction_tick, 500);
            } else {
                panic!("unexpected key in loaded field");
            }
        }
    }

    // ------------------------------------------------------------------
    // Test 2: Empty database -- load returns 0
    // ------------------------------------------------------------------
    #[test]
    fn test_empty_database() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");
        let mut field = CoherenceField::new();

        let loaded = load_from_db(&conn, &mut field).expect("load should succeed");
        assert_eq!(loaded, 0, "Empty DB should load 0 accumulators");
        assert_eq!(field.context_count(), 0);
    }

    // ------------------------------------------------------------------
    // Test 3: Overwrite -- save, modify, save again, load reflects latest
    // ------------------------------------------------------------------
    #[test]
    fn test_overwrite() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");

        // First save: 2 accumulators
        let mut field1 = CoherenceField::new();
        let key_a = make_key(BrightnessBand::Dim, NoiseBand::Moderate, PresenceSignature::Retreating);
        {
            let acc = field1.get_or_create(&key_a);
            acc.value = 0.50;
            acc.interaction_count = 20;
            acc.last_interaction_tick = 100;
        }
        let key_b = make_key(BrightnessBand::Bright, NoiseBand::Quiet, PresenceSignature::Absent);
        {
            let acc = field1.get_or_create(&key_b);
            acc.value = 0.10;
            acc.interaction_count = 3;
            acc.last_interaction_tick = 50;
        }
        save_to_db(&conn, &field1).expect("first save");

        // Second save: only 1 accumulator with updated values
        let mut field2 = CoherenceField::new();
        {
            let acc = field2.get_or_create(&key_a);
            acc.value = 0.90;
            acc.interaction_count = 55;
            acc.last_interaction_tick = 999;
        }
        save_to_db(&conn, &field2).expect("second save");

        // Load should only reflect the second save
        let mut loaded = CoherenceField::new();
        let count = load_from_db(&conn, &mut loaded).expect("load");
        assert_eq!(count, 1, "Should load 1 accumulator after overwrite");

        let ctx_a = loaded.context_coherence(&key_a);
        assert!(
            (ctx_a - 0.90).abs() < 0.001,
            "key_a value should be 0.90, got {}",
            ctx_a
        );

        // key_b should not be present (uses fallback or 0.0)
        let ctx_b = loaded.context_coherence(&key_b);
        assert!(
            ctx_b < 0.01,
            "key_b should not be present, got {}",
            ctx_b
        );
    }

    // ------------------------------------------------------------------
    // Test 4: Unknown enum variant in DB is gracefully skipped
    // ------------------------------------------------------------------
    #[test]
    fn test_unknown_variant_skipped() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");

        create_table(&conn).expect("create table");

        // Insert a row with unknown brightness variant
        conn.execute(
            "INSERT INTO coherence_accumulators \
             (context_hash_u32, brightness, noise, presence, motion, \
              orientation, time_period, value, interaction_count, last_interaction_tick) \
             VALUES (1, 'UltraBright', 'Quiet', 'Static', 'Stationary', 'Upright', 'Morning', 0.5, 5, 100)",
            [],
        )
        .expect("insert unknown variant");

        // Insert a valid row
        conn.execute(
            "INSERT INTO coherence_accumulators \
             (context_hash_u32, brightness, noise, presence, motion, \
              orientation, time_period, value, interaction_count, last_interaction_tick) \
             VALUES (2, 'Dark', 'Quiet', 'Static', 'Stationary', 'Upright', 'Morning', 0.6, 8, 200)",
            [],
        )
        .expect("insert valid row");

        let mut field = CoherenceField::new();
        let count = load_from_db(&conn, &mut field).expect("load");

        assert_eq!(count, 1, "Only valid row should be loaded");
        assert_eq!(field.context_count(), 1);
    }

    // ------------------------------------------------------------------
    // Test 5: Values are clamped to [0.0, 1.0] on load (CCF-002)
    // ------------------------------------------------------------------
    #[test]
    fn test_value_clamped_on_load() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");

        create_table(&conn).expect("create table");

        // Insert a row with out-of-range value
        conn.execute(
            "INSERT INTO coherence_accumulators \
             (context_hash_u32, brightness, noise, presence, motion, \
              orientation, time_period, value, interaction_count, last_interaction_tick) \
             VALUES (1, 'Bright', 'Loud', 'Absent', 'Stationary', 'Upright', 'Night', 1.5, 99, 999)",
            [],
        )
        .expect("insert out-of-range value");

        let mut field = CoherenceField::new();
        let count = load_from_db(&conn, &mut field).expect("load");
        assert_eq!(count, 1);

        let key = ContextKey {
            brightness: BrightnessBand::Bright,
            noise: NoiseBand::Loud,
            presence: PresenceSignature::Absent,
            motion: MotionContext::Stationary,
            orientation: Orientation::Upright,
            time_period: TimePeriod::Night,
        };

        let val = field.context_coherence(&key);
        assert!(
            val <= 1.0,
            "value should be clamped to 1.0, got {}",
            val
        );
    }

    // ------------------------------------------------------------------
    // Test 6: create_table is idempotent
    // ------------------------------------------------------------------
    #[test]
    fn test_create_table_idempotent() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");
        create_table(&conn).expect("first create");
        create_table(&conn).expect("second create should not fail");
    }
}
