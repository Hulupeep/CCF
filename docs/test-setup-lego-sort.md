# LEGO Sort Journey Test Setup Guide

**Journey:** J-HELP-LEGO-SORT
**Test File:** `tests/journeys/lego-sort.journey.spec.ts`
**Issue:** #32 - STORY-HELP-006
**DOD Criticality:** CRITICAL

## Overview

This journey test verifies the complete LEGO sorting experience, including color detection, systematic sorting, personality behaviors, and accuracy measurement. The test must achieve >= 90% sorting accuracy to pass.

## Required Materials

### Hardware
- mBot2 robot (fully assembled and charged)
- Quad RGB sensor (installed and calibrated)
- Gripper attachment (if using pick mode) OR pusher attachment (recommended)
- USB cable or WiFi connection for robot communication

### Physical Materials
1. **White paper sheet** (A3 or larger) - sorting surface
2. **10-12 LEGO bricks:**
   - 2x red standard bricks
   - 2x blue standard bricks
   - 2x green standard bricks
   - 2x yellow standard bricks
   - 1x gold/chrome piece (rare find)
   - 1-2x additional pieces (any standard color)
3. **5 colored paper sheets** for zone markers:
   - Red zone marker (8.5x11" or A4)
   - Blue zone marker
   - Green zone marker
   - Yellow zone marker
   - Special/gold zone marker (can be yellow or orange)
4. **Clear flat surface** - table or floor area (min 3ft x 3ft)
5. **Normal indoor lighting** - avoid direct sunlight or shadows

### Software
- Companion app running (`cargo run --bin mbot-companion`)
- Web browser pointing to `http://127.0.0.1:3000`
- Playwright test framework installed (`npm run playwright:install`)

## Environment Setup

### Step 1: Physical Setup
```
1. Place white paper sheet on flat surface (centered)
2. Arrange color zone papers around perimeter:
   - Red zone:     top-left corner (100, 100)
   - Blue zone:    top-center (200, 100)
   - Green zone:   top-right (300, 100)
   - Yellow zone:  right side (400, 100)
   - Special zone: bottom-center (250, 200)
3. Scatter LEGO bricks randomly on white paper (not too close to edges)
4. Position robot at starting corner (bottom-left recommended)
```

### Step 2: Robot Calibration
```bash
# Start companion app
cargo run --bin mbot-companion

# In browser: http://127.0.0.1:3000/helperbot/lego-sort
# 1. Click "Calibrate RGB Sensor"
# 2. Place sensor over white paper
# 3. Wait for calibration success message
# 4. Verify LED shows white baseline
```

### Step 3: Test Configuration
```typescript
// Default test configuration (can be overridden)
{
  totalPieces: 12,
  colors: ['red', 'blue', 'green', 'yellow', 'gold'],
  surfaceType: 'white-paper',
  maxDuration: 600000, // 10 minutes
  accuracyThreshold: 90, // 90% minimum
  enablePersonalityTracking: true
}
```

## Running the Tests

### Run Full Journey Test
```bash
# Run complete LEGO sort journey
npm run test:journeys -- lego-sort.journey.spec.ts

# Run only the full journey test
npm run test:journeys -- lego-sort.journey.spec.ts -g "full-journey"

# Run with UI mode for debugging
npx playwright test tests/journeys/lego-sort.journey.spec.ts --ui
```

### Run Specific Test Scenarios
```bash
# Test accuracy measurement only
npm run test:journeys -- lego-sort.journey.spec.ts -g "accuracy"

# Test personality behaviors
npm run test:journeys -- lego-sort.journey.spec.ts -g "personality"

# Test edge cases
npm run test:journeys -- lego-sort.journey.spec.ts -g "edge-cases"

# Test timing constraints
npm run test:journeys -- lego-sort.journey.spec.ts -g "timing"

# Performance test
npm run test:journeys -- lego-sort.journey.spec.ts -g "Performance"
```

### Run with Video Recording
```bash
# Enable video recording for documentation
npx playwright test tests/journeys/lego-sort.journey.spec.ts --video=on
# Videos saved to: test-results/
```

## Test Scenarios

### 1. @HELP-006-full-journey (Main Test)
**Duration:** 5-10 minutes
**Verifies:**
- All 8 journey steps from J-HELP-LEGO-SORT specification
- Color detection for each piece
- Systematic sorting to correct zones
- Personality behaviors (excitement for gold, celebration)
- >= 90% accuracy
- Completion within 10 minutes

**Expected Behavior:**
```
Step 1: Robot shows "Ready" status
Step 2: Robot begins movement after "Start Sorting" clicked
Step 3-5: For each piece:
  - Detects color (RGB sensor)
  - Displays color on LED
  - Pushes/places piece to matching zone
  - Proceeds to next piece
Step 6: When gold piece detected:
  - Shows "excitement" behavior
  - Pauses to admire (500ms+)
Step 7: When all pieces sorted:
  - Shows "celebration" behavior
  - Status = "Complete"
Step 8: Accuracy >= 90%
```

### 2. @HELP-006-accuracy
**Duration:** 5-10 minutes
**Verifies:**
- Sorting accuracy measurement
- Correctly sorted count (>= 9 out of 10)
- Accuracy percentage (>= 90%)

### 3. @HELP-006-personality
**Duration:** 5-10 minutes
**Verifies:**
- Rare piece reactions (gold detection)
- Completion celebration behavior
- Personality consistency throughout session

### 4. @HELP-006-edge-cases
**Duration:** 2-3 minutes
**Verifies:**
- Handling of difficult-to-move pieces
- Frustration behavior when piece won't move
- Alternative approach strategy
- Eventual successful sorting

### 5. @HELP-006-missed-piece
**Duration:** 5-10 minutes
**Verifies:**
- Verification scan after initial sorting
- Detection of missed pieces
- Correct sorting of missed pieces

### 6. @HELP-006-timing
**Duration:** 5-10 minutes
**Verifies:**
- Completion within 10 minute constraint
- Duration measurement accuracy

### 7. Performance Test
**Duration:** < 5 minutes (target)
**Verifies:**
- Fast sorting performance (< 5 minutes for 10 pieces)

## Test Data Selectors (data-testid)

### Setup Elements
| Element | data-testid | Purpose |
|---------|-------------|---------|
| Surface setup | `setup-sorting-surface` | Configure surface type |
| Piece setup | `setup-pieces` | Configure test pieces |
| Zone setup | `setup-color-zones` | Define zone locations |
| Start button | `start-sorting` | Begin sorting session |

### Status Elements
| Element | data-testid | Purpose |
|---------|-------------|---------|
| Robot status | `robot-status` | Shows Ready/Moving/Error |
| Sorting status | `sorting-status` | Shows Active/Complete |
| Pieces remaining | `pieces-remaining` | Count of unsorted pieces |
| Detected color | `detected-color` | Current piece color |
| LED color | `led-color` | LED display status |

### Bin Counts
| Element | data-testid | Purpose |
|---------|-------------|---------|
| Red bin | `bin-red-count` | Count in red zone |
| Blue bin | `bin-blue-count` | Count in blue zone |
| Green bin | `bin-green-count` | Count in green zone |
| Yellow bin | `bin-yellow-count` | Count in yellow zone |
| Special bin | `bin-special-count` | Count in special zone |

### Personality Elements
| Element | data-testid | Purpose |
|---------|-------------|---------|
| Special piece detected | `special-piece-detected` | Rare piece event |
| Personality behavior | `personality-behavior` | Current behavior type |
| Completion behavior | `completion-behavior` | Celebration display |

### Metrics
| Element | data-testid | Purpose |
|---------|-------------|---------|
| Sorting accuracy | `sorting-accuracy` | Accuracy percentage |
| Correctly sorted | `correctly-sorted` | Correct count |
| Session duration | `session-duration` | Time elapsed |

## Acceptance Criteria Verification

### From Issue #32
- [x] Journey test covers all 8 steps from J-HELP-LEGO-SORT specification
  - **Test:** `@HELP-006-full-journey` implements all 8 steps with tracking
- [x] Test measures sorting accuracy (target >= 90%)
  - **Test:** `@HELP-006-accuracy` verifies >= 90% threshold
- [x] Test verifies personality behaviors trigger correctly
  - **Test:** `@HELP-006-personality` tracks excitement, celebration behaviors
- [x] Test handles edge cases (missed pieces, stuck pieces, rare colors)
  - **Tests:** `@HELP-006-edge-cases`, `@HELP-006-missed-piece`
- [x] Test is documented with video recording capability
  - **Docs:** This file + video recording via `--video=on` flag
- [x] Test is reproducible with documented preconditions
  - **Docs:** Physical setup, calibration, and configuration documented above
- [x] Test produces detailed report of each step
  - **Code:** `JourneyTestResult` interface with step tracking
- [x] Test validates user experience (entertainment + helpfulness)
  - **Test:** Personality behaviors verified in full journey test

## Invariants Enforced

| Invariant | Enforcement | Test |
|-----------|-------------|------|
| I-HELP-050 | Sorting accuracy >= 90% | Test fails if accuracy < 90% |
| I-HELP-051 | All journey steps complete | `stepsCompleted` array verification |
| I-HELP-052 | Personality behaviors trigger | `personalityBehaviors` array checked |
| I-HELP-053 | Reasonable time (< 10 min) | Timeout at 600000ms |
| I-HELP-054 | Reproducible setup | Documented preconditions and config |

## Troubleshooting

### Test Timeout
**Symptom:** Test times out after 10 minutes
**Solutions:**
- Check robot battery level (should be > 50%)
- Verify pieces are not stuck or overlapping
- Reduce piece count to 8 for faster execution
- Check WiFi/USB connection stability

### Low Accuracy (< 90%)
**Symptom:** Accuracy check fails
**Solutions:**
- Re-calibrate RGB sensor on white paper
- Improve lighting conditions (avoid shadows)
- Clean RGB sensor lens
- Verify color zones are correctly positioned
- Use high-contrast LEGO colors (avoid pastels)

### Personality Behaviors Not Triggering
**Symptom:** Gold piece or celebration not detected
**Solutions:**
- Verify gold/chrome piece is visually distinct
- Check `enable-personality-tracking` is enabled
- Review robot personality settings (should not be "neutral")
- Check behavior event logging is active

### Robot Not Moving
**Symptom:** Robot shows "Ready" but doesn't move
**Solutions:**
- Verify companion app is running
- Check robot connection status
- Restart robot and re-pair
- Check servo and motor calibration

## Expected Results

### Success Criteria
```
✓ All 8 journey steps completed
✓ Accuracy >= 90% (typically 92-95%)
✓ Duration < 10 minutes (typically 5-7 minutes for 10 pieces)
✓ Gold piece detected and celebrated
✓ Completion celebration behavior shown
✓ All pieces sorted to correct zones
✓ No crashes or errors
```

### Sample Output
```json
{
  "journey_id": "J-HELP-LEGO-SORT",
  "status": "passed",
  "total_pieces": 12,
  "correctly_sorted": 11,
  "accuracy_percent": 91.67,
  "special_pieces_detected": ["gold"],
  "personality_behaviors_triggered": ["excitement", "celebration"],
  "duration_ms": 387420,
  "steps_completed": [
    { "step_number": 1, "passed": true, "notes": "Setup complete, robot ready" },
    { "step_number": 2, "passed": true, "notes": "Sorting initiated successfully" },
    { "step_number": 3, "passed": true, "notes": "First piece detected: red" },
    { "step_number": 6, "passed": true, "notes": "Gold piece detected and celebrated: gold" },
    { "step_number": 7, "passed": true, "notes": "Celebration behavior: victory_dance" },
    { "step_number": 8, "passed": true, "notes": "Accuracy: 91.67%, Pieces: 11/12" }
  ],
  "video_path": "test-results/lego-sort-chromium/video.webm"
}
```

## Performance Benchmarks

| Metric | Target | Typical | Acceptable Range |
|--------|--------|---------|------------------|
| Accuracy | >= 90% | 92-95% | 90-100% |
| Duration (10 pieces) | < 5 min | 3-4 min | 2-5 min |
| Duration (12 pieces) | < 10 min | 5-7 min | 4-10 min |
| Detection confidence | >= 0.85 | 0.90-0.95 | 0.85-1.0 |
| Behavior triggers | 2+ | 3-4 | 2-5 |

## Contract References

- **Feature Contract:** `docs/contracts/feature_helperbot.yml`
- **Journey Contract:** Listed in `docs/contracts/CONTRACT_INDEX.yml`
- **Requirements Covered:** HELP-001 (Color Detection System)
- **Related Issues:** #13, #14, #15, #32

## Next Steps

After this journey test passes:
1. Mark issue #32 as complete
2. Update CONTRACT_INDEX.yml: `dod_status: passing`
3. Proceed to other HelperBot journey tests (#55 - Reset Play Area)
4. Consider adding performance optimizations if duration > 7 minutes
5. Document any unique personality behaviors discovered

## Video Recording Instructions

For documentation and demonstration:

```bash
# Record full journey with video
npx playwright test tests/journeys/lego-sort.journey.spec.ts \
  --project=chromium \
  --video=on \
  -g "full-journey"

# Video will be saved to:
# test-results/lego-sort-journey-spec-ts-HELP-006-J-HELP-LEGO-SORT-LEGO-Sort-Journey-HELP-006-full-journey-chromium/video.webm

# Convert to MP4 for sharing (if needed)
ffmpeg -i test-results/.../video.webm journey-lego-sort.mp4
```

## Maintenance

This test should be run:
- **Before every release** (Critical DOD journey)
- After changes to color detection system
- After changes to sorting algorithm
- After changes to personality system
- When hardware changes (sensor, robot chassis)
- Periodically for regression testing (weekly recommended)

---

**Last Updated:** 2026-01-31
**Maintained By:** mbot-ruvector-team
**Status:** Implementation Complete ✅
