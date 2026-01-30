# Journey Tests

End-to-end journey tests for mBot RuVector. These tests validate complete user workflows from start to finish.

## Running Tests

```bash
# Install Playwright browsers (first time only)
npm run playwright:install

# Run all journey tests
npm run test:journeys

# Run specific journey
npx playwright test tests/journeys/first-drawing.journey.spec.ts

# Run with UI mode
npx playwright test --ui

# Debug a test
npx playwright test --debug tests/journeys/tictactoe.journey.spec.ts
```

## Journey Coverage

| Journey ID | Test File | Issue | Status |
|------------|-----------|-------|--------|
| J-ART-FIRST-DRAWING | `first-drawing.journey.spec.ts` | #16 | ⚠️ Template |
| J-PERS-MEET-PERSONALITY | `meet-personality.journey.spec.ts` | #29 | ⚠️ Template |
| J-SORT-RESET | `lego-sort.journey.spec.ts` | #32, #55 | ⚠️ Template |
| J-LEARN-FIRST-EXPERIMENT | `first-experiment.journey.spec.ts` | #33 | ⚠️ Template |
| J-GAME-TICTACTOE | `tictactoe.journey.spec.ts` | #37 | ⚠️ Template |

## Test Structure

Each journey test follows this pattern:

```typescript
/**
 * Journey Test: J-JOURNEY-ID
 * Issue: #XX - Story title
 *
 * Scenario: User action description
 *   Given precondition
 *   When action
 *   Then expected result
 */

test.describe('J-JOURNEY-ID: Journey Name', () => {
  test('main happy path', async ({ page }) => {
    // Test implementation
  });

  test('error handling', async ({ page }) => {
    // Error scenarios
  });
});
```

## Prerequisites

Before running journey tests:

1. **Frontend must be running** - Tests expect `http://127.0.0.1:3000`
2. **mBot companion app** - Backend services must be available
3. **Test data** - Some tests require specific setup (see individual test files)

## data-testid Convention

All tests use `data-testid` attributes for stable selectors:

```typescript
// Good: Stable, semantic selector
await page.getByTestId('start-drawing').click();

// Bad: Fragile CSS selector
await page.locator('.btn-primary').first().click();
```

See issue templates for required `data-testid` values per feature.

## Debugging Failed Tests

```bash
# Run test with trace
npx playwright test --trace on

# View trace in UI
npx playwright show-trace trace.zip

# Run with headed browser
npx playwright test --headed

# Run with slow motion
npx playwright test --headed --slow-mo=500
```

## Definition of Done

A journey test is complete when:
- [ ] All Gherkin scenarios from the issue are covered
- [ ] Happy path test passes
- [ ] Error handling tests pass
- [ ] All required `data-testid` selectors are used
- [ ] Test runs in < 60 seconds
- [ ] No flaky behavior (passes 10 times in a row)

## Contract Validation

Journey tests also verify contract compliance:
- Personality bounds (0.0-1.0)
- Servo angles (0-180)
- Motor speeds (-100 to 100)
- Response times (< 100ms for reflexes)

See `docs/contracts/` for full contract specifications.
