# Specflow Workflow

## What You Do

```
1. Point at issues     →  "Make #107-#112 specflow-compliant"
2. Generate contracts  →  "Create YAML contracts for these"
3. Execute build       →  "Run the sprint"
4. Validate & close    →  "Check readiness, close tickets"
```

The agents handle the details. You provide direction and review results.

---

## Example Session

### Morning: Prep the Sprint

```
YOU: "Run board-auditor on all open issues labeled 'admin'"

CLAUDE: [Spawns board-auditor agent]

AGENT RETURNS:
  Compliance Matrix:
  #107 Zone CRUD         Ghk=Y SQL=Y RLS=N TSi=Y Jrn=N  → Partial
  #108 Zone Reorder      Ghk=Y SQL=Y RLS=Y TSi=Y Jrn=N  → Partial
  #109 Default Zone      Ghk=N SQL=Y RLS=Y TSi=N Jrn=N  → Partial

  3 issues need work before sprint.

YOU: "Run specflow-uplifter on #107, #108, #109 — add missing sections"

CLAUDE: [Spawns 3 specflow-uplifter agents in parallel]

AGENTS RETURN:
  #107: Added RLS policy, added J-ADM-ZONE-CRUD journey reference
  #108: Added J-ADM-ZONE-CRUD journey reference
  #109: Added Gherkin scenarios, TypeScript interface, journey reference
```

### Mid-Morning: Generate Contracts

```
YOU: "Generate YAML contracts for the admin zone features"

CLAUDE: [Spawns contract-generator agent]

AGENT RETURNS:
  Created:
  - docs/contracts/feature_admin_zones.yml (ADM-003, ADM-004, ADM-006)
  - docs/contracts/journey_admin_zone_crud.yml (J-ADM-ZONE-CRUD)
  - Updated CONTRACT_INDEX.yml

YOU: "Generate Jest tests for the contracts"

CLAUDE: [Spawns contract-test-generator agent]

AGENT RETURNS:
  Created:
  - src/__tests__/contracts/admin_zones.test.ts

  Run: npm test -- contracts
```

### Afternoon: Build

```
YOU: "Map dependencies and show me the sprint waves"

CLAUDE: [Spawns dependency-mapper agent]

AGENT RETURNS:
  Wave 0 (no dependencies): #109 (default zone trigger)
  Wave 1 (depends on #109): #107 (zone CRUD), #108 (zone reorder)

YOU: "Execute wave 0, then wave 1. Last migration was 035."

CLAUDE: [Spawns sprint-executor, which dispatches:]
  Wave 0:
    - migration-builder for #109 → creates 036_default_zone_trigger.sql

  Wave 1 (after 0 completes):
    - migration-builder for #107 → creates 037_zone_crud.sql
    - migration-builder for #108 → creates 038_zone_reorder.sql
    - frontend-builder for #107 → creates useZones hook
    - frontend-builder for #108 → creates useZoneReorder hook

BUILD: npm test -- contracts
  ✓ ADM-003: Zone minimum constraint
  ✓ ADM-004: Zone name uniqueness
  ✓ ADM-006: Admin mutations audited
```

### End of Day: Validate & Close

```
YOU: "Validate and check release readiness"

CLAUDE: [Spawns contract-validator, then journey-enforcer]

RETURNS:
  Contracts: All satisfied
  Critical Journeys:
  - J-ADM-ZONE-CRUD: ✓ passing
  - J-LEAVE-LIFECYCLE: ✓ passing
  - J-PAYROLL-EXPORT: ⚠️ no test

  Status: NOT READY (1 critical journey untested)

YOU: "Generate the payroll export journey test"

CLAUDE: [Spawns journey-tester agent]

YOU: "Now close #107, #108, #109"

CLAUDE: [Spawns ticket-closer agent]
```

---

## The Pattern

Every interaction:

```
YOU: [Direction] "Do X on issues Y"
     ↓
CLAUDE: [Spawns appropriate agent(s)]
     ↓
AGENT: [Does work, returns result]
     ↓
YOU: [Review, next direction]
```

You're directing traffic, not writing code.

---

## When Things Go Wrong

**Build fails:**
```
CONTRACT VIOLATION: ADM-006 - missing audit call
  src/features/zones/hooks/useDeleteZone.ts:42
```
→ "Fix the violation in useDeleteZone"

**Journey fails:**
```
Step 3: Default space auto-created → FAILED
  Element not found: space-card
```
→ "The default space isn't being created. Check the trigger."

**Issue missing specs:**
```
#112 Zone Colors - Non-compliant (missing: Gherkin, SQL)
```
→ "Run specflow-writer on #112"

---

## Checklist

**Before sprint:**
- [ ] All issues have Gherkin, SQL, RLS, TypeScript, journey refs
- [ ] YAML contracts generated
- [ ] Jest tests generated
- [ ] Dependencies mapped

**Before release:**
- [ ] `npm test -- contracts` passes
- [ ] All critical journey tests exist and pass

---

## The Magic

You never write specs by hand. Agents generate them.
You never write contracts by hand. Agents generate them.
You never write tests by hand. Agents generate them.
You never track dependencies by hand. SQL REFERENCES tells us.

You point. Agents work. Build catches mistakes.
