//! Reflex Lab Educational Experiments
//!
//! Provides four core experiments for understanding stimulus-response relationships:
//! 1. Startle Response - How sensitive the robot is to sudden changes
//! 2. Approach/Avoid - What makes the robot curious vs. scared
//! 3. Recovery Time - How long it takes the robot to calm down
//! 4. Mode Transitions - What triggers each personality mode
//!
//! Each experiment uses age-appropriate language and provides step-by-step guidance
//! with hints for students who get stuck. (I-LEARN-020, I-LEARN-023)

use serde::{Deserialize, Serialize};
// ARCH-LEARN-003: Timing must use monotonic clock (Instant::now() for real measurements)
// Currently not used in module-level API, but required by contract for future timer integration

/// All possible experiment types in Reflex Lab
///
/// Each experiment teaches a different concept about the robot's nervous system.
/// (LEARN-003: Must implement all four types)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentType {
    /// Startle Response - How the robot reacts to sudden changes
    StartleResponse,
    /// Approach/Avoid - What triggers curiosity vs. fear
    ApproachAvoid,
    /// Recovery Time - How long it takes to calm down
    RecoveryTime,
    /// Mode Transitions - What causes the robot to switch moods
    ModeTransitions,
}

/// Action type for each step in an experiment
///
/// Guides students through the experiment with clear actions:
/// - observe: Watch what happens
/// - adjust: Change a robot setting
/// - stimulate: Do something to trigger the robot
/// - record: Write down what you see
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepActionType {
    /// "Just watch and observe"
    Observe,
    /// "Change this setting"
    Adjust,
    /// "Do this to the robot"
    Stimulate,
    /// "Write down what you notice"
    Record,
}

/// A single step in an experiment with guidance and success criteria
///
/// Each step provides clear instructions and a hint if the student needs help.
/// (I-LEARN-020: Age-appropriate language, I-LEARN-023: Hints available)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentStep {
    /// Which step number (1, 2, 3, ...)
    pub step_number: u8,
    /// What to do: "Set the sensitivity slider all the way up"
    /// Language is simple and action-oriented for 10-year-olds
    pub instruction: String,
    /// Type of action: observe, adjust, stimulate, or record
    pub action_type: StepActionType,
    /// Optional: The parameter to adjust (e.g., "startle_sensitivity")
    pub parameter: Option<String>,
    /// What should happen if done correctly
    /// Example: "The robot will jump when you clap"
    pub expected_outcome: Option<String>,
    /// A helpful hint if student is stuck (I-LEARN-023)
    pub hint: Option<String>,
    /// Has this step been completed?
    pub completed: bool,
}

impl ExperimentStep {
    /// Create a new experiment step
    pub fn new(step_number: u8, instruction: String, action_type: StepActionType) -> Self {
        Self {
            step_number,
            instruction,
            action_type,
            parameter: None,
            expected_outcome: None,
            hint: None,
            completed: false,
        }
    }

    /// Add the parameter being adjusted
    pub fn with_parameter(mut self, parameter: String) -> Self {
        self.parameter = Some(parameter);
        self
    }

    /// Add what should happen if done correctly
    pub fn with_expected_outcome(mut self, outcome: String) -> Self {
        self.expected_outcome = Some(outcome);
        self
    }

    /// Add a helpful hint for stuck students
    pub fn with_hint(mut self, hint: String) -> Self {
        self.hint = Some(hint);
        self
    }

    /// Mark this step as completed
    pub fn mark_completed(&mut self) {
        self.completed = true;
    }
}

/// Core experiment definition
///
/// Defines all the metadata and steps for an experiment.
/// Provides learning objectives and difficulty level.
/// (I-LEARN-021: Clear learning objectives)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Experiment {
    /// Unique identifier (e.g., "startle-001")
    pub id: String,
    /// Friendly name (e.g., "Startle Response")
    pub name: String,
    /// Simple description for 10-year-olds
    /// Example: "Learn how sensitive the robot is to sudden changes"
    pub description: String,
    /// What students will learn
    /// (I-LEARN-021: Required by contract)
    pub learning_objectives: Vec<String>,
    /// How long the experiment takes (in minutes)
    pub duration_minutes: u16,
    /// Is this for beginners, intermediate, or advanced students?
    pub difficulty: DifficultyLevel,
    /// All the steps in this experiment
    pub steps: Vec<ExperimentStep>,
    /// Parameters that students can adjust during the experiment
    pub parameters_to_adjust: Vec<String>,
    /// What observations students should make
    pub observations_expected: Vec<String>,
}

/// Difficulty level for age-appropriate progression
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
}

impl Experiment {
    /// Create a new experiment definition
    pub fn new(
        id: String,
        name: String,
        description: String,
        duration_minutes: u16,
        difficulty: DifficultyLevel,
    ) -> Self {
        Self {
            id,
            name,
            description,
            learning_objectives: Vec::new(),
            duration_minutes,
            difficulty,
            steps: Vec::new(),
            parameters_to_adjust: Vec::new(),
            observations_expected: Vec::new(),
        }
    }

    /// Add a learning objective
    pub fn add_learning_objective(mut self, objective: String) -> Self {
        self.learning_objectives.push(objective);
        self
    }

    /// Add a step to the experiment
    pub fn add_step(mut self, step: ExperimentStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add a parameter that can be adjusted
    pub fn add_adjustable_parameter(mut self, parameter: String) -> Self {
        self.parameters_to_adjust.push(parameter);
        self
    }

    /// Add an observation students should make
    pub fn add_observation(mut self, observation: String) -> Self {
        self.observations_expected.push(observation);
        self
    }

    /// Get the current step (None if experiment is complete)
    pub fn current_step(&self) -> Option<&ExperimentStep> {
        self.steps.iter().find(|s| !s.completed)
    }

    /// Get progress as percentage (0-100)
    pub fn progress_percent(&self) -> u8 {
        if self.steps.is_empty() {
            return 0;
        }
        let completed = self.steps.iter().filter(|s| s.completed).count();
        ((completed as f32 / self.steps.len() as f32) * 100.0) as u8
    }

    /// Is the experiment complete?
    pub fn is_complete(&self) -> bool {
        !self.steps.is_empty() && self.steps.iter().all(|s| s.completed)
    }
}

/// Observation record during an experiment
///
/// Each observation is tagged with:
/// - Step number
/// - Timestamp (using monotonic clock - ARCH-LEARN-003)
/// - Description of what was observed
/// - Optional metrics from the robot's nervous system
/// (I-LEARN-022: Observations correlate with actual robot state)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    /// Which step generated this observation
    pub step: u8,
    /// When this was observed (milliseconds from session start)
    pub elapsed_ms: u64,
    /// What the student observed (age-appropriate language)
    /// Example: "The robot jumped and the tension went way up!"
    pub description: String,
    /// Optional metrics from robot nervous system
    pub metrics: Option<ObservationMetrics>,
}

/// Nervous system metrics captured during an observation
///
/// These are raw measurements that we explain in simple language.
/// (I-LEARN-022: Must use actual robot homeostasis state)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservationMetrics {
    /// How nervous was the robot before (0.0-1.0)?
    pub tension_before: f32,
    /// How nervous was the robot after (0.0-1.0)?
    pub tension_after: f32,
    /// How long from stimulus to robot response (milliseconds)?
    pub response_time_ms: u32,
}

/// Parameter change during an experiment
///
/// Tracks when a student adjusts robot settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParameterChange {
    /// Which parameter was changed (e.g., "startle_sensitivity")
    pub parameter_name: String,
    /// What was the old value?
    pub old_value: f32,
    /// What was the new value?
    pub new_value: f32,
    /// When did this change happen (milliseconds from start)?
    pub elapsed_ms: u64,
}

/// Reaction time measurement
///
/// Used to track how fast the robot responds to stimuli.
/// (ARCH-LEARN-003: Must use monotonic Instant::now())
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimingResult {
    /// Time from stimulus to first motor response (milliseconds)
    pub stimulus_to_response_ms: u32,
    /// Time from first response to peak tension (milliseconds)
    pub response_to_peak_ms: u32,
    /// Time from peak tension back to baseline (milliseconds)
    pub peak_to_baseline_ms: u32,
}

/// Active experiment session
///
/// Tracks a student's progress through a single experiment run.
/// Stores all observations and parameter changes made during the session.
/// (ARCH-LEARN-002: Must derive Serialize/Deserialize)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperimentSession {
    /// Which experiment are we running?
    pub experiment_id: String,
    /// When did this session start? (Real-time timestamp)
    pub started_at: u64,
    /// Current step number (0-indexed)
    pub current_step: usize,
    /// All observations made so far
    pub observations: Vec<Observation>,
    /// All parameter adjustments made so far
    pub parameter_changes: Vec<ParameterChange>,
    /// Has the session been completed?
    pub completed: bool,
    /// Optional timing results for reflex measurements
    pub timing_results: Vec<TimingResult>,
}

impl ExperimentSession {
    /// Start a new experiment session
    pub fn new(experiment_id: String, started_at: u64) -> Self {
        Self {
            experiment_id,
            started_at,
            current_step: 0,
            observations: Vec::new(),
            parameter_changes: Vec::new(),
            completed: false,
            timing_results: Vec::new(),
        }
    }

    /// Record an observation
    pub fn record_observation(&mut self, step: u8, elapsed_ms: u64, description: String) {
        self.observations.push(Observation {
            step,
            elapsed_ms,
            description,
            metrics: None,
        });
    }

    /// Record an observation with metrics from the nervous system
    pub fn record_observation_with_metrics(
        &mut self,
        step: u8,
        elapsed_ms: u64,
        description: String,
        metrics: ObservationMetrics,
    ) {
        self.observations.push(Observation {
            step,
            elapsed_ms,
            description,
            metrics: Some(metrics),
        });
    }

    /// Record a parameter change
    pub fn record_parameter_change(
        &mut self,
        parameter_name: String,
        old_value: f32,
        new_value: f32,
        elapsed_ms: u64,
    ) {
        self.parameter_changes.push(ParameterChange {
            parameter_name,
            old_value,
            new_value,
            elapsed_ms,
        });
    }

    /// Record timing results
    pub fn record_timing_result(&mut self, timing: TimingResult) {
        self.timing_results.push(timing);
    }

    /// Move to next step
    pub fn advance_step(&mut self) {
        self.current_step += 1;
    }

    /// Mark session as complete
    pub fn mark_complete(&mut self) {
        self.completed = true;
    }

    /// Export observations to CSV string
    ///
    /// Format: timestamp_ms, step, description, tension_before, tension_after, response_time_ms
    pub fn export_observations_csv(&self) -> String {
        let mut csv = String::from("timestamp_ms,step,description,tension_before,tension_after,response_time_ms\n");

        for obs in &self.observations {
            csv.push_str(&format!(
                "{},{},\"{}\",",
                obs.elapsed_ms, obs.step, obs.description
            ));

            if let Some(metrics) = &obs.metrics {
                csv.push_str(&format!(
                    "{},{},{}",
                    metrics.tension_before, metrics.tension_after, metrics.response_time_ms
                ));
            } else {
                csv.push_str(",,");
            }
            csv.push('\n');
        }

        csv
    }

    /// Export parameter changes to CSV string
    ///
    /// Format: timestamp_ms, parameter_name, old_value, new_value
    pub fn export_parameters_csv(&self) -> String {
        let mut csv = String::from("timestamp_ms,parameter_name,old_value,new_value\n");

        for change in &self.parameter_changes {
            csv.push_str(&format!(
                "{},{},{},{}\n",
                change.elapsed_ms, change.parameter_name, change.old_value, change.new_value
            ));
        }

        csv
    }
}

/// Reflex Lab Experiments - Main API
///
/// Provides all available experiments and manages experiment sessions.
/// This is the entry point for students to start learning about reflexes.
pub struct ReflexLabExperiments;

impl ReflexLabExperiments {
    /// Get the Startle Response experiment
    ///
    /// Students learn: "How sensitive is the robot to sudden changes?"
    /// - Adjust sensitivity from low to high
    /// - Observe how the robot reacts
    /// - Understand cause and effect
    pub fn startle_response() -> Experiment {
        Experiment::new(
            "startle-001".to_string(),
            "Startle Response".to_string(),
            "Learn how sensitive the robot is to sudden changes! When you clap or make a loud sound, the robot might jump. Let's see how it reacts at different sensitivity levels.".to_string(),
            8,
            DifficultyLevel::Beginner,
        )
        .add_learning_objective("Understand how robots sense sudden changes".to_string())
        .add_learning_objective("Learn about sensitivity and response thresholds".to_string())
        .add_learning_objective("See cause and effect in action".to_string())
        // Step 1: Baseline measurement
        .add_step(
            ExperimentStep::new(
                1,
                "First, let's measure how the robot reacts normally. Set the sensitivity to the MIDDLE. This is the normal setting.".to_string(),
                StepActionType::Adjust,
            )
            .with_parameter("startle_sensitivity".to_string())
            .with_expected_outcome("The sensitivity slider is at 50%".to_string())
            .with_hint("Look for the slider that says 'Sensitivity' and drag it to the middle.".to_string())
        )
        // Step 2: Make a stimulus
        .add_step(
            ExperimentStep::new(
                2,
                "Now make a sudden sound - you can clap, snap your fingers, or shout 'HEY!' Watch what happens to the robot!".to_string(),
                StepActionType::Stimulate,
            )
            .with_expected_outcome("The robot makes a movement and the 'nervousness' gauge goes up.".to_string())
            .with_hint("Make the sound or movement sudden - not slow! The robot should notice it.".to_string())
        )
        // Step 3: Record observation
        .add_step(
            ExperimentStep::new(
                3,
                "What did you see? Did the robot jump? Look at the nervousness gauge - how high did it go?".to_string(),
                StepActionType::Record,
            )
            .with_expected_outcome("You write down what you observed.".to_string())
            .with_hint("Write something like: 'The robot jumped and the nervousness went up to 70%'".to_string())
        )
        // Step 4: Change to low sensitivity
        .add_step(
            ExperimentStep::new(
                4,
                "Now let's make the robot LESS sensitive. Move the sensitivity slider ALL THE WAY DOWN to the minimum.".to_string(),
                StepActionType::Adjust,
            )
            .with_parameter("startle_sensitivity".to_string())
            .with_expected_outcome("The sensitivity slider is all the way at 0%".to_string())
            .with_hint("Drag the slider to the very left. It should say 'Low' or '0%'.".to_string())
        )
        // Step 5: Test at low sensitivity
        .add_step(
            ExperimentStep::new(
                5,
                "Make the same sound again - clap, snap, or shout. Do you think the robot will react as much?".to_string(),
                StepActionType::Stimulate,
            )
            .with_expected_outcome("The robot barely reacts, and the nervousness gauge stays low.".to_string())
            .with_hint("Remember to make it sudden! The robot is just not as sensitive now.".to_string())
        )
        // Step 6: Record low sensitivity observation
        .add_step(
            ExperimentStep::new(
                6,
                "Compare! Did the robot react LESS this time? Look at the nervousness gauge - is it lower than before?".to_string(),
                StepActionType::Record,
            )
            .with_expected_outcome("You notice that at low sensitivity, the robot barely reacted.".to_string())
            .with_hint("Write something like: 'At low sensitivity, the robot only went up to 20%'".to_string())
        )
        // Step 7: Change to high sensitivity
        .add_step(
            ExperimentStep::new(
                7,
                "Now let's make the robot SUPER sensitive! Move the sensitivity slider ALL THE WAY UP to the maximum.".to_string(),
                StepActionType::Adjust,
            )
            .with_parameter("startle_sensitivity".to_string())
            .with_expected_outcome("The sensitivity slider is all the way at 100%".to_string())
            .with_hint("Drag the slider to the very right. It should say 'High' or '100%'.".to_string())
        )
        // Step 8: Test at high sensitivity
        .add_step(
            ExperimentStep::new(
                8,
                "Make the same sound one more time. Now that the robot is SUPER sensitive, what do you think will happen?".to_string(),
                StepActionType::Stimulate,
            )
            .with_expected_outcome("The robot reacts MUCH MORE, and the nervousness gauge shoots way up!".to_string())
            .with_hint("The robot is now very sensitive - even a small sound might make it jump!".to_string())
        )
        .add_adjustable_parameter("startle_sensitivity".to_string())
        .add_observation("At low sensitivity, the robot barely notices sudden sounds.".to_string())
        .add_observation("At medium sensitivity, the robot reacts normally.".to_string())
        .add_observation("At high sensitivity, the robot overreacts to every sound.".to_string())
    }

    /// Get the Approach/Avoid experiment
    ///
    /// Students learn: "What makes the robot curious vs. scared?"
    /// - Adjust curiosity and fear thresholds
    /// - Present the same stimulus
    /// - Observe different reactions
    pub fn approach_avoid() -> Experiment {
        Experiment::new(
            "approach-avoid-001".to_string(),
            "Approach / Avoid".to_string(),
            "Learn what makes the robot curious and what makes it scared! We'll use the same object but change the robot's personality settings. Sometimes it will want to approach (get closer), and sometimes it will want to avoid (move away).".to_string(),
            10,
            DifficultyLevel::Intermediate,
        )
        .add_learning_objective("Understand how curiosity vs. fear affects behavior".to_string())
        .add_learning_objective("Learn about personality parameters and thresholds".to_string())
        .add_learning_objective("See how different settings create different personalities".to_string())
        // Step 1: Set curious mode
        .add_step(
            ExperimentStep::new(
                1,
                "First, let's make the robot VERY curious! Turn UP the 'curiosity' setting and turn DOWN the 'fear' setting.".to_string(),
                StepActionType::Adjust,
            )
            .with_parameter("curiosity_threshold".to_string())
            .with_expected_outcome("Curiosity is high (80%+) and fear is low (20% or less)".to_string())
            .with_hint("The robot should be in 'explorer mode' - ready to investigate anything!".to_string())
        )
        // Step 2: Present stimulus to curious robot
        .add_step(
            ExperimentStep::new(
                2,
                "Now hold a hand or toy in front of the robot. What does it do - does it move toward it or away from it?".to_string(),
                StepActionType::Stimulate,
            )
            .with_expected_outcome("The robot moves TOWARD the stimulus (approaches)!".to_string())
            .with_hint("Watch the robot's wheels - which way are they turning?".to_string())
        )
        // Step 3: Record curious behavior
        .add_step(
            ExperimentStep::new(
                3,
                "Write down what the curious robot did. Did it come toward you?".to_string(),
                StepActionType::Record,
            )
            .with_expected_outcome("You observe and record that the robot approached.".to_string())
            .with_hint("Example: 'The robot moved toward my hand because it was curious'".to_string())
        )
        // Step 4: Change to cautious/fearful mode
        .add_step(
            ExperimentStep::new(
                4,
                "Now let's make the robot VERY cautious! Turn DOWN the 'curiosity' setting and turn UP the 'fear' setting.".to_string(),
                StepActionType::Adjust,
            )
            .with_parameter("fear_threshold".to_string())
            .with_expected_outcome("Curiosity is low (20% or less) and fear is high (80%+)".to_string())
            .with_hint("The robot should be in 'protection mode' - scared of anything unfamiliar!".to_string())
        )
        // Step 5: Present the same stimulus to fearful robot
        .add_step(
            ExperimentStep::new(
                5,
                "Present the same hand or toy again. What does the robot do NOW - is it different?".to_string(),
                StepActionType::Stimulate,
            )
            .with_expected_outcome("The robot moves AWAY from the stimulus (avoids)!".to_string())
            .with_hint("Watch carefully - the robot should back up or turn away.".to_string())
        )
        // Step 6: Record fearful behavior
        .add_step(
            ExperimentStep::new(
                6,
                "The robot just saw the SAME stimulus, but acted completely differently! Write down what you saw.".to_string(),
                StepActionType::Record,
            )
            .with_expected_outcome("You observe that the same stimulus caused opposite behavior.".to_string())
            .with_hint("Example: 'The curious robot approached, but the scared robot backed away'".to_string())
        )
        // Step 7: Try balanced mode
        .add_step(
            ExperimentStep::new(
                7,
                "Let's try a middle setting - equal curiosity and fear. Set both to 50%.".to_string(),
                StepActionType::Adjust,
            )
            .with_parameter("curiosity_threshold".to_string())
            .with_expected_outcome("Both curiosity and fear are at 50% - balanced!".to_string())
            .with_hint("This is like the robot being 'careful but interested'.".to_string())
        )
        // Step 8: Observe balanced behavior
        .add_step(
            ExperimentStep::new(
                8,
                "Present the stimulus to this balanced robot. What does it do - approach or avoid?".to_string(),
                StepActionType::Stimulate,
            )
            .with_expected_outcome("The robot might hesitate or move slowly - it's unsure!".to_string())
            .with_hint("Watch for slow, cautious movements - the robot is making a decision!".to_string())
        )
        .add_adjustable_parameter("curiosity_threshold".to_string())
        .add_adjustable_parameter("fear_threshold".to_string())
        .add_observation("Curious robot = approaches new things".to_string())
        .add_observation("Fearful robot = avoids new things".to_string())
        .add_observation("Balanced robot = carefully investigates".to_string())
    }

    /// Get the Recovery Time experiment
    ///
    /// Students learn: "How long does it take the robot to calm down?"
    /// - Startle the robot
    /// - Watch tension spike
    /// - Observe tension gradually decrease
    /// - Measure recovery time
    pub fn recovery_time() -> Experiment {
        Experiment::new(
            "recovery-001".to_string(),
            "Recovery Time".to_string(),
            "After the robot gets scared or excited, how long does it take to calm down? Let's measure! We'll startle the robot and then watch as it relaxes.".to_string(),
            7,
            DifficultyLevel::Intermediate,
        )
        .add_learning_objective("Understand how the robot calms down over time".to_string())
        .add_learning_objective("Learn to measure reaction duration".to_string())
        .add_learning_objective("See how tension changes after stimulation".to_string())
        // Step 1: Baseline measurement
        .add_step(
            ExperimentStep::new(
                1,
                "First, let's see the robot calm and relaxed. Don't touch or make sounds. Watch the nervousness gauge - it should be LOW and steady.".to_string(),
                StepActionType::Observe,
            )
            .with_expected_outcome("The nervousness gauge shows a low number (maybe 10-20%).".to_string())
            .with_hint("This is the robot's 'baseline' or starting point. Remember this number!".to_string())
        )
        // Step 2: Startle the robot
        .add_step(
            ExperimentStep::new(
                2,
                "NOW - make a sudden sound or movement! Clap, snap, or shout! Watch what happens to the nervousness gauge - it should JUMP UP immediately!".to_string(),
                StepActionType::Stimulate,
            )
            .with_expected_outcome("The nervousness gauge shoots up quickly - to maybe 80-90%!".to_string())
            .with_hint("Make it sudden and unexpected! The gauge should jump right away.".to_string())
        )
        // Step 3: Start observing recovery
        .add_step(
            ExperimentStep::new(
                3,
                "STOP making sounds! Just watch. The nervousness gauge should start going DOWN. How long does it take?".to_string(),
                StepActionType::Observe,
            )
            .with_expected_outcome("The gauge slowly decreases back toward the baseline.".to_string())
            .with_hint("Count the seconds or watch a timer. How long until it gets back to normal?".to_string())
        )
        // Step 4: Record recovery time
        .add_step(
            ExperimentStep::new(
                4,
                "Write down how long recovery took. Start from when you made the sound and end when the gauge got back to baseline.".to_string(),
                StepActionType::Record,
            )
            .with_expected_outcome("You have a recovery time measurement (maybe 5-10 seconds).".to_string())
            .with_hint("Example: 'The robot took 7 seconds to calm down'".to_string())
        )
        // Step 5: Try again with different stimulus
        .add_step(
            ExperimentStep::new(
                5,
                "Let's test again! Make sure the nervousness gauge is back to baseline (calm), then try a DIFFERENT sound - maybe whistle or tap.".to_string(),
                StepActionType::Stimulate,
            )
            .with_expected_outcome("Same spike and recovery, but you can compare different stimuli.".to_string())
            .with_hint("Does a louder sound take longer to recover from? Does a softer sound recover faster?".to_string())
        )
        // Step 6: Measure second recovery time
        .add_step(
            ExperimentStep::new(
                6,
                "Measure the recovery time again. Is it the same as before or different?".to_string(),
                StepActionType::Record,
            )
            .with_expected_outcome("You have a second recovery time measurement to compare.".to_string())
            .with_hint("If it's very different, try to figure out why!".to_string())
        )
        // Step 7: Conclude
        .add_step(
            ExperimentStep::new(
                7,
                "Great work! You've discovered how the robot's nervous system recovers. Just like you feel better after being scared, the robot calms down too!".to_string(),
                StepActionType::Observe,
            )
            .with_expected_outcome("You understand how recovery time works.".to_string())
            .with_hint("This is how real nervous systems work - they take time to calm down!".to_string())
        )
        .add_adjustable_parameter("recovery_acceleration".to_string())
        .add_observation("The robot's nervousness spikes immediately when startled.".to_string())
        .add_observation("The nervousness slowly decreases over several seconds.".to_string())
        .add_observation("Different stimuli might cause different recovery times.".to_string())
    }

    /// Get the Mode Transitions experiment
    ///
    /// Students learn: "What triggers each mode? How does the robot switch between calm, active, spike, and protect?"
    /// - See all four modes on a diagram
    /// - Trigger each mode with different stimuli
    /// - Understand mode transitions
    pub fn mode_transitions() -> Experiment {
        Experiment::new(
            "modes-001".to_string(),
            "Mode Transitions".to_string(),
            "The robot has four moods or 'modes': Calm (relaxed), Active (alert), Spike (startled), and Protect (scared). Let's see what triggers each mode and how the robot switches between them!".to_string(),
            12,
            DifficultyLevel::Advanced,
        )
        .add_learning_objective("Understand the robot's four personality modes".to_string())
        .add_learning_objective("Learn what triggers each mode".to_string())
        .add_learning_objective("See how stimulus intensity affects mode transitions".to_string())
        // Step 1: Explain the modes
        .add_step(
            ExperimentStep::new(
                1,
                "Look at the Mode Diagram. It shows four moods: CALM (green, relaxed), ACTIVE (blue, alert), SPIKE (yellow, startled), and PROTECT (red, scared). The dots show what triggers each mode.".to_string(),
                StepActionType::Observe,
            )
            .with_expected_outcome("You understand the four modes and what triggers them.".to_string())
            .with_hint("Think: Calm = boring, Active = interesting, Spike = sudden!, Protect = danger!".to_string())
        )
        // Step 2: Start in calm mode
        .add_step(
            ExperimentStep::new(
                2,
                "Make sure the robot is in CALM mode. The mode indicator should show green. No sounds, no movement - just let it relax.".to_string(),
                StepActionType::Observe,
            )
            .with_expected_outcome("The mode indicator is green (CALM).".to_string())
            .with_hint("The robot needs to settle down first. Wait a few seconds if needed.".to_string())
        )
        // Step 3: Trigger active mode
        .add_step(
            ExperimentStep::new(
                3,
                "Now make a GENTLE, INTERESTING stimulus. Hold your hand in front slowly, or make a soft sound. Watch - the mode should change to ACTIVE (blue)!".to_string(),
                StepActionType::Stimulate,
            )
            .with_expected_outcome("The mode indicator changes to blue (ACTIVE)!".to_string())
            .with_hint("Be gentle - you want 'interesting' not 'scary'. The robot should be curious, not afraid.".to_string())
        )
        // Step 4: Observe active behavior
        .add_step(
            ExperimentStep::new(
                4,
                "In ACTIVE mode, the robot might move toward you, light up, or make sounds. Write down what the ACTIVE robot does.".to_string(),
                StepActionType::Record,
            )
            .with_expected_outcome("You observe what the robot does in ACTIVE mode.".to_string())
            .with_hint("Example: 'In ACTIVE mode, the robot moved toward me and flashed its LED'".to_string())
        )
        // Step 5: Let it calm down
        .add_step(
            ExperimentStep::new(
                5,
                "Stop the stimulus and wait. The mode should return to CALM (green). Let the robot relax for a few seconds.".to_string(),
                StepActionType::Observe,
            )
            .with_expected_outcome("The mode indicator returns to green (CALM).".to_string())
            .with_hint("Be patient - give the robot time to relax between tests!".to_string())
        )
        // Step 6: Trigger spike mode
        .add_step(
            ExperimentStep::new(
                6,
                "Now make a SUDDEN stimulus - clap, snap, or quick movement! This should trigger SPIKE mode (yellow). Watch the mode change!".to_string(),
                StepActionType::Stimulate,
            )
            .with_expected_outcome("The mode indicator changes to yellow (SPIKE)!".to_string())
            .with_hint("Make it sudden and unexpected! This is like jumping from a surprise.".to_string())
        )
        // Step 7: Observe spike behavior
        .add_step(
            ExperimentStep::new(
                7,
                "In SPIKE mode, the robot jumps or makes a defensive movement. Write down what happens!".to_string(),
                StepActionType::Record,
            )
            .with_expected_outcome("You see the dramatic reaction of SPIKE mode.".to_string())
            .with_hint("Example: 'In SPIKE mode, the robot jumped and backed away!'".to_string())
        )
        // Step 8: Let it calm down again
        .add_step(
            ExperimentStep::new(
                8,
                "Stop and wait for the robot to calm down. The mode should go from SPIKE back to ACTIVE, then to CALM.".to_string(),
                StepActionType::Observe,
            )
            .with_expected_outcome("The mode gradually returns to green (CALM).".to_string())
            .with_hint("Notice how it recovers step-by-step? That's the nervous system working!".to_string())
        )
        // Step 9: Trigger protect mode (optional advanced)
        .add_step(
            ExperimentStep::new(
                9,
                "ADVANCED: Try to trigger PROTECT mode (red) by doing multiple sudden stimuli in a row. The robot should get very scared!".to_string(),
                StepActionType::Stimulate,
            )
            .with_expected_outcome("The mode might change to red (PROTECT) if very threatened.".to_string())
            .with_hint("This is like the robot feeling cornered. Multiple threats make it extra defensive.".to_string())
        )
        // Step 10: Observe protect behavior
        .add_step(
            ExperimentStep::new(
                10,
                "In PROTECT mode, the robot might make defensive sounds or movements. Describe what you see!".to_string(),
                StepActionType::Record,
            )
            .with_expected_outcome("You understand the extreme defensive reaction.".to_string())
            .with_hint("Example: 'In PROTECT mode, the robot made loud beeps and spun in circles!'".to_string())
        )
        // Step 11: Let it recover completely
        .add_step(
            ExperimentStep::new(
                11,
                "Stop all stimuli and let the robot fully relax. It might take longer to recover from PROTECT mode!".to_string(),
                StepActionType::Observe,
            )
            .with_expected_outcome("The mode gradually returns to CALM.".to_string())
            .with_hint("Recovery from fear takes time - just like with real creatures!".to_string())
        )
        // Step 12: Conclude with insights
        .add_step(
            ExperimentStep::new(
                12,
                "You've now triggered all four modes! The robot has a complete nervous system - just like animals do. Mild stimuli = curiosity, sudden stimuli = startled, repeated threats = protective.".to_string(),
                StepActionType::Record,
            )
            .with_expected_outcome("You understand how the mode system works.".to_string())
            .with_hint("The same nervous system patterns exist in real animals too!".to_string())
        )
        .add_adjustable_parameter("active_threshold".to_string())
        .add_adjustable_parameter("spike_threshold".to_string())
        .add_adjustable_parameter("protect_threshold".to_string())
        .add_observation("CALM mode: No stimulus, robot is relaxed".to_string())
        .add_observation("ACTIVE mode: Gentle stimulus triggers curiosity and investigation".to_string())
        .add_observation("SPIKE mode: Sudden stimulus triggers startled response".to_string())
        .add_observation("PROTECT mode: Repeated/intense stimulus triggers defensive reaction".to_string())
    }

    /// Get all available experiments
    pub fn all_experiments() -> Vec<Experiment> {
        vec![
            Self::startle_response(),
            Self::approach_avoid(),
            Self::recovery_time(),
            Self::mode_transitions(),
        ]
    }

    /// Get an experiment by ID
    pub fn get_experiment(id: &str) -> Option<Experiment> {
        Self::all_experiments().into_iter().find(|e| e.id == id)
    }

    /// Create a new experiment session
    pub fn start_session(experiment_id: String, started_at: u64) -> ExperimentSession {
        ExperimentSession::new(experiment_id, started_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startle_response_structure() {
        let exp = ReflexLabExperiments::startle_response();
        assert_eq!(exp.id, "startle-001");
        assert_eq!(exp.steps.len(), 8);
        assert!(!exp.learning_objectives.is_empty());
        assert!(!exp.parameters_to_adjust.is_empty());
    }

    #[test]
    fn test_approach_avoid_structure() {
        let exp = ReflexLabExperiments::approach_avoid();
        assert_eq!(exp.id, "approach-avoid-001");
        assert_eq!(exp.steps.len(), 8);
        assert!(exp.difficulty == DifficultyLevel::Intermediate);
    }

    #[test]
    fn test_recovery_time_structure() {
        let exp = ReflexLabExperiments::recovery_time();
        assert_eq!(exp.id, "recovery-001");
        assert_eq!(exp.steps.len(), 7);
    }

    #[test]
    fn test_mode_transitions_structure() {
        let exp = ReflexLabExperiments::mode_transitions();
        assert_eq!(exp.id, "modes-001");
        assert_eq!(exp.steps.len(), 12);
        assert!(exp.difficulty == DifficultyLevel::Advanced);
    }

    #[test]
    fn test_experiment_progress_tracking() {
        let mut exp = ReflexLabExperiments::startle_response();
        assert_eq!(exp.progress_percent(), 0);
        assert!(!exp.is_complete());

        // Mark first step as complete
        if let Some(step) = exp.steps.get_mut(0) {
            step.mark_completed();
        }
        assert_eq!(exp.progress_percent(), 12); // 1 of 8 steps

        // Mark all steps as complete
        for step in &mut exp.steps {
            step.mark_completed();
        }
        assert_eq!(exp.progress_percent(), 100);
        assert!(exp.is_complete());
    }

    #[test]
    fn test_experiment_session_tracking() {
        let mut session = ExperimentSession::new("startle-001".to_string(), 1000);
        assert_eq!(session.observations.len(), 0);
        assert!(!session.completed);

        // Record observations
        session.record_observation(1, 100, "Baseline measured".to_string());
        assert_eq!(session.observations.len(), 1);

        // Record parameter changes
        session.record_parameter_change(
            "startle_sensitivity".to_string(),
            0.5,
            0.8,
            150,
        );
        assert_eq!(session.parameter_changes.len(), 1);

        // Mark complete
        session.mark_complete();
        assert!(session.completed);
    }

    #[test]
    fn test_observation_metrics_tracking() {
        let mut session = ExperimentSession::new("recovery-001".to_string(), 1000);

        let metrics = ObservationMetrics {
            tension_before: 0.2,
            tension_after: 0.8,
            response_time_ms: 250,
        };

        session.record_observation_with_metrics(
            1,
            500,
            "Strong startle response!".to_string(),
            metrics,
        );

        assert_eq!(session.observations.len(), 1);
        let obs = &session.observations[0];
        assert!(obs.metrics.is_some());
        if let Some(m) = &obs.metrics {
            assert_eq!(m.tension_before, 0.2);
            assert_eq!(m.tension_after, 0.8);
            assert_eq!(m.response_time_ms, 250);
        }
    }

    #[test]
    fn test_csv_export() {
        let mut session = ExperimentSession::new("startle-001".to_string(), 1000);

        session.record_observation(1, 100, "Baseline".to_string());
        session.record_parameter_change("sensitivity".to_string(), 0.5, 0.8, 200);

        let obs_csv = session.export_observations_csv();
        assert!(obs_csv.contains("timestamp_ms,step,description"));
        assert!(obs_csv.contains("100,1,\"Baseline\""));

        let param_csv = session.export_parameters_csv();
        assert!(param_csv.contains("timestamp_ms,parameter_name,old_value,new_value"));
        assert!(param_csv.contains("200,sensitivity,0.5,0.8"));
    }

    #[test]
    fn test_experiment_step_builder() {
        let step = ExperimentStep::new(1, "Test instruction".to_string(), StepActionType::Observe)
            .with_parameter("test_param".to_string())
            .with_expected_outcome("Expected result".to_string())
            .with_hint("This is a hint".to_string());

        assert_eq!(step.parameter, Some("test_param".to_string()));
        assert_eq!(step.expected_outcome, Some("Expected result".to_string()));
        assert_eq!(step.hint, Some("This is a hint".to_string()));
    }

    #[test]
    fn test_all_experiments_available() {
        let experiments = ReflexLabExperiments::all_experiments();
        assert_eq!(experiments.len(), 4);

        let ids: Vec<_> = experiments.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"startle-001"));
        assert!(ids.contains(&"approach-avoid-001"));
        assert!(ids.contains(&"recovery-001"));
        assert!(ids.contains(&"modes-001"));
    }

    #[test]
    fn test_get_experiment_by_id() {
        let exp = ReflexLabExperiments::get_experiment("startle-001");
        assert!(exp.is_some());
        assert_eq!(exp.unwrap().name, "Startle Response");

        let missing = ReflexLabExperiments::get_experiment("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_serialization() {
        let exp = ReflexLabExperiments::startle_response();
        let json = serde_json::to_string(&exp);
        assert!(json.is_ok());

        let session = ExperimentSession::new("startle-001".to_string(), 1000);
        let json = serde_json::to_string(&session);
        assert!(json.is_ok());
    }

    #[test]
    fn test_age_appropriate_language_check() {
        let experiments = ReflexLabExperiments::all_experiments();

        // Verify no forbidden technical terms
        let forbidden = ["algorithm", "coefficient", "derivative", "integral", "tensor"];

        for exp in &experiments {
            let text = format!("{} {}", exp.name, exp.description);
            for word in &forbidden {
                assert!(
                    !text.to_lowercase().contains(word),
                    "Found forbidden term '{}' in experiment: {}",
                    word,
                    exp.id
                );
            }
        }
    }

    #[test]
    fn test_learning_objectives_present() {
        let experiments = ReflexLabExperiments::all_experiments();
        for exp in experiments {
            assert!(
                !exp.learning_objectives.is_empty(),
                "Experiment {} missing learning objectives",
                exp.id
            );
        }
    }

    #[test]
    fn test_hints_available() {
        let exp = ReflexLabExperiments::startle_response();
        let has_hints = exp.steps.iter().any(|s| s.hint.is_some());
        assert!(
            has_hints,
            "Experiment should have hints for stuck students (I-LEARN-023)"
        );
    }
}
