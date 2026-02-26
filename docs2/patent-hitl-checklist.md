# Patent Claim HITL Verification Checklist

**Document purpose:** Human-in-the-loop (HITL) verification procedure for the five
major claims of USP provisional patent application 63/988,438.

**When to run:** Before any public demo, major release, or patent review meeting.

**Who runs it:** A non-engineer evaluator using the live mBot2 hardware and the
companion dashboard. No code changes are permitted during a verification session.

**Related automated tests:** `crates/mbot-companion/tests/patent_claims_integration.rs`

---

## Pre-Session Setup

- [ ] mBot2 is fully charged (LED ring shows solid blue at startup)
- [ ] Companion dashboard is open and connected (`ws://localhost:8080` or production URL)
- [ ] Context coherence panel is visible on the dashboard
- [ ] Social phase indicator is visible (ShyObserver / QuietlyBeloved / etc.)
- [ ] Session log is recording (check "Record Session" checkbox)
- [ ] Room A (bright, quiet) and Room B (dark, noisy) are prepared in advance

---

## Claim 1 — Context Isolation

**Claim text:** Coherence accumulated in one environment is not transferred to a
structurally dissimilar environment. Each context must independently earn trust.

### Steps

1. Place robot in **Room A** (bright overhead light, ambient noise below conversational).

2. Interact positively with the robot for **3 minutes**:
   - Wave hand slowly at 30–50 cm distance
   - Speak calmly ("hello", "good robot")
   - Observe LED color shifting from muted blue toward warm blue

3. OBSERVE: On the dashboard, the context coherence panel should show the
   `Bright · Quiet · Static` context climbing above **0.30**.

   - [ ] Context coherence for `Bright · Quiet · Static` is >= 0.30

4. Without resetting the robot, move it to **Room B** (dim or dark, noisy —
   music or crowd noise at > 60 dB, or cover the light sensor).

5. OBSERVE immediately after entering Room B:

   - [ ] The active context label changes to a dark/noisy variant
   - [ ] The context coherence indicator drops to **0.00** for the new context
   - [ ] The robot's LED ring dims or shifts to muted/cautious colours
   - [ ] The robot does NOT immediately display the warm expressive behaviours
         it showed in Room A (no large flourishes, no approach movement)

### PASS criteria

Both checkboxes in steps 3 and 5 are ticked. The robot visibly withdraws or
remains cautious in Room B despite positive interaction history in Room A.

### FAIL indicator

The robot immediately shows high-coherence expressive behaviour (full LED
brightness, active approach movement) upon entering Room B without any
interaction history there.

---

## Claim 2 — Earned Trust

**Claim text:** Relational coherence grows strictly through repeated positive
interaction. It is not granted at startup and cannot jump to high values without
interaction history.

### Steps

1. **Reboot the robot** (clear all session state). Confirm the dashboard shows
   context coherence = **0.00** for all contexts and social phase = **ShyObserver**.

   - [ ] All context coherence values are 0.00 at startup

2. Begin a timed positive interaction in a fixed room. Use a stopwatch.

3. At **T = 0 min** (immediately after startup):
   - OBSERVE: LED is muted blue, movement is minimal, social phase is ShyObserver
   - [ ] Social phase shown as `ShyObserver`

4. Interact continuously and positively for **2 minutes** (slow waves, calm speech,
   gentle approach within 30–40 cm, then gradual retreat).

5. At **T = 2 min**:
   - OBSERVE the active context's coherence on the dashboard
   - [ ] Coherence value is visibly higher than at T = 0 (should be > 0.15)
   - [ ] The robot's LED has brightened or shifted toward warmer blue
   - [ ] Movement range or curiosity gestures have increased

6. Continue for another **3 minutes** (total 5 min of positive interaction).

7. At **T = 5 min**:
   - [ ] Coherence value is >= 0.30 (crosses into familiar territory)
   - [ ] Social phase has transitioned to `QuietlyBeloved` or is close to it
   - [ ] Robot exhibits small expressive flourishes (brief spin, tone, LED pulse)

### PASS criteria

All checkboxes in steps 1, 3, 5, and 7 are ticked. The dashboard shows a
continuously rising coherence curve over the 5-minute session.

### FAIL indicator

Coherence starts above 0.0 at reboot (pre-loaded), or jumps to >= 0.30 within
the first 30 seconds without sustained positive interaction.

---

## Claim 3 — Warm Start

**Claim text:** When the robot resumes in the same context as a prior session,
coherence restores from stored history (warm start). A different context always
starts at zero (cold start).

### Steps

#### Part A — Warm start in same room

1. Run Claim 2 verification until coherence >= 0.30 in **Room A**.

2. Note the exact coherence value shown on dashboard (e.g., "0.41").

3. **Reboot the robot** (power off, power on). Keep it in Room A.

4. Wait 10 seconds for the companion to reconnect and load persisted state.

5. OBSERVE the dashboard immediately after reconnection:
   - [ ] The `Room A` context shows coherence close to the value noted in step 2
         (within +/- 0.05; some decay is expected)
   - [ ] Social phase is NOT ShyObserver if earned coherence was >= 0.30
   - [ ] Robot's LED reflects the restored coherence (warmer blue, not muted grey)

#### Part B — Cold start in different room

6. After confirming the warm start, move the robot to **Room B** (different
   brightness/noise context not visited this session).

7. OBSERVE the dashboard after the context key updates (within 5 seconds):
   - [ ] Room B context shows coherence **0.00**
   - [ ] Social phase drops to `ShyObserver`
   - [ ] Robot behaviour is cautious, not expressive

### PASS criteria

All checkboxes in Part A and Part B are ticked. The warm start restores prior
coherence and the cold start shows zero in the new context.

### FAIL indicator

Reboot in Room A shows 0.00 (persistence not working), or Room B immediately
shows the same coherence as Room A (no isolation).

---

## Claim 4 — Deliberative Gate

**Claim text:** A significant contextual boundary change (SocialPhase transition)
fires the deliberative gate exactly once. Sustained stimuli do not cause
repeated firing. A minimum cooldown is enforced before the gate can fire again.

### Steps

1. Establish a calm baseline: robot in Room A with coherence >= 0.30, social
   phase = `QuietlyBeloved`.

   - [ ] Dashboard shows `QuietlyBeloved` at baseline

2. Introduce a **sudden loud noise**: clap sharply or play a loud tone (> 80 dB)
   for 1 second, then maintain that same noise level continuously.

3. OBSERVE in the first 2 seconds after the noise onset:
   - [ ] Social phase transitions to `StartledRetreat` or `ProtectiveGuardian`
   - [ ] The robot performs exactly ONE deliberative gesture (reverse motion,
         red LED flash, or alarm tone)
   - [ ] The transition event is logged ONCE in the session log

4. Wait **10 seconds** while the noise remains at the same level.

5. OBSERVE during sustained noise (ticks 50–600 after onset):
   - [ ] No additional deliberative motor reversals occur
   - [ ] No repeated alarm tones
   - [ ] Social phase remains in the same quadrant without flickering

6. Remove the noise source. Wait 5 seconds.

7. OBSERVE recovery:
   - [ ] Social phase transitions back toward `QuietlyBeloved` exactly once
   - [ ] The recovery transition appears as a single entry in the session log

### PASS criteria

Steps 3, 5, and 7 checkboxes all ticked. Exactly two phase-transition events
appear in the session log for this scenario (onset and recovery).

### FAIL indicator

Multiple deliberative motor reversals occur during sustained noise (gate fires
repeatedly), or the session log shows more than one onset-transition event.

---

## Claim 5 — Suppression Integrity

**Claim text:** A general suppression rule (e.g., suppress LoudnessSpike in
all contexts) does not override or weaken a more specific rule configured for
a precise context. Exact-match rules always return their own factor.

### Steps

1. Using the companion dashboard's suppression rule panel, configure:
   - **General rule**: `LoudnessSpike` in `Dark · Loud · *` context →
     suppression factor **0.40** (robot has learned this noise is routine in
     dark rooms)
   - **Specific rule**: `LoudnessSpike` in `Bright · Loud · Static` context →
     suppression factor **0.95** (this precise context remains fully trusted)

   - [ ] Both rules are visible in the suppression rule table
   - [ ] General rule shows factor 0.40
   - [ ] Specific rule shows factor 0.95

2. Move the robot to **Room B** (dark, loud). Create a LoudnessSpike stimulus
   (sharp clap).

3. OBSERVE the robot's response to the clap in Room B:
   - [ ] Robot shows a reduced startle response (the 0.40 suppression is applied)
   - [ ] LED and motor response is noticeably muted compared to an unsuppressed clap
   - [ ] The "suppression applied" indicator on the dashboard shows the 0.40 factor

4. Move the robot to **Room A** configured as `Bright · Loud` (keep noise on).
   Create another LoudnessSpike stimulus with the same clap.

5. OBSERVE the robot's response to the clap in Room A (bright + loud):
   - [ ] Robot shows a FULL or near-full trust response (0.95 factor applied)
   - [ ] The LED and motor behaviour is noticeably MORE expressive than in Room B
   - [ ] The "suppression factor" indicator on the dashboard shows **0.95**, not 0.40
   - [ ] The general rule (0.40) did NOT override the specific rule (0.95)

### PASS criteria

All checkboxes in steps 1, 3, and 5 are ticked. The specific rule's factor
(0.95) is applied in Room A despite the general rule (0.40) existing in the map.

### FAIL indicator

The dashboard shows factor 0.40 in Room A (general rule bled into specific rule),
or both rooms show the same suppression factor.

---

## Post-Session Sign-Off

Evaluator name: _________________________________

Date: _________________________________

Robot serial / firmware version: _________________________________

| Claim | Result |
|-------|--------|
| 1 — Context Isolation | PASS / FAIL |
| 2 — Earned Trust | PASS / FAIL |
| 3 — Warm Start | PASS / FAIL |
| 4 — Deliberative Gate | PASS / FAIL |
| 5 — Suppression Integrity | PASS / FAIL |

Overall session: **PASS / FAIL**

Notes (attach photos or dashboard screenshots if available):

_______________________________________________________________________________

_______________________________________________________________________________
