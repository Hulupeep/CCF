# Predictive Behavior Engine

**Issue:** #87
**Status:** Implemented
**Contract:** `docs/contracts/feature_ai_learning.yml`
**Invariants:** I-AI-005, I-AI-006, I-AI-007

## Overview

The Predictive Behavior Engine anticipates user actions and adapts robot behavior proactively based on learned patterns. It works standalone but is designed to integrate with the Reinforcement Learning system (#86) when available.

## Architecture

### Core Components

1. **Pattern Detection** (`PredictiveEngine`)
   - Temporal patterns (time-based activities)
   - Sequence patterns (activity chains)
   - Preference patterns (usage frequency)

2. **Prediction Engine**
   - Confidence scoring (0.0-1.0)
   - Minimum confidence threshold (70% default)
   - Human-readable reasoning

3. **User Controls**
   - Enable/disable predictions
   - Clear all data
   - Privacy settings

## Contract Compliance

### I-AI-005: Minimum Confidence Threshold
```rust
// Default 70% confidence required for proactive actions
if prediction.confidence >= self.settings.min_confidence {
    suggest_action(prediction);
}
```

**Default:** 0.7 (70%)
**Rationale:** Low-confidence predictions should not trigger actions

### I-AI-006: Minimum Observations
```rust
// Patterns require minimum 10 observations before use
if pattern.observations >= self.settings.min_observations {
    patterns.push(pattern);
}
```

**Default:** 10 observations
**Rationale:** Avoid false patterns from insufficient data

### I-AI-007: User Control
```rust
// User can disable predictions
if !self.settings.enabled {
    return None;
}

// User can clear all data
pub fn clear_all_data(&mut self) {
    self.activity_history.clear();
    self.patterns.clear();
    self.prediction_history.clear();
}
```

**Features:**
- Enable/disable toggle
- Clear all prediction data
- Never mandatory

## Storage Limits (I-AI-003)

- **Activity History:** 1,000 entries (LRU eviction)
- **Patterns:** 100 patterns (oldest evicted)
- **Prediction History:** 100 predictions (FIFO)

All storage is bounded to respect embedded system constraints.

## Pattern Types

### 1. Temporal Patterns

Detect time-based activities:

```rust
Pattern::temporal(
    id: "temporal_14_Chase".into(),
    description: "User often plays Chase at hour 14".into(),
    time_window: (14, 15),  // 2-3 PM
    predicted_activity: "Chase".into(),
    observations: 7,
    probability: 0.85,
    timestamp,
)
```

**Example:** User plays Chase game every day at 3 PM.

### 2. Sequence Patterns

Detect activity chains:

```rust
Pattern::sequence(
    id: "sequence_Drawing_Game".into(),
    description: "After Drawing, user often does Game".into(),
    preceding_activity: "Drawing".into(),
    predicted_activity: "Game".into(),
    observations: 5,
    probability: 0.75,
    timestamp,
)
```

**Example:** User alternates between drawing and games.

### 3. Preference Patterns

Detect usage frequency:

```rust
Pattern::preference(
    id: "preference_Calm".into(),
    description: "User prefers Calm personality (82% of time)".into(),
    predicted_activity: "Calm".into(),
    observations: 41,
    probability: 0.82,
    timestamp,
)
```

**Example:** User chooses Calm personality 82% of the time.

## Integration with Reinforcement Learning (#86)

The prediction engine is designed to work standalone but can enhance RL:

```rust
pub trait ReinforcementLearning {
    fn record_transition(&mut self, state: &Context, action: &str, reward: f32, next_state: &Context);
    fn get_value(&self, state: &Context, action: &str) -> f32;
    fn update_policy(&mut self);
    fn exploration_rate(&self) -> f32;
}
```

**Stub Implementation:** `StubRL` provides basic functionality until #86 is complete.

**Integration Plan:**
1. Replace `StubRL` with full RL implementation from #86
2. Use RL value estimates to improve prediction confidence
3. Combine pattern recognition with learned Q-values
4. Use predictions to guide exploration strategy

## Usage Example

```rust
use mbot_core::learning::prediction::{PredictiveEngine, UserActivity, ActivityType, Context};

// Create engine
let mut engine = PredictiveEngine::new();

// Record user activities
let activity = UserActivity::new(
    "1".into(),
    ActivityType::GameStart,
    "Chase".into(),
    1000000,  // Timestamp
).with_duration(300);  // 5 minutes

engine.record_activity(activity);

// Detect patterns (after collecting data)
let patterns = engine.detect_patterns(2000000);

// Make prediction
let context = Context::new(
    "Idle".into(),
    "Calm".into(),
    15,  // 3 PM
    1,   // Monday
);

if let Some(prediction) = engine.predict_next_action(&context) {
    println!("Prediction: {} (confidence: {:.0}%)",
             prediction.predicted_activity,
             prediction.confidence * 100.0);
    println!("Reasoning: {}", prediction.reasoning);
}

// User can disable
engine.settings.enabled = false;

// User can clear data
engine.clear_all_data();
```

## Testing

### Unit Tests (in `prediction.rs`)

- ✅ `test_activity_creation`
- ✅ `test_engine_bounded_storage` (I-AI-003)
- ✅ `test_pattern_confidence_bounded` (I-AI-001)
- ✅ `test_min_confidence_threshold` (I-AI-005)
- ✅ `test_min_observations_required` (I-AI-006)
- ✅ `test_user_can_disable_predictions` (I-AI-007)
- ✅ `test_user_can_clear_data` (I-AI-007)
- ✅ `test_temporal_pattern_detection`
- ✅ `test_sequence_pattern_detection`
- ✅ `test_prediction_with_reasoning` (I-AI-004)

### Contract Tests

Located at: `crates/mbot-core/tests/ai_learning_test.rs`

Tests verify compliance with invariants I-AI-005, I-AI-006, I-AI-007.

### E2E Journey Test

**Planned:** `tests/journeys/predictive-behavior.journey.spec.ts`

Will test complete user workflows:
- Temporal pattern learning
- Activity sequence prediction
- Preference adaptation
- User override scenarios

## API Reference

### `PredictiveEngine`

**Methods:**
- `new() -> Self` - Create with default settings
- `with_settings(settings: PredictionSettings) -> Self` - Create with custom settings
- `record_activity(&mut self, activity: UserActivity)` - Record user activity
- `detect_patterns(&mut self, timestamp: u64) -> Vec<Pattern>` - Find patterns
- `predict_next_action(&mut self, context: &Context) -> Option<Prediction>` - Make prediction
- `get_patterns(&self) -> &[Pattern]` - Get all detected patterns
- `get_prediction_history(&self) -> &VecDeque<Prediction>` - Get past predictions
- `clear_all_data(&mut self)` - Clear all data (I-AI-007)

### `PredictionSettings`

**Fields:**
- `enabled: bool` - Enable/disable predictions (default: true)
- `min_confidence: f32` - Confidence threshold (default: 0.7)
- `min_observations: u32` - Minimum observations (default: 10)
- `show_suggestions: bool` - Show proactive suggestions (default: true)
- `auto_adapt: bool` - Auto-adapt vs manual confirmation (default: false)

### `Pattern`

**Fields:**
- `id: String` - Unique identifier
- `pattern_type: PatternType` - Temporal, Sequence, or Preference
- `description: String` - Human-readable description
- `confidence: f32` - Pattern confidence (0.0-1.0)
- `observations: u32` - Number of times observed
- `predicted_activity: String` - What the pattern predicts
- `probability: f32` - Prediction probability (0.0-1.0)

### `Prediction`

**Fields:**
- `pattern_id: String` - Source pattern ID
- `confidence: f32` - Prediction confidence (0.0-1.0)
- `predicted_activity: String` - Predicted next activity
- `reasoning: String` - Human-readable explanation
- `suggested_action: Option<Action>` - Proactive suggestion
- `timestamp: u64` - When prediction was made

## Privacy & Safety

1. **User Control:** Users can disable predictions at any time (I-AI-007)
2. **Data Ownership:** Users can clear all prediction data (I-AI-007)
3. **Bounded Storage:** Limited memory usage on embedded systems (I-AI-003)
4. **High Confidence Required:** Only >70% confidence triggers actions (I-AI-005)
5. **Sufficient Data Required:** Minimum 10 observations for patterns (I-AI-006)
6. **Transparency:** All predictions include human-readable reasoning (I-AI-004)

## Future Enhancements

When #86 (Reinforcement Learning) is complete:

1. **RL-Enhanced Confidence**
   - Combine pattern probability with Q-value estimates
   - Use RL to validate pattern predictions

2. **Exploration Guidance**
   - Use predictions to guide exploration strategy
   - Balance pattern-following with RL exploration

3. **Adaptive Learning Rates**
   - Adjust pattern detection based on RL convergence
   - Faster learning for stable behaviors

4. **Multi-Modal Integration**
   - Combine temporal, sequence, and RL signals
   - Weighted ensemble predictions

## Files

- **Implementation:** `crates/mbot-core/src/learning/prediction.rs`
- **Module Export:** `crates/mbot-core/src/learning/mod.rs`
- **Contract:** `docs/contracts/feature_ai_learning.yml`
- **Tests:** `crates/mbot-core/tests/ai_learning_test.rs`
- **Journey Test:** `tests/journeys/predictive-behavior.journey.spec.ts` (planned)

## References

- **Issue #87:** STORY-AI-002: Predictive Behavior Engine
- **Issue #86:** STORY-AI-001: Learning from Play (Reinforcement Learning)
- **Contract:** feature_ai_learning.yml (I-AI-005, I-AI-006, I-AI-007)
- **PRD:** docs/PRD.md (Epic 6: AI & Learning)
