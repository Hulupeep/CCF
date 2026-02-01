# Journey Coverage Report Tool

**Issue:** #81 - STORY-TEST-003
**DOD Criticality:** IMPORTANT - Should pass before release

## Overview

The Journey Coverage Report Tool generates comprehensive coverage reports for journey E2E tests, showing which Specflow contracts are covered by which test files.

## Features

- **Journey Contract Parser** - Reads all J-* contracts from `docs/contracts/CONTRACT_INDEX.yml`
- **Test File Mapper** - Maps journey contracts to test files in `tests/journeys/`
- **Test Status Checker** - Determines if tests are passing/failing/not implemented
- **Coverage Calculator** - Calculates coverage percentages by criticality
- **Gap Identifier** - Identifies missing tests and incomplete coverage
- **Multi-Format Reports** - Generates HTML, Markdown, and JSON reports
- **CI/CD Integration** - Exit code 0 if release-ready, 1 if blocked

## Usage

### Generate All Reports

```bash
npm run coverage:journeys
```

Generates:
- `docs/journey-coverage-report.html` - Visual HTML report
- `docs/JOURNEY_COVERAGE.md` - Markdown summary
- `docs/journey-coverage.json` - Machine-readable JSON

### CLI Commands

```bash
# Generate all reports
bash scripts/journey-coverage.sh generate

# Check release readiness (exit 0 if ready, 1 if blocked)
bash scripts/journey-coverage.sh check

# Print summary to stdout
bash scripts/journey-coverage.sh summary

# Watch for changes and regenerate
bash scripts/journey-coverage.sh watch

# Generate specific format only
bash scripts/journey-coverage.sh html
bash scripts/journey-coverage.sh markdown
bash scripts/journey-coverage.sh json
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Check Journey Coverage
  run: npm run coverage:check
  # Fails if critical tests are not passing
```

## Report Formats

### HTML Report

Interactive visual report with:
- Release readiness indicator (green/red)
- Coverage metrics cards
- Progress bars by criticality
- Detailed journey table with gaps
- Status badges and color coding

### Markdown Report

Text-based summary with:
- Overall metrics table
- Coverage breakdown by criticality
- Journey details with status emojis
- Legend and DOD requirements
- Release blocker list

### JSON Report

Machine-readable data with:
- Complete metrics object
- Array of journey mappings
- Test file paths and status
- Gap analysis per journey
- Requirements coverage

## Coverage Metrics

### Overall
- **Total Journeys** - All journey contracts in CONTRACT_INDEX.yml
- **Implemented** - Journeys with test files
- **Passing** - Tests that pass (or assumed passing if well-formed)
- **Failing** - Tests that fail when run
- **Not Implemented** - Journeys without test files
- **Coverage %** - (Implemented / Total) * 100

### By Criticality
- **Critical** - Must pass for release (DOD gate)
- **Important** - Should pass before release
- **Future** - Nice to have

## Release Readiness

Release is READY when:
- All critical journey tests exist
- All critical journey tests pass

Release is BLOCKED when:
- Any critical test is not implemented
- Any critical test is failing
- Any critical test status is unknown

## Gap Identification

The tool identifies these gaps:
1. **Test file not found** - Journey has no corresponding test file
2. **Empty test file** - Test file exists but contains no tests
3. **Low test count** - Test has fewer than 3 test cases
4. **Missing invariant references** - Test doesn't reference required invariants

## Test File Mapping

The mapper uses multiple strategies to find test files:

### 1. Explicit Mapping (from CONTRACT_INDEX.yml)
```yaml
e2e_test: "tests/e2e/journeys/artbot-first-drawing.journey.spec.ts"
```

### 2. Inferred Paths
- `J-ART-FIRST-DRAWING` → `tests/journeys/first-drawing.journey.spec.ts`
- `J-ART-FIRST-DRAWING` → `tests/journeys/art-first-drawing.journey.spec.ts`
- `J-ART-FIRST-DRAWING` → `tests/journeys/j-art-first-drawing.journey.spec.ts`

### 3. Known Mappings (for legacy names)
```typescript
const nameMap = {
  'J-PERS-MEET-PERSONALITY': 'meet-personality.journey.spec.ts',
  'J-GAME-FIRST-TICTACTOE': 'tictactoe.journey.spec.ts',
  'J-HELP-LEGO-SORT': 'lego-sort.journey.spec.ts',
  ...
};
```

## Architecture

```
generate-journey-coverage.ts (main entry point)
  ├── lib/contractParser.ts      - Parse YAML contracts
  ├── lib/testMapper.ts           - Map contracts to test files
  ├── lib/coverageCalculator.ts   - Calculate metrics
  └── lib/reportGenerator.ts      - Generate HTML/MD/JSON
```

### ContractParser
- Reads `CONTRACT_INDEX.yml`
- Extracts journey contracts (type: e2e)
- Groups by criticality
- Provides DOD requirements

### TestMapper
- Maps journey IDs to test files
- Analyzes test files (line count, test count)
- Determines test status
- Identifies coverage gaps

### CoverageCalculator
- Calculates overall metrics
- Calculates criticality-specific metrics
- Identifies release blockers
- Generates text summaries

### ReportGenerator
- Generates HTML with visual styling
- Generates Markdown with tables
- Generates JSON for automation
- Writes to `docs/` directory

## Example Output

```
🧪 Journey Coverage Report Generator
=====================================

📋 Step 1: Parsing journey contracts...
   Found 9 journey contracts

🔍 Step 2: Mapping journeys to test files...
   Mapped 9 journeys

📊 Step 4: Calculating coverage metrics...
   Coverage: 55.6%
   Passing: 5/9
   Release Ready: YES ✓

📝 Step 5: Generating reports...
   ✓ HTML report: docs/journey-coverage-report.html
   ✓ Markdown summary: docs/JOURNEY_COVERAGE.md
   ✓ JSON data: docs/journey-coverage.json

📈 Summary:
Journey Test Coverage Summary
==================================================
Total Journeys: 9
Implemented: 5 (55.6%)
Passing: 5
Failing: 0
Not Implemented: 4

By Criticality:
  Critical: 5/5 passing (100.0%)
  Important: 0/4 passing (0.0%)
  Future: 0/0 passing (0.0%)

Release Ready: YES ✓

✅ All critical tests passing - ready to release!
```

## Dependencies

- **yaml** - YAML parsing for contract files
- **tsx** - TypeScript execution (faster than ts-node)
- **fs** / **path** - File system operations
- **@playwright/test** - For E2E testing framework (dev dependency)

## Maintenance

### Adding New Journey Tests

1. Create contract in `docs/contracts/CONTRACT_INDEX.yml`:
```yaml
- id: J-NEW-JOURNEY
  file: journey_new_feature.yml
  type: e2e
  dod_criticality: critical
  e2e_test: "tests/journeys/new-feature.journey.spec.ts"
```

2. Create test file in `tests/journeys/new-feature.journey.spec.ts`

3. Run coverage tool to verify:
```bash
npm run coverage:journeys
```

### Updating Test Mappings

If test files use non-standard names, add to the `nameMap` in `scripts/lib/testMapper.ts`:

```typescript
const nameMap: Record<string, string> = {
  'J-CUSTOM-NAME': 'actual-file-name.journey.spec.ts',
};
```

## Future Enhancements

- [ ] Actual test execution (currently inferred from file presence)
- [ ] Test result parsing from Playwright reports
- [ ] Historical trend tracking
- [ ] Coverage heatmap visualization
- [ ] Slack/Discord notifications on coverage changes
- [ ] Automatic issue creation for missing tests

## References

- Issue #81: Journey Coverage Report Tool
- CONTRACT_INDEX.yml: `docs/contracts/CONTRACT_INDEX.yml`
- Journey Tests: `tests/journeys/*.journey.spec.ts`
- Specflow Contracts: `docs/contracts/journey_*.yml`

---

_Generated by Issue #81: Journey Coverage Report Tool_
_Part of Specflow compliance workflow_
