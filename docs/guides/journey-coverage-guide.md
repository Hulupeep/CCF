# Journey Coverage Tool Guide

**Feature:** Wave 6 Sprint 3
**Difficulty:** Intermediate
**Time:** 15 minutes

## Overview
Intelligent tool that maps journey contracts to test files, calculates coverage by criticality, and determines release readiness.

## Quick Start
\`\`\`bash
# Generate all reports
npm run coverage:journeys

# Check release readiness (exits with code 0 if ready)
npm run coverage:check

# Generate specific format
npm run coverage:journeys -- --format html
npm run coverage:journeys -- --format markdown
npm run coverage:journeys -- --format json

# Watch mode for development
bash scripts/journey-coverage.sh watch
\`\`\`

## Current Coverage
- Total Journeys: 9
- Implemented: 5 (55.6%)
- **Critical**: 5/5 passing (100.0%) ✅
- **Release Ready**: YES ✓

## Reports Generated
- `docs/journey-coverage-report.html` - Visual HTML
- `docs/JOURNEY_COVERAGE.md` - Markdown summary
- `docs/journey-coverage.json` - Machine-readable

## Release Criteria
- All Critical journeys: MUST pass
- Important journeys: SHOULD pass
- Future journeys: Optional

## API
See: [Journey Coverage API](../api/WAVE_6_APIs.md#journey-coverage)

**Last Updated:** 2026-02-01
