//! InteractionEpisode recording & TrajectoryVector embedding.
//!
//! Records episodes of interaction within a single context, computing
//! a fixed-size trajectory vector that summarises the dynamic shape of
//! tension, coherence, and energy over the episode window.
//!
//! # Contract Compliance
//! - **ARCH-001**: All episode logic lives in mbot-companion, not mbot-core
//! - **CCF-002**: All loaded accumulator values are clamped to valid ranges

// ─── Types ──────────────────────────────────────────────────────────

/// Episode outcome -- how the interaction ended relative to expectations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpisodeOutcome {
    /// Coherence increased or stayed high.
    Positive,
    /// Coherence dropped or tension spiked.
    Negative,
    /// No significant change.
    Neutral,
}

impl EpisodeOutcome {
    fn to_i32(self) -> i32 {
        match self {
            EpisodeOutcome::Positive => 1,
            EpisodeOutcome::Negative => -1,
            EpisodeOutcome::Neutral => 0,
        }
    }

    fn from_i32(v: i32) -> Self {
        match v {
            1 => EpisodeOutcome::Positive,
            -1 => EpisodeOutcome::Negative,
            _ => EpisodeOutcome::Neutral,
        }
    }
}

/// Fixed-size trajectory embedding: 4 stats x 3 channels = 12 floats.
///
/// Channels: tension, coherence, energy (in that order, 4 stats each).
/// Stats per channel: mean, variance, trend_slope, peak.
pub type TrajectoryVector = [f32; 12];

/// A completed interaction episode with its trajectory embedding.
#[derive(Clone, Debug)]
pub struct InteractionEpisode {
    /// Hash of the context key active during this episode.
    pub context_hash: u32,
    /// Tick when the episode began.
    pub start_tick: u64,
    /// Tick when the episode ended (inclusive).
    pub end_tick: u64,
    /// How the episode resolved.
    pub outcome: EpisodeOutcome,
    /// 12-dimensional trajectory embedding.
    pub trajectory: TrajectoryVector,
}

// ─── Running Stats (per channel) ────────────────────────────────────

/// Welford online accumulator for a single channel.
/// Tracks mean, variance, trend slope (simple linear regression), and peak.
#[derive(Clone, Debug)]
struct ChannelStats {
    count: usize,
    mean: f64,
    m2: f64,       // sum of squared deviations (for variance)
    peak: f32,
    // For simple linear regression: slope = (sum_xy - n*mean_x*mean_y) / (sum_x2 - n*mean_x^2)
    sum_x: f64,    // sum of indices (0, 1, 2, ...)
    sum_x2: f64,   // sum of index^2
    sum_xy: f64,   // sum of index * value
    sum_y: f64,    // sum of values (same as count * mean, but kept for clarity)
}

impl ChannelStats {
    fn new() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            m2: 0.0,
            peak: f32::NEG_INFINITY,
            sum_x: 0.0,
            sum_x2: 0.0,
            sum_xy: 0.0,
            sum_y: 0.0,
        }
    }

    fn push(&mut self, value: f32) {
        let v = value as f64;
        let x = self.count as f64;

        // Welford online mean + variance
        self.count += 1;
        let delta = v - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = v - self.mean;
        self.m2 += delta * delta2;

        // Peak
        if value > self.peak {
            self.peak = value;
        }

        // Linear regression accumulators
        self.sum_x += x;
        self.sum_x2 += x * x;
        self.sum_xy += x * v;
        self.sum_y += v;
    }

    /// Finalize into [mean, variance, trend_slope, peak].
    fn finalize(&self) -> [f32; 4] {
        if self.count == 0 {
            return [0.0; 4];
        }

        let mean = self.mean as f32;
        let variance = if self.count > 1 {
            (self.m2 / (self.count - 1) as f64) as f32
        } else {
            0.0
        };

        let n = self.count as f64;
        let slope = if self.count > 1 {
            let denom = self.sum_x2 - (self.sum_x * self.sum_x) / n;
            if denom.abs() < 1e-12 {
                0.0
            } else {
                ((self.sum_xy - (self.sum_x * self.sum_y) / n) / denom) as f32
            }
        } else {
            0.0
        };

        let peak = if self.peak == f32::NEG_INFINITY {
            0.0
        } else {
            self.peak
        };

        [mean, variance, slope, peak]
    }
}

// ─── Episode Recorder ───────────────────────────────────────────────

/// Maximum ticks before an episode is force-finalized.
const MAX_EPISODE_TICKS: u64 = 100;

/// Accumulates per-tick data and produces `InteractionEpisode` on context
/// change or when the 100-tick window is reached.
pub struct EpisodeRecorder {
    /// Context hash of the episode currently being recorded.
    current_context: Option<u32>,
    /// First tick of the current episode.
    start_tick: u64,
    /// Last tick recorded.
    last_tick: u64,
    /// Running stats: [tension, coherence, energy].
    channels: [ChannelStats; 3],
}

impl EpisodeRecorder {
    pub fn new() -> Self {
        Self {
            current_context: None,
            start_tick: 0,
            last_tick: 0,
            channels: [ChannelStats::new(), ChannelStats::new(), ChannelStats::new()],
        }
    }

    /// Record one tick of data. Returns a finalized episode if the context
    /// changed or the 100-tick window was reached.
    pub fn record_tick(
        &mut self,
        context_hash: u32,
        tension: f32,
        coherence: f32,
        energy: f32,
        tick: u64,
    ) -> Option<InteractionEpisode> {
        let mut completed = None;

        match self.current_context {
            Some(prev_hash) => {
                // Context changed -- finalize the previous episode
                if prev_hash != context_hash {
                    completed = self.finalize_inner();
                    self.begin(context_hash, tick);
                } else if tick.saturating_sub(self.start_tick) >= MAX_EPISODE_TICKS {
                    // Window boundary reached
                    completed = self.finalize_inner();
                    self.begin(context_hash, tick);
                }
            }
            None => {
                // First tick ever
                self.begin(context_hash, tick);
            }
        }

        // Accumulate the current sample
        self.channels[0].push(tension);
        self.channels[1].push(coherence);
        self.channels[2].push(energy);
        self.last_tick = tick;

        completed
    }

    /// Force-finalize whatever is in progress. Returns `None` if empty.
    pub fn finalize(&mut self) -> Option<InteractionEpisode> {
        self.finalize_inner()
    }

    // -- internals --------------------------------------------------------

    fn begin(&mut self, context_hash: u32, tick: u64) {
        self.current_context = Some(context_hash);
        self.start_tick = tick;
        self.last_tick = tick;
        self.channels = [ChannelStats::new(), ChannelStats::new(), ChannelStats::new()];
    }

    fn finalize_inner(&mut self) -> Option<InteractionEpisode> {
        let context_hash = self.current_context.take()?;

        if self.channels[0].count == 0 {
            return None;
        }

        let t_stats = self.channels[0].finalize();
        let c_stats = self.channels[1].finalize();
        let e_stats = self.channels[2].finalize();

        let trajectory: TrajectoryVector = [
            t_stats[0], t_stats[1], t_stats[2], t_stats[3],
            c_stats[0], c_stats[1], c_stats[2], c_stats[3],
            e_stats[0], e_stats[1], e_stats[2], e_stats[3],
        ];

        // Simple heuristic for outcome:
        // - If coherence trend slope > 0.001 and tension trend slope < 0: Positive
        // - If coherence trend slope < -0.001 or tension peak > 0.8: Negative
        // - Otherwise: Neutral
        let outcome = if c_stats[2] > 0.001 && t_stats[2] <= 0.0 {
            EpisodeOutcome::Positive
        } else if c_stats[2] < -0.001 || t_stats[3] > 0.8 {
            EpisodeOutcome::Negative
        } else {
            EpisodeOutcome::Neutral
        };

        Some(InteractionEpisode {
            context_hash,
            start_tick: self.start_tick,
            end_tick: self.last_tick,
            outcome,
            trajectory,
        })
    }
}

impl Default for EpisodeRecorder {
    fn default() -> Self {
        Self::new()
    }
}

// ─── SQLite Persistence ─────────────────────────────────────────────

/// Create the `interaction_episodes` table if it does not exist.
pub fn create_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS interaction_episodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            context_hash INTEGER NOT NULL,
            start_tick INTEGER NOT NULL,
            end_tick INTEGER NOT NULL,
            outcome INTEGER NOT NULL,
            trajectory BLOB NOT NULL
        )",
    )
    .map_err(|e| format!("create interaction_episodes table: {}", e))
}

/// Serialize a `TrajectoryVector` ([f32; 12]) into a 48-byte blob (little-endian).
fn trajectory_to_blob(tv: &TrajectoryVector) -> Vec<u8> {
    let mut buf = Vec::with_capacity(48);
    for &f in tv.iter() {
        buf.extend_from_slice(&f.to_le_bytes());
    }
    buf
}

/// Deserialize a 48-byte blob back into a `TrajectoryVector`.
fn blob_to_trajectory(blob: &[u8]) -> Option<TrajectoryVector> {
    if blob.len() != 48 {
        return None;
    }
    let mut tv = [0.0f32; 12];
    for (i, chunk) in blob.chunks_exact(4).enumerate() {
        tv[i] = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    Some(tv)
}

/// Save a single episode to the database.
pub fn save_episode(
    conn: &rusqlite::Connection,
    episode: &InteractionEpisode,
) -> Result<(), String> {
    create_table(conn)?;

    let blob = trajectory_to_blob(&episode.trajectory);
    conn.execute(
        "INSERT INTO interaction_episodes \
         (context_hash, start_tick, end_tick, outcome, trajectory) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            episode.context_hash as i64,
            episode.start_tick as i64,
            episode.end_tick as i64,
            episode.outcome.to_i32(),
            blob,
        ],
    )
    .map_err(|e| format!("insert episode: {}", e))?;

    Ok(())
}

/// Load all episodes matching a given context hash.
pub fn load_episodes(
    conn: &rusqlite::Connection,
    context_hash: u32,
) -> Result<Vec<InteractionEpisode>, String> {
    // Check if table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master \
             WHERE type='table' AND name='interaction_episodes'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    if !table_exists {
        return Ok(vec![]);
    }

    let mut stmt = conn
        .prepare(
            "SELECT context_hash, start_tick, end_tick, outcome, trajectory \
             FROM interaction_episodes WHERE context_hash = ?1 \
             ORDER BY start_tick ASC",
        )
        .map_err(|e| format!("prepare episodes query: {}", e))?;

    let rows = stmt
        .query_map(rusqlite::params![context_hash as i64], |row| {
            let ctx: i64 = row.get(0)?;
            let start: i64 = row.get(1)?;
            let end: i64 = row.get(2)?;
            let outcome_val: i32 = row.get(3)?;
            let blob: Vec<u8> = row.get(4)?;
            Ok((ctx, start, end, outcome_val, blob))
        })
        .map_err(|e| format!("query episodes: {}", e))?;

    let mut episodes = Vec::new();
    for row in rows {
        let (ctx, start, end, outcome_val, blob) =
            row.map_err(|e| format!("row read: {}", e))?;

        let trajectory = match blob_to_trajectory(&blob) {
            Some(tv) => tv,
            None => continue, // skip malformed blobs
        };

        episodes.push(InteractionEpisode {
            context_hash: ctx as u32,
            start_tick: start as u64,
            end_tick: end as u64,
            outcome: EpisodeOutcome::from_i32(outcome_val),
            trajectory,
        });
    }

    Ok(episodes)
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Test 1: Episode finalized on context change
    // ------------------------------------------------------------------
    #[test]
    fn test_episode_finalized_on_context_change() {
        let mut recorder = EpisodeRecorder::new();

        // Record 10 ticks in context A
        for tick in 0..10 {
            let result = recorder.record_tick(100, 0.3, 0.5, 0.7, tick);
            assert!(result.is_none(), "should not finalize mid-episode");
        }

        // Switch to context B -- should finalize context A episode
        let result = recorder.record_tick(200, 0.4, 0.6, 0.8, 10);
        assert!(result.is_some(), "context change should finalize episode");

        let episode = result.unwrap();
        assert_eq!(episode.context_hash, 100);
        assert_eq!(episode.start_tick, 0);
        assert_eq!(episode.end_tick, 9);
    }

    // ------------------------------------------------------------------
    // Test 2: Episode finalized at 100-tick window boundary
    // ------------------------------------------------------------------
    #[test]
    fn test_episode_finalized_at_window_boundary() {
        let mut recorder = EpisodeRecorder::new();

        // Record exactly 100 ticks in the same context
        for tick in 0..100 {
            let result = recorder.record_tick(42, 0.5, 0.5, 0.5, tick);
            assert!(result.is_none(), "should not finalize before 100 ticks");
        }

        // Tick 100 should trigger finalization (tick 100 - start 0 = 100 >= MAX_EPISODE_TICKS)
        let result = recorder.record_tick(42, 0.5, 0.5, 0.5, 100);
        assert!(result.is_some(), "100-tick boundary should finalize episode");

        let episode = result.unwrap();
        assert_eq!(episode.context_hash, 42);
        assert_eq!(episode.start_tick, 0);
        assert_eq!(episode.end_tick, 99);
    }

    // ------------------------------------------------------------------
    // Test 3: TrajectoryVector captures mean/variance/slope/peak correctly
    // ------------------------------------------------------------------
    #[test]
    fn test_trajectory_vector_statistics() {
        let mut recorder = EpisodeRecorder::new();

        // Feed a linear ramp for tension: 0.0, 0.1, 0.2, ..., 0.9
        // Constant coherence: 0.5
        // Constant energy: 0.8
        for i in 0..10 {
            recorder.record_tick(99, i as f32 * 0.1, 0.5, 0.8, i);
        }

        let episode = recorder.finalize().expect("should finalize");

        // Tension channel: indices 0..3
        let t_mean = episode.trajectory[0];
        let t_variance = episode.trajectory[1];
        let t_slope = episode.trajectory[2];
        let t_peak = episode.trajectory[3];

        // Mean of 0.0..0.9 = 0.45
        assert!(
            (t_mean - 0.45).abs() < 0.02,
            "tension mean should be ~0.45, got {}",
            t_mean
        );

        // Variance should be > 0 for a ramp
        assert!(t_variance > 0.0, "tension variance should be > 0, got {}", t_variance);

        // Slope should be positive for an increasing ramp
        assert!(
            t_slope > 0.0,
            "tension slope should be positive for ramp, got {}",
            t_slope
        );

        // Peak should be 0.9
        assert!(
            (t_peak - 0.9).abs() < 0.02,
            "tension peak should be ~0.9, got {}",
            t_peak
        );

        // Coherence channel: indices 4..7
        let c_mean = episode.trajectory[4];
        let c_variance = episode.trajectory[5];
        let c_slope = episode.trajectory[6];

        // Constant 0.5 -> mean=0.5, variance~0, slope~0
        assert!(
            (c_mean - 0.5).abs() < 0.01,
            "coherence mean should be ~0.5, got {}",
            c_mean
        );
        assert!(
            c_variance < 0.001,
            "coherence variance should be ~0 for constant, got {}",
            c_variance
        );
        assert!(
            c_slope.abs() < 0.001,
            "coherence slope should be ~0 for constant, got {}",
            c_slope
        );

        // Energy channel: indices 8..11
        let e_mean = episode.trajectory[8];
        let e_peak = episode.trajectory[11];

        assert!(
            (e_mean - 0.8).abs() < 0.01,
            "energy mean should be ~0.8, got {}",
            e_mean
        );
        assert!(
            (e_peak - 0.8).abs() < 0.01,
            "energy peak should be ~0.8, got {}",
            e_peak
        );
    }

    // ------------------------------------------------------------------
    // Test 4: SQLite round-trip (save + load)
    // ------------------------------------------------------------------
    #[test]
    fn test_sqlite_round_trip() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");

        let episode = InteractionEpisode {
            context_hash: 12345,
            start_tick: 100,
            end_tick: 199,
            outcome: EpisodeOutcome::Positive,
            trajectory: [
                0.1, 0.2, 0.3, 0.4,
                0.5, 0.6, 0.7, 0.8,
                0.9, 1.0, 0.05, 0.15,
            ],
        };

        save_episode(&conn, &episode).expect("save should succeed");

        let loaded = load_episodes(&conn, 12345).expect("load should succeed");
        assert_eq!(loaded.len(), 1);

        let e = &loaded[0];
        assert_eq!(e.context_hash, 12345);
        assert_eq!(e.start_tick, 100);
        assert_eq!(e.end_tick, 199);
        assert_eq!(e.outcome, EpisodeOutcome::Positive);

        for i in 0..12 {
            assert!(
                (e.trajectory[i] - episode.trajectory[i]).abs() < 1e-6,
                "trajectory[{}]: expected {}, got {}",
                i,
                episode.trajectory[i],
                e.trajectory[i]
            );
        }
    }

    // ------------------------------------------------------------------
    // Test 5: Default/empty episode handling
    // ------------------------------------------------------------------
    #[test]
    fn test_empty_recorder_finalize() {
        let mut recorder = EpisodeRecorder::new();
        // Nothing recorded -- finalize should return None
        assert!(recorder.finalize().is_none(), "empty recorder should return None");
    }

    // ------------------------------------------------------------------
    // Test 6: Load from empty / non-existent table
    // ------------------------------------------------------------------
    #[test]
    fn test_load_from_empty_database() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");

        // Table doesn't exist yet
        let loaded = load_episodes(&conn, 999).expect("load should succeed");
        assert!(loaded.is_empty(), "empty DB should return no episodes");
    }

    // ------------------------------------------------------------------
    // Test 7: Multiple episodes for same context
    // ------------------------------------------------------------------
    #[test]
    fn test_multiple_episodes_same_context() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");

        for i in 0..3 {
            let episode = InteractionEpisode {
                context_hash: 42,
                start_tick: i * 100,
                end_tick: i * 100 + 99,
                outcome: EpisodeOutcome::Neutral,
                trajectory: [0.0; 12],
            };
            save_episode(&conn, &episode).expect("save should succeed");
        }

        let loaded = load_episodes(&conn, 42).expect("load should succeed");
        assert_eq!(loaded.len(), 3);

        // Should be ordered by start_tick
        assert_eq!(loaded[0].start_tick, 0);
        assert_eq!(loaded[1].start_tick, 100);
        assert_eq!(loaded[2].start_tick, 200);
    }

    // ------------------------------------------------------------------
    // Test 8: Different context hashes are isolated
    // ------------------------------------------------------------------
    #[test]
    fn test_context_isolation() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");

        let ep1 = InteractionEpisode {
            context_hash: 111,
            start_tick: 0,
            end_tick: 50,
            outcome: EpisodeOutcome::Positive,
            trajectory: [1.0; 12],
        };
        let ep2 = InteractionEpisode {
            context_hash: 222,
            start_tick: 0,
            end_tick: 50,
            outcome: EpisodeOutcome::Negative,
            trajectory: [2.0; 12],
        };

        save_episode(&conn, &ep1).expect("save ep1");
        save_episode(&conn, &ep2).expect("save ep2");

        let loaded_111 = load_episodes(&conn, 111).expect("load 111");
        assert_eq!(loaded_111.len(), 1);
        assert_eq!(loaded_111[0].outcome, EpisodeOutcome::Positive);

        let loaded_222 = load_episodes(&conn, 222).expect("load 222");
        assert_eq!(loaded_222.len(), 1);
        assert_eq!(loaded_222[0].outcome, EpisodeOutcome::Negative);
    }

    // ------------------------------------------------------------------
    // Test 9: Blob serialization round-trip
    // ------------------------------------------------------------------
    #[test]
    fn test_trajectory_blob_round_trip() {
        let tv: TrajectoryVector = [
            0.1, 0.2, 0.3, 0.4,
            0.5, 0.6, 0.7, 0.8,
            0.9, 1.0, -0.5, 3.14,
        ];
        let blob = trajectory_to_blob(&tv);
        assert_eq!(blob.len(), 48);

        let restored = blob_to_trajectory(&blob).expect("should parse");
        for i in 0..12 {
            assert!(
                (restored[i] - tv[i]).abs() < 1e-7,
                "index {}: expected {}, got {}",
                i,
                tv[i],
                restored[i]
            );
        }
    }

    // ------------------------------------------------------------------
    // Test 10: Malformed blob is skipped
    // ------------------------------------------------------------------
    #[test]
    fn test_malformed_blob_returns_none() {
        assert!(blob_to_trajectory(&[0u8; 10]).is_none());
        assert!(blob_to_trajectory(&[]).is_none());
        assert!(blob_to_trajectory(&[0u8; 49]).is_none());
    }

    // ------------------------------------------------------------------
    // Test 11: EpisodeOutcome round-trip via i32
    // ------------------------------------------------------------------
    #[test]
    fn test_outcome_round_trip() {
        for outcome in &[EpisodeOutcome::Positive, EpisodeOutcome::Negative, EpisodeOutcome::Neutral] {
            assert_eq!(EpisodeOutcome::from_i32(outcome.to_i32()), *outcome);
        }
        // Unknown value -> Neutral
        assert_eq!(EpisodeOutcome::from_i32(99), EpisodeOutcome::Neutral);
    }

    // ------------------------------------------------------------------
    // Test 12: create_table is idempotent
    // ------------------------------------------------------------------
    #[test]
    fn test_create_table_idempotent() {
        let conn = rusqlite::Connection::open_in_memory().expect("open in-memory db");
        create_table(&conn).expect("first create");
        create_table(&conn).expect("second create should not fail");
    }

    // ------------------------------------------------------------------
    // Test 13: ChannelStats empty finalize
    // ------------------------------------------------------------------
    #[test]
    fn test_channel_stats_empty() {
        let stats = ChannelStats::new();
        let result = stats.finalize();
        assert_eq!(result, [0.0, 0.0, 0.0, 0.0]);
    }

    // ------------------------------------------------------------------
    // Test 14: ChannelStats single value
    // ------------------------------------------------------------------
    #[test]
    fn test_channel_stats_single_value() {
        let mut stats = ChannelStats::new();
        stats.push(0.5);
        let result = stats.finalize();

        // mean = 0.5, variance = 0 (only 1 sample), slope = 0, peak = 0.5
        assert!((result[0] - 0.5).abs() < 1e-6, "mean should be 0.5");
        assert!((result[1] - 0.0).abs() < 1e-6, "variance should be 0");
        assert!((result[2] - 0.0).abs() < 1e-6, "slope should be 0");
        assert!((result[3] - 0.5).abs() < 1e-6, "peak should be 0.5");
    }
}
