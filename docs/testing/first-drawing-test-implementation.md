# First Drawing Journey Test - Implementation Guide

**Journey Contract:** J-ART-FIRST-DRAWING (CRITICAL)
**Issue:** #16 - STORY-ART-005: First Drawing Journey Test
**Test File:** `tests/journeys/first-drawing.journey.spec.ts`
**Status:** ✅ Implemented

## Overview

This document describes the implementation of the E2E journey test for the first drawing experience, covering the complete emotional arc from Calm → Spike → Calm with automated stimulus injection and artifact capture.

## Implementation Summary

### Test Architecture

The implementation uses three main classes:

1. **StimulusInjector** - Injects test stimuli (loud noise, quiet periods)
2. **JourneyRecorder** - Captures screenshots, records mood transitions, exports results
3. **Test Suite** - 8 individual tests plus 1 complete journey test

### Data Contracts Implemented

All data contracts from issue #16 are fully implemented:

```typescript
interface MoodTransition {
  from_mode: 'Calm' | 'Active' | 'Spike' | 'Protect';
  to_mode: 'Calm' | 'Active' | 'Spike' | 'Protect';
  timestamp: number;
  trigger: 'stimulus' | 'natural_decay' | 'session_event';
}

interface JourneyTestResult {
  journey_id: 'J-ART-FIRST-DRAWING';
  passed: boolean;
  steps_completed: number;
  total_steps: number;
  mood_transitions: MoodTransition[];
  duration_ms: number;
  artifacts: JourneyArtifact[];
}

interface JourneyArtifact {
  type: 'photo' | 'video' | 'log' | 'session_export';
  path: string;
  description: string;
}
```

## Test Scenarios Implemented

### 1. Step 1: Robot Indicates Ready State
**Gherkin Tag:** `@ART-JOURNEY-STEP-1`

**Verifies:**
- `status-ready` element visible
- LED shows "blue pulse"
- Pen status is "Up"

**Artifacts:**
- Screenshot: "Robot ready state - LED blue pulse, pen up"
- JSON export with test result

### 2. Step 2: Drawing Begins with Calm Spiral
**Gherkin Tag:** `@ART-JOURNEY-STEP-2`

**Verifies:**
- Start button clickable
- Pen lowers (status "Down")
- Drawing mode shows "Calm"
- Pattern type is "spiral"
- Drawing style contains "smooth"

**Artifacts:**
- Screenshot: "Calm spiral drawing - smooth curves"
- Mood transition: Calm → Calm (session_event)

### 3. Step 3: Loud Noise Triggers Spike Mode
**Gherkin Tag:** `@ART-JOURNEY-STEP-3`

**Verifies:**
- Stimulus injection via `StimulusInjector`
- Transition to Spike mode within 2 seconds
- Drawing style becomes "jagged"
- Max angle drawn > 45 degrees

**Artifacts:**
- Screenshot (before): "Before stimulus - Calm mode"
- Screenshot (after): "After stimulus - Spike mode with jagged lines"
- Mood transition: Calm → Spike (stimulus)

### 4. Step 4: Quiet Environment Restores Calm
**Gherkin Tag:** `@ART-JOURNEY-STEP-4`

**Verifies:**
- Natural decay after 5 seconds of quiet
- Return to Calm mode
- Drawing style returns to "smooth"
- Tension level < 0.3

**Artifacts:**
- Screenshot: "Returned to Calm mode - smooth curves restored"
- Mood transition: Spike → Calm (natural_decay)

### 5. Step 5: Session Ends with Signing
**Gherkin Tag:** `@ART-JOURNEY-STEP-5`

**Verifies:**
- Finish button triggers signing
- Robot action shows "Signing"
- Pen lifts after signature
- Session status is "Complete"

**Artifacts:**
- Screenshot: "Session complete - pen up, signature visible"

### 6. Step 6: Physical Artwork Inspection
**Gherkin Tag:** `@ART-JOURNEY-STEP-6`

**Verifies:**
- Complete emotional arc in single session
- Canvas captures all phases
- Visual inspection protocol output

**Artifacts:**
- 4 phase screenshots (Calm, Spike, Return, Signature)
- Final artwork PNG
- Console output with inspection checklist

### 7. Complete Journey: Full Emotional Arc
**Gherkin Tag:** `@ART-JOURNEY-COMPLETE`

**Verifies:**
- All 6 steps in sequence
- Complete mood transition flow
- Duration < 120 seconds
- Mood sequence includes:
  - Calm → Spike (stimulus trigger)
  - Spike → Calm (natural decay)

**Artifacts:**
- 5 step screenshots
- Complete journey artwork
- JSON export with full journey data

### 8. Error Handling: Servo Failure
**Additional coverage**

**Verifies:**
- Graceful failure on servo error
- Error message displayed
- Pen status shows "Error"

## Helper Classes

### StimulusInjector

```typescript
class StimulusInjector {
  async injectLoudNoise(intensity: number = 0.9, duration_ms: number = 1000)
  async injectQuietPeriod(duration_ms: number = 5000)
}
```

**Usage:**
```typescript
const stimulator = new StimulusInjector(page);
await stimulator.injectLoudNoise(0.9, 1000); // 90% intensity, 1 second
await stimulator.injectQuietPeriod(5000);    // 5 seconds quiet
```

**Mechanism:**
Injects events into `window.mBot.stimulusSystem` to trigger nervous system responses without external hardware.

### JourneyRecorder

```typescript
class JourneyRecorder {
  async captureScreenshot(description: string)
  async recordMoodTransition(from_mode, to_mode, trigger)
  getResult(passed, steps_completed, total_steps): JourneyTestResult
  async exportResult(result: JourneyTestResult)
}
```

**Usage:**
```typescript
const recorder = new JourneyRecorder(page, testInfo);
await recorder.captureScreenshot('Step description');
await recorder.recordMoodTransition('Calm', 'Spike', 'stimulus');
const result = recorder.getResult(true, 6, 6);
await recorder.exportResult(result);
```

**Output:**
All artifacts saved to `test-results/[test-name]/`:
- `step-N-{timestamp}ms.png` - Sequential screenshots
- `journey-result.json` - Complete test metadata

## Required data-testid Selectors

The test requires these elements in the web UI:

| Element | data-testid | Purpose |
|---------|-------------|---------|
| Status indicator | `status-ready` | Robot connection status |
| LED status | `led-status` | LED color and animation |
| Pen position | `pen-status` | Up/Down/Error |
| Start button | `start-drawing` | Begin drawing session |
| Finish button | `finish-drawing` | End session |
| Drawing mode | `drawing-mode` | Calm/Active/Spike/Protect |
| Pattern type | `pattern-type` | spiral/circle/zigzag |
| Drawing style | `drawing-style` | smooth/jagged/erratic |
| Max angle | `max-angle-drawn` | Maximum angle in degrees |
| Tension level | `tension-level` | 0.0-1.0 float |
| Robot action | `robot-action` | Current action (Signing) |
| Session status | `session-status` | Complete/Active |
| Session ID | `session-id` | Unique session identifier |
| Canvas | `drawing-canvas` | Main drawing surface |
| Error message | `error-message` | Error display |

## Running the Tests

### Run All Journey Tests
```bash
npm run test:journeys
```

### Run Specific Test
```bash
npx playwright test tests/journeys/first-drawing.journey.spec.ts
```

### Run Only Complete Journey
```bash
npx playwright test -g "Complete journey"
```

### Debug Mode
```bash
npx playwright test tests/journeys/first-drawing.journey.spec.ts --debug
```

### View Test Report
```bash
npx playwright show-report
```

## Test Artifacts Location

After running tests, artifacts are saved to:

```
test-results/
  j-art-first-drawing-first-drawing-journey-step-1-robot-indicates-ready-state-chromium/
    step-1-{timestamp}ms.png
    journey-result.json

  j-art-first-drawing-first-drawing-journey-complete-journey-full-emotional-arc-chromium/
    step-1-{timestamp}ms.png
    step-2-{timestamp}ms.png
    step-3-{timestamp}ms.png
    step-4-{timestamp}ms.png
    step-5-{timestamp}ms.png
    complete-journey-artwork.png
    journey-result.json
```

## Interpreting Results

### Success Output

```json
{
  "journey_id": "J-ART-FIRST-DRAWING",
  "passed": true,
  "steps_completed": 6,
  "total_steps": 6,
  "mood_transitions": [
    {
      "from_mode": "Calm",
      "to_mode": "Spike",
      "timestamp": 10234,
      "trigger": "stimulus"
    },
    {
      "from_mode": "Spike",
      "to_mode": "Calm",
      "timestamp": 20567,
      "trigger": "natural_decay"
    }
  ],
  "duration_ms": 47234,
  "artifacts": [
    { "type": "photo", "path": "...", "description": "..." },
    { "type": "session_export", "path": "...", "description": "..." }
  ]
}
```

**Console Output:**
```
=== JOURNEY COMPLETED SUCCESSFULLY ===
Duration: 47234ms
Steps: 6/6
Mood transitions: 2
Artifacts: 8

Expected mood flow: Calm -> Spike (stimulus) -> Calm (decay) ✓
```

### Failure Scenarios

**Incomplete Journey:**
```json
{
  "passed": false,
  "steps_completed": 3,
  "total_steps": 6,
  "duration_ms": 25000
}
```

**Missing Mood Transitions:**
- Test will fail if Calm → Spike transition missing
- Test will fail if Spike → Calm (natural_decay) missing

## Integration with CI/CD

### GitHub Actions Example

```yaml
- name: Install Playwright
  run: npm run playwright:install

- name: Run Journey Tests
  run: npm run test:journeys

- name: Upload test artifacts
  if: always()
  uses: actions/upload-artifact@v3
  with:
    name: journey-test-results
    path: test-results/
```

### Acceptance Criteria

From issue #16 Definition of Done:

- [x] Journey test file created at `tests/journeys/first-drawing.journey.spec.ts`
- [x] All 7 Gherkin scenarios implemented (6 steps + 1 complete + 1 error)
- [x] Stimulus injection framework implemented (`StimulusInjector`)
- [x] Mood transitions verified programmatically
- [x] Visual inspection protocol documented (`docs/testing/visual-inspection-protocol.md`)
- [x] Test captures photos of output (via `JourneyRecorder`)
- [x] Journey documented with example structure
- [ ] Test runs reliably (>95% pass rate) - **Requires web UI implementation**
- [x] Performance: journey completes within 120s (enforced via `test.setTimeout(120000)`)
- [ ] Code reviewed and merged to main - **Pending PR**

## Next Steps

1. **Implement Web UI** - Add required `data-testid` elements
2. **Implement Stimulus System** - Wire `window.mBot.stimulusSystem`
3. **Connect Nervous System** - Integrate mood → drawing translation
4. **Run Tests** - Execute against live system
5. **Validate Physical Output** - Follow visual inspection protocol
6. **Document Journey** - Add example photos to `docs/testing/journey-examples/`

## Related Documentation

- Visual Inspection Protocol: `docs/testing/visual-inspection-protocol.md`
- Issue #16: https://github.com/Hulupeep/mbot_ruvector/issues/16
- Journey Contract: `docs/contracts/journey_artbot.yml`
- Feature Contract: `docs/contracts/feature_artbot.yml`

## Questions or Issues

For questions about this test implementation, see:
- Issue #16 for acceptance criteria
- `docs/testing/visual-inspection-protocol.md` for physical verification
- Contract files in `docs/contracts/` for invariants
