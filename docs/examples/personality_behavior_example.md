# Personality-to-Behavior Mapping Example

## Overview

This example demonstrates how personality parameters influence robot behaviors in the mBot RuVector system.

## Basic Usage

### 1. Creating a Personality Mapper

```rust
use mbot_core::personality::{Personality, PersonalityMapper, PersonalityPreset};

// Option 1: Start with a preset
let curious = PersonalityPreset::Curious.to_personality();
let mapper = PersonalityMapper::with_personality(&curious);

// Option 2: Create custom personality
let mut custom = Personality::default();
custom.set_tension_baseline(0.7).unwrap();
custom.set_curiosity_drive(0.9).unwrap();
let mapper = PersonalityMapper::with_personality(&custom);
```

### 2. Applying Personality to Homeostasis

```rust
use mbot_core::personality::{
    apply_tension_influence,
    apply_curiosity_influence,
};

// Get current personality influence
let influence = mapper.current_influence();

// Apply to tension calculation
let raw_tension = 0.5; // From sensor processing
let current_tension = state.tension;
let modified_tension = apply_tension_influence(
    raw_tension,
    current_tension,
    influence,
    0.01, // Gravity strength toward baseline
);

// Apply to curiosity
let raw_curiosity = 0.6;
let modified_curiosity = apply_curiosity_influence(raw_curiosity, influence);
```

### 3. Scaling Motor Outputs

```rust
use mbot_core::personality::scale_motor_output;

// Get motor commands from nervous system
let (left, right) = (80, 80);

// Scale by personality's movement expressiveness
let (scaled_left, scaled_right) = scale_motor_output(left, right, influence);

// Apply to motors
motor_command.left = scaled_left;
motor_command.right = scaled_right;
```

### 4. Smooth Personality Transitions

```rust
// Transition from one personality to another over 200 ticks (~3 seconds at 60 Hz)
let new_personality = PersonalityPreset::Zen.to_personality();
mapper.transition_to(&new_personality, 200);

// In your main loop
loop {
    // Update transition state
    let influence = mapper.tick_transition();

    // Check if transition complete
    if !mapper.is_transitioning() {
        println!("Transition complete!");
    }

    // Use influence for behavior calculations...
}
```

## Complete Integration Example

### MBotBrain with Personality

```rust
use mbot_core::{
    MBotBrain, MBotSensors, MotorCommand,
    personality::{PersonalityMapper, PersonalityPreset, apply_tension_influence, scale_motor_output},
};

pub struct PersonalityBrain {
    brain: MBotBrain,
    mapper: PersonalityMapper,
}

impl PersonalityBrain {
    pub fn new() -> Self {
        // Start with Mellow personality
        let personality = PersonalityPreset::Mellow.to_personality();
        let mapper = PersonalityMapper::with_personality(&personality);

        Self {
            brain: MBotBrain::new(),
            mapper,
        }
    }

    pub fn tick(&mut self, sensors: &MBotSensors) -> MotorCommand {
        // Update transition if active
        let influence = self.mapper.tick_transition();

        // Get base nervous system response
        let (state, mut cmd) = self.brain.tick(sensors);

        // Apply personality influence to outputs
        let (scaled_left, scaled_right) = scale_motor_output(
            cmd.left,
            cmd.right,
            influence,
        );

        cmd.left = scaled_left;
        cmd.right = scaled_right;

        // Scale LED brightness by light_expressiveness
        cmd.led_color = [
            (cmd.led_color[0] as f32 * influence.light_scale) as u8,
            (cmd.led_color[1] as f32 * influence.light_scale) as u8,
            (cmd.led_color[2] as f32 * influence.light_scale) as u8,
        ];

        cmd
    }

    pub fn change_personality(&mut self, new_personality: &Personality, duration_ticks: u32) {
        self.mapper.transition_to(new_personality, duration_ticks);
    }
}
```

## Personality Influence Explained

### Baseline Targets

Personality baselines act as gravitational centers for homeostasis:

- **tension_baseline**: Where tension "wants to be" when idle
- **coherence_baseline**: Target internal stability
- **energy_baseline**: Resting energy level

Example:
```rust
// High tension baseline = nervous personality
personality.set_tension_baseline(0.7);

// Over time, tension gravitates toward 0.7 even without external stimuli
```

### Reactivity Multipliers

These modify how the robot processes incoming stimuli:

- **startle_sensitivity** → `stimulus_multiplier` (0.5 to 1.5)
  - High: Strong reactions to stimuli
  - Low: Dampened reactions

- **recovery_speed** → `recovery_rate` (0.5 to 1.5)
  - High: Quick return to baseline
  - Low: Slow recovery

- **curiosity_drive** → `curiosity_multiplier` (0.0 to 1.0)
  - High: Strong interest in novelty
  - Low: Ignores new stimuli

### Expression Scaling

These control output intensity:

- **movement_expressiveness** (0.0 to 1.0)
  - 0.0: Almost no movement
  - 1.0: Full range of motion

- **sound_expressiveness** (0.0 to 1.0)
  - Controls buzzer frequency/volume

- **light_expressiveness** (0.0 to 1.0)
  - Controls LED brightness

## Observable Behavior Differences

### Mellow vs Excitable

```rust
let mellow = PersonalityPreset::Mellow.to_personality();
let excitable = PersonalityPreset::Excitable.to_personality();

// Same stimulus, different reactions:
// Mellow: Slow, gentle movements, dim LEDs
// Excitable: Fast, energetic movements, bright LEDs
```

### Kitchen Table Test

Place both robots on a table with an obstacle:

- **Mellow**: Approaches slowly, gentle turns, soft beeps
- **Excitable**: Approaches quickly, sharp turns, loud beeps
- **Zen**: Minimal movement, very dim lights, no sounds
- **Timid**: Backs away early, nervous movements

## Contract Compliance

### I-PERS-004: Influence, Don't Override

```rust
// ✅ CORRECT: Personality influences nervous system
let modified = apply_tension_influence(raw, current, influence, gravity);

// ❌ WRONG: Directly setting values
state.tension = personality.tension_baseline(); // Violates contract!
```

### I-PERS-005: Emergent Behavior

Behavior emerges from: **Personality + Nervous System State**

```rust
// Same personality, different states = different behaviors
let curious_calm = apply_tension_influence(0.1, 0.2, curious_influence, 0.01);
let curious_excited = apply_tension_influence(0.8, 0.9, curious_influence, 0.01);

// Results are different despite same personality
```

### I-PERS-006: Gradual Transitions

```rust
// Smooth 3-second transition
mapper.transition_to(&new_personality, 180); // 180 ticks at 60 Hz

// Each tick makes small incremental changes
// No sudden jumps in behavior
```

## Testing

Run comprehensive tests:

```bash
# Unit tests for behavior mapping
cargo test --package mbot-core --lib personality::behavior_mapping

# Integration tests
cargo test personality_behavior_mapping

# All personality tests
cargo test --package mbot-core personality
```

## Further Reading

- [Personality Data Structure (STORY-PERS-001)](../contracts/feature_personality.yml)
- [Behavior Mapping Contract (I-PERS-004..006)](../contracts/feature_personality.yml)
- [Preset Personalities (#18)](https://github.com/Hulupeep/mbot_ruvector/issues/18)
