# Journey Test Quick Reference

## LEGO Sort Journey Test (Issue #32)

### Quick Start
```bash
# 1. Start companion app
cargo run --bin mbot-companion

# 2. Run full journey test
npm run test:journeys -- lego-sort.journey.spec.ts

# 3. Run with UI for debugging
npx playwright test tests/journeys/lego-sort.journey.spec.ts --ui
```

### Test Commands

```bash
# Full journey (all 8 steps)
npm run test:journeys -- lego-sort -g "full-journey"

# Accuracy test only
npm run test:journeys -- lego-sort -g "accuracy"

# Personality behaviors test
npm run test:journeys -- lego-sort -g "personality"

# Edge cases test
npm run test:journeys -- lego-sort -g "edge-cases"

# Performance test (< 5 min target)
npm run test:journeys -- lego-sort -g "Performance"

# With video recording
npx playwright test tests/journeys/lego-sort.journey.spec.ts --video=on
```

### Physical Setup Checklist

- [ ] White paper (A3+) on flat surface
- [ ] 10-12 LEGO bricks (red, blue, green, yellow, gold)
- [ ] 5 color zone papers arranged around perimeter
- [ ] Robot calibrated on white paper
- [ ] Battery > 50%
- [ ] Companion app running on port 3000

### Success Criteria

✅ All 8 journey steps pass
✅ Accuracy >= 90%
✅ Duration < 10 minutes
✅ Gold piece detected and celebrated
✅ Completion behavior shown

### Troubleshooting

| Problem | Solution |
|---------|----------|
| Test timeout | Reduce pieces to 8, check battery |
| Low accuracy | Re-calibrate sensor, improve lighting |
| No personality | Enable personality tracking, check settings |
| Robot won't move | Check connection, restart robot |

### Test Files

- Test: `tests/journeys/lego-sort.journey.spec.ts`
- Setup Guide: `docs/test-setup-lego-sort.md`
- Contract: `docs/contracts/feature_helperbot.yml`
- Issue: #32 - STORY-HELP-006

### Contract: J-HELP-LEGO-SORT

**DOD Criticality:** CRITICAL
**Requirements:** HELP-001 (Color Detection System)
**Invariants:** I-HELP-050 through I-HELP-054

---

For detailed setup instructions, see: `docs/test-setup-lego-sort.md`
