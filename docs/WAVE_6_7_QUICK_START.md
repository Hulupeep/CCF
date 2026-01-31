# Wave 6-7 Quick Start Guide

**Execute Waves 6-7 in 5 minutes or less**

---

## 🚀 Step 1: Create All GitHub Issues (2 minutes)

```bash
# Navigate to project root
cd /home/xanacan/projects/code/mbot/mbot_ruvector

# Run the issue creation script
./scripts/create-wave-6-7-issues.sh

# Expected output: "✅ Wave 6-7 issue creation complete!"
```

This creates 22 Specflow-compliant issues:
- Wave 6: #58-#69 (12 stories)
- Wave 7: #70-#79 (10 stories)

---

## ✅ Step 2: Verify Issues Created (1 minute)

```bash
# List Wave 6 issues
gh issue list -R Hulupeep/mbot_ruvector --label wave-6

# List Wave 7 issues
gh issue list -R Hulupeep/mbot_ruvector --label wave-7

# View specific issue
gh issue view 58 -R Hulupeep/mbot_ruvector
```

Expected: 22 total issues with full Specflow compliance.

---

## 📋 Step 3: Create Project Board (2 minutes)

### Option A: GitHub CLI
```bash
# Create project board
gh project create \
  --owner Hulupeep \
  --title "mBot RuVector: Waves 6-7" \
  --body "UI Completion & Advanced Features (7 weeks)"

# Add issues to board (repeat for #58-79)
gh project item-add <PROJECT_ID> --owner Hulupeep --url "https://github.com/Hulupeep/mbot_ruvector/issues/58"
```

### Option B: GitHub Web UI
1. Go to https://github.com/Hulupeep/mbot_ruvector/projects
2. Click "New project"
3. Choose "Board" template
4. Title: "Waves 6-7"
5. Add columns:
   - 🔜 **Backlog** (Wave 6 + Wave 7 stories)
   - 🏗️ **In Progress** (Active work)
   - 🧪 **In Review** (PR submitted)
   - ✅ **Done** (Merged + tested)
6. Drag issues #58-#79 into Backlog

---

## 👥 Step 4: Assign Stories to Team (varies)

### Critical Path First (Assign immediately)
```bash
# #64 WebSocket V2 (CRITICAL - blocks mobile + multi-robot)
gh issue edit 64 -R Hulupeep/mbot_ruvector --add-assignee backend-dev-lead

# #67 Integration Tests (CRITICAL - blocks Wave 7)
gh issue edit 67 -R Hulupeep/mbot_ruvector --add-assignee qa-engineer-lead
```

### Wave 6 Week 1: UI Components (Assign to 5 developers)
```bash
gh issue edit 58 -R Hulupeep/mbot_ruvector --add-assignee frontend-dev-1  # Personality Mixer
gh issue edit 59 -R Hulupeep/mbot_ruvector --add-assignee frontend-dev-2  # Neural Visualizer
gh issue edit 60 -R Hulupeep/mbot_ruvector --add-assignee frontend-dev-3  # Drawing Gallery
gh issue edit 61 -R Hulupeep/mbot_ruvector --add-assignee frontend-dev-4  # Game Stats
gh issue edit 62 -R Hulupeep/mbot_ruvector --add-assignee frontend-dev-5  # Inventory Dashboard
```

### Track Assignments
```bash
# See who's assigned what
gh issue list -R Hulupeep/mbot_ruvector --assignee @me
gh issue list -R Hulupeep/mbot_ruvector --assignee frontend-dev-1
```

---

## 📅 Step 5: Schedule Kickoff Meeting

### Agenda (1 hour)

**Attendees:** All developers, QA, PM, architect

**Topics:**
1. **Wave 6-7 Overview (10 min)**
   - Read: `docs/WAVE_6_7_SUMMARY.md`
   - 22 stories, 7 weeks, all Specflow-compliant

2. **Dependency Review (15 min)**
   - Show: `docs/WAVE_6_7_VISUAL_TIMELINE.md` (Mermaid graphs)
   - Highlight: #64 WebSocket V2 is bottleneck
   - Discuss: Parallel work strategies

3. **Story Walkthrough (20 min)**
   - Review: #58 (Personality Mixer) as example
   - Explain: Gherkin, Invariants, data-testid tables
   - Q&A: Clarify any confusion

4. **Sprint Planning (10 min)**
   - Week 1 focus: 5 UI components in parallel
   - Daily stand-ups: 9am
   - PR review protocol: 2 approvals required

5. **Action Items (5 min)**
   - Assign remaining stories
   - Set up dev environments
   - Clone repos, install dependencies

---

## 🛠️ Step 6: Set Up Development Environment

### For Web Dashboard Development (Wave 6 UI stories)
```bash
# Install dependencies
cd /home/xanacan/projects/code/mbot/mbot_ruvector/web
npm install

# Start dev server
npm start

# Open: http://localhost:3000
```

### For Backend Development (Wave 6 integration stories)
```bash
# Install Rust dependencies
cd /home/xanacan/projects/code/mbot/mbot_ruvector
cargo build

# Run companion app
cargo run --bin mbot-companion -- --serial /dev/ttyUSB0
```

### For Testing (Wave 6 testing stories)
```bash
# Run unit tests
npm test

# Run E2E journey tests
npm run test:journeys

# Run contract tests
npm test -- contracts
```

---

## 📊 Step 7: Track Progress Daily

### Daily Ritual (5 minutes/day)

#### Morning: Check Status
```bash
# What's in progress?
gh issue list -R Hulupeep/mbot_ruvector --label wave-6 --state open

# Any blockers?
gh issue list -R Hulupeep/mbot_ruvector --label blocked

# What's ready for review?
gh pr list -R Hulupeep/mbot_ruvector --label wave-6
```

#### Stand-up: Share Updates
- **Yesterday:** "Completed #58 Personality Mixer UI"
- **Today:** "Starting #63 Cross-App Persistence"
- **Blockers:** "Waiting for #64 WebSocket design doc"

#### Evening: Update Progress
```bash
# Mark story in progress
gh issue comment 58 -R Hulupeep/mbot_ruvector --body "🏗️ In progress: 50% complete, sliders working"

# Mark story complete
gh issue close 58 -R Hulupeep/mbot_ruvector --comment "✅ Complete: UI implemented, E2E test passing"
```

---

## 🎯 Success Metrics to Track

### Week 1 Target: 5/5 UI stories complete
```bash
# Check completion
gh issue list -R Hulupeep/mbot_ruvector --label wave-6 --search "STORY-*-UI" --state closed
```

### Week 2 Target: WebSocket V2 deployed
```bash
# Check if #64 closed
gh issue view 64 -R Hulupeep/mbot_ruvector --json state
```

### Week 3 Target: Integration tests >80% coverage
```bash
# Run coverage report
npm test -- --coverage

# Check: Is coverage >80%?
```

### Wave 6 Release Gate: All critical stories done
```bash
# Critical stories: #64, #67
gh issue list -R Hulupeep/mbot_ruvector --label wave-6 --search "CRITICAL" --state closed

# Expected: 2 closed
```

---

## 🚨 When Things Go Wrong

### Issue: Story blocked by dependency
```bash
# Add "blocked" label
gh issue edit 76 -R Hulupeep/mbot_ruvector --add-label blocked

# Add comment explaining blocker
gh issue comment 76 -R Hulupeep/mbot_ruvector --body "🚧 Blocked by #64 WebSocket V2"

# Find alternative work
gh issue list -R Hulupeep/mbot_ruvector --label wave-6 --no-label blocked
```

### Issue: Test failing
```bash
# Don't mark story complete!
# Add "failing-tests" label
gh issue edit 58 -R Hulupeep/mbot_ruvector --add-label failing-tests

# Link to test results
gh issue comment 58 -R Hulupeep/mbot_ruvector --body "❌ E2E test failing: [CI log](link)"

# Fix test, re-run, then close
```

### Issue: Scope creep detected
```bash
# Reference "Not In Scope" section
gh issue comment 58 -R Hulupeep/mbot_ruvector --body "⚠️ This is out of scope (see issue body). Creating new story for Wave 8."

# Create follow-up story
gh issue create -R Hulupeep/mbot_ruvector --title "FOLLOW-UP: Animation previews for Personality Mixer" --label wave-8
```

---

## 📚 Documentation to Read

### Before Starting Development
1. **CLAUDE.md** - Project rules and Specflow requirements
2. **WAVE_6_7_PLAN.md** - Detailed story descriptions
3. **WAVE_6_7_SUMMARY.md** - Quick reference
4. **WAVE_6_7_VISUAL_TIMELINE.md** - Dependency graph

### During Development
1. **APP_GUIDES.md** - How each app works
2. **MASTER_GUIDE.md** - Hardware setup, building firmware
3. **docs/contracts/*.yml** - Feature contracts and invariants

### For Testing
1. **tests/journeys/*.spec.ts** - E2E test examples
2. **tests/contracts/*.test.ts** - Contract test patterns

---

## 🎉 Celebrate Milestones

### Week 1 Done: "UI Complete" 🎨
- [ ] Take screenshots of all 5 components
- [ ] Record demo video (2 minutes)
- [ ] Post in team Slack: "Wave 6 UI Complete! 🎉"

### Week 3 Done: "Wave 6 Release" 🚀
- [ ] Deploy to production
- [ ] Send release notes to stakeholders
- [ ] Team lunch/happy hour

### Week 7 Done: "Wave 7 Release" 🏆
- [ ] Final demo to company
- [ ] Publish blog post
- [ ] Plan Wave 8!

---

## 🤝 Getting Help

### Questions About Stories
- **Specflow compliance:** Read `.claude/agents/specflow-writer.md`
- **Gherkin syntax:** See existing issues #58-#79 for examples
- **data-testid:** Look at `tests/journeys/*.spec.ts`

### Technical Questions
- **WebSocket:** Ask backend lead about #64 design doc
- **React:** Check `web/src/components/*.tsx` examples
- **Rust:** Read `crates/mbot-core/src/*.rs`

### Process Questions
- **Issue creation:** Re-run `./scripts/create-wave-6-7-issues.sh` (idempotent)
- **Sprint planning:** PM or Scrum Master
- **Release process:** DevOps engineer

---

## ✅ Final Checklist Before Starting

- [ ] All 22 issues created (#58-#79)
- [ ] Project board set up with 4 columns
- [ ] Critical path stories assigned (#64, #67)
- [ ] Wave 6 Week 1 stories assigned (5 developers)
- [ ] Kickoff meeting scheduled
- [ ] Dev environments set up (web + Rust)
- [ ] Documentation read (CLAUDE.md, WAVE_6_7_*.md)
- [ ] Daily stand-up time agreed (9am suggested)
- [ ] Slack/communication channel ready

---

## 🚀 Ready to Go!

You now have:
- ✅ 22 Specflow-compliant GitHub issues
- ✅ Clear dependency graph
- ✅ 7-week execution timeline
- ✅ Success metrics defined
- ✅ Team assignments planned
- ✅ Development environment ready

**Next action:** Run `./scripts/create-wave-6-7-issues.sh` and let's build! 🎉

---

**Time to complete this guide:** 10-15 minutes
**Time to execute Waves 6-7:** 7 weeks
**Stories delivered:** 22 features

**Good luck, and happy coding! 🤖🚀**
