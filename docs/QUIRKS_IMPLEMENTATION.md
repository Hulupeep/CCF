# Quirks System Implementation (#26)

## Overview

This document describes the implementation of the quirks system for mBot RuVector personalities. Quirks are unique, surprising behaviors that make each personality distinct and add character to the robot's behavior.

## Implementation Summary

### Core Components

#### 1. Quirk Enum (9 Total Quirks)
All quirks are defined as enum variants for type safety and compile-time validation:

- `RandomSigh` - Occasional sigh sound when idle
- `SpinWhenHappy` - Spin in place when coherence is high
- `BackUpWhenScared` - Always reverse first when startled
- `ChaseTail` - Sometimes chases own "tail" when bored
- `CollectorInstinct` - Stops near objects, doesn't want to leave
- `NightOwl` - More active in darker environments
- `EarlyBird` - More active in brighter environments
- `SocialButterfly` - Seeks out movement/sound sources
- `Hermit` - Avoids movement/sound sources

#### 2. Trigger System
Quirks trigger based on four condition types:
- **Idle**: Triggers after robot has been idle for a duration
- **StateThreshold**: Triggers when nervous system state crosses threshold
- **Stimulus**: Triggers on sudden stimuli (sound, touch)
- **Environment**: Triggers based on environmental conditions

#### 3. Behavior System
Quirks execute behaviors when triggered:
- **Movement**: Spin, backup, chase tail, stay, approach, retreat
- **Sound**: Sigh, chirp, beep, silence
- **Light**: Pulse, flash, dim, bright
- **ParameterModifier**: Temporarily modify personality parameters
- **Compound**: Execute multiple behaviors

#### 4. QuirkEngine
Manages active quirks and enforces cooldowns:
- Add/remove quirks dynamically
- Check trigger conditions
- Enforce activation chances (0.0-1.0)
- Track cooldown periods
- Prevent quirk spam

### Contract Compliance

#### I-PERS-016: Safety First
- `BackUpWhenScared` has no cooldown (can activate repeatedly for safety)
- Safety quirks never interfere with emergency behaviors
- Quirks are suggestions, not commands (higher-level safety systems can override)

#### I-PERS-017: Configurable Activation Rates
- Each quirk has configurable `activation_chance` (0.0-1.0)
- Default rates range from 0.1 (rare) to 1.0 (always active)
- Can be customized per personality

#### I-PERS-018: Cooldown Periods
- Quirks respect cooldown periods to prevent spam
- Safety quirks have 0ms cooldown
- Fun quirks have medium cooldowns (10-15 seconds)
- Rare behaviors have long cooldowns (20-30 seconds)
- Continuous modifiers have no cooldown

#### I-PERS-019: Multiple Quirks Coexist
- Multiple quirks can be active simultaneously
- Independent cooldown tracking per quirk
- No conflicts between different quirks
- Each personality can have multiple quirks

### Personality Preset Quirks

Each personality preset includes thematically appropriate quirks:

| Personality | Quirks |
|-------------|--------|
| Mellow | random_sigh |
| Curious | social_butterfly |
| Zen | random_sigh |
| Excitable | spin_when_happy, chase_tail |
| Timid | back_up_when_scared, hermit |
| Adventurous | social_butterfly |
| Shy | hermit, back_up_when_scared |
| Grumpy | hermit |
| Cheerful | spin_when_happy |
| Cautious | back_up_when_scared |
| Playful | spin_when_happy, chase_tail, social_butterfly |
| Serious | collector_instinct |
| Energetic | spin_when_happy, chase_tail, early_bird |
| Calm | random_sigh, night_owl |
| Anxious | back_up_when_scared, hermit |

### Architecture

#### no_std Compatibility (ARCH-001)
- Uses `alloc` crate for dynamic collections
- No standard library dependencies in core quirks module
- Compatible with embedded systems

#### Deterministic Behavior (ARCH-002)
- Trigger conditions are deterministic given state
- Activation uses provided RNG function (deterministic with seed)
- Cooldown tracking uses millisecond timestamps
- Reproducible behavior for testing

#### State Isolation
- `QuirkCheckState` snapshot pattern
- Quirks read state but don't modify directly
- Behavior execution is separate from trigger checking
- Clean separation of concerns

### Testing

#### Unit Tests (19 tests in quirks.rs)
- Quirk enum operations (to_str, from_str, all)
- Default configurations for all quirks
- Cooldown tracking and enforcement
- Trigger condition checking
- Activation chance respect
- QuirkEngine operations

#### Integration Tests (19 tests in test_personality_quirks.rs)
- All quirks have valid default configs
- Quirk engine add/remove operations
- Cooldown behavior verification
- Multiple quirks coexist without conflict
- Safety quirks have no cooldown
- Continuous modifier quirks work correctly
- String parsing and roundtrip
- Personality quirk loading

#### Test Coverage
- All 9 quirks tested
- All trigger types tested
- All behavior types defined
- Cooldown edge cases covered
- Safety requirements verified

### API Usage

#### Basic Usage
```rust
use mbot_core::personality::quirks::{QuirkEngine, QuirkConfig, Quirk, QuirkCheckState};

// Create engine and add quirks
let mut engine = QuirkEngine::new();
engine.add_quirk(QuirkConfig::default_for(Quirk::RandomSigh));
engine.add_quirk(QuirkConfig::default_for(Quirk::SpinWhenHappy));

// Check triggers
let state = QuirkCheckState {
    tension: 0.5,
    coherence: 0.9,  // High coherence
    energy: 0.7,
    idle_duration_ms: 12_000,  // Idle for 12 seconds
    stimulus_intensity: 0.0,
    light_level: 0.5,
    movement_detected: false,
    sound_detected: false,
    near_object: false,
};

// Evaluate which quirks should activate
let mut rng = || rand::random::<f32>();
let to_activate = engine.evaluate(&state, current_time_ms, &mut rng);

// Execute behaviors
for config in to_activate {
    match &config.behavior {
        QuirkBehavior::Movement { pattern } => {
            // Execute movement
        }
        QuirkBehavior::Sound { sound } => {
            // Play sound
        }
        // ... handle other behaviors
    }
}
```

#### Loading Personality Quirks
```rust
use mbot_core::personality::PersonalityPreset;

let excitable = PersonalityPreset::Excitable.to_personality();

let mut engine = QuirkEngine::new();
for quirk_str in &excitable.quirks {
    engine.add_quirk_from_str(quirk_str);
}

// Engine now has spin_when_happy and chase_tail quirks
```

### File Structure

```
crates/mbot-core/src/personality/
├── quirks.rs                    # Core quirks implementation (820 lines)
├── mod.rs                       # Updated with quirk support
└── presets.rs                   # Quirk assignments per preset

crates/mbot-core/tests/
├── test_personality_quirks.rs   # E2E integration tests
└── test_personality_simple.rs   # Updated unit tests
```

### Performance Considerations

1. **Efficient Trigger Checking**: O(n) where n = active quirks (typically < 5)
2. **Cooldown Tracking**:
   - std: HashMap lookup O(1)
   - no_std: Vec scan O(n), acceptable for small n
3. **Memory Usage**: Minimal, ~100 bytes per active quirk config

### Future Enhancements

Potential future improvements (not in scope for #26):
1. Custom quirk definitions via scripting
2. Quirk chaining (one quirk triggers another)
3. Quirk learning (adjust activation rates based on user feedback)
4. Quirk animations synchronized with companion app
5. Sound pack integration for quirk sounds

### Compliance Checklist

- [x] All 9 quirks implemented
- [x] Quirk triggers fire under correct conditions
- [x] Activation chances respected
- [x] Cooldowns prevent spam
- [x] Quirks configurable per personality
- [x] Multiple quirks can be active simultaneously
- [x] Quirks don't override safety behaviors
- [x] All tests pass (38 total tests)
- [x] Documentation complete
- [x] Contract compliant (I-PERS-016, 017, 018, 019)
- [x] no_std compatible (ARCH-001)
- [x] Deterministic behavior (ARCH-002)

### Issue Closure

This implementation fully satisfies GitHub issue #26 (STORY-PERS-006: Quirks System).

**Acceptance Criteria Met:**
- [x] All 9 quirks implemented
- [x] Quirk triggers fire under correct conditions
- [x] Activation chances respected (20% means ~20% activation)
- [x] Cooldowns prevent spam
- [x] Quirks configurable per personality
- [x] Multiple quirks can coexist simultaneously
- [x] Quirks don't override safety behaviors

**Test Coverage:**
- 19 unit tests in quirks.rs
- 19 integration tests in test_personality_quirks.rs
- All tests passing (100% pass rate)

**Gherkin Scenarios Satisfied:**
- [x] SpinWhenHappy quirk activates appropriately
- [x] RandomSigh quirk during idle
- [x] BackUpWhenScared overrides normal startle
- [x] ChaseTail when bored
- [x] NightOwl increases activity in dark
- [x] Quirk cooldown prevents spam
- [x] Multiple quirks coexist

### Related Issues

- #12 - STORY-PERS-001: Personality Data Structure (dependency)
- #18 - STORY-PERS-003: Preset Personalities (integration)
- #23 - STORY-PERS-004: Personality Persistence (future integration)

### Commit Message

```
feat(personality): implement quirks system (#26)

- Add Quirk enum with 9 unique quirk types
- Implement trigger system (idle, state threshold, stimulus, environment)
- Create behavior system (movement, sound, light, parameter modifier)
- Add QuirkEngine for managing active quirks and cooldowns
- Integrate quirks with personality presets
- Ensure contract compliance (I-PERS-016, 017, 018, 019)
- Maintain no_std compatibility (ARCH-001)
- Maintain deterministic behavior (ARCH-002)
- Add 38 comprehensive tests (100% pass rate)

Closes #26
```
