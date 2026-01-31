# Specflow Agent Library

## The Problem

LLMs drift. You explain a requirement, they build something, and three prompts later they've "optimized" your auth flow into a security hole. They confidently break things while appearing to understand perfectly.

**Traditional fixes don't work:**
- **More instructions?** LLMs attend to what feels salient, not what you emphasize
- **Better prompts?** Works until context window fills and early instructions fade
- **Code review?** You're now the bottleneck, reviewing AI output line by line
- **Unit tests?** Test implementation details, not architectural invariants

## The Solution

**Make requirements executable.** Turn "tokens must be in httpOnly cookies" into a pattern test that fails the build if violated.

```
Spec → YAML Contract → Jest Test → npm test → Build fails on violation
```

The LLM can drift all it wants. The build catches it.

## Why This Works

| Approach | Problem |
|----------|---------|
| Prompting | LLM attention drifts; instructions compete with priors |
| Code review | Human bottleneck; doesn't scale |
| Unit tests | Test functions, not invariants; break on refactors |
| **Contracts** | Test what must stay true; survive refactors; fail loudly |

Contracts answer: "What must NEVER change, regardless of how we build it?"

## Your Role (Human)

Two jobs:

### 1. Ensure stories are Specflow-compliant

Before work starts, issues should have:
- Gherkin scenarios (what behavior looks like)
- Data contracts (SQL, RLS, TypeScript interfaces)
- Journey references (which user flow this enables)

Run `board-auditor` to check. Run `specflow-uplifter` to fix gaps.

### 2. Execute with the right agents

Tell Claude Code what to do:

```
"Run specflow-writer on issues #107-#112"
"Generate YAML contracts for the admin features"
"Execute sprint 0: issues #107, #108, #109"
"Check if we're release-ready"
```

The agents know the patterns. You provide direction.

---

## The 16 Agents

### Writing Specs
| Agent | What it does |
|-------|--------------|
| **specflow-writer** | Turns feature descriptions into build-ready issues with Gherkin, SQL, RLS, TypeScript interfaces |
| **board-auditor** | Scans issues for compliance; produces a matrix showing what's missing |
| **specflow-uplifter** | Fills gaps in partially-compliant issues |

### Generating Contracts
| Agent | What it does |
|-------|--------------|
| **contract-generator** | Creates YAML contracts from specs (`docs/contracts/*.yml`) |
| **contract-test-generator** | Creates Jest tests from YAML contracts (`npm test -- contracts`) |

### Planning & Building
| Agent | What it does |
|-------|--------------|
| **dependency-mapper** | Extracts dependencies from SQL REFERENCES, builds sprint order |
| **sprint-executor** | Coordinates parallel build waves; dispatches other agents |
| **migration-builder** | Creates Supabase migrations from SQL contracts |
| **frontend-builder** | Creates React hooks and components |
| **edge-function-builder** | Creates Deno Edge Functions |

### Testing & Enforcement
| Agent | What it does |
|-------|--------------|
| **contract-validator** | Verifies code matches contracts; produces gap report |
| **journey-enforcer** | Ensures UI stories have journeys; blocks release if critical journeys fail |
| **playwright-from-specflow** | Generates Playwright tests from Gherkin |
| **journey-tester** | Creates cross-feature E2E journey tests |
| **test-runner** | Executes Playwright tests, parses results, reports failures with file:line details |

### Closing
| Agent | What it does |
|-------|--------------|
| **ticket-closer** | Posts summaries, links commits, closes validated issues |

---

## The Pipeline

```
YOU: "Make issues #107-#112 specflow-compliant"
  │
  ↓
Phase 1: SPECIFICATION
  specflow-writer → board-auditor → specflow-uplifter
  │
  ↓
YOU: "Generate contracts for these issues"
  │
  ↓
Phase 2: CONTRACTS
  contract-generator → contract-test-generator
  │
  ↓
YOU: "Map dependencies and execute the sprint"
  │
  ↓
Phase 3: BUILD
  dependency-mapper → sprint-executor
    ├─ migration-builder (parallel)
    ├─ frontend-builder (parallel)
    └─ edge-function-builder (parallel)
  │
  ↓
  npm test -- contracts ← BUILD FAILS IF PATTERNS VIOLATED
  │
  ↓
Phase 4: VALIDATE
  contract-validator → journey-enforcer
    ├─ playwright-from-specflow (parallel)
    └─ journey-tester (parallel)
  │
  ↓
Phase 5: TEST EXECUTION
  test-runner → (runs tests, reports failures)
  │
  ↓
  All tests pass? → Continue
  Tests fail? → Fix and re-run
  │
  ↓
YOU: "Close the completed issues"
  │
  ↓
Phase 6: CLOSE
  ticket-closer
```

---

## Quick Commands

| Goal | Say this |
|------|----------|
| Make issues spec-ready | "Run specflow-writer on #107-#112" |
| Check compliance | "Run board-auditor on all open issues" |
| Fill spec gaps | "Run specflow-uplifter on issues missing RLS" |
| Generate YAML contracts | "Run contract-generator on #107-#112" |
| Generate Jest tests | "Run contract-test-generator for all contracts" |
| Plan the sprint | "Run dependency-mapper, show me the waves" |
| Build a wave | "Execute sprint 0: #107, #108, #109" |
| Validate contracts | "Run contract-validator on the implemented issues" |
| Run tests | "Run test-runner" or "Run tests" |
| Check what's failing | "What tests are failing?" |
| Check release readiness | "Are critical journeys passing?" |
| Close tickets | "Run ticket-closer on #107-#112" |

---

## Two Enforcement Layers

| Layer | When | What it catches |
|-------|------|-----------------|
| **Pattern tests** | `npm test -- contracts` (build time) | Code that violates architectural rules |
| **Journey tests** | Playwright (post-build) | User flows that don't work end-to-end |

Pattern tests catch: `localStorage` in service workers, direct Supabase calls in components, hardcoded secrets.

Journey tests catch: User can't complete checkout, approval flow broken, data not syncing.

**test-runner agent** executes both layers and produces actionable reports with file:line references.

---

## The Key Insight

**Contracts in tickets ARE the dependency graph.**

A SQL `REFERENCES` clause is a dependency. A TypeScript interface import is a dependency. The `dependency-mapper` agent reads these and builds the sprint order automatically.

No manual linking. No Gantt charts. The code tells us what depends on what.

---

## Adding Agents

Create `agents/{name}.md` with:
- **Role**: What this agent does
- **Trigger**: When to use it
- **Process**: Step-by-step with examples
- **Quality gates**: What must be true when done

See existing agents for the pattern.
