# CCF Patent Demo Runbook

**Story:** #36
**Purpose:** 5-minute video proving CCF patent claims on a physical mBot2 robot.
**Companion binary:** `crates/mbot-companion`
**Dashboard:** served by the `voice-api` feature on port 8080 (default)

**Invariants enforced:**
- I-DEMO-001: Every shell command is verbatim copy-pasteable and matches the current `main.rs` Args struct.
- I-DEMO-002: Clean-state verification procedure is defined and leaves `context_coherence = 0%` and `social_phase = ShyObserver` before each demo section.

---

## 1. Hardware Checklist

Complete this checklist before powering anything on.

- [ ] **Firmware version** — mBot2 CyberPi firmware v2.1.0 or later. Flash via Makeblock app if out of date. Confirm with robot's on-screen version display after power-on.
- [ ] **Battery level** — Battery must read > 80% on the CyberPi display. Charge if below threshold; charging takes ~90 min from 20%.
- [ ] **BLE pairing** — Confirm laptop Bluetooth is on. Power on the mBot2. The CyberPi display should show "BLE Ready". Pair using the OS Bluetooth settings if first time. Note: the companion binary pairs automatically once the OS has registered the device.
- [ ] **Clear zone** — Table cleared to at least 40 cm radius around the robot. No cables or objects in motor path.
- [ ] **Camera framing** — Camera on tripod, angled 45 degrees to robot, capturing the full body and the LED strip. White balance set to daylight (5500 K). Verify LED colour is visible in viewfinder before filming.
- [ ] **Second screen** — Dashboard browser tab open on second screen or laptop lid closed with external monitor. Dashboard must be visible to operator during the run.
- [ ] **Ambient environment** — Lighting stable (no windows with passing clouds if possible). Close doors to reduce sudden noise events before Section 1 and 2.

---

## 2. Software Start Sequence

Run these commands in order. Every command is verbatim; replace nothing unless instructed.

### Step 1 — Navigate to repo root

```bash
cd /home/xanacan/projects/code/mbot/CCF
```

### Step 2 — Start the companion binary (Bluetooth + voice-api dashboard)

```bash
cargo run -p mbot-companion --features "brain,voice-api" -- \
  --bluetooth \
  --voice-api \
  --voice-port 8080
```

Expected startup output:

```
INFO  mbot_companion > CCF on RuVector Companion starting...
INFO  mbot_companion > Connecting via Bluetooth...
INFO  mbot_companion > Voice API server listening on 0.0.0.0:8080
```

If Bluetooth is not available on the build host, run in simulation mode for dashboard verification only:

```bash
cargo run -p mbot-companion --features "brain,voice-api" -- \
  --simulate \
  --voice-api \
  --voice-port 8080
```

### Step 3 — Open the dashboard

```bash
xdg-open http://localhost:8080
```

On macOS:

```bash
open http://localhost:8080
```

### Step 4 — Run sensor diagnostics (optional, recommended before first take)

Open a second terminal, navigate to the repo root, then:

```bash
cargo run -p mbot-companion -- --bluetooth --diagnose
```

Confirm the output shows all sensors OK before proceeding.

---

## 3. Pre-Roll Verification Table (I-DEMO-002)

Before pressing record on any section, verify every row of this table against the live dashboard at `http://localhost:8080`.

| Signal | Expected value | Action if wrong |
|---|---|---|
| `social_phase` | `ShyObserver` | Run the clean-state procedure below |
| `context_coherence` | `0%` | Run the clean-state procedure below |
| `effective_coherence` | `0%` | Run the clean-state procedure below |
| `context_count` | `0` or `1` (first context card only) | Run the clean-state procedure below |
| LED colour (RGB) | `[40, 40, 80]` muted blue-grey | Check ambient light; re-run clean-state |
| Motor amplitude | `25%` (barely moving) | Check that no prior interactions inflated trust |
| BLE connection | `Connected` | See Recovery — BLE drop |

**This table must pass before filming any section. No exceptions.**

### Clean-State Procedure (I-DEMO-002)

Run this procedure between every demo take or whenever the table above fails.

```bash
# 1. Stop the running companion process
#    Press Ctrl+C in the terminal running cargo run

# 2. Delete the on-disk coherence accumulator to erase all earned trust
rm -f ~/.mbot/coherence.db

# 3. Restart the companion from repo root
cargo run -p mbot-companion --features "brain,voice-api" -- \
  --bluetooth \
  --voice-api \
  --voice-port 8080
```

After restart, wait 5 seconds for Bluetooth reconnection, then reload the dashboard tab and re-verify the pre-roll table before filming.

---

## 4. Demo Script Cue Sheet

### Section 1 — Fresh Start / ShyObserver Baseline (~60 seconds)

**What the audience sees:** Robot is powered and connected but has no history. Trust is at the absolute floor.

**Pre-section checklist:**
- [ ] Pre-roll verification table passes (see Section 3)
- [ ] Operator is not moving or making noise near the robot

**Camera cue:** Wide shot capturing both LED strip and full body. No cuts.

**Expected observable state:**

| Observable | Value |
|---|---|
| `social_phase` | `ShyObserver` |
| `context_coherence` | `0%` |
| LED colour | `[40, 40, 80]` muted blue-grey |
| Motor amplitude | `25%` — brief, tentative movements |

**Operator action:** None. Let the robot idle. The minimal motor amplitude and muted LED are the demonstration. Hold this shot for the full 60 seconds.

**Patent claims evidenced:** Claims 2-5 (CoherenceAccumulator at floor), Claim 14 (SocialPhase baseline).

---

### Section 2 — Gradual Trust Accumulation to QuietlyBeloved (~90 seconds)

**What the audience sees:** Repeated positive interactions in a stable environment raise the coherence floor. The robot transitions from ShyObserver to QuietlyBeloved, visibly changing LED and motor behaviour.

**Pre-section checklist:**
- [ ] Starting from ShyObserver / 0% coherence (carry over from Section 1 or re-run clean-state)
- [ ] Ambient environment is stable — same lighting, no sudden sounds

**Camera cue:** Start with close-up on LED strip. Hold until colour shifts. Then wide shot to show motor amplitude increase.

**Operator action:** Stay present but calm. Allow 60-90 seconds of normal idle behaviour. The accumulator earns coherence passively from stable context readings. Do not interrupt.

**Transition event:** Dashboard shows `social_phase` flipping to `QuietlyBeloved`. LED changes to warm blue `[60, 120, 200]`.

**Expected observable state after transition:**

| Observable | Value |
|---|---|
| `social_phase` | `QuietlyBeloved` |
| `context_coherence` | ~22% or above (earned floor) |
| LED colour | `[60, 120, 200]` warm blue |
| Motor amplitude | `100%` — confident, fluid movements |

**Patent claims evidenced:** Claims 2-5 (earned coherence floor), Claims 14-17 (phase transition and permeability).

---

### Section 3 — Startle Event to StartledRetreat (~30 seconds)

**What the audience sees:** A sudden environmental disturbance triggers the startle processor. The robot immediately retreats — dark red LED, limited movement — then recovers back toward QuietlyBeloved as the disturbance clears.

**Pre-section checklist:**
- [ ] Robot is in QuietlyBeloved state (carry over from Section 2)
- [ ] Camera operator ready for fast cut to close-up on LED

**Camera cue:** Stay on close-up of LED for the full section to capture the colour flip and recovery.

**Operator action — startle trigger:** Clap loudly once, or cover the CyberPi light sensor firmly with your hand for 2 seconds, then remove it.

**Expected observable state during startle:**

| Observable | Value |
|---|---|
| `social_phase` | `StartledRetreat` |
| LED colour | `[180, 20, 20]` dark red |
| Motor amplitude | limited, defensive |

**Expected recovery (after disturbance clears, ~5-10 seconds):**

| Observable | Value |
|---|---|
| `social_phase` | returning toward `QuietlyBeloved` (hysteresis) |
| LED colour | transitioning back to warm blue |
| Motor amplitude | increasing back toward 100% |

**Patent claims evidenced:** Claims 14-18 (Schmitt trigger hysteresis, StartledRetreat phase, recovery trajectory).

---

### Section 4 — Context Switch / New Context Card at 0% (~60 seconds)

**What the audience sees:** The robot is moved to a meaningfully different environment (different room or dramatically changed lighting). The dashboard shows a new context card appearing at 0% coherence while the original context card preserves its earned coherence intact.

**Pre-section checklist:**
- [ ] Robot is in QuietlyBeloved in original context (from Section 2 or 3)
- [ ] Second room or strong lighting change prepared
- [ ] Screen recording of dashboard is active (this section is best captured by recording the dashboard, not just the robot)

**Camera cue:** Split recording — one camera on robot, screen recording on dashboard. The key visual is the two context cards side by side.

**Operator action:** Physically move the robot (or the operator) to a significantly different environment. A different room with different ambient light is ideal. Alternatively, cover all light sensors and open a window to change the light profile dramatically.

**Expected observable state after context switch:**

| Observable | Value |
|---|---|
| New context card `context_coherence` | `0%` |
| Original context card `context_coherence` | preserved (~22% from Section 2) |
| `social_phase` | resets to `ShyObserver` in new context |
| LED colour | returns to `[40, 40, 80]` muted blue-grey |

**Patent claims evidenced:** Claims 1, 8, 13 (independent per-context accumulation; original context preserved; new context starts fresh).

---

## 5. Recovery Procedures

### BLE Drop Mid-Demo

Symptom: Dashboard shows `Disconnected`. Robot is unresponsive.

```bash
# 1. Press Ctrl+C to stop the companion

# 2. Restart from repo root — coherence DB is preserved on disk
cargo run -p mbot-companion --features "brain,voice-api" -- \
  --bluetooth \
  --voice-api \
  --voice-port 8080
```

Wait 10 seconds for BLE reconnection. Reload the dashboard tab. Verify the pre-roll table before resuming filming. If the take was in progress, restart the section from the clean-state procedure.

### Mid-Demo Crash (Companion Process Exits Unexpectedly)

Symptom: Terminal shows a panic or the process exits with a non-zero code.

```bash
# 1. Note the error message from the terminal — save it for debugging

# 2. Restart the companion
cargo run -p mbot-companion --features "brain,voice-api" -- \
  --bluetooth \
  --voice-api \
  --voice-port 8080
```

If the crash repeats on startup, run with verbose logging to capture more detail:

```bash
cargo run -p mbot-companion --features "brain,voice-api" -- \
  --bluetooth \
  --voice-api \
  --voice-port 8080 \
  --verbose
```

Do not continue filming until the binary starts cleanly. Run the clean-state procedure and re-verify the pre-roll table before the next take.

### Robot Unresponsive (Hardware)

Symptom: BLE connects but robot does not move or LED does not change.

1. Check battery level on CyberPi display. Charge if below 20%.
2. Press the reset button on the mBot2 (small button on the CyberPi board).
3. Wait 10 seconds for the robot to reboot. The CyberPi display should show "BLE Ready".
4. The companion will automatically reconnect. Verify pre-roll table before filming.

### Wrong Social Phase at Section Start

Symptom: Dashboard shows a phase other than `ShyObserver` when Section 1 or 2 should start clean.

```bash
# Run the clean-state procedure
rm -f ~/.mbot/coherence.db

cargo run -p mbot-companion --features "brain,voice-api" -- \
  --bluetooth \
  --voice-api \
  --voice-port 8080
```

Reload the dashboard and verify `social_phase = ShyObserver` and `context_coherence = 0%` before filming.

### Dashboard Not Updating

Symptom: Browser shows stale values or a blank page.

1. Confirm the companion process is running (check the terminal for active log output).
2. Hard-reload the browser tab: `Ctrl+Shift+R` (Linux/Windows) or `Cmd+Shift+R` (macOS).
3. If the page does not load, confirm the companion started with `--voice-api` and `--voice-port 8080` and that no other process is bound to port 8080.

To check port 8080:

```bash
ss -tlnp | grep 8080
```

---

## 6. Post-Demo: Save Session Log

Run these commands immediately after filming wraps. The coherence DB captures the full trust accumulation trajectory observed in the demo.

```bash
# 1. Create the archive directory if it does not exist
mkdir -p ~/demo-sessions

# 2. Copy the coherence DB with a timestamped filename
cp ~/.mbot/coherence.db ~/demo-sessions/$(date +%Y%m%d-%H%M%S)-coherence.db

# 3. Confirm the file was saved
ls -lh ~/demo-sessions/
```

The saved file is the on-disk evidence corresponding to the video. Keep both together. The DB can be inspected post-filming to verify the coherence trajectory matches what was visible in the dashboard during filming.
