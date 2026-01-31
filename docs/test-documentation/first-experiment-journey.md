# First Experiment Journey Test Documentation

**Journey ID:** J-LEARN-FIRST-EXPERIMENT
**Issue:** #33 - STORY-LEARN-006
**Contract:** J-LEARN-FIRST-EXPERIMENT (Critical)
**Test File:** `tests/journeys/first-experiment.journey.spec.ts`

## Overview

This journey test validates the complete first experiment flow for students using mBot's LearningLab mode. The journey guides students through discovering AI behavior by observing the robot's nervous system, experimenting with stimuli, and creating their first custom personality.

## Journey Steps

### Step 1: Student Opens Visualizer
**Learning Outcome:** "The robot has a 'brain'"

- Navigate to LearningLab mode
- Click "Start Exploring" button
- Verify nervous system visualizer displays with real-time data
- Check child-friendly labels are present

**Key Test IDs:**
- `mode-selector`
- `mode-learning-lab`
- `start-exploring`
- `nervous-system-visualizer`
- `data-streaming-indicator`

### Step 2: Observe Idle Robot Baseline
**Learning Outcome:** "It has a baseline state"

- Wait 5 seconds for robot to settle
- Verify tension is low (< 0.3)
- Verify mode is "calm"
- Check gauge stability (variability < 0.1)

**Key Test IDs:**
- `tension-value`
- `current-reflex-mode`

### Step 3: Robot Detects Stimulus
**Learning Outcome:** "It detected me!"

- Simulate ultrasonic sensor trigger (20cm distance)
- Verify tension spikes visibly (increase > 0.2)
- Verify mode changes to "active" or "spike"
- Check response time < 100ms

**Key Test IDs:**
- `simulate-stimulus`
- `stimulus-type`
- `stimulus-distance`
- `apply-stimulus`
- `response-time`

### Step 4: Robot Calms Down
**Learning Outcome:** "It calms down over time"

- Remove stimulus
- Sample tension over 5 seconds
- Verify downward trend in tension
- Verify mode returns to "calm"
- Check recovery time is 5-15 seconds

**Key Test IDs:**
- `remove-stimulus`

### Step 5: Open Personality Mixer
**Learning Outcome:** "I can adjust its 'personality'"

- Click "Adjust Personality" button
- Verify personality mixer panel opens
- Verify all 5 parameter sliders are visible
- Check labels are present

**Key Test IDs:**
- `adjust-personality`
- `personality-mixer-panel`
- `slider-startle-sensitivity`
- `slider-tension-decay`
- `slider-curiosity-gain`
- `slider-energy-baseline`
- `slider-social-response`

### Step 6: Adjust Startle Sensitivity
**Learning Outcome:** "Let's see what happens"

- Move startle_sensitivity slider to 0.8 (high)
- Verify slider value updates
- Click "Apply Parameters"
- Verify parameter is transmitted to robot

**Key Test IDs:**
- `apply-parameters`
- `parameter-sync-status`
- `startle-sensitivity-value`

### Step 7: Test with New Sensitivity
**Learning Outcome:** "Parameters change behavior!"

- Close mixer panel
- Apply same stimulus as Step 3
- Verify larger startle response (spike > 0.6)
- Compare with Step 3 baseline

**Key Test IDs:**
- `close-mixer`

### Step 8: Save Custom Personality
**Learning Outcome:** "I designed an AI"

- Click "Save Personality"
- Enter name "My First AI"
- Click confirm
- Verify save success message
- Check personality appears in library

**Key Test IDs:**
- `save-personality`
- `personality-name-input`
- `confirm-save`
- `save-success-message`
- `personality-library`

## Test Suites

### Main Journey Tests
- Individual step tests (Steps 1-8)
- Complete journey test (all steps sequenced)
- Accessibility test (WCAG 2.1 AA)
- Tablet compatibility test (768px viewport)

### Cross-Browser Tests
- Chrome (chromium)
- Firefox
- Safari (webkit)

## Invariants Tested

| Invariant | Description | Verification |
|-----------|-------------|--------------|
| I-LEARN-050 | Journey completes in < 10 minutes | `expect(duration).toBeLessThan(600000)` |
| I-LEARN-051 | Observable robot response per step | Each step checks for visible UI changes |
| I-LEARN-052 | Verifiable learning outcomes | Learning outcomes validated through UI state |
| I-LEARN-053 | Keyboard-only navigation | Accessibility test checks tab navigation |
| I-LEARN-054 | Tablet device compatibility | Dedicated tablet viewport test |

## Data Contract

```typescript
interface JourneyTestResult {
  journey_id: 'J-LEARN-FIRST-EXPERIMENT';
  executed_at: number;
  duration_ms: number;
  steps: JourneyStepResult[];
  overall_result: 'pass' | 'fail' | 'partial';
  accessibility_score: number;
  artifacts: {
    video_path?: string;
    screenshots: string[];
    logs: string[];
  };
}

interface JourneyStepResult {
  step_number: number;
  action: string;
  expected_response: string;
  actual_response: string;
  learning_outcome: string;
  result: 'pass' | 'fail' | 'skip';
  duration_ms: number;
  screenshot_path?: string;
  notes?: string;
}
```

## Running the Tests

### Install Dependencies
```bash
npm install
npm run playwright:install
```

### Run All Journey Tests
```bash
npm run test:journeys
```

### Run Specific Journey
```bash
npx playwright test first-experiment.journey.spec.ts
```

### Run Specific Browser
```bash
npx playwright test --project=chromium first-experiment.journey.spec.ts
npx playwright test --project=firefox first-experiment.journey.spec.ts
npx playwright test --project=webkit first-experiment.journey.spec.ts
```

### Run Tablet Tests
```bash
npx playwright test --project=tablet first-experiment.journey.spec.ts
```

### Run with UI Mode (Debug)
```bash
npx playwright test --ui first-experiment.journey.spec.ts
```

### Run Specific Test
```bash
npx playwright test -g "@LEARN-006-003" first-experiment.journey.spec.ts
```

## Accessibility Testing

The test includes comprehensive accessibility checks using axe-core:

- WCAG 2.1 Level A compliance
- WCAG 2.1 Level AA compliance
- Keyboard navigation validation
- Focus indicator visibility
- Screen reader compatibility
- Color contrast validation

## Test Artifacts

Each test run produces:

1. **Screenshots:** Captured for each step
   - Location: `test-results/journey-step-{N}.png`
   - Full-page screenshots

2. **Videos:** Recorded on failure
   - Location: `test-results/` (automatic)
   - Configurable retention policy

3. **Traces:** Captured on retry
   - Location: `test-results/` (automatic)
   - Viewable with `npx playwright show-trace`

4. **Journey Results:** JSON output
   - Console output after each test
   - Contains all step results and metrics

## CI/CD Integration

### GitHub Actions Configuration

```yaml
- name: Install dependencies
  run: npm ci

- name: Install Playwright browsers
  run: npx playwright install --with-deps

- name: Run journey tests
  run: npm run test:journeys

- name: Upload test results
  if: always()
  uses: actions/upload-artifact@v3
  with:
    name: playwright-results
    path: test-results/
```

### Required Environment
- Node.js 18+
- 2GB RAM minimum
- Robot connection mocked in CI

## Troubleshooting

### Test Timeout
**Issue:** Journey exceeds 10-minute limit
**Solution:** Check for blocked async operations or slow robot responses

### Robot Not Connected
**Issue:** `mbot-status` shows disconnected
**Solution:** Ensure mock robot is initialized in test environment

### Accessibility Violations
**Issue:** WCAG violations detected
**Solution:** Review violation details in console output and fix UI components

### Slider Interaction Failure
**Issue:** Slider doesn't update on fill()
**Solution:** Verify slider is visible and not disabled before interaction

### Cross-Browser Failures
**Issue:** Tests pass in Chrome but fail in Firefox/Safari
**Solution:** Check for browser-specific CSS or API differences

## Test Maintenance

### When to Update

1. **UI Changes:** Update `data-testid` selectors
2. **Flow Changes:** Update step sequences
3. **New Features:** Add new test cases
4. **Invariant Changes:** Update assertions

### Best Practices

1. Keep `data-testid` values stable
2. Use descriptive test names
3. Document learning outcomes clearly
4. Maintain screenshot artifacts
5. Update documentation with code changes

## Success Criteria

Journey test is considered successful when:

- ✅ All 8 steps pass individually
- ✅ Complete journey test passes
- ✅ Accessibility score > 90%
- ✅ Total duration < 10 minutes
- ✅ Tests pass on Chrome, Firefox, Safari
- ✅ Tests pass on tablet viewport
- ✅ All learning outcomes verified

## Related Documentation

- Issue: [#33 - STORY-LEARN-006](https://github.com/Hulupeep/mbot_ruvector/issues/33)
- Contract: `docs/contracts/journey_learn.yml`
- Parent Epic: [#5 - EPIC-005: LearningLab](https://github.com/Hulupeep/mbot_ruvector/issues/5)
- Test Infrastructure: `playwright.config.ts`
- Package Config: `package.json`

## Version History

- **v1.0.0** (2026-01-31): Initial comprehensive implementation
  - All 8 journey steps
  - Accessibility testing
  - Multi-browser support
  - Tablet compatibility
  - Full documentation
