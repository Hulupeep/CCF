# mBot RuVector - Development Guide

## 🚨 RULE 1: No Ticket = No Code

**ALL work MUST have a GitHub issue before ANY code is written.**

```
NO TICKET → NO CODE
```

### Before Starting ANY Task

1. Check issues: `gh issue list -R Hulupeep/mbot_ruvector`
2. Find or create ticket with acceptance criteria
3. Reference issue in commits: `feat(scope): description (#123)`
4. Link PRs to the issue they close

### Issue Format (Specflow-Compliant)

Every issue MUST have these sections:

```markdown
## Description
Brief description of the work

## DOD Criticality
- [ ] **Critical** - Blocks release if failing
- [ ] **Important** - Should pass before release
- [ ] **Future** - Can release without

## Contract References
- **Feature Contracts:** [ARCH-001, ART-001]
- **Journey Contract:** [J-ART-FIRST-DRAWING]

## Acceptance Criteria (Gherkin)
Scenario: [Scenario name]
  Given [precondition]
  When [action]
  Then [expected result]

## data-testid Requirements
| Element | data-testid | Purpose |
|---------|-------------|---------|
| Start button | `start-drawing` | Begin drawing session |

## E2E Test File
`tests/journeys/[feature].journey.spec.ts`
```

**Quick Checklist** before submitting issues:
- [ ] Has Gherkin acceptance criteria
- [ ] Lists all data-testid selectors
- [ ] References applicable contracts
- [ ] Names the E2E test file

---

## 🚨 RULE 2: Contracts Are Non-Negotiable

This project uses **Specflow contracts** (`docs/contracts/*.yml`) enforced by tests.

**The rule:** Violate a contract → build fails → PR blocked.

### Before Modifying Code

1. Check if file is protected (see table below)
2. Read the contract in `docs/contracts/`
3. Run `cargo test` or `npm test -- contracts`
4. Fix any `CONTRACT VIOLATION` errors

### Protected Files

| Files | Contract | Key Rules |
|-------|----------|-----------|
| `src/mbot-core/**` | `feature_architecture.yml` | ARCH-001: no_std compatible |
| `src/**/*.rs` | `feature_architecture.yml` | ARCH-002: Deterministic nervous system |
| `src/**/*.rs` | `feature_architecture.yml` | SAFE-001: Kitchen Table Test |
| `src/personality/**` | `feature_personality.yml` | PERS-001: Bounded parameters |

### Contract Violation Example

```
❌ CONTRACT VIOLATION: ARCH-001
File: src/mbot-core/homeostasis.rs
Pattern: Found std:: usage in no_std module
See: docs/contracts/feature_architecture.yml
```

### Override Protocol

Only humans can override. User must say:
```
override_contract: [contract_id]
```

Then Claude MUST:
1. Explain what rule is broken and why
2. Warn about consequences
3. Ask if contract should be updated

---

## 🚨 RULE 3: Tests Must Pass Before Closing

Work is NOT complete until tests pass.

### After ANY Code Changes

```bash
# 1. Rust tests
cargo test

# 2. Contract tests (pattern enforcement)
npm test -- contracts

# 3. E2E tests (user journeys)
npm run test:journeys
```

**Do NOT mark work complete if tests fail.**

---

## 🤖 Claude Flow Swarm Integration

This project uses **Claude Flow** for parallel agent execution and memory coordination.

### Quick Commands

```bash
# Check status
npx @claude-flow/cli@latest swarm status

# Search memory for patterns
npx @claude-flow/cli@latest memory search --query "personality data structure"

# Spawn an agent
npx @claude-flow/cli@latest agent spawn -t coder --name "dev-1" --task "Implement #12"

# Run pre-task hook (get agent recommendations)
npx @claude-flow/cli@latest hooks pre-task --description "implement personality system"
```

### Sprint Waves (Stored in Memory)

```
Wave 0: #12, #35, #48, #50, #8, #13, #15, #24, #25, #28, #31 (11 parallel)
Wave 1: #14, #18, #23, #7, #20, #19, #27, #30, #49 (9 parallel)
Wave 2: #21, #11, #22, #34, #36, #17, #51 (7 parallel)
Wave 3: #26, #52, #53, #56 (4 parallel)
Wave 4: #54 (1)
Wave 5: #16, #29, #37, #32, #33, #55 (6 journey tests)
Future: #57
```

**Critical Bottleneck:** #12 (Personality Data Structure) - blocks 14 downstream stories

### Swarm Execution Pattern

When spawning agents for wave execution, use Claude Code's Task tool:

```javascript
// Spawn agents for Wave 0 in ONE message
Task("Implement #12 Personality Data Structure", "...", "coder", { run_in_background: true })
Task("Implement #35 Physical Turn Detection", "...", "coder", { run_in_background: true })
Task("Implement #48 Servo Calibration", "...", "coder", { run_in_background: true })
// ... etc
```

---

## Specflow Subagent Library

Reusable agents live in `.claude/agents/*.md`. Claude Code reads and follows these instructions automatically.

### 🎯 Agent Registry (16 Agents)

| Agent | File | When to Use | Trigger Phrases |
|-------|------|-------------|-----------------|
| **specflow-writer** | `specflow-writer.md` | New feature needs Gherkin, contracts, acceptance criteria | "write specflow for...", "spec out...", "create tickets for...", "make specflow compliant" |
| **board-auditor** | `board-auditor.md` | Check which issues are specflow-compliant | "audit the board", "check compliance", "which issues are compliant" |
| **specflow-uplifter** | `specflow-uplifter.md` | Fill gaps in partially-compliant issues | "uplift issues", "fill spec gaps", "fix non-compliant issues" |
| **contract-generator** | `contract-generator.md` | Generate YAML contracts from specs | "generate contracts", "create YAML contracts", "write contract files" |
| **contract-test-generator** | `contract-test-generator.md` | Create Jest tests from YAML contracts | "generate contract tests", "create contract enforcement tests" |
| **contract-validator** | `contract-validator.md` | Verify implementation matches spec | "validate contracts", "check contract compliance", "verify implementation" |
| **dependency-mapper** | `dependency-mapper.md` | Extract dependencies, build sprint waves | "map dependencies", "show me the waves", "analyze dependencies", "create sprint plan" |
| **sprint-executor** | `sprint-executor.md` | Execute parallel build waves | "execute sprint", "build wave 0", "run the waves", "execute backlog" |
| **migration-builder** | `migration-builder.md` | Feature needs database schema changes | "create migrations", "build schema", "database changes" |
| **frontend-builder** | `frontend-builder.md` | Create React hooks and components | "build the frontend", "create components", "implement UI" |
| **edge-function-builder** | `edge-function-builder.md` | Create serverless functions | "create edge functions", "build API endpoints" |
| **journey-enforcer** | `journey-enforcer.md` | Verify journey coverage, release readiness | "check journeys", "are we release ready?", "verify journey coverage" |
| **journey-tester** | `journey-tester.md` | Create E2E journey tests | "create journey tests", "write E2E tests", "build playwright tests" |
| **playwright-from-specflow** | `playwright-from-specflow.md` | Generate tests from Gherkin | "generate playwright tests", "convert gherkin to playwright" |
| **ticket-closer** | `ticket-closer.md` | Close validated issues with summaries | "close tickets", "close completed issues", "mark done" |
| **test-runner** | `test-runner.md` | Run tests, report failures | "run tests", "what's failing", "check test status" |

### 🔥 Auto-Trigger Rules (MANDATORY)

**Claude MUST use these agents automatically based on user phrases:**

#### 1. Specflow Compliance Work
**Triggers:** "specflow compliant", "write specflow", "spec out", "create tickets"
```
→ Read `.claude/agents/specflow-writer.md`
→ Follow its process for each issue
→ Ensure all 8 compliance markers present
```

#### 2. Board Audit
**Triggers:** "audit the board", "check compliance", "which issues are compliant"
```
→ Read `.claude/agents/board-auditor.md`
→ Produce compliance matrix
→ List missing markers per issue
```

#### 3. Dependency Mapping
**Triggers:** "map dependencies", "show me the waves", "sprint plan"
```
→ Read `.claude/agents/dependency-mapper.md`
→ Analyze all open issues
→ Produce Mermaid graph + wave table
```

#### 4. Sprint Execution
**Triggers:** "execute sprint", "build wave", "execute backlog"
```
→ Read `.claude/agents/sprint-executor.md`
→ Spawn parallel agents per wave
→ Coordinate via Claude Flow memory
```

#### 5. After ANY Code Changes (MANDATORY)
```
→ Run tests: cargo test && npm test -- contracts
→ Do NOT mark complete if tests fail
→ If tests pass, run journey-enforcer
```

#### 6. Closing Tickets
**Triggers:** "close tickets", "mark done", "close completed"
```
→ Read `.claude/agents/ticket-closer.md`
→ Verify tests pass
→ Post summary and close issue
```

#### 7. Contract Validation
**Triggers:** "validate contracts", "check contracts"
```
→ Read `.claude/agents/contract-validator.md`
→ Scan implementation against YAML contracts
→ Report violations
```

#### 8. Journey Testing
**Triggers:** "create journey tests", "write E2E tests"
```
→ Read `.claude/agents/journey-tester.md`
→ Generate Playwright tests from Gherkin
```

### Specflow Compliance Markers

An issue is **Specflow-compliant** when it has ALL 8 markers:

| # | Marker | Pattern | Required |
|---|--------|---------|----------|
| 1 | Gherkin | `Scenario:` or `Given/When/Then` | ✅ |
| 2 | Invariants | `I-*-NNN` pattern | ✅ |
| 3 | Acceptance Criteria | `- [ ]` checkboxes | ✅ |
| 4 | Data Contract | `interface` or `struct` | ✅ |
| 5 | Scope | "In Scope" / "Not In Scope" | ✅ |
| 6 | Journey Reference | `J-*-*` pattern | ✅ |
| 7 | Test IDs | `data-testid` table | ✅ |
| 8 | Definition of Done | DOD section | ✅ |

### Orchestration Pipeline

```
specflow-writer → board-auditor → dependency-mapper
       ↓
sprint-executor → [implementation agents via Task tool]
       ↓
contract-validator → journey-enforcer → ticket-closer
```

### Execute Full Backlog

Tell Claude Code:
```
Make my GitHub issues specflow-compliant, then map dependencies and execute my backlog in waves.
```

This triggers the full pipeline automatically.

---

## Active Contracts

### Feature Contracts

| ID | Contract | Description |
|----|----------|-------------|
| ARCH-001 | `feature_architecture.yml` | Core must be no_std compatible |
| ARCH-002 | `feature_architecture.yml` | Nervous system must be deterministic |
| ARCH-003 | `feature_architecture.yml` | Kitchen Table Test - no harmful behaviors |
| ARCH-004 | `feature_architecture.yml` | Personality parameters must be bounded |
| ARCH-005 | `feature_architecture.yml` | Transport layer must be abstracted |
| SORT-001 | `feature_sorter.yml` | Servo calibration repeatable (±2°) |
| SORT-002 | `feature_sorter.yml` | Sorting loop deterministic |
| SORT-003 | `feature_sorter.yml` | Safety - no pinch events |
| SORT-004 | `feature_sorter.yml` | Inventory must persist |
| SORT-005 | `feature_sorter.yml` | Offline-first operation |

### Journey Contracts (Definition of Done)

A feature is DONE when its journeys pass.

#### Critical (MUST pass for release)

| Journey | Description | Test File |
|---------|-------------|-----------|
| `J-ART-FIRST-DRAWING` | User creates first artwork | `tests/journeys/first-drawing.journey.spec.ts` |
| `J-PERS-MEET-PERSONALITY` | User experiences personality switch | `tests/journeys/meet-personality.journey.spec.ts` |
| `J-SORT-RESET` | User sorts mixed LEGO pile | `tests/journeys/reset-play-area.journey.spec.ts` |

#### Important (SHOULD pass)

| Journey | Description | Test File |
|---------|-------------|-----------|
| `J-GAME-TICTACTOE` | User plays Tic-Tac-Toe | `tests/journeys/tictactoe.journey.spec.ts` |
| `J-LEARN-FIRST-EXPERIMENT` | Student runs first experiment | `tests/journeys/first-experiment.journey.spec.ts` |
| `J-HELP-LEGO-SORT` | User sorts LEGO by color | `tests/journeys/lego-sort.journey.spec.ts` |

---

## Contract Locations

| Type | Location |
|------|----------|
| Feature contracts | `docs/contracts/feature_*.yml` |
| Journey contracts | `docs/contracts/journey_*.yml` |
| Contract index | `docs/contracts/CONTRACT_INDEX.yml` |
| Contract tests | `tests/contracts/*.test.ts` |
| Journey tests | `tests/journeys/*.journey.spec.ts` |

---

## Invariant Domains

| Domain | Prefix | Scope |
|--------|--------|-------|
| Architecture | ARCH-XXX | All code |
| Safety | SAFE-XXX | Kitchen Table Test |
| ArtBot | ART-XXX | Drawing features |
| Personality | PERS-XXX | Personality system |
| GameBot | GAME-XXX | Games |
| HelperBot | HELP-XXX | Utility features |
| LearningLab | LEARN-XXX | Education features |
| LEGOSorter | SORT-XXX | Sorting system |

---

## GitHub Repository

- **Repo:** `Hulupeep/mbot_ruvector`
- **Issues:** `gh issue list -R Hulupeep/mbot_ruvector`
- **Labels:** `epic`, `story`, `enhancement`

---

## Quick Reference Commands

```bash
# Specflow workflow
gh issue list -R Hulupeep/mbot_ruvector --state open

# Claude Flow
npx @claude-flow/cli@latest swarm status
npx @claude-flow/cli@latest memory search --query "wave 0"
npx @claude-flow/cli@latest hooks pre-task --description "task description"

# Tests
cargo test
npm test -- contracts
npm run test:journeys
```
