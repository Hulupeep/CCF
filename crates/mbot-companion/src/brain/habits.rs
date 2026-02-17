//! Habit compilation — detects frequently repeated behavioral sequences
//! in specific contexts and "compiles" them into autonomous routines.
//!
//! When sensor conditions shift from the learned distribution, the compiled
//! routine halts, preventing stale behavior in changed environments.
//!
//! # Contract Compliance
//! - **ARCH-001**: All habit logic lives in mbot-companion, not mbot-core
//! - **INVARIANT-004**: Personality modulates compilation speed (threshold), not structure
//! - **INVARIANT-006**: Distribution shift z-score test is pure math

use mbot_core::coherence::SocialPhase;

// ─── Sensor Distribution ────────────────────────────────────────────

/// Running statistics for sensor values during a compiled routine's context.
///
/// Uses Welford's online algorithm for numerically stable mean/variance.
/// The z-score method detects when current sensor readings have shifted
/// away from the distribution observed during routine learning.
#[derive(Clone, Debug)]
pub struct SensorDistribution {
    pub loudness_mean: f32,
    pub loudness_std: f32,
    pub brightness_mean: f32,
    pub brightness_std: f32,
    pub distance_mean: f32,
    pub distance_std: f32,
    pub sample_count: u32,
    // Internal Welford accumulators (M2 for variance computation)
    loudness_m2: f64,
    brightness_m2: f64,
    distance_m2: f64,
}

impl SensorDistribution {
    /// Create an empty distribution with no samples.
    pub fn new() -> Self {
        Self {
            loudness_mean: 0.0,
            loudness_std: 0.0,
            brightness_mean: 0.0,
            brightness_std: 0.0,
            distance_mean: 0.0,
            distance_std: 0.0,
            sample_count: 0,
            loudness_m2: 0.0,
            brightness_m2: 0.0,
            distance_m2: 0.0,
        }
    }

    /// Update the distribution with a new sensor reading using Welford's online algorithm.
    pub fn update(&mut self, loudness: f32, brightness: f32, distance: f32) {
        self.sample_count += 1;
        let n = self.sample_count as f64;

        // Loudness channel
        let loud_v = loudness as f64;
        let loud_delta = loud_v - self.loudness_mean as f64;
        self.loudness_mean += (loud_delta / n) as f32;
        let loud_delta2 = loud_v - self.loudness_mean as f64;
        self.loudness_m2 += loud_delta * loud_delta2;

        // Brightness channel
        let bright_v = brightness as f64;
        let bright_delta = bright_v - self.brightness_mean as f64;
        self.brightness_mean += (bright_delta / n) as f32;
        let bright_delta2 = bright_v - self.brightness_mean as f64;
        self.brightness_m2 += bright_delta * bright_delta2;

        // Distance channel
        let dist_v = distance as f64;
        let dist_delta = dist_v - self.distance_mean as f64;
        self.distance_mean += (dist_delta / n) as f32;
        let dist_delta2 = dist_v - self.distance_mean as f64;
        self.distance_m2 += dist_delta * dist_delta2;

        // Recompute standard deviations
        if self.sample_count > 1 {
            let n_minus_1 = (self.sample_count - 1) as f64;
            self.loudness_std = (self.loudness_m2 / n_minus_1).sqrt() as f32;
            self.brightness_std = (self.brightness_m2 / n_minus_1).sqrt() as f32;
            self.distance_std = (self.distance_m2 / n_minus_1).sqrt() as f32;
        }
    }

    /// Compute the maximum absolute z-score across all three sensor channels.
    ///
    /// Returns 0.0 if fewer than 5 samples have been recorded (not enough
    /// data for a meaningful distribution).
    pub fn z_score(&self, loudness: f32, brightness: f32, distance: f32) -> f32 {
        if self.sample_count < 5 {
            return 0.0;
        }

        let z_loud = if self.loudness_std > f32::EPSILON {
            ((loudness - self.loudness_mean) / self.loudness_std).abs()
        } else {
            0.0
        };

        let z_bright = if self.brightness_std > f32::EPSILON {
            ((brightness - self.brightness_mean) / self.brightness_std).abs()
        } else {
            0.0
        };

        let z_dist = if self.distance_std > f32::EPSILON {
            ((distance - self.distance_mean) / self.distance_std).abs()
        } else {
            0.0
        };

        // Return the maximum z-score across all channels
        z_loud.max(z_bright).max(z_dist)
    }
}

impl Default for SensorDistribution {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Motor Step ─────────────────────────────────────────────────────

/// A single step in a compiled routine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MotorStep {
    pub left: i32,
    pub right: i32,
    pub duration_ticks: u32,
}

// ─── Behavioral Sequence ────────────────────────────────────────────

/// Tracks a repeated pattern of motor commands in a specific context.
///
/// As the same sequence of motor steps is observed in the same context
/// and social phase, the execution count grows. Once it crosses the
/// compilation threshold with a sufficient success rate, the sequence
/// can be compiled into a `CompiledRoutine`.
pub struct BehavioralSequence {
    pub context_hash: u32,
    pub quadrant: SocialPhase,
    pub steps: Vec<MotorStep>,
    pub execution_count: u32,
    pub success_count: u32,
    pub distribution: SensorDistribution,
}

impl BehavioralSequence {
    /// Create a new behavioral sequence with zero executions.
    pub fn new(context_hash: u32, quadrant: SocialPhase, steps: Vec<MotorStep>) -> Self {
        Self {
            context_hash,
            quadrant,
            steps,
            execution_count: 0,
            success_count: 0,
            distribution: SensorDistribution::new(),
        }
    }

    /// Record one execution of this sequence.
    pub fn record_execution(&mut self, success: bool) {
        self.execution_count += 1;
        if success {
            self.success_count += 1;
        }
    }

    /// Compute the success rate (0.0 if no executions yet).
    pub fn success_rate(&self) -> f32 {
        if self.execution_count == 0 {
            0.0
        } else {
            self.success_count as f32 / self.execution_count as f32
        }
    }
}

// ─── Compiled Routine ───────────────────────────────────────────────

/// A finalized routine ready for autonomous execution.
///
/// Once a `BehavioralSequence` has been executed enough times with
/// sufficient success, it is "compiled" into this form. The routine
/// includes the sensor distribution learned during training so that
/// distribution shifts (changed environment) can be detected and
/// the routine halted.
pub struct CompiledRoutine {
    pub context_hash: u32,
    pub quadrant: SocialPhase,
    pub steps: Vec<MotorStep>,
    pub distribution: SensorDistribution,
    /// Z-score threshold above which the routine should halt (default 2.5).
    pub shift_threshold: f32,
}

impl CompiledRoutine {
    /// Create a compiled routine from a behavioral sequence.
    pub fn from_sequence(seq: &BehavioralSequence, shift_threshold: f32) -> Self {
        Self {
            context_hash: seq.context_hash,
            quadrant: seq.quadrant,
            steps: seq.steps.clone(),
            distribution: seq.distribution.clone(),
            shift_threshold,
        }
    }

    /// Check whether the current sensor readings represent a distribution shift.
    ///
    /// Returns `true` if the z-score exceeds the threshold (routine should halt).
    /// Returns `false` if readings are within the learned distribution.
    pub fn check_distribution(&self, loudness: f32, brightness: f32, distance: f32) -> bool {
        self.distribution.z_score(loudness, brightness, distance) > self.shift_threshold
    }
}

// ─── Habit Compiler ─────────────────────────────────────────────────

/// The main orchestrator for habit detection and compilation.
///
/// Records behavioral sequences as they are observed, and compiles
/// them into routines when they have been executed frequently enough
/// with a high success rate.
///
/// The compilation threshold is modulated by personality: higher
/// `curiosity_drive` lowers the threshold, letting curious robots
/// compile habits faster (INVARIANT-004).
pub struct HabitCompiler {
    pub sequences: Vec<BehavioralSequence>,
    pub compiled: Vec<CompiledRoutine>,
    /// Minimum executions required before compilation (default 20).
    pub base_threshold: u32,
}

impl HabitCompiler {
    /// Create a new habit compiler with default threshold of 20.
    pub fn new() -> Self {
        Self {
            sequences: Vec::new(),
            compiled: Vec::new(),
            base_threshold: 20,
        }
    }

    /// Record an observed behavioral sequence.
    ///
    /// If a matching sequence already exists (same context_hash, quadrant,
    /// and steps), it is updated with the new execution. Otherwise a new
    /// sequence is created.
    pub fn record_sequence(
        &mut self,
        context_hash: u32,
        quadrant: SocialPhase,
        steps: Vec<MotorStep>,
        success: bool,
    ) {
        // Look for an existing matching sequence
        if let Some(seq) = self.sequences.iter_mut().find(|s| {
            s.context_hash == context_hash && s.quadrant == quadrant && s.steps == steps
        }) {
            seq.record_execution(success);
            return;
        }

        // No match found -- create a new sequence
        let mut seq = BehavioralSequence::new(context_hash, quadrant, steps);
        seq.record_execution(success);
        self.sequences.push(seq);
    }

    /// Try to compile sequences that meet the threshold and success criteria.
    ///
    /// The effective threshold is `base_threshold / curiosity_drive.max(0.1)`,
    /// so higher curiosity lowers the bar for compilation (INVARIANT-004).
    ///
    /// Only sequences with a success rate above 80% are compiled.
    ///
    /// Returns the list of newly compiled routines.
    pub fn try_compile(&mut self, curiosity_drive: f32) -> Vec<CompiledRoutine> {
        let effective_threshold =
            (self.base_threshold as f32 / curiosity_drive.max(0.1)) as u32;

        let mut newly_compiled = Vec::new();

        for seq in &self.sequences {
            // Skip if already compiled (same context/quadrant/steps)
            let already_compiled = self.compiled.iter().any(|c| {
                c.context_hash == seq.context_hash
                    && c.quadrant == seq.quadrant
                    && c.steps == seq.steps
            });
            if already_compiled {
                continue;
            }

            // Check compilation criteria
            if seq.execution_count >= effective_threshold && seq.success_rate() > 0.8 {
                let routine = CompiledRoutine::from_sequence(seq, 2.5);
                newly_compiled.push(routine);
            }
        }

        // Add the newly compiled routines to the compiled list
        for routine in &newly_compiled {
            self.compiled.push(CompiledRoutine {
                context_hash: routine.context_hash,
                quadrant: routine.quadrant,
                steps: routine.steps.clone(),
                distribution: routine.distribution.clone(),
                shift_threshold: routine.shift_threshold,
            });
        }

        newly_compiled
    }

    /// Look up a compiled routine by context hash and social phase.
    pub fn find_routine(&self, context_hash: u32, quadrant: SocialPhase) -> Option<&CompiledRoutine> {
        self.compiled
            .iter()
            .find(|r| r.context_hash == context_hash && r.quadrant == quadrant)
    }

    /// Number of compiled routines.
    pub fn compiled_count(&self) -> usize {
        self.compiled.len()
    }
}

impl Default for HabitCompiler {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Test 1: SensorDistribution z_score returns 0 with < 5 samples
    // ------------------------------------------------------------------
    #[test]
    fn test_z_score_returns_zero_with_few_samples() {
        let mut dist = SensorDistribution::new();

        // Feed only 4 samples (below the threshold of 5)
        for i in 0..4 {
            dist.update(50.0 + i as f32, 50.0, 50.0);
        }

        // Even a wildly different reading should return 0.0
        let z = dist.z_score(999.0, 999.0, 999.0);
        assert!(
            (z - 0.0).abs() < f32::EPSILON,
            "z_score should be 0.0 with < 5 samples, got {}",
            z
        );
    }

    // ------------------------------------------------------------------
    // Test 2: SensorDistribution z_score detects shift
    // ------------------------------------------------------------------
    #[test]
    fn test_z_score_detects_shift() {
        let mut dist = SensorDistribution::new();

        // Feed 20 samples clustered around mean=50, std~5
        for _ in 0..20 {
            dist.update(50.0, 50.0, 50.0);
        }
        // Add some variance
        for _ in 0..10 {
            dist.update(45.0, 50.0, 50.0);
        }
        for _ in 0..10 {
            dist.update(55.0, 50.0, 50.0);
        }

        // A value of 80 should produce a high z-score (6 std devs away)
        let z = dist.z_score(80.0, 50.0, 50.0);
        assert!(
            z > 2.0,
            "z_score should be high for value far from mean, got {}",
            z
        );
    }

    // ------------------------------------------------------------------
    // Test 3: BehavioralSequence tracks execution/success counts
    // ------------------------------------------------------------------
    #[test]
    fn test_behavioral_sequence_counts() {
        let steps = vec![
            MotorStep { left: 50, right: 50, duration_ticks: 10 },
            MotorStep { left: -30, right: 30, duration_ticks: 5 },
        ];

        let mut seq = BehavioralSequence::new(42, SocialPhase::QuietlyBeloved, steps);
        assert_eq!(seq.execution_count, 0);
        assert_eq!(seq.success_count, 0);

        seq.record_execution(true);
        seq.record_execution(true);
        seq.record_execution(false);

        assert_eq!(seq.execution_count, 3);
        assert_eq!(seq.success_count, 2);
    }

    // ------------------------------------------------------------------
    // Test 4: BehavioralSequence success_rate calculation
    // ------------------------------------------------------------------
    #[test]
    fn test_behavioral_sequence_success_rate() {
        let steps = vec![MotorStep { left: 10, right: 10, duration_ticks: 5 }];
        let mut seq = BehavioralSequence::new(1, SocialPhase::ShyObserver, steps);

        // Zero executions -> 0.0
        assert!((seq.success_rate() - 0.0).abs() < f32::EPSILON);

        // 4 successes, 1 failure -> 0.8
        for _ in 0..4 {
            seq.record_execution(true);
        }
        seq.record_execution(false);

        assert!(
            (seq.success_rate() - 0.8).abs() < 0.01,
            "success_rate should be 0.8, got {}",
            seq.success_rate()
        );
    }

    // ------------------------------------------------------------------
    // Test 5: CompiledRoutine check_distribution detects shift
    // ------------------------------------------------------------------
    #[test]
    fn test_compiled_routine_detects_shift() {
        let mut dist = SensorDistribution::new();
        // Build a distribution: mean ~50, some variance
        for _ in 0..20 {
            dist.update(50.0, 50.0, 50.0);
        }
        for _ in 0..10 {
            dist.update(45.0, 45.0, 45.0);
        }
        for _ in 0..10 {
            dist.update(55.0, 55.0, 55.0);
        }

        let routine = CompiledRoutine {
            context_hash: 1,
            quadrant: SocialPhase::QuietlyBeloved,
            steps: vec![],
            distribution: dist,
            shift_threshold: 2.5,
        };

        // Way outside the distribution
        assert!(
            routine.check_distribution(200.0, 50.0, 50.0),
            "should detect shift for value far from mean"
        );
    }

    // ------------------------------------------------------------------
    // Test 6: CompiledRoutine check_distribution passes for in-range values
    // ------------------------------------------------------------------
    #[test]
    fn test_compiled_routine_passes_for_in_range() {
        let mut dist = SensorDistribution::new();
        for _ in 0..20 {
            dist.update(50.0, 50.0, 50.0);
        }
        for _ in 0..10 {
            dist.update(45.0, 45.0, 45.0);
        }
        for _ in 0..10 {
            dist.update(55.0, 55.0, 55.0);
        }

        let routine = CompiledRoutine {
            context_hash: 1,
            quadrant: SocialPhase::QuietlyBeloved,
            steps: vec![],
            distribution: dist,
            shift_threshold: 2.5,
        };

        // Value near the mean
        assert!(
            !routine.check_distribution(51.0, 49.0, 50.0),
            "should NOT detect shift for values near mean"
        );
    }

    // ------------------------------------------------------------------
    // Test 7: HabitCompiler records new sequence
    // ------------------------------------------------------------------
    #[test]
    fn test_habit_compiler_records_new_sequence() {
        let mut compiler = HabitCompiler::new();
        assert_eq!(compiler.sequences.len(), 0);

        let steps = vec![MotorStep { left: 50, right: 50, duration_ticks: 10 }];
        compiler.record_sequence(42, SocialPhase::ShyObserver, steps, true);

        assert_eq!(compiler.sequences.len(), 1);
        assert_eq!(compiler.sequences[0].execution_count, 1);
        assert_eq!(compiler.sequences[0].success_count, 1);
    }

    // ------------------------------------------------------------------
    // Test 8: HabitCompiler matches existing sequence
    // ------------------------------------------------------------------
    #[test]
    fn test_habit_compiler_matches_existing_sequence() {
        let mut compiler = HabitCompiler::new();
        let steps = vec![MotorStep { left: 50, right: 50, duration_ticks: 10 }];

        // Record the same sequence twice
        compiler.record_sequence(42, SocialPhase::ShyObserver, steps.clone(), true);
        compiler.record_sequence(42, SocialPhase::ShyObserver, steps.clone(), true);

        // Should be ONE sequence with 2 executions, not two sequences
        assert_eq!(compiler.sequences.len(), 1);
        assert_eq!(compiler.sequences[0].execution_count, 2);
    }

    // ------------------------------------------------------------------
    // Test 9: HabitCompiler compiles after threshold met with >80% success
    // ------------------------------------------------------------------
    #[test]
    fn test_habit_compiler_compiles_after_threshold() {
        let mut compiler = HabitCompiler::new();
        compiler.base_threshold = 10; // Lower threshold for testing
        let steps = vec![MotorStep { left: 50, right: 50, duration_ticks: 10 }];

        // Record 10 successful executions (100% success rate)
        for _ in 0..10 {
            compiler.record_sequence(42, SocialPhase::QuietlyBeloved, steps.clone(), true);
        }

        // curiosity_drive = 1.0 -> effective_threshold = 10/1.0 = 10
        let newly_compiled = compiler.try_compile(1.0);
        assert_eq!(
            newly_compiled.len(),
            1,
            "should compile 1 routine after threshold met"
        );
        assert_eq!(compiler.compiled_count(), 1);
    }

    // ------------------------------------------------------------------
    // Test 10: HabitCompiler personality modulates threshold
    // ------------------------------------------------------------------
    #[test]
    fn test_personality_modulates_threshold() {
        let mut compiler = HabitCompiler::new();
        compiler.base_threshold = 20;
        let steps = vec![MotorStep { left: 50, right: 50, duration_ticks: 10 }];

        // Record 10 successful executions (all successes)
        for _ in 0..10 {
            compiler.record_sequence(42, SocialPhase::QuietlyBeloved, steps.clone(), true);
        }

        // With curiosity_drive = 0.5 -> threshold = 20/0.5 = 40 -> NOT enough
        let result_low = compiler.try_compile(0.5);
        assert_eq!(
            result_low.len(),
            0,
            "should NOT compile with low curiosity (threshold too high)"
        );

        // With curiosity_drive = 5.0 -> threshold = 20/5.0 = 4 -> enough (10 >= 4)
        let result_high = compiler.try_compile(5.0);
        assert_eq!(
            result_high.len(),
            1,
            "should compile with high curiosity (threshold lowered)"
        );
    }

    // ------------------------------------------------------------------
    // Test 11: HabitCompiler does NOT compile below 80% success rate
    // ------------------------------------------------------------------
    #[test]
    fn test_does_not_compile_below_success_threshold() {
        let mut compiler = HabitCompiler::new();
        compiler.base_threshold = 10;
        let steps = vec![MotorStep { left: 50, right: 50, duration_ticks: 10 }];

        // Record 10 executions with only 7 successes (70% < 80%)
        for _ in 0..7 {
            compiler.record_sequence(42, SocialPhase::QuietlyBeloved, steps.clone(), true);
        }
        for _ in 0..3 {
            compiler.record_sequence(42, SocialPhase::QuietlyBeloved, steps.clone(), false);
        }

        let result = compiler.try_compile(1.0);
        assert_eq!(
            result.len(),
            0,
            "should NOT compile with success rate below 80%"
        );
        assert_eq!(compiler.compiled_count(), 0);
    }

    // ------------------------------------------------------------------
    // Test 12: find_routine returns compiled routine
    // ------------------------------------------------------------------
    #[test]
    fn test_find_routine_returns_compiled() {
        let mut compiler = HabitCompiler::new();
        compiler.base_threshold = 5;
        let steps = vec![MotorStep { left: 50, right: 50, duration_ticks: 10 }];

        // Build a compilable sequence
        for _ in 0..5 {
            compiler.record_sequence(42, SocialPhase::QuietlyBeloved, steps.clone(), true);
        }
        compiler.try_compile(1.0);

        // Should find the routine
        let found = compiler.find_routine(42, SocialPhase::QuietlyBeloved);
        assert!(found.is_some(), "should find compiled routine");
        assert_eq!(found.unwrap().context_hash, 42);

        // Should NOT find a routine for a different context
        let not_found = compiler.find_routine(999, SocialPhase::ShyObserver);
        assert!(not_found.is_none(), "should not find routine for wrong context");
    }

    // ------------------------------------------------------------------
    // Test 13: Different quadrants create separate sequences
    // ------------------------------------------------------------------
    #[test]
    fn test_different_quadrants_separate_sequences() {
        let mut compiler = HabitCompiler::new();
        let steps = vec![MotorStep { left: 50, right: 50, duration_ticks: 10 }];

        // Same steps and context_hash but different quadrants
        compiler.record_sequence(42, SocialPhase::ShyObserver, steps.clone(), true);
        compiler.record_sequence(42, SocialPhase::QuietlyBeloved, steps.clone(), true);

        assert_eq!(
            compiler.sequences.len(),
            2,
            "different quadrants should create separate sequences"
        );
    }

    // ------------------------------------------------------------------
    // Test 14: Welford accumulator numerical stability
    // ------------------------------------------------------------------
    #[test]
    fn test_welford_numerical_stability() {
        let mut dist = SensorDistribution::new();

        // Feed many identical values: std should be ~0
        for _ in 0..100 {
            dist.update(42.0, 42.0, 42.0);
        }

        assert!(
            (dist.loudness_mean - 42.0).abs() < 0.01,
            "mean should be ~42.0, got {}",
            dist.loudness_mean
        );
        assert!(
            dist.loudness_std < 0.01,
            "std should be ~0 for constant input, got {}",
            dist.loudness_std
        );
    }

    // ------------------------------------------------------------------
    // Test 15: z_score returns 0 when std is zero (all identical samples)
    // ------------------------------------------------------------------
    #[test]
    fn test_z_score_zero_std() {
        let mut dist = SensorDistribution::new();

        // Feed 10 identical values -> std = 0
        for _ in 0..10 {
            dist.update(50.0, 50.0, 50.0);
        }

        // z_score for a different value should return 0.0 (not inf/nan)
        // because std is 0 and we guard against division by zero
        let z = dist.z_score(100.0, 100.0, 100.0);
        assert!(
            z.is_finite(),
            "z_score should be finite when std is zero, got {}",
            z
        );
    }
}
