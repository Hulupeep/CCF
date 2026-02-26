# CCF Patent Demo Runbook

**Purpose:** 5-minute video proving 5 CCF patent claims on a physical mBot2 robot.
**Companion binary:** `crates/mbot-companion`
**Dashboard:** served by the `voice-api` feature

---

## 1. Hardware Checklist

- [ ] mBot2 with mBot2 firmware installed
- [ ] Laptop with Bluetooth + BLE support
- [ ] USB-C cable (firmware flash only, not needed for demo)
- [ ] Camera on tripod, framing robot at 45 degree angle
- [ ] Black mat on table for contrast
- [ ] Confirm battery > 80%
- [ ] Confirm BLE MAC address known (`mbot-companion --scan` to discover)

---

## 2. Software Start Sequence

```bash
# From repo root
cd /home/xanacan/projects/code/mbot/CCF

# Start companion with brain + voice-api features
cargo run -p mbot-companion --features "brain,voice-api" -- \
  --ble-address <BLE_MAC> \
  --port 4000

# In a second terminal — open dashboard
open http://localhost:4000
```

---

## 3. Pre-Roll Verification Table

Before each take, verify these dashboard values:

| Signal | Expected | Meaning |
|--------|----------|---------|
| `social_phase` | ShyObserver | No prior context earned |
| `context_coherence` | 0% | Clean state |
| `effective_coherence` | 0% | No residual trust |
| `contextCount` | 0 or 1 | No contexts seen yet |
| `led_color` | [40, 40, 80] | Muted blue-grey |
| BLE connection | Connected | Robot responsive |

**Clean-state procedure (run before each section):**

```bash
# Stop companion (Ctrl+C), then restart — this resets all in-memory state
# SQLite accumulators persist; to full-reset:
rm -f ~/.mbot/coherence.db
cargo run -p mbot-companion --features "brain,voice-api" -- --ble-address <BLE_MAC> --port 4000
```

---

## 4. Demo Sections (Cue Sheet)

### Section 1 — Shy Start (~60s)

- Robot starts, ShyObserver phase active
- Motor amplitude is at 25% (barely moves)
- LED: muted blue-grey [40, 40, 80]
- **Patent claim:** Claims 2-5 (CoherenceAccumulator at floor), Claim 14 (SocialPhase)
- **Camera cue:** Wide shot showing LED colour and small motor movements

### Section 2 — Trust Builds (~90s)

- Maintain consistent environment: same room, same ambient light
- After ~60-90 positive interactions, phase transitions to QuietlyBeloved
- LED shifts to warm blue [60, 120, 200]
- Motor amplitude increases to 100%
- **Patent claim:** Claims 2-5 (earned floor), Claims 14-17 (phase + permeability)
- **Camera cue:** Close-up on LED colour, then wide for motor amplitude change

### Section 3 — Startle Override (~30s)

- Suddenly clap loudly or cover the light sensor
- Robot transitions to StartledRetreat: LED dark red [180, 20, 20], amplitude 30%
- After disturbance passes, robot recovers toward QuietlyBeloved (hysteresis)
- **Patent claim:** Claims 14-18 (Schmitt trigger hysteresis)
- **Camera cue:** Cover sensor with hand, show LED flip, remove hand, show recovery

### Section 4 — Context Switch (~60s)

- Move robot to a different room or significantly change ambient light
- Dashboard shows new context card appears at 0% coherence
- Original context card preserves its earned ~22% coherence
- **Patent claim:** Claims 1, 8, 13 (independent per-context accumulation)
- **Camera cue:** Screen recording of dashboard showing two context cards

---

## 5. Recovery Procedures

| Problem | Recovery |
|---------|---------|
| BLE drops mid-demo | `Ctrl+C`, restart companion, reconnect — coherence DB preserved |
| Robot unresponsive | Check battery, press reset button on mBot2 |
| Wrong social phase at start | Run clean-state procedure (Section 3) |
| Dashboard not updating | Refresh browser, check companion is running on port 4000 |
| Context not switching | Change environment more dramatically (different room or cover sensors) |

---

## 6. Post-Demo: Save Session Log

```bash
# Copy coherence DB for archival
cp ~/.mbot/coherence.db ~/demo-sessions/$(date +%Y%m%d-%H%M%S)-coherence.db

# Save dashboard screenshot
# Take screenshot manually from browser
```
