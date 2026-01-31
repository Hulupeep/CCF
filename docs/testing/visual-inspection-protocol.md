# Visual Inspection Protocol - First Drawing Journey

**Journey Contract:** J-ART-FIRST-DRAWING (CRITICAL)
**Issue:** #16 - STORY-ART-005: First Drawing Journey Test
**Last Updated:** 2026-01-31

## Overview

This protocol guides manual verification of physical robot artwork to validate the complete emotional journey captured during automated E2E testing.

## Purpose

While automated tests verify digital UI elements and mood transitions programmatically, physical robot output requires human visual inspection to confirm that mood states translate correctly to mechanical drawing behavior.

## Test File Location

`tests/journeys/first-drawing.journey.spec.ts`

## Artifacts Generated

Each test run generates artifacts in Playwright's test output directory:

```
test-results/
  j-art-first-drawing-{test-name}/
    step-1-{timestamp}ms.png       # Robot ready state
    step-2-{timestamp}ms.png       # Calm drawing phase
    step-3-{timestamp}ms.png       # Spike mode triggered
    step-4-{timestamp}ms.png       # Return to calm
    step-5-{timestamp}ms.png       # Session complete
    final-artwork.png              # Complete canvas capture
    complete-journey-artwork.png   # Full E2E test artwork
    journey-result.json            # Test metadata and transitions
```

## Inspection Checklist

### Section 1: Start (Calm Mode)

**Expected Pattern:** Smooth spirals or curves

- [ ] Lines are continuous and flowing
- [ ] Curves have consistent radius
- [ ] No sharp angles or jagged transitions
- [ ] Pen pressure appears even
- [ ] Pattern resembles relaxed, deliberate movement

**Photo Reference:** `step-2-*.png` or `Section 1: Smooth spirals (Calm)`

### Section 2: Middle (Spike Mode Transition)

**Expected Pattern:** Jagged, angular marks

- [ ] Sharp angles present (visually >45 degrees)
- [ ] Lines show abrupt direction changes
- [ ] Pattern indicates "startled" response
- [ ] Clear visual contrast with Calm section
- [ ] Transition occurs within ~2 seconds of stimulus

**Photo Reference:** `step-3-*.png` or `Section 2: Jagged transitions (Spike)`

### Section 3: End (Return to Calm)

**Expected Pattern:** Return to smooth curves

- [ ] Lines become smoother again
- [ ] Gradual transition from jagged to smooth
- [ ] Pattern stabilizes over ~5 seconds
- [ ] Final curves resemble initial Calm section
- [ ] No residual jitter or nervousness

**Photo Reference:** `step-4-*.png` or `Section 3: Return to smooth`

### Section 4: Signature

**Expected Pattern:** Distinctive signing behavior

- [ ] Signature is present at end of drawing
- [ ] Signature is legible or recognizable
- [ ] Pen lifted cleanly after signature
- [ ] No smudging or dragging

**Photo Reference:** `step-5-*.png` or `Section 4: Signature`

## Verification Steps

### 1. Setup

- Locate test output directory in `test-results/`
- Open `journey-result.json` to view test metadata
- Review mood transition timestamps

### 2. Digital Verification

Review captured screenshots:
- Compare `step-2` (before stimulus) vs `step-3` (after stimulus)
- Verify visual difference between Calm and Spike modes
- Check that `final-artwork.png` shows complete emotional arc

### 3. Physical Verification (If Available)

If testing with physical robot:
- Place completed artwork on flat, well-lit surface
- Photograph artwork with smartphone or camera
- Compare physical output to captured screenshots
- Mark checklist items above

### 4. Mood Transition Validation

Open `journey-result.json` and verify:

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
      "timestamp": 10000,
      "trigger": "stimulus"
    },
    {
      "from_mode": "Spike",
      "to_mode": "Calm",
      "timestamp": 20000,
      "trigger": "natural_decay"
    }
  ],
  "duration_ms": 45000,
  "artifacts": [...]
}
```

**Required transitions:**
- [ ] At least one `Calm -> Spike` transition with trigger `stimulus`
- [ ] At least one `Spike -> Calm` transition with trigger `natural_decay`
- [ ] Total journey duration < 120,000ms (120 seconds)

## Pass/Fail Criteria

### PASS Requirements

All of the following must be true:

1. Automated test reports `"passed": true`
2. All 6 journey steps completed
3. Mood transitions include Calm → Spike → Calm sequence
4. Visual inspection checklist has >80% items checked
5. Physical output (if available) matches expected patterns
6. Duration < 120 seconds

### FAIL Scenarios

Test fails if any of:

- Mood transition missing or incorrect sequence
- No visible difference between Calm and Spike sections
- Pen did not lift at end
- No signature present
- Duration exceeds 120 seconds
- Any step throws exception

## Stimulus Injection Verification

The test uses `StimulusInjector` to trigger mood changes:

```typescript
// Loud noise (90% intensity, 1 second)
await stimulator.injectLoudNoise(0.9, 1000);

// Quiet period (5 seconds)
await stimulator.injectQuietPeriod(5000);
```

**Verify in browser console:**
```javascript
window.mBot?.stimulusSystem?.inject({
  type: 'sound',
  intensity: 0.9,
  duration_ms: 1000,
  timestamp: Date.now()
});
```

## Common Issues

### Issue: No Spike Transition Observed

**Symptoms:** Drawing remains smooth throughout
**Possible Causes:**
- Stimulus system not wired to nervous system
- Threshold too high for 0.9 intensity stimulus
- Nervous system not updating drawing engine

**Debug:**
- Check `window.mBot.stimulusSystem` exists
- Verify `drawing-mode` element updates to "Spike"
- Review nervous system integration

### Issue: Pen Does Not Lift

**Symptoms:** Pen status stays "Down" after finish
**Possible Causes:**
- Servo control not integrated
- Signing behavior not implemented
- Error in finish handler

**Debug:**
- Check `pen-status` element value
- Verify servo commands sent
- Review servo calibration

### Issue: Signature Missing

**Symptoms:** No visible signature at end
**Possible Causes:**
- Signing behavior not implemented
- Pen lifted before signature
- Canvas cropping signature area

**Debug:**
- Extend canvas capture area
- Increase wait time after "Finish"
- Check signing coordinates

## Document Journey Results

After completing inspection:

1. Take photo of physical output (if applicable)
2. Save photo to `docs/testing/journey-examples/`
3. Update this document with:
   - Date tested
   - Pass/fail result
   - Notable observations
   - Any deviations from expected behavior

## Example Journey Result

**Date:** 2026-01-31
**Test Run:** J-ART-FIRST-DRAWING Complete Journey
**Result:** PASS
**Duration:** 47,234ms
**Observations:**
- Clear visual contrast between Calm (smooth spirals) and Spike (jagged zigzags)
- Mood transition occurred within 1.2s of stimulus injection
- Return to calm took ~4.8s (natural decay)
- Signature legible and correctly positioned

**Photos:**
- `journey-examples/2026-01-31-first-drawing-pass.jpg`

## Related Contracts

- **J-ART-FIRST-DRAWING:** Complete first drawing journey (CRITICAL)
- **ART-001:** Mood-to-Movement Translation
- **ART-002:** Pen Servo Control
- **ART-005:** Emotional Art Session

## References

- Issue: #16 (STORY-ART-005)
- Test file: `tests/journeys/first-drawing.journey.spec.ts`
- Architecture: `docs/contracts/feature_artbot.yml`
