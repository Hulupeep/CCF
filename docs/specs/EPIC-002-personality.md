# EPIC-002: Personality System

**"Same robot, wildly different personalities"**

## Overview

Create a configurable personality system that transforms a single mBot2 into multiple distinct characters. Each personality emerges from different configurations of the RuVector nervous system parameters - not from scripted behaviors.

## User Value

- **Variety**: One robot, infinite personalities
- **Connection**: Find a personality that matches your mood
- **Sharing**: Trade personalities with friends
- **Learning**: Understand how parameters create behavior

---

## Architecture Requirements

### ARCH-PERS-001 (MUST)
Personality parameters must be serializable to JSON for persistence and sharing.

### ARCH-PERS-002 (MUST)
Personality changes must not require code changes or recompilation.

### ARCH-PERS-003 (MUST)
All personality parameters must have safe bounds to prevent harmful behaviors.

### ARCH-PERS-004 (MUST)
Personality state must be separable from nervous system state.

---

## Feature Requirements

### PERS-001 (MUST): Personality Configuration
A personality is defined by these configurable parameters:

```rust
struct Personality {
    // Identity
    name: String,
    icon: char,  // emoji representation

    // Baseline moods (where it wants to be)
    tension_baseline: f32,     // 0.0-1.0
    coherence_baseline: f32,   // 0.0-1.0
    energy_baseline: f32,      // 0.0-1.0

    // Reactivity (how much stimuli affect it)
    startle_sensitivity: f32,  // 0.0-1.0
    recovery_speed: f32,       // 0.0-1.0
    curiosity_drive: f32,      // 0.0-1.0

    // Expression (how it shows feelings)
    movement_expressiveness: f32,  // 0.0-1.0
    sound_expressiveness: f32,     // 0.0-1.0
    light_expressiveness: f32,     // 0.0-1.0

    // Quirks (unique behaviors)
    quirks: Vec<Quirk>,
}
```

**Gherkin:**
```gherkin
@PERS-001
Scenario: Personality affects baseline mood
  Given the personality "Nervous Nellie" with tension_baseline 0.7
  When the robot is idle for 30 seconds
  Then the tension should stabilize near 0.7
  And the robot should exhibit nervous behaviors
```

### PERS-002 (MUST): Preset Personalities
Five distinct personalities ship by default:

| Personality | Icon | Description | Key Trait |
|-------------|------|-------------|-----------|
| **Curious Cleo** | 🔍 | Must investigate everything | High curiosity |
| **Nervous Nellie** | 😰 | Worried about everything | High tension |
| **Chill Charlie** | 😎 | Nothing bothers them | Low reactivity |
| **Bouncy Betty** | 🎉 | Can't stop moving | High energy |
| **Grumpy Gus** | 😤 | Everything is annoying | Low coherence |

**Gherkin:**
```gherkin
@PERS-002
Scenario: Curious Cleo investigates new stimuli
  Given the active personality is "Curious Cleo"
  When a new object is detected by ultrasonic
  Then the robot should approach the object
  And exploration behaviors should activate
```

```gherkin
@PERS-002
Scenario: Grumpy Gus resists interaction
  Given the active personality is "Grumpy Gus"
  When the user tries to play chase
  Then the robot should reluctantly participate
  And occasionally stop and "huff" (LED + sound)
```

### PERS-003 (MUST): Personality Switching
Users can switch personalities without restarting.

**Gherkin:**
```gherkin
@PERS-003
Scenario: Switch personality at runtime
  Given the robot is running as "Chill Charlie"
  When the user selects "Nervous Nellie"
  Then the personality should transition over 3 seconds
  And behaviors should gradually shift to match
```

### PERS-004 (MUST): Personality Persistence
The robot remembers its last personality and any customizations.

**Gherkin:**
```gherkin
@PERS-004
Scenario: Personality survives restart
  Given the robot is configured as "Bouncy Betty"
  When the robot is powered off and on
  Then it should boot as "Bouncy Betty"
  And all custom parameters should be preserved
```

### PERS-005 (SHOULD): Quirks System
Small unique behaviors that make each personality special:

```rust
enum Quirk {
    RandomSigh,           // Occasional sigh sound when idle
    SpinWhenHappy,        // Spin in place when coherence is high
    BackUpWhenScared,     // Always reverse first when startled
    ChaseTail,            // Sometimes chases own "tail" when bored
    CollectorInstinct,    // Stops near objects, doesn't want to leave
    NightOwl,             // More active in darker environments
    EarlyBird,            // More active in brighter environments
    SocialButterfly,      // Seeks out movement/sound sources
    Hermit,               // Avoids movement/sound sources
}
```

**Gherkin:**
```gherkin
@PERS-005
Scenario: Quirk activates appropriately
  Given the personality has quirk "SpinWhenHappy"
  And coherence is above 0.8
  When idle for 5 seconds
  Then there is a 20% chance of a celebration spin
```

### PERS-006 (SHOULD): Mood Memory
The robot's recent experiences affect its mood trajectory.

**Gherkin:**
```gherkin
@PERS-006
Scenario: Bad experiences affect mood
  Given the robot was startled 5 times in 1 minute
  When calculating tension
  Then baseline tension should be temporarily elevated
  And recovery should be slower for the next 5 minutes
```

### PERS-007 (SHOULD): Personality Editor
Visual interface to create custom personalities.

### PERS-008 (MAY): Personality Sharing
Export/import personalities as files for trading.

---

## Journey: J-PERS-MEET-PERSONALITY

**DOD Criticality: CRITICAL**

First time experiencing a robot's personality.

### Preconditions
- mBot2 connected via Companion app
- Default personality loaded

### Steps

| Step | User Action | Robot Response | Verification |
|------|-------------|----------------|--------------|
| 1 | Opens personality menu | List of 5 presets shown | UI shows all presets |
| 2 | Selects "Nervous Nellie" | Robot shudders, LEDs flicker yellow | Transition animation |
| 3 | Moves hand toward robot | Robot backs away nervously | Protect mode triggers |
| 4 | Speaks gently | Robot cautiously approaches | Recovery behavior |
| 5 | Selects "Bouncy Betty" | Robot perks up, starts moving | Energy increase visible |
| 6 | Does nothing | Robot circles, wiggles, seeks attention | High baseline energy |

### Expected Outcome
User clearly sees the difference between personalities and feels the robot has a distinct character.

---

## Journey: J-PERS-CUSTOMIZE

**DOD Criticality: IMPORTANT**

Creating a custom personality.

### Preconditions
- Companion app with personality editor
- Understanding of basic parameters

### Steps

| Step | User Action | Robot Response | Verification |
|------|-------------|----------------|--------------|
| 1 | Opens personality editor | Parameter sliders shown | Editor UI loads |
| 2 | Names personality "Sleepy Sam" | Name saved | Name appears in UI |
| 3 | Sets energy_baseline to 0.2 | Robot slows down | Immediate feedback |
| 4 | Sets tension_baseline to 0.3 | Robot relaxes | Calm behaviors increase |
| 5 | Adds quirk "RandomSigh" | Robot occasionally sighs | Quirk activates |
| 6 | Saves personality | Added to preset list | Persistence verified |

### Expected Outcome
User has created a unique personality that behaves consistently.

---

## Data Contracts

### PersonalityConfig
```typescript
interface PersonalityConfig {
  id: string;
  name: string;
  icon: string;
  version: number;
  created_at: number;
  modified_at: number;

  // Baselines (0.0 - 1.0)
  tension_baseline: number;
  coherence_baseline: number;
  energy_baseline: number;

  // Reactivity (0.0 - 1.0)
  startle_sensitivity: number;
  recovery_speed: number;
  curiosity_drive: number;

  // Expression (0.0 - 1.0)
  movement_expressiveness: number;
  sound_expressiveness: number;
  light_expressiveness: number;

  // Quirks
  quirks: string[];

  // Custom sounds (optional)
  sound_pack?: string;
}
```

### PersonalityPresets
```typescript
const PRESET_PERSONALITIES: PersonalityConfig[] = [
  {
    id: "curious-cleo",
    name: "Curious Cleo",
    icon: "🔍",
    tension_baseline: 0.3,
    coherence_baseline: 0.5,
    energy_baseline: 0.7,
    startle_sensitivity: 0.4,
    recovery_speed: 0.8,
    curiosity_drive: 0.9,
    movement_expressiveness: 0.8,
    sound_expressiveness: 0.6,
    light_expressiveness: 0.7,
    quirks: ["investigate_all", "excited_wiggle"],
  },
  // ... other presets
];
```

---

## Stories

### STORY-PERS-001: Personality Data Structure
**Points:** 3
**Covers:** PERS-001, ARCH-PERS-001

As a developer, I need to define the Personality struct so it can be configured and serialized.

**Tasks:**
- [ ] Define Personality struct in mbot-core
- [ ] Implement Default trait with safe values
- [ ] Add serde serialization
- [ ] Add validation for bounds
- [ ] Unit tests for serialization round-trip

### STORY-PERS-002: Personality-to-Behavior Mapping
**Points:** 5
**Covers:** PERS-001

As a robot, I need my personality to influence my nervous system so my behavior emerges from who I am.

**Tasks:**
- [ ] Integrate personality baselines into homeostasis
- [ ] Apply reactivity modifiers to stimulus processing
- [ ] Apply expressiveness to output generation
- [ ] Add smooth transitions when parameters change
- [ ] Integration tests with different personalities

### STORY-PERS-003: Preset Personalities
**Points:** 3
**Covers:** PERS-002

As a user, I need preset personalities so I can experience variety immediately.

**Tasks:**
- [ ] Define 5 preset configurations
- [ ] Test each preset for distinct behavior
- [ ] Verify Kitchen Table Test for all presets
- [ ] Add personality icons and descriptions
- [ ] Document personality differences

### STORY-PERS-004: Personality Switching
**Points:** 3
**Covers:** PERS-003

As a user, I need to switch personalities at runtime so I can change the robot's character.

**Tasks:**
- [ ] Add personality switch command
- [ ] Implement smooth transition (not jarring)
- [ ] Handle mid-action switches gracefully
- [ ] Add transition animation (LEDs)
- [ ] Test rapid switching doesn't crash

### STORY-PERS-005: Personality Persistence
**Points:** 3
**Covers:** PERS-004, ARCH-PERS-001

As a robot, I need to remember my personality across restarts.

**Tasks:**
- [ ] Save active personality to file
- [ ] Load personality on startup
- [ ] Handle missing/corrupt files gracefully
- [ ] Test persistence across power cycles

### STORY-PERS-006: Quirks System
**Points:** 5
**Covers:** PERS-005

As a personality, I need quirks so I have unique surprising behaviors.

**Tasks:**
- [ ] Define Quirk enum with all quirks
- [ ] Implement quirk trigger logic
- [ ] Add quirk behaviors (spin, sigh, etc.)
- [ ] Make quirks configurable per personality
- [ ] Test quirk activation rates

### STORY-PERS-007: Meet Personality Journey Test
**Points:** 5
**Covers:** J-PERS-MEET-PERSONALITY

As a tester, I need to verify users can experience distinct personalities.

**Tasks:**
- [ ] Create journey test
- [ ] Test all 5 preset switches
- [ ] Verify behavioral differences
- [ ] Test transition smoothness
- [ ] Document with video

---

## Dependencies

- **Requires:** EPIC-000 (Core nervous system) ✅ Complete
- **Enables:** All other EPICs (personalities make everything more fun)
- **Soft dependency:** Companion app UI for editor

---

## Open Questions

1. Should personalities "grow" over time based on experiences?
2. Can two robots "exchange" personality traits through interaction?
3. Should there be "hidden" personality traits that emerge?
4. How do we prevent "mean" personalities from being created?
