# Reinforcement Learning System - mBot RuVector

## Overview

The mBot RuVector Reinforcement Learning (RL) system enables the robot to learn from game outcomes and user feedback, improving its behavior over time through Q-learning.

## Features

- **Q-Learning Algorithm**: Classic tabular Q-learning with epsilon-greedy exploration
- **Multi-Game Support**: Separate Q-tables for different games (tic-tac-toe, connect-four, etc.)
- **Policy Persistence**: Save and load learned policies across sessions
- **User Feedback**: Incorporate human ratings to guide learning
- **Convergence Detection**: Automatic detection when learning has stabilized
- **Adaptive Exploration**: Epsilon decay for explore-exploit balance
- **Learning Rate Scheduling**: Decay prevents oscillation after convergence

## Architecture

```
learning/
├── mod.rs              # Public API and exports
├── q_learning.rs       # Q-learning algorithm implementation
├── policy.rs           # Policy storage and persistence
├── reward.rs           # Reward functions and user feedback
└── metrics.rs          # Learning metrics and configuration
```

## Invariants (Contracts)

The RL system enforces these architectural invariants from `feature_ai_learning.yml`:

### I-AI-001: Convergence within 1000 episodes
- **Requirement**: Q-learning must converge for simple games like tic-tac-toe within 1000 training episodes
- **Implementation**: Tracks variance of recent rewards; convergence score > 0.9 indicates stable learning
- **Test**: `test_q_learning_convergence()` in journey tests

### I-AI-002: Policy persistence across sessions
- **Requirement**: Learned policies must be saved and restored without loss
- **Implementation**: `save_policy()` and `load_policy()` serialize Q-tables
- **Test**: `test_policy_save_load()` verifies roundtrip preservation

### I-AI-003: Learning rate decay
- **Requirement**: Learning rate must decrease over time to prevent unlearning
- **Implementation**: `α_t = α_0 / (1 + episode_count / 100)`
- **Test**: `test_learning_rate_decay()` verifies monotonic decrease

### I-AI-004: Epsilon-greedy exploration decay
- **Requirement**: Exploration rate starts high (1.0) and decreases to minimum (0.1)
- **Implementation**: `ε_t = max(ε_min, ε_t-1 * decay)`
- **Test**: `test_epsilon_decay()` checks bounds and monotonicity

### I-AI-005: Q-value bounding
- **Requirement**: Q-values must be numerically stable (no overflow)
- **Implementation**: Q-values clamped to reasonable bounds
- **Test**: Verified in `test_q_learning_update()`

### I-AI-006: Deterministic learning
- **Requirement**: Same seed produces same learning trajectory
- **Implementation**: Uses episode count for pseudo-random selection (no `rand::thread_rng`)
- **Test**: `test_deterministic_learning()`

### I-AI-007: User feedback responsiveness
- **Requirement**: Feedback must affect policy within 10 episodes
- **Implementation**: Immediate Q-value adjustment from user ratings
- **Test**: `test_user_feedback_effect()`

## Usage

### Basic Q-Learning

```rust
use mbot_core::learning::{QLearner, LearningConfig, State, Action};

// Create learner with default config
let config = LearningConfig::default();
let mut learner = QLearner::new(config);

// Define state and action
let state = State::new("tictactoe".into(), "X__O__X__".into());
let action = Action::new("place_marker".into(), (1, 1));
let next_state = State::new("tictactoe".into(), "X__OX_X__".into());

// Learn from experience
learner.learn(&state, &action, 1.0, &next_state);

// Select best action (exploit)
let actions = vec![
    Action::new("place_marker".into(), (0, 1)),
    Action::new("place_marker".into(), (2, 2)),
];
let best_action = learner.select_action(&state, &actions, false);

// Or explore
let explore_action = learner.select_action(&state, &actions, true);
```

### Training Loop

```rust
use mbot_core::learning::{QLearner, LearningConfig, RewardFunction};

let config = LearningConfig::fast_convergence();
let reward_fn = RewardFunction::tictactoe();
let mut learner = QLearner::new(config).with_reward_function(reward_fn);

for episode in 0..1000 {
    // Play game...
    let outcome = "win"; // or "loss", "draw"
    let total_reward = 1.0;

    learner.complete_episode(total_reward, outcome);

    // Check convergence
    if learner.get_metrics().is_converged() {
        println!("Converged after {} episodes!", episode);
        break;
    }
}
```

### User Feedback

```rust
use mbot_core::learning::{UserFeedback, FeedbackRating};

// User rates robot behavior
let feedback = UserFeedback::new(123, FeedbackRating::Good);
learner.update_from_feedback("behavior_cautious_explore", feedback);

// Bad feedback
let bad_feedback = UserFeedback::new(124, FeedbackRating::Bad);
learner.update_from_feedback("behavior_reckless_move", bad_feedback);
```

### Policy Persistence

```rust
// Save learned policy
let policy_data = learner.save_policy("tictactoe")?;
// Store policy_data to file or database

// Later: load policy
learner.load_policy("tictactoe", &policy_data)?;

// Robot maintains performance without retraining
assert!(learner.get_metrics().win_rate > 0.7);
```

## Configuration Presets

### Default (Balanced)
```rust
LearningConfig::default()
// - Learning rate: 0.1
// - Discount factor: 0.9
// - Epsilon: 1.0 → 0.1 (decay 0.995)
// - Max episodes: 1000
```

### Fast Convergence
```rust
LearningConfig::fast_convergence()
// - Higher learning rate (0.2)
// - Faster epsilon decay (0.99)
// - Fewer max episodes (500)
// Use for simple games or quick iteration
```

### Careful Exploration
```rust
LearningConfig::careful_exploration()
// - Lower learning rate (0.05)
// - Slower epsilon decay (0.998)
// - More max episodes (2000)
// Use for complex games or safety-critical learning
```

## Metrics

The `LearningMetrics` struct tracks:

- **episode_count**: Total training episodes
- **average_reward**: Mean reward over recent episodes
- **win_rate**: Win percentage (last 100 games)
- **loss_rate**: Loss percentage
- **draw_rate**: Draw percentage
- **epsilon_current**: Current exploration rate
- **learning_rate_current**: Current learning rate
- **convergence_score**: 0-1, 1 = fully converged

### Checking Progress

```rust
let metrics = learner.get_metrics();

println!("Episode: {}", metrics.episode_count);
println!("Win rate: {:.1}%", metrics.win_rate * 100.0);
println!("Convergence: {:.1}%", metrics.convergence_score * 100.0);

if metrics.is_ready() {
    println!("Ready for deployment!");
}
```

## Web Dashboard (TypeScript)

The `learningMonitor.ts` service provides WebSocket-based monitoring:

```typescript
import { createLearningMonitor } from '@/services/learningMonitor';

const monitor = createLearningMonitor('ws://localhost:8080/learning');

// Connect
await monitor.connect();

// Enable learning for a game
monitor.enableLearning('tictactoe');

// Listen for metrics updates
monitor.on('metrics', (metrics) => {
  console.log('Win rate:', metrics.winRate);
  console.log('Episodes:', metrics.episodeCount);
});

// Listen for episode completion
monitor.on('episode', (data) => {
  console.log(`Episode ${data.episode}: ${data.outcome} (reward: ${data.reward})`);
});

// Send user feedback
monitor.sendFeedback('behavior_123', 'good');

// Save policy
await monitor.savePolicy('tictactoe');

// Reset learning
monitor.resetLearning('tictactoe');
```

## Game Integration

### Adding a New Game

1. **Define State Representation**

```rust
fn tictactoe_state(board: &[[char; 3]; 3]) -> State {
    let board_str = board.iter()
        .flat_map(|row| row.iter())
        .collect::<String>();

    State::new("tictactoe".into(), board_str)
        .with_features(extract_features(board))
}

fn extract_features(board: &[[char; 3]; 3]) -> Vec<i32> {
    vec![
        count_x(board),
        count_o(board),
        check_threats(board),
        // ... more features
    ]
}
```

2. **Define Actions**

```rust
fn available_actions(board: &[[char; 3]; 3]) -> Vec<Action> {
    let mut actions = Vec::new();
    for row in 0..3 {
        for col in 0..3 {
            if board[row][col] == '_' {
                actions.push(Action::new("place_marker".into(), (row as i32, col as i32)));
            }
        }
    }
    actions
}
```

3. **Create Reward Function**

```rust
impl RewardFunction {
    pub fn my_game() -> Self {
        Self {
            game_type: "my_game",
            win_reward: 1.0,
            loss_reward: -1.0,
            draw_reward: 0.0,
            good_move_reward: 0.15,
            bad_move_reward: -0.15,
            user_good_reward: 0.5,
            user_bad_reward: -0.5,
        }
    }
}
```

4. **Training Loop**

```rust
let config = LearningConfig::default();
let reward_fn = RewardFunction::my_game();
let mut learner = QLearner::new(config).with_reward_function(reward_fn);

for episode in 0..1000 {
    let mut state = initial_state();

    loop {
        let actions = available_actions(&state);
        let action = learner.select_action(&state, &actions, true).unwrap();

        let (next_state, reward, done) = execute_action(&state, &action);

        learner.learn(&state, &action, reward, &next_state);

        if done {
            let outcome = determine_outcome(&next_state);
            learner.complete_episode(reward, outcome);
            break;
        }

        state = next_state;
    }
}
```

## Testing

### Unit Tests

```bash
cargo test --lib learning
```

### Journey Tests (E2E)

```bash
npm run test:journeys -- reinforcement-learning.journey.spec.ts
```

### Contract Tests

```bash
npm test -- contracts/ai_learning.test.ts
```

## Performance

- **Memory**: Q-table size = O(states × actions)
- **Training Speed**: ~1000 episodes/minute for tic-tac-toe
- **Convergence**: Typically 200-500 episodes for simple games
- **Storage**: ~1KB per game policy (serialized Q-table)

## Limitations

- **Tabular Q-learning**: Not suitable for large state spaces (use deep RL for complex games)
- **No transfer learning**: Each game learns independently
- **Synchronous training**: No parallel episode execution
- **Fixed features**: State representation is game-specific

## Future Enhancements

- Experience replay buffer for sample efficiency
- Double Q-learning to reduce overestimation bias
- Prioritized experience replay
- Transfer learning across similar games
- Deep Q-Networks (DQN) for complex state spaces
- Multi-agent reinforcement learning

## References

- Sutton & Barto, "Reinforcement Learning: An Introduction"
- Watkins, "Q-Learning", Machine Learning (1992)
- Issue #86: Implementation spec
- Contract: `docs/contracts/feature_ai_learning.yml`
- Tests: `tests/journeys/reinforcement-learning.journey.spec.ts`
