# Meet Personality Journey Test - Implementation Requirements

**Issue:** #29 - STORY-PERS-007: Meet Personality Journey Test
**Contract:** J-PERS-MEET-PERSONALITY
**Test File:** `tests/journeys/meet-personality.journey.spec.ts`

## Test Overview

This E2E journey test validates the complete "Meet Personality" experience where users first encounter the robot's personality system and observe distinct behavioral differences between personality presets.

### Journey Structure (6 Steps)

1. Opens personality menu
2. Selects 'Nervous Nellie' (Timid preset)
3. Hand approach triggers nervous response
4. Gentle speech triggers recovery
5. Switch to 'Bouncy Betty' (Energetic preset)
6. Observe high-energy idle behavior

## Required data-testid Selectors

The companion app UI must implement these test IDs for journey testing:

### Core Status
| Element | data-testid | Purpose |
|---------|-------------|---------|
| Connection status | `mbot-status` | Shows "Connected" when ready |
| Current personality | `current-personality` | Displays active personality name |
| Tension display | `tension-level` | Shows "Low", "Medium", or "High" |
| Tension numeric | `tension-value` | Numeric tension value (0.0-1.0) |
| Energy display | `energy-level` | Shows "Low", "Medium", or "High" |
| Energy numeric | `energy-value` | Numeric energy value (0.0-1.0) |
| Behavior mode | `behavior-mode` | Current behavior (Calm, Protect, Explore, etc.) |
| Behavior log | `behavior-log` | Text log of recent behaviors |

### Personality Menu
| Element | data-testid | Purpose |
|---------|-------------|---------|
| Menu button | `personality-menu-button` | Opens personality selector |
| Menu container | `personality-menu` | Personality selection UI |
| Preset: Curious Cleo | `personality-preset-curious-cleo` | Button to select Curious preset |
| Preset: Nervous Nellie | `personality-preset-nervous-nellie` | Button to select Timid preset |
| Preset: Chill Charlie | `personality-preset-chill-charlie` | Button to select Calm preset |
| Preset: Bouncy Betty | `personality-preset-bouncy-betty` | Button to select Energetic preset |
| Preset: Grumpy Gus | `personality-preset-grumpy-gus` | Button to select Grumpy preset |
| Preset icon | `personality-icon-{preset-id}` | Visual icon for preset |
| Preset description | `personality-description-{preset-id}` | Text description |
| Transition animation | `transition-animation` | Visual transition indicator |

### Personality Presets Mapping

| UI Name | Internal Preset | Key Traits |
|---------|----------------|------------|
| Curious Cleo | Curious | High curiosity, social butterfly |
| Nervous Nellie | Timid | High tension baseline, backs up when scared |
| Chill Charlie | Calm | Low reactivity, relaxed |
| Bouncy Betty | Energetic | High energy, spins when happy |
| Grumpy Gus | Grumpy | Low coherence, hermit |

### Simulation Controls
| Element | data-testid | Purpose |
|---------|-------------|---------|
| Simulate obstacle button | `simulate-obstacle` | Opens obstacle simulation |
| Obstacle distance input | `obstacle-distance` | Distance in cm (0-400) |
| Apply simulation | `apply-simulation` | Applies the simulated input |
| Simulate sound button | `simulate-sound` | Opens sound simulation |
| Sound level input | `sound-level` | Sound level (0-100) |
| Sound type selector | `sound-type` | Type: voice, noise, music |
| Apply sound | `apply-sound` | Applies sound stimulus |
| Clear all stimuli | `clear-all-stimuli` | Resets all simulations |

### Motor & LED Status
| Element | data-testid | Purpose |
|---------|-------------|---------|
| Left motor | `motor-left` | Left motor speed (-100 to +100) |
| Right motor | `motor-right` | Right motor speed (-100 to +100) |
| Motor speed | `motor-speed` | Average absolute speed |
| LED color | `led-color` | Current LED color name or RGB |

### Journey Completion
| Element | data-testid | Purpose |
|---------|-------------|---------|
| Journey summary | `journey-summary` | Summary of journey completion |

## Test Scenarios

### 1. Individual Step Tests (6 tests)
- `@step-1`: Opens personality menu
- `@step-2`: Selects Nervous Nellie
- `@step-3`: Hand approach triggers nervous response
- `@step-4`: Gentle speech triggers recovery
- `@step-5`: Switch to Bouncy Betty
- `@step-6`: Bouncy Betty idle behavior

### 2. Complete Journey Test
- `@full-journey`: Executes all 6 steps in sequence

### 3. All Presets Test
- `@all-presets`: Tests all 5 personalities are distinguishable within 30 seconds (I-PERS-007)
- Generates screenshots for documentation

### 4. Transition Smoothness Test
- `@I-PERS-022`: Verifies smooth transitions with no sudden jumps (<0.15 delta per sample)

### 5. Persistence Test
- Tests personality persists across page reloads

## Invariants Tested

| Invariant | Description | Test Coverage |
|-----------|-------------|---------------|
| I-PERS-007 | All presets distinguishable in 30s | `@all-presets` test |
| I-PERS-020 | Journey completes without errors | `@full-journey` test |
| I-PERS-021 | Each step verification is observable | All step tests |
| I-PERS-022 | Transitions are smooth and visible | `@I-PERS-022` test |

## Expected Behaviors

### Nervous Nellie (Timid Preset)
- **Tension baseline:** High (0.7+)
- **Response to approach:** Backs away, motors negative
- **LED color:** Yellow/warning when stressed
- **Quirks:** back_up_when_scared, hermit
- **Observable in 30s:** High tension, defensive posture

### Bouncy Betty (Energetic Preset)
- **Energy baseline:** High (0.7+)
- **Idle behavior:** Circles, wiggles, seeks attention
- **Movement speed:** Fast (50+)
- **Quirks:** spin_when_happy, chase_tail, early_bird
- **Observable in 30s:** High energy, constant movement

### Transition Behavior
- **Duration:** 2-3 seconds (100 ticks)
- **Animation:** Visual transition indicator
- **LED feedback:** Color shifts during transition
- **Smoothness:** Max delta <0.03 per tick, <0.15 per 300ms sample

## Running the Tests

```bash
# Run all journey tests
npm run test:journeys

# Run only meet-personality tests
npx playwright test tests/journeys/meet-personality.journey.spec.ts

# Run specific step test
npx playwright test tests/journeys/meet-personality.journey.spec.ts -g "step-1"

# Run with UI
npx playwright test tests/journeys/meet-personality.journey.spec.ts --ui

# Generate HTML report
npx playwright show-report
```

## Prerequisites

1. **Companion App Running:** `cargo run --bin mbot-companion` on port 3000
2. **Playwright Installed:** `npm run playwright:install`
3. **mBot2 Connected:** Real or simulated robot connection
4. **All data-testid attributes:** Implemented in UI components

## Documentation Requirements

Per issue #29, the following documentation is required:

- [x] Journey test script created
- [x] All 6 steps implemented
- [x] All 5 presets tested
- [ ] Video documentation of journey execution
- [ ] Screenshot documentation (auto-generated by `@all-presets` test)

## Success Criteria

All tests must pass for issue #29 to be considered complete:

- All 6 individual step tests pass
- Full journey test passes
- All 5 presets are distinguishable in test
- Transitions are smooth (no jumps)
- Personality persists across reloads
- Video documentation created
- Non-developer verification completed

## Related Files

- **Test:** `tests/journeys/meet-personality.journey.spec.ts`
- **Issue:** #29 - STORY-PERS-007
- **Contract:** J-PERS-MEET-PERSONALITY
- **Rust Implementation:** `crates/mbot-core/src/personality/`
- **Presets:** `crates/mbot-core/src/personality/presets.rs`
- **Integration Tests:** `tests/integration/personality_behavior_mapping.rs`
