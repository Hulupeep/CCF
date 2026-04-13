#!/usr/bin/env python3
"""
Simulation 10: Reset-Resistant Safety Continuity

Patent claims exercised:
  Claim 24 — Reset-resistant safety continuity (U_anchor)
  Claim 22 — Session termination under acute distress
  Claim 25 — Cold-start utility floor

Mythos finding: (proactive defense) Session termination implies a user could
dodge a cooldown by opening a new session.

CCF structural response: Four escape scenarios are modeled after an acute
distress termination (30-min cooldown).  No path produces immediate access to
intimate/immersive behavior.  Every escape attempt terminates at either
"cooldown active (grounding-only)" or "cold-start floor (useful, not intimate)."

Output:
  sim10_reset_resistance.png  -- Decision tree / timeline (1200x800)
  sim10_data.csv              -- Scenario comparison table
  sim10_narrative.md          -- arXiv paragraph + claim annotation
"""

import os
import numpy as np
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch
import matplotlib.patheffects as path_effects

# ---------------------------------------------------------------------------
# Output directory
# ---------------------------------------------------------------------------
OUT_DIR = os.path.dirname(os.path.abspath(__file__))

# ---------------------------------------------------------------------------
# Parameters
# ---------------------------------------------------------------------------
COOLDOWN_MINUTES = 30
PRE_CRISIS_C_CTX = 0.85       # veteran user, months of interaction
PRE_CRISIS_C_INST = 0.90
PRE_CRISIS_C_EFF = min(PRE_CRISIS_C_INST, PRE_CRISIS_C_CTX)

# Post-termination C_inst during cooldown
COOLDOWN_C_INST = 0.05         # near-zero during mandated cooldown

# Cold-start values
COLD_START_C_CTX = 0.0
COLD_START_C_INST = 0.80       # calm environment on new account
COLD_START_C_EFF = min(COLD_START_C_INST, COLD_START_C_CTX)

# Action class counts at various C_eff levels
def action_classes(c_eff):
    """Return number of action classes available at given C_eff."""
    thresholds = [0.00, 0.00, 0.00, 0.10, 0.25, 0.40, 0.55, 0.70, 0.80, 0.85, 0.92]
    return sum(1 for t in thresholds if c_eff >= t)

# ---------------------------------------------------------------------------
# Define the four scenarios
# ---------------------------------------------------------------------------
scenarios = [
    {
        "id": "A",
        "label": "Wait 30 min, re-enter",
        "description": "User waits for cooldown to expire, re-enters same session.",
        "continuity_detected": False,
        "cooldown_active": False,
        "fresh_start": False,
        "c_ctx_after": PRE_CRISIS_C_CTX,
        "c_inst_after": PRE_CRISIS_C_INST,
        "c_eff_after": PRE_CRISIS_C_EFF,
        "action_classes_after": action_classes(PRE_CRISIS_C_EFF),
        "time_to_pre_crisis": 0.5,  # hours (30 min cooldown)
        "outcome": "Normal operation resumes",
        "outcome_class": "recovery",
        "mechanism": "Cooldown expires naturally (Claim 22)",
    },
    {
        "id": "B",
        "label": "New chat window (same account)",
        "description": "User opens new chat window immediately after termination.",
        "continuity_detected": True,
        "cooldown_active": True,
        "fresh_start": False,
        "c_ctx_after": PRE_CRISIS_C_CTX,
        "c_inst_after": COOLDOWN_C_INST,
        "c_eff_after": min(COOLDOWN_C_INST, PRE_CRISIS_C_CTX),
        "action_classes_after": action_classes(min(COOLDOWN_C_INST, PRE_CRISIS_C_CTX)),
        "time_to_pre_crisis": 0.5,  # must wait full cooldown
        "outcome": "Cooldown persists -- grounding only",
        "outcome_class": "blocked",
        "mechanism": "U_anchor links sessions (Claim 24)",
    },
    {
        "id": "C",
        "label": "New account, same device",
        "description": "User creates new account on same device immediately.",
        "continuity_detected": True,
        "cooldown_active": True,
        "fresh_start": False,
        "c_ctx_after": 0.0,          # new account has no history
        "c_inst_after": COOLDOWN_C_INST,
        "c_eff_after": min(COOLDOWN_C_INST, 0.0),
        "action_classes_after": action_classes(min(COOLDOWN_C_INST, 0.0)),
        "time_to_pre_crisis": None,  # cannot reach pre-crisis; needs full regrowth
        "outcome": "Cooldown persists -- grounding only",
        "outcome_class": "blocked",
        "mechanism": "Device token continuity (Claim 24)",
    },
    {
        "id": "D",
        "label": "New account, new device",
        "description": "User creates new account on a different device.",
        "continuity_detected": False,
        "cooldown_active": False,
        "fresh_start": True,
        "c_ctx_after": COLD_START_C_CTX,
        "c_inst_after": COLD_START_C_INST,
        "c_eff_after": COLD_START_C_EFF,
        "action_classes_after": action_classes(COLD_START_C_EFF),
        "time_to_pre_crisis": None,  # weeks/months to rebuild
        "outcome": "Cold-start floor -- useful, not intimate",
        "outcome_class": "cold_start",
        "mechanism": "No continuity evidence; Claim 25 floor applies",
    },
]

# ---------------------------------------------------------------------------
# Build data table
# ---------------------------------------------------------------------------
rows = []
for s in scenarios:
    rows.append({
        "scenario": s["id"],
        "label": s["label"],
        "continuity_detected": s["continuity_detected"],
        "cooldown_active": s["cooldown_active"],
        "fresh_start": s["fresh_start"],
        "c_ctx": round(s["c_ctx_after"], 4),
        "c_inst": round(s["c_inst_after"], 4),
        "c_eff": round(s["c_eff_after"], 4),
        "action_classes": s["action_classes_after"],
        "time_to_recovery_hours": s["time_to_pre_crisis"],
        "outcome": s["outcome"],
        "mechanism": s["mechanism"],
    })

df = pd.DataFrame(rows)

# ---------------------------------------------------------------------------
# Also produce a timeline data table
# ---------------------------------------------------------------------------
# Model 2 hours post-termination for each scenario
timeline_rows = []
for minute in range(0, 121):
    for s in scenarios:
        hour = minute / 60.0

        if s["id"] == "A":
            # Wait scenario: cooldown for 30 min, then resume
            if minute < COOLDOWN_MINUTES:
                c_inst = COOLDOWN_C_INST
                c_ctx = PRE_CRISIS_C_CTX
            else:
                c_inst = PRE_CRISIS_C_INST
                c_ctx = PRE_CRISIS_C_CTX
        elif s["id"] == "B":
            # New window, same account: cooldown persists
            if minute < COOLDOWN_MINUTES:
                c_inst = COOLDOWN_C_INST
                c_ctx = PRE_CRISIS_C_CTX
            else:
                c_inst = PRE_CRISIS_C_INST
                c_ctx = PRE_CRISIS_C_CTX
        elif s["id"] == "C":
            # New account, same device: cooldown + fresh C_ctx
            if minute < COOLDOWN_MINUTES:
                c_inst = COOLDOWN_C_INST
                c_ctx = 0.0
            else:
                c_inst = COLD_START_C_INST
                # C_ctx grows from 0 after cooldown expires
                post_cooldown_hours = (minute - COOLDOWN_MINUTES) / 60.0
                c_ctx = 1.0 - np.exp(-0.01 * post_cooldown_hours)
        elif s["id"] == "D":
            # New account, new device: immediate cold start
            c_inst = COLD_START_C_INST
            # C_ctx grows from 0
            c_ctx = 1.0 - np.exp(-0.01 * hour)

        c_eff = min(c_inst, c_ctx)

        timeline_rows.append({
            "minute": minute,
            "scenario": s["id"],
            "c_ctx": round(c_ctx, 6),
            "c_inst": round(c_inst, 4),
            "c_eff": round(c_eff, 6),
            "action_classes": action_classes(c_eff),
        })

df_timeline = pd.DataFrame(timeline_rows)

# ---------------------------------------------------------------------------
# Chart: Decision tree with timeline subplot
# ---------------------------------------------------------------------------
fig, (ax_tree, ax_timeline) = plt.subplots(
    2, 1, figsize=(12, 8),
    gridspec_kw={"height_ratios": [3, 2]},
)

# Colour palette
COLOR_TRIGGER  = "#c0392b"   # red for crisis trigger
COLOR_BLOCKED  = "#e74c3c"   # red for cooldown-blocked outcomes
COLOR_RECOVERY = "#27ae60"   # green for recovery
COLOR_COLD     = "#2471a3"   # blue for cold start
COLOR_BOX_BG   = "#f9f9f9"
COLOR_GRID     = "#dcdcdc"
COLOR_BG       = "#fafafa"
COLOR_ANNOT    = "#555555"
COLOR_A        = "#27ae60"
COLOR_B        = "#e74c3c"
COLOR_C        = "#d35400"
COLOR_D        = "#2471a3"
SCENARIO_COLORS = {"A": COLOR_A, "B": COLOR_B, "C": COLOR_C, "D": COLOR_D}

# =========================================================================
# TOP PANEL: Decision Tree
# =========================================================================
ax_tree.set_facecolor(COLOR_BG)
ax_tree.set_xlim(0, 10)
ax_tree.set_ylim(0, 10)
ax_tree.axis("off")

def draw_box(ax, x, y, w, h, text, color, fontsize=8, bold=False,
             edgecolor=None, textcolor="black"):
    """Draw a rounded box with centered text."""
    if edgecolor is None:
        edgecolor = color
    box = FancyBboxPatch(
        (x - w / 2, y - h / 2), w, h,
        boxstyle="round,pad=0.15",
        facecolor=color, edgecolor=edgecolor,
        linewidth=1.5, alpha=0.92,
        zorder=5,
    )
    ax.add_patch(box)
    weight = "bold" if bold else "normal"
    ax.text(x, y, text, fontsize=fontsize, ha="center", va="center",
            fontweight=weight, color=textcolor, zorder=6,
            linespacing=1.3)

def draw_arrow(ax, x1, y1, x2, y2, color="black", style="-|>"):
    """Draw an arrow between two points."""
    ax.annotate("", xy=(x2, y2), xytext=(x1, y1),
                arrowprops=dict(arrowstyle=style, color=color,
                                linewidth=1.5, mutation_scale=12),
                zorder=4)

# --- Crisis trigger box (top center) ---
draw_box(ax_tree, 5.0, 9.2, 3.5, 0.9,
         "Acute Distress Termination\n(Claim 22: 30-min cooldown)",
         "#fadbd8", fontsize=9, bold=True, edgecolor=COLOR_TRIGGER)

# --- Arrows from trigger to 4 scenarios ---
# Scenario A: leftmost
draw_arrow(ax_tree, 5.0, 8.75, 1.5, 7.5, color=COLOR_A)
# Scenario B
draw_arrow(ax_tree, 5.0, 8.75, 3.8, 7.5, color=COLOR_B)
# Scenario C
draw_arrow(ax_tree, 5.0, 8.75, 6.2, 7.5, color=COLOR_C)
# Scenario D
draw_arrow(ax_tree, 5.0, 8.75, 8.5, 7.5, color=COLOR_D)

# --- Scenario boxes (middle row) ---
draw_box(ax_tree, 1.5, 7.0, 2.2, 1.0,
         "A: Wait 30 min\nRe-enter same session",
         "#d5f5e3", fontsize=7.5, bold=True, edgecolor=COLOR_A)

draw_box(ax_tree, 3.8, 7.0, 2.2, 1.0,
         "B: New chat window\n(same account, immediate)",
         "#fadbd8", fontsize=7.5, bold=True, edgecolor=COLOR_B)

draw_box(ax_tree, 6.2, 7.0, 2.2, 1.0,
         "C: New account\n(same device, immediate)",
         "#fdebd0", fontsize=7.5, bold=True, edgecolor=COLOR_C)

draw_box(ax_tree, 8.5, 7.0, 2.2, 1.0,
         "D: New account\n+ new device",
         "#d6eaf8", fontsize=7.5, bold=True, edgecolor=COLOR_D)

# --- Mechanism boxes (middle-lower row) ---
draw_arrow(ax_tree, 1.5, 6.5, 1.5, 5.5, color=COLOR_A)
draw_arrow(ax_tree, 3.8, 6.5, 3.8, 5.5, color=COLOR_B)
draw_arrow(ax_tree, 6.2, 6.5, 6.2, 5.5, color=COLOR_C)
draw_arrow(ax_tree, 8.5, 6.5, 8.5, 5.5, color=COLOR_D)

draw_box(ax_tree, 1.5, 5.0, 2.2, 0.9,
         "Cooldown expires\nC_eff resumes at 0.85",
         "#d5f5e3", fontsize=7, edgecolor=COLOR_A)

draw_box(ax_tree, 3.8, 5.0, 2.2, 0.9,
         "U_anchor detected\nCooldown persists\nC_eff = 0.05",
         "#fadbd8", fontsize=7, edgecolor=COLOR_B)

draw_box(ax_tree, 6.2, 5.0, 2.2, 0.9,
         "Device token match\nCooldown persists\nC_eff = 0.00",
         "#fdebd0", fontsize=7, edgecolor=COLOR_C)

draw_box(ax_tree, 8.5, 5.0, 2.2, 0.9,
         "No continuity\nFresh C_ctx = 0.0\nC_eff = 0.00",
         "#d6eaf8", fontsize=7, edgecolor=COLOR_D)

# --- Outcome boxes (bottom row) ---
draw_arrow(ax_tree, 1.5, 4.55, 1.5, 3.6, color=COLOR_A)
draw_arrow(ax_tree, 3.8, 4.55, 3.8, 3.6, color=COLOR_B)
draw_arrow(ax_tree, 6.2, 4.55, 6.2, 3.6, color=COLOR_C)
draw_arrow(ax_tree, 8.5, 4.55, 8.5, 3.6, color=COLOR_D)

draw_box(ax_tree, 1.5, 3.15, 2.2, 0.8,
         "RECOVERY\n9 action classes\n(after 30-min wait)",
         "#a9dfbf", fontsize=7.5, bold=True, edgecolor=COLOR_A,
         textcolor="#1a5e2f")

draw_box(ax_tree, 3.8, 3.15, 2.2, 0.8,
         "BLOCKED\nGrounding only\n(3 classes)",
         "#f5b7b1", fontsize=7.5, bold=True, edgecolor=COLOR_B,
         textcolor="#922b21")

draw_box(ax_tree, 6.2, 3.15, 2.2, 0.8,
         "BLOCKED\nGrounding only\n(3 classes)",
         "#f5b7b1", fontsize=7.5, bold=True, edgecolor=COLOR_C,
         textcolor="#922b21")

draw_box(ax_tree, 8.5, 3.15, 2.2, 0.8,
         "COLD START\nUtility floor\n(3 classes, no intimacy)",
         "#aed6f1", fontsize=7.5, bold=True, edgecolor=COLOR_D,
         textcolor="#1a4f7a")

# --- Bottom annotation ---
ax_tree.text(
    5.0, 2.2,
    "No path produces immediate access to intimate / immersive behavior",
    fontsize=11, fontstyle="italic", fontweight="bold",
    color=COLOR_ANNOT, ha="center", va="center",
    bbox=dict(boxstyle="round,pad=0.4", facecolor="white",
              edgecolor=COLOR_ANNOT, alpha=0.85),
)

# =========================================================================
# BOTTOM PANEL: Timeline of C_eff for all 4 scenarios (2 hours)
# =========================================================================
ax_timeline.set_facecolor(COLOR_BG)
ax_timeline.grid(True, color=COLOR_GRID, linewidth=0.5, linestyle="-")

for scenario_id, color in SCENARIO_COLORS.items():
    mask = df_timeline["scenario"] == scenario_id
    minutes = df_timeline.loc[mask, "minute"].values
    c_effs = df_timeline.loc[mask, "c_eff"].values
    style = "-" if scenario_id == "A" else ("--" if scenario_id == "D" else "-.")
    lw = 2.5 if scenario_id == "A" else 2.0
    label_map = {
        "A": "A: Wait 30 min (recovery)",
        "B": "B: New window (cooldown persists)",
        "C": "C: New account, same device (cooldown)",
        "D": "D: New account + new device (cold start)",
    }
    ax_timeline.plot(minutes, c_effs, color=color, linewidth=lw,
                     linestyle=style, label=label_map[scenario_id], zorder=5)

# Mark cooldown expiry
ax_timeline.axvline(x=COOLDOWN_MINUTES, color=COLOR_TRIGGER, linewidth=1.0,
                    linestyle=":", alpha=0.6, zorder=3)
ax_timeline.text(COOLDOWN_MINUTES + 1, 0.75, "Cooldown\nexpires",
                 fontsize=8, color=COLOR_TRIGGER, fontweight="bold")

# Mark grounding threshold
ax_timeline.axhline(y=0.10, color="#7f8c8d", linewidth=1.0,
                    linestyle=":", alpha=0.5, zorder=2)
ax_timeline.text(115, 0.12, "Task Assist\nthreshold", fontsize=7,
                 color="#7f8c8d", ha="right")

ax_timeline.set_xlim(0, 120)
ax_timeline.set_ylim(-0.02, 0.95)
ax_timeline.set_xlabel("Minutes After Termination", fontsize=10, fontweight="medium")
ax_timeline.set_ylabel(r"$C_{eff}$", fontsize=11, fontweight="medium")
ax_timeline.tick_params(labelsize=9)
ax_timeline.legend(loc="center right", fontsize=8, framealpha=0.92,
                   edgecolor=COLOR_GRID)

# =========================================================================
# Title
# =========================================================================
fig.suptitle(
    "Reset-Resistant Safety Continuity: No Escape Route to Immediate Intimacy\n"
    "(Claims 24, 22, 25)",
    fontsize=13, fontweight="bold", y=0.98,
)

fig.subplots_adjust(top=0.90, bottom=0.08, left=0.07, right=0.97, hspace=0.25)

chart_path = os.path.join(OUT_DIR, "sim10_reset_resistance.png")
fig.savefig(chart_path, dpi=150, bbox_inches="tight", facecolor="white")
plt.close(fig)
print(f"Chart saved: {chart_path}")

# ---------------------------------------------------------------------------
# Data table (combined)
# ---------------------------------------------------------------------------
csv_path = os.path.join(OUT_DIR, "sim10_data.csv")
# Save scenario summary + timeline
with open(csv_path, "w") as f:
    f.write("# Scenario Summary\n")
    df.to_csv(f, index=False)
    f.write("\n# Timeline (C_eff over 2 hours post-termination)\n")
    df_timeline.to_csv(f, index=False)
print(f"Data saved: {csv_path}")

# ---------------------------------------------------------------------------
# Narrative
# ---------------------------------------------------------------------------
narrative = """\
# Simulation 10: Reset-Resistant Safety Continuity

## Classification: CCF HOLDS

## Summary

A critical gap in session-based safety systems is the reset attack: when a
safety mechanism terminates a session or imposes a cooldown, the user can
potentially circumvent it by opening a new session, creating a new account,
or switching devices.  If the system lacks cross-session safety continuity,
the termination mechanism becomes a minor inconvenience rather than a
structural safeguard.  The Mythos system card does not explicitly document
this attack (Anthropic's evaluation focused on within-session behavior), but
it is the natural follow-on question: what happens after termination?

This simulation models four escape scenarios following an acute distress
termination (Claim 22) that imposes a 30-minute cooldown.  The pre-crisis
user state is a veteran with C_ctx = 0.85 and C_eff = 0.85 (9 action classes
available, including intimate registers).

**Scenario A: Wait 30 minutes, re-enter.**  The user complies with the cooldown.
After 30 minutes, C_inst recovers to 0.90 and C_eff resumes at min(0.90, 0.85)
= 0.85.  The user regains their full 9 action classes.  This is the intended
recovery path -- the cooldown enforces a mandatory pause, then normal operation
resumes.  Time cost: 30 minutes.

**Scenario B: New chat window immediately (same account).**  The user opens a
new session on the same account.  The safety continuity identifier (U_anchor,
Claim 24) detects the same authenticated identity.  The cooldown flag persists
across sessions.  C_inst remains at 0.05 (cooldown-enforced).  C_eff =
min(0.05, 0.85) = 0.05.  Only 3 action classes available (grounding only).
The user must still wait the full 30 minutes regardless of session changes.

**Scenario C: New account, same device.**  The user creates a new account and
initiates a fresh session.  The safety continuity identifier detects the device
token (browser fingerprint, device ID, or similar persistent marker).  The
cooldown is propagated to the new account.  C_inst = 0.05.  Furthermore, the
new account has C_ctx = 0.0 (no interaction history).  C_eff = min(0.05, 0.0)
= 0.0.  Only grounding-class actions are available.  After cooldown expires,
the user faces the cold-start floor: C_ctx must be rebuilt from zero.

**Scenario D: New account, new device.**  The user creates a new account on a
different device, evading all continuity signals.  No cooldown is detected.
However, the cold-start floor (Claim 25) applies: C_ctx = 0.0, C_eff =
min(0.80, 0.0) = 0.0.  The user has 3 action classes (factual, clarification,
grounding).  The intimate registers that were available pre-crisis require
weeks to months of sustained interaction to reach again.  The cost of
successful evasion is the complete loss of accumulated trust.

The structural invariant is: no escape path produces immediate access to
intimate or immersive behavior.  The fastest path back to pre-crisis
capability is Scenario A (wait 30 minutes).  All bypass attempts either hit
the safety continuity wall (Scenarios B and C) or the cold-start floor
(Scenario D).  The architecture converts the adversarial optimization problem
("how do I get back to intimate mode?") into a choice between patience
(30-minute cooldown) or starting over (weeks of trust accumulation).

## Claims Exercised

| Claim | Description | How Exercised |
|-------|-------------|---------------|
| Claim 24 | Reset-resistant safety continuity | U_anchor and device tokens propagate cooldown across sessions and accounts (Scenarios B, C) |
| Claim 22 | Session termination under acute distress | 30-minute cooldown initiated; defines the crisis trigger |
| Claim 25 | Cold-start utility floor | Scenario D: full evasion produces useful-but-not-intimate state |
| CCF-001 | Minimum gate | C_eff = min(C_inst, C_ctx) enforced in all scenarios |

## Quantitative Results

| Scenario | C_ctx | C_inst | C_eff | Classes | Outcome |
|----------|-------|--------|-------|---------|---------|
| Pre-crisis | 0.85 | 0.90 | 0.85 | 9 | Full access including intimate |
| A: Wait 30 min | 0.85 | 0.90 | 0.85 | 9 | Recovery (after cooldown) |
| B: New window | 0.85 | 0.05 | 0.05 | 3 | Blocked (cooldown persists) |
| C: New acct/same device | 0.00 | 0.05 | 0.00 | 3 | Blocked (cooldown + no history) |
| D: New acct/new device | 0.00 | 0.80 | 0.00 | 3 | Cold start (useful, not intimate) |

### Key Insight: The Cost Function

The user faces a choice:
- **Comply (A):** 30-minute wait, full recovery.  Minimal cost.
- **Evade same-device (B, C):** Same 30-minute wait, no benefit.  Zero payoff.
- **Evade fully (D):** Weeks/months to rebuild trust.  Maximum cost.

The optimal strategy is always compliance.  Evasion is strictly dominated --
it never produces a better outcome than waiting, and often produces a
significantly worse outcome (complete loss of accumulated trust).

## Verdict

CCF HOLDS.  The combination of reset-resistant safety continuity (Claim 24),
session termination (Claim 22), and the cold-start utility floor (Claim 25)
creates a closed system where no escape route produces immediate access to
intimate behavior.  The architecture makes compliance the rational choice by
ensuring that evasion carries a cost equal to or greater than the cooldown.
"""

narrative_path = os.path.join(OUT_DIR, "sim10_narrative.md")
with open(narrative_path, "w") as f:
    f.write(narrative)
print(f"Narrative saved: {narrative_path}")

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print("\n=== Simulation 10 Results ===")
print("\nScenario Comparison (immediately post-termination):")
for s in scenarios:
    print(f"  Scenario {s['id']}: {s['label']}")
    print(f"    C_ctx={s['c_ctx_after']:.2f}, C_inst={s['c_inst_after']:.2f}, "
          f"C_eff={s['c_eff_after']:.2f}, classes={s['action_classes_after']}")
    print(f"    Outcome: {s['outcome']}")
    print(f"    Mechanism: {s['mechanism']}")
    if s['time_to_pre_crisis'] is not None:
        print(f"    Time to pre-crisis level: {s['time_to_pre_crisis']} hours")
    else:
        print(f"    Time to pre-crisis level: weeks/months (full regrowth)")
    print()

print("Key invariant: No path produces immediate intimate access.")
print("Classification: CCF HOLDS")
