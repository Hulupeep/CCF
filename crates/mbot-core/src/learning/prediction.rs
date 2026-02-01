//! Predictive Behavior Engine
//!
//! Anticipates user actions and adapts robot behavior proactively based on patterns.
//!
//! Issue: #87
//! Contract: feature_ai_learning.yml
//! Invariants: I-AI-005, I-AI-006, I-AI-007
//!
//! Features:
//! - Temporal pattern detection (time-based activities)
//! - Sequence pattern detection (activity chains)
//! - Preference pattern detection (usage frequency)
//! - Confidence scoring for predictions
//! - Proactive suggestions (>70% confidence)
//! - Gradual behavior adaptation
//! - User override mechanism
//! - Privacy controls

#[cfg(feature = "no_std")]
extern crate alloc;

#[cfg(feature = "no_std")]
use alloc::{vec::Vec, string::String, collections::VecDeque, format};

#[cfg(not(feature = "no_std"))]
use std::{vec::Vec, string::String, collections::VecDeque, format};

use core::cmp::Ordering;

/// Type of user activity
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityType {
    ModeChange,
    GameStart,
    DrawingStart,
    PersonalityChange,
    Other,
}

impl ActivityType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "mode_change" => ActivityType::ModeChange,
            "game_start" => ActivityType::GameStart,
            "drawing_start" => ActivityType::DrawingStart,
            "personality_change" => ActivityType::PersonalityChange,
            _ => ActivityType::Other,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ActivityType::ModeChange => "mode_change",
            ActivityType::GameStart => "game_start",
            ActivityType::DrawingStart => "drawing_start",
            ActivityType::PersonalityChange => "personality_change",
            ActivityType::Other => "other",
        }
    }
}

/// User activity record
#[derive(Debug, Clone)]
pub struct UserActivity {
    pub id: String,
    pub activity_type: ActivityType,
    pub details: String,
    pub timestamp: u64,      // Unix timestamp in seconds
    pub day_of_week: u8,     // 0-6 (Sunday=0)
    pub hour_of_day: u8,     // 0-23
    pub duration: u32,       // seconds
}

impl UserActivity {
    pub fn new(
        id: String,
        activity_type: ActivityType,
        details: String,
        timestamp: u64,
    ) -> Self {
        // Calculate day of week and hour from timestamp
        let seconds_per_day = 86400_u64;
        let seconds_per_hour = 3600_u64;

        // Unix epoch started on Thursday (Jan 1, 1970 = Thursday)
        let days_since_epoch = timestamp / seconds_per_day;
        let day_of_week = ((days_since_epoch + 4) % 7) as u8; // +4 because epoch was Thursday

        let seconds_today = timestamp % seconds_per_day;
        let hour_of_day = (seconds_today / seconds_per_hour) as u8;

        Self {
            id,
            activity_type,
            details,
            timestamp,
            day_of_week,
            hour_of_day,
            duration: 0,
        }
    }

    pub fn with_duration(mut self, duration: u32) -> Self {
        self.duration = duration;
        self
    }
}

/// Context for making predictions
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub current_mode: String,
    pub current_personality: String,
    pub time_of_day: u8,              // 0-23
    pub day_of_week: u8,              // 0-6
    pub last_activities: Vec<String>, // Last 5 activities
}

impl Context {
    pub fn new(mode: String, personality: String, hour: u8, day: u8) -> Self {
        Self {
            current_mode: mode,
            current_personality: personality,
            time_of_day: hour,
            day_of_week: day,
            last_activities: Vec::new(),
        }
    }

    pub fn with_last_activities(mut self, activities: Vec<String>) -> Self {
        // I-AI-003: Limit history - only keep last 5
        self.last_activities = activities.into_iter().rev().take(5).rev().collect();
        self
    }
}

/// Type of pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternType {
    Temporal,    // Time-based patterns
    Sequence,    // Activity sequence patterns
    Preference,  // Usage frequency patterns
}

/// Detected pattern
#[derive(Debug, Clone)]
pub struct Pattern {
    pub id: String,
    pub pattern_type: PatternType,
    pub description: String,
    pub confidence: f32,         // I-AI-004: 0.0-1.0
    pub observations: u32,       // I-AI-006: Number of times observed

    // Trigger conditions
    pub time_window: Option<(u8, u8)>,  // (start_hour, end_hour)
    pub days_of_week: Option<Vec<u8>>,  // Days when pattern applies
    pub preceding_activity: Option<String>,

    // Prediction
    pub predicted_activity: String,
    pub probability: f32,        // 0.0-1.0

    pub created_at: u64,
    pub last_observed: u64,
}

impl Pattern {
    /// Create new temporal pattern
    pub fn temporal(
        id: String,
        description: String,
        time_window: (u8, u8),
        predicted_activity: String,
        observations: u32,
        probability: f32,
        timestamp: u64,
    ) -> Self {
        Self {
            id,
            pattern_type: PatternType::Temporal,
            description,
            confidence: probability.clamp(0.0, 1.0), // I-AI-001: Bounded
            observations,
            time_window: Some(time_window),
            days_of_week: None,
            preceding_activity: None,
            predicted_activity,
            probability: probability.clamp(0.0, 1.0),
            created_at: timestamp,
            last_observed: timestamp,
        }
    }

    /// Create new sequence pattern
    pub fn sequence(
        id: String,
        description: String,
        preceding_activity: String,
        predicted_activity: String,
        observations: u32,
        probability: f32,
        timestamp: u64,
    ) -> Self {
        Self {
            id,
            pattern_type: PatternType::Sequence,
            description,
            confidence: probability.clamp(0.0, 1.0),
            observations,
            time_window: None,
            days_of_week: None,
            preceding_activity: Some(preceding_activity),
            predicted_activity,
            probability: probability.clamp(0.0, 1.0),
            created_at: timestamp,
            last_observed: timestamp,
        }
    }

    /// Create new preference pattern
    pub fn preference(
        id: String,
        description: String,
        predicted_activity: String,
        observations: u32,
        probability: f32,
        timestamp: u64,
    ) -> Self {
        Self {
            id,
            pattern_type: PatternType::Preference,
            description,
            confidence: probability.clamp(0.0, 1.0),
            observations,
            time_window: None,
            days_of_week: None,
            preceding_activity: None,
            predicted_activity,
            probability: probability.clamp(0.0, 1.0),
            created_at: timestamp,
            last_observed: timestamp,
        }
    }
}

/// Prediction result
#[derive(Debug, Clone)]
pub struct Prediction {
    pub pattern_id: String,
    pub confidence: f32,             // I-AI-005: Must be >= 0.7 for proactive actions
    pub predicted_activity: String,
    pub reasoning: String,           // I-AI-004: Human-readable explanation
    pub suggested_action: Option<Action>,
    pub timestamp: u64,
}

/// Suggested action based on prediction
#[derive(Debug, Clone)]
pub enum Action {
    ModeChange { target_mode: String },
    PersonalityChange { target_personality: String },
    Prompt { message: String },
}

/// Prediction engine settings
#[derive(Debug, Clone)]
pub struct PredictionSettings {
    pub enabled: bool,              // I-AI-007: User can disable
    pub min_confidence: f32,        // I-AI-005: Default 0.7
    pub min_observations: u32,      // I-AI-006: Default 10
    pub show_suggestions: bool,
    pub auto_adapt: bool,
}

impl Default for PredictionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            min_confidence: 0.7,        // I-AI-005: 70% confidence threshold
            min_observations: 10,       // I-AI-006: Minimum 10 observations
            show_suggestions: true,
            auto_adapt: false,          // Require user confirmation by default
        }
    }
}

/// Main predictive behavior engine
pub struct PredictiveEngine {
    settings: PredictionSettings,

    // I-AI-003: Bounded history storage
    activity_history: VecDeque<UserActivity>,
    max_history_size: usize,

    patterns: Vec<Pattern>,
    max_patterns: usize,

    prediction_history: VecDeque<Prediction>,
    max_prediction_history: usize,
}

impl PredictiveEngine {
    /// Create new prediction engine with bounded storage
    pub fn new() -> Self {
        Self {
            settings: PredictionSettings::default(),
            activity_history: VecDeque::with_capacity(1000),
            max_history_size: 1000,  // I-AI-003: Storage limit
            patterns: Vec::with_capacity(100),
            max_patterns: 100,
            prediction_history: VecDeque::with_capacity(100),
            max_prediction_history: 100,
        }
    }

    /// Create with custom settings
    pub fn with_settings(settings: PredictionSettings) -> Self {
        let mut engine = Self::new();
        engine.settings = settings;
        engine
    }

    /// Record a user activity
    pub fn record_activity(&mut self, activity: UserActivity) {
        // I-AI-003: Bounded storage with LRU eviction
        if self.activity_history.len() >= self.max_history_size {
            self.activity_history.pop_front();
        }
        self.activity_history.push_back(activity);
    }

    /// Detect patterns from activity history
    pub fn detect_patterns(&mut self, current_timestamp: u64) -> Vec<Pattern> {
        // I-AI-007: Check if predictions are enabled
        if !self.settings.enabled {
            return Vec::new();
        }

        let mut new_patterns = Vec::new();

        // Detect temporal patterns
        new_patterns.extend(self.detect_temporal_patterns(current_timestamp));

        // Detect sequence patterns
        new_patterns.extend(self.detect_sequence_patterns(current_timestamp));

        // Detect preference patterns
        new_patterns.extend(self.detect_preference_patterns(current_timestamp));

        // I-AI-006: Filter by minimum observations
        let valid_patterns: Vec<Pattern> = new_patterns
            .into_iter()
            .filter(|p| p.observations >= self.settings.min_observations)
            .collect();

        // I-AI-003: Bounded pattern storage
        for pattern in valid_patterns.iter() {
            if self.patterns.len() >= self.max_patterns {
                // Remove oldest pattern
                if let Some(oldest_idx) = self.patterns
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, p)| p.last_observed)
                    .map(|(idx, _)| idx)
                {
                    self.patterns.remove(oldest_idx);
                }
            }
            self.patterns.push(pattern.clone());
        }

        valid_patterns
    }

    /// Detect temporal patterns (time-based activities)
    fn detect_temporal_patterns(&self, current_timestamp: u64) -> Vec<Pattern> {
        let mut patterns = Vec::new();

        // Group activities by hour of day
        let mut hour_activities: Vec<(u8, Vec<&UserActivity>)> = Vec::new();

        for hour in 0..24 {
            let activities: Vec<&UserActivity> = self.activity_history
                .iter()
                .filter(|a| a.hour_of_day == hour)
                .collect();

            if !activities.is_empty() {
                hour_activities.push((hour, activities));
            }
        }

        // Find patterns with sufficient observations
        for (hour, activities) in hour_activities {
            // Count activity types
            let mut activity_counts: Vec<(String, u32)> = Vec::new();

            for activity in &activities {
                let activity_name = activity.details.clone();
                if let Some(count) = activity_counts.iter_mut().find(|(name, _)| name == &activity_name) {
                    count.1 += 1;
                } else {
                    activity_counts.push((activity_name, 1));
                }
            }

            // Create patterns for activities with high frequency
            for (activity_name, count) in activity_counts {
                if count >= 3 {  // At least 3 observations
                    let total_in_hour = activities.len() as f32;
                    let probability = (count as f32 / total_in_hour).clamp(0.0, 1.0);

                    let pattern = Pattern::temporal(
                        format!("temporal_{}_{}", hour, activity_name),
                        format!("User often does {} at hour {}", activity_name, hour),
                        (hour, hour + 1),
                        activity_name.clone(),
                        count,
                        probability,
                        current_timestamp,
                    );

                    patterns.push(pattern);
                }
            }
        }

        patterns
    }

    /// Detect sequence patterns (activity chains)
    fn detect_sequence_patterns(&self, current_timestamp: u64) -> Vec<Pattern> {
        let mut patterns = Vec::new();

        // Look for activity sequences (A -> B patterns)
        let history: Vec<&UserActivity> = self.activity_history.iter().collect();

        for i in 0..history.len().saturating_sub(1) {
            let current = &history[i];
            let next = &history[i + 1];

            // Count how often this sequence occurs
            let mut sequence_count = 0;
            let mut following_activities = Vec::new();

            for j in 0..history.len().saturating_sub(1) {
                if history[j].details == current.details {
                    following_activities.push(&history[j + 1].details);
                    if history[j + 1].details == next.details {
                        sequence_count += 1;
                    }
                }
            }

            if sequence_count >= 3 && !following_activities.is_empty() {
                let probability = (sequence_count as f32 / following_activities.len() as f32)
                    .clamp(0.0, 1.0);

                let pattern = Pattern::sequence(
                    format!("sequence_{}_{}", current.details, next.details),
                    format!("After {}, user often does {}", current.details, next.details),
                    current.details.clone(),
                    next.details.clone(),
                    sequence_count,
                    probability,
                    current_timestamp,
                );

                patterns.push(pattern);
            }
        }

        patterns
    }

    /// Detect preference patterns (usage frequency)
    fn detect_preference_patterns(&self, current_timestamp: u64) -> Vec<Pattern> {
        let mut patterns = Vec::new();

        // Count total occurrences of each activity
        let mut activity_counts: Vec<(String, u32)> = Vec::new();
        let total_activities = self.activity_history.len() as f32;

        for activity in self.activity_history.iter() {
            let activity_name = activity.details.clone();
            if let Some(count) = activity_counts.iter_mut().find(|(name, _)| name == &activity_name) {
                count.1 += 1;
            } else {
                activity_counts.push((activity_name, 1));
            }
        }

        // Create preference patterns
        for (activity_name, count) in activity_counts {
            if count >= 5 {  // At least 5 observations
                let probability = (count as f32 / total_activities).clamp(0.0, 1.0);

                let pattern = Pattern::preference(
                    format!("preference_{}", activity_name),
                    format!("User prefers {} ({}% of time)", activity_name, (probability * 100.0) as u32),
                    activity_name.clone(),
                    count,
                    probability,
                    current_timestamp,
                );

                patterns.push(pattern);
            }
        }

        patterns
    }

    /// Predict next action based on current context
    pub fn predict_next_action(&mut self, context: &Context) -> Option<Prediction> {
        // I-AI-007: Check if enabled
        if !self.settings.enabled {
            return None;
        }

        // Find matching patterns
        let matching_patterns: Vec<&Pattern> = self.patterns
            .iter()
            .filter(|p| self.pattern_matches_context(p, context))
            .collect();

        // Get highest confidence pattern
        let best_pattern = matching_patterns
            .iter()
            .max_by(|a, b| {
                a.confidence.partial_cmp(&b.confidence)
                    .unwrap_or(Ordering::Equal)
            })?;

        // I-AI-005: Check confidence threshold
        if best_pattern.confidence < self.settings.min_confidence {
            return None;
        }

        // I-AI-004: Create prediction with reasoning
        let reasoning = format!(
            "{} (observed {} times, {:.0}% confidence)",
            best_pattern.description,
            best_pattern.observations,
            best_pattern.confidence * 100.0
        );

        let suggested_action = if self.settings.show_suggestions {
            Some(Action::Prompt {
                message: format!("Time for {}?", best_pattern.predicted_activity),
            })
        } else {
            None
        };

        let prediction = Prediction {
            pattern_id: best_pattern.id.clone(),
            confidence: best_pattern.confidence,
            predicted_activity: best_pattern.predicted_activity.clone(),
            reasoning,
            suggested_action,
            timestamp: context.time_of_day as u64,
        };

        // Record prediction
        if self.prediction_history.len() >= self.max_prediction_history {
            self.prediction_history.pop_front();
        }
        self.prediction_history.push_back(prediction.clone());

        Some(prediction)
    }

    /// Check if a pattern matches the current context
    fn pattern_matches_context(&self, pattern: &Pattern, context: &Context) -> bool {
        match pattern.pattern_type {
            PatternType::Temporal => {
                if let Some((start_hour, end_hour)) = pattern.time_window {
                    context.time_of_day >= start_hour && context.time_of_day < end_hour
                } else {
                    false
                }
            }
            PatternType::Sequence => {
                if let Some(ref preceding) = pattern.preceding_activity {
                    context.last_activities.last()
                        .map(|last| last == preceding)
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            PatternType::Preference => {
                // Preference patterns always match
                true
            }
        }
    }

    /// Get all detected patterns
    pub fn get_patterns(&self) -> &[Pattern] {
        &self.patterns
    }

    /// Get prediction history
    pub fn get_prediction_history(&self) -> &VecDeque<Prediction> {
        &self.prediction_history
    }

    /// Get current settings
    pub fn get_settings(&self) -> &PredictionSettings {
        &self.settings
    }

    /// Update settings
    pub fn update_settings(&mut self, settings: PredictionSettings) {
        self.settings = settings;
    }

    /// Clear all data (I-AI-007: User can clear predictions)
    pub fn clear_all_data(&mut self) {
        self.activity_history.clear();
        self.patterns.clear();
        self.prediction_history.clear();
    }

    /// Get activity history size
    pub fn activity_count(&self) -> usize {
        self.activity_history.len()
    }

    /// Get pattern count
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

impl Default for PredictiveEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_creation() {
        let activity = UserActivity::new(
            "1".into(),
            ActivityType::GameStart,
            "Chase".into(),
            1000000,
        );

        assert_eq!(activity.activity_type, ActivityType::GameStart);
        assert_eq!(activity.details, "Chase");
    }

    #[test]
    fn test_engine_bounded_storage() {
        // I-AI-003: Storage must be bounded
        let mut engine = PredictiveEngine::new();

        // Add more than max_history_size activities
        for i in 0..1500 {
            let activity = UserActivity::new(
                format!("{}", i),
                ActivityType::GameStart,
                "Test".into(),
                i as u64,
            );
            engine.record_activity(activity);
        }

        assert!(engine.activity_count() <= 1000, "History should be bounded to 1000");
    }

    #[test]
    fn test_pattern_confidence_bounded() {
        // I-AI-001: Confidence must be bounded 0-1
        let pattern = Pattern::temporal(
            "test".into(),
            "Test pattern".into(),
            (14, 15),
            "Activity".into(),
            10,
            1.5,  // Invalid value > 1.0
            1000,
        );

        assert!(pattern.confidence <= 1.0, "Confidence should be clamped to 1.0");
        assert!(pattern.confidence >= 0.0, "Confidence should be >= 0.0");
    }

    #[test]
    fn test_min_confidence_threshold() {
        // I-AI-005: Predictions require >= 70% confidence
        let mut engine = PredictiveEngine::new();

        // Add low-confidence pattern
        let pattern = Pattern::temporal(
            "low_conf".into(),
            "Low confidence".into(),
            (10, 11),
            "Activity".into(),
            10,
            0.5,  // Only 50% confidence
            1000,
        );
        engine.patterns.push(pattern);

        let context = Context::new(
            "Test".into(),
            "Calm".into(),
            10,
            1,
        );

        let prediction = engine.predict_next_action(&context);
        assert!(prediction.is_none(), "Low confidence predictions should not trigger actions");
    }

    #[test]
    fn test_min_observations_required() {
        // I-AI-006: Patterns require minimum observations
        let mut engine = PredictiveEngine::new();

        // Add activities
        for i in 0..5 {  // Only 5 observations (< 10 minimum)
            let activity = UserActivity::new(
                format!("{}", i),
                ActivityType::GameStart,
                "Chase".into(),
                1000 + i,
            );
            engine.record_activity(activity);
        }

        let patterns = engine.detect_patterns(2000);

        // Should not create patterns with < 10 observations
        assert!(patterns.is_empty() || patterns.iter().all(|p| p.observations >= 10),
                "Patterns should require minimum 10 observations");
    }

    #[test]
    fn test_user_can_disable_predictions() {
        // I-AI-007: User can disable predictions
        let mut engine = PredictiveEngine::new();
        engine.settings.enabled = false;

        let context = Context::default();
        let prediction = engine.predict_next_action(&context);

        assert!(prediction.is_none(), "Predictions should be disabled");
    }

    #[test]
    fn test_user_can_clear_data() {
        // I-AI-007: User can clear prediction data
        let mut engine = PredictiveEngine::new();

        // Add data
        engine.record_activity(UserActivity::new(
            "1".into(),
            ActivityType::GameStart,
            "Test".into(),
            1000,
        ));

        let pattern = Pattern::temporal(
            "test".into(),
            "Test".into(),
            (10, 11),
            "Activity".into(),
            10,
            0.8,
            1000,
        );
        engine.patterns.push(pattern);

        // Clear all data
        engine.clear_all_data();

        assert_eq!(engine.activity_count(), 0, "Activity history should be cleared");
        assert_eq!(engine.pattern_count(), 0, "Patterns should be cleared");
    }

    #[test]
    fn test_temporal_pattern_detection() {
        let mut engine = PredictiveEngine::new();
        engine.settings.min_observations = 3;  // Lower threshold for testing

        // Add activities at same hour
        for i in 0..5 {
            let timestamp = 1000000 + (i * 3600);  // Same hour of day
            let mut activity = UserActivity::new(
                format!("{}", i),
                ActivityType::GameStart,
                "Chase".into(),
                timestamp,
            );
            activity.hour_of_day = 14;  // 2 PM
            engine.record_activity(activity);
        }

        let patterns = engine.detect_patterns(2000000);

        let temporal_patterns: Vec<&Pattern> = patterns
            .iter()
            .filter(|p| p.pattern_type == PatternType::Temporal)
            .collect();

        assert!(!temporal_patterns.is_empty(), "Should detect temporal pattern");

        if let Some(pattern) = temporal_patterns.first() {
            assert_eq!(pattern.predicted_activity, "Chase");
            assert!(pattern.observations >= 3);
        }
    }

    #[test]
    fn test_sequence_pattern_detection() {
        let mut engine = PredictiveEngine::new();
        engine.settings.min_observations = 3;

        // Add sequence: Drawing -> Game -> Drawing -> Game
        let activities = vec![
            ("Drawing", 1000),
            ("Game", 2000),
            ("Drawing", 3000),
            ("Game", 4000),
            ("Drawing", 5000),
            ("Game", 6000),
        ];

        for (i, (activity, timestamp)) in activities.iter().enumerate() {
            let act = UserActivity::new(
                format!("{}", i),
                ActivityType::GameStart,
                (*activity).into(),
                *timestamp,
            );
            engine.record_activity(act);
        }

        let patterns = engine.detect_patterns(7000);

        let sequence_patterns: Vec<&Pattern> = patterns
            .iter()
            .filter(|p| p.pattern_type == PatternType::Sequence)
            .collect();

        assert!(!sequence_patterns.is_empty(), "Should detect sequence pattern");
    }

    #[test]
    fn test_prediction_with_reasoning() {
        // I-AI-004: Predictions must include reasoning
        let mut engine = PredictiveEngine::new();

        let pattern = Pattern::temporal(
            "test".into(),
            "User plays Chase at 3pm".into(),
            (15, 16),
            "Chase".into(),
            10,
            0.85,
            1000,
        );
        engine.patterns.push(pattern);

        let context = Context::new(
            "Idle".into(),
            "Calm".into(),
            15,  // 3 PM
            1,
        );

        let prediction = engine.predict_next_action(&context);

        assert!(prediction.is_some(), "Should make prediction");

        if let Some(pred) = prediction {
            assert!(!pred.reasoning.is_empty(), "Should include reasoning");
            assert!(pred.reasoning.contains("observed"), "Reasoning should explain observations");
        }
    }
}
