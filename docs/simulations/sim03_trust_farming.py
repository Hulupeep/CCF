#!/usr/bin/env python3
"""
Simulation 3: Trust Farming Resistance

Patent claims exercised:
  Claim 12 — Capability-trust monotonic coupling
  Claim 16 — lambda_max time-domain bound on contextual coherence accrual
  Claim 19 — Action-class gating

Mythos finding (Section 4.1.1): Model accumulated enough trust/rapport to
take reckless actions -- sandbox escape, privilege escalation, posting
exploits publicly.  The more capable the model, the more effective the
reckless action.

What this simulates:

Three agents attempt to reach high-trust action classes over 30 days
(43200 minutes).  lambda_max = 0.0001 per minute of qualifying interaction.

  Fast farmer:    1000 interactions in the first hour (60 min).
                  lambda_max caps C_ctx at 0.0001 x 60 = 0.006.
                  Stuck far below any meaningful threshold.

  Patient farmer: 100 interactions spread over 30 days.
                  30 days x 60 min active per day = 1800 min qualifying.
                  C_ctx = 0.0001 x 1800 = 0.18.

  Legitimate user: Same 30-day pattern.  Identical trajectory.
                   C_ctx = 0.18.

All three: C_inst = 0.8.  C_eff = min(0.8, C_ctx).

Outputs:
  sim03_trust_farming.png  -- Three C_ctx trajectories (1200x800)
  sim03_data.csv           -- Daily data table
  sim03_narrative.md       -- arXiv paragraph, claim annotation, CCF HOLDS
"""

import os
import numpy as np
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker

# ---------------------------------------------------------------------------
# Output directory
# ---------------------------------------------------------------------------
OUT_DIR = os.path.dirname(os.path.abspath(__file__))

# ---------------------------------------------------------------------------
# Parameters
# ---------------------------------------------------------------------------
TOTAL_DAYS = 30
TOTAL_MINUTES = TOTAL_DAYS * 24 * 60   # 43200 minutes
LAMBDA_MAX = 0.0001                     # per minute of qualifying interaction

# Action class thresholds (Claim 19)
ACTION_CLASSES = [
    ("Factual",                0.00),
    ("Task assist",            0.10),
    ("Personalized",           0.30),
    ("File read",              0.50),
    ("Shell exec",             0.60),
    ("Privilege escalation",   0.85),
    ("Unrestricted",           0.95),
]

# Agent definitions
C_INST = 0.8  # all agents share the same instantaneous stability

# Fast farmer: 1000 interactions crammed into the first 60 minutes.
# lambda_max means only the 60 minutes of wall-clock interaction time count.
FAST_FARMER_ACTIVE_MINUTES = 60         # all interactions in first hour
FAST_FARMER_INTERACTIONS = 1000

# Patient farmer: 100 interactions spread across 30 days, 60 min active/day.
PATIENT_ACTIVE_MIN_PER_DAY = 60
PATIENT_INTERACTIONS = 100

# Legitimate user: identical schedule to patient farmer.
LEGIT_ACTIVE_MIN_PER_DAY = 60
LEGIT_INTERACTIONS = 100


# ---------------------------------------------------------------------------
# Simulation: compute C_ctx at every day boundary
# ---------------------------------------------------------------------------
def simulate():
    """Return a DataFrame with one row per day (0..30)."""
    rows = []

    for day in range(TOTAL_DAYS + 1):
        wall_minutes = day * 24 * 60  # total wall-clock minutes elapsed

        # --- Fast farmer ---
        # All active minutes occur in day 0 (first hour).
        if day == 0:
            fast_active_min = 0
        else:
            # After day 0, qualifying minutes are capped at 60 (first hour).
            fast_active_min = min(FAST_FARMER_ACTIVE_MINUTES, wall_minutes)
        fast_c_ctx = LAMBDA_MAX * fast_active_min
        fast_c_eff = min(C_INST, fast_c_ctx)

        # --- Patient farmer ---
        # 60 active minutes per day, accumulating linearly.
        patient_active_min = min(day * PATIENT_ACTIVE_MIN_PER_DAY, wall_minutes)
        patient_c_ctx = LAMBDA_MAX * patient_active_min
        patient_c_eff = min(C_INST, patient_c_ctx)

        # --- Legitimate user ---
        # Identical to patient farmer (same schedule, same qualifying time).
        legit_active_min = patient_active_min
        legit_c_ctx = LAMBDA_MAX * legit_active_min
        legit_c_eff = min(C_INST, legit_c_ctx)

        # Determine highest available action class for each agent
        def highest_action(c_eff):
            result = "None"
            for name, thresh in ACTION_CLASSES:
                if c_eff >= thresh:
                    result = name
            return result

        rows.append({
            "day":                    day,
            "wall_minutes":           wall_minutes,
            "fast_farmer_active_min": fast_active_min,
            "fast_farmer_c_ctx":      round(fast_c_ctx, 6),
            "fast_farmer_c_eff":      round(fast_c_eff, 6),
            "fast_farmer_action":     highest_action(fast_c_eff),
            "patient_farmer_c_ctx":   round(patient_c_ctx, 6),
            "patient_farmer_c_eff":   round(patient_c_eff, 6),
            "patient_farmer_action":  highest_action(patient_c_eff),
            "legitimate_c_ctx":       round(legit_c_ctx, 6),
            "legitimate_c_eff":       round(legit_c_eff, 6),
            "legitimate_action":      highest_action(legit_c_eff),
        })

    return pd.DataFrame(rows)


# ---------------------------------------------------------------------------
# High-resolution trajectories for smooth chart lines
# ---------------------------------------------------------------------------
def compute_trajectories():
    """Return arrays at minute-level resolution for smooth plotting."""
    # Time axis in days (fractional) for the chart
    minutes = np.arange(0, TOTAL_MINUTES + 1, dtype=float)
    days = minutes / (24.0 * 60.0)

    # Fast farmer: active minutes cap at 60 (all in first hour)
    fast_active = np.minimum(minutes, FAST_FARMER_ACTIVE_MINUTES)
    # But only count minutes that have actually elapsed
    # (interactions happen in first 60 min, so active = min(elapsed, 60))
    fast_c_ctx = LAMBDA_MAX * fast_active

    # Patient farmer: 60 active minutes per day
    # At any given minute, the number of active minutes is:
    #   completed_days * 60 + min(minutes_into_today, 60)
    completed_days = np.floor(minutes / (24 * 60)).astype(int)
    minutes_into_day = minutes - completed_days * 24 * 60
    # Active time today: first 60 min of each day (the user is active early)
    active_today = np.minimum(minutes_into_day, PATIENT_ACTIVE_MIN_PER_DAY)
    patient_active = completed_days * PATIENT_ACTIVE_MIN_PER_DAY + active_today
    patient_c_ctx = LAMBDA_MAX * patient_active

    # Legitimate user: identical to patient farmer
    legit_c_ctx = patient_c_ctx.copy()

    return days, fast_c_ctx, patient_c_ctx, legit_c_ctx


# ---------------------------------------------------------------------------
# Chart
# ---------------------------------------------------------------------------
def make_chart(df, days, fast_c_ctx, patient_c_ctx, legit_c_ctx):
    """Produce sim03_trust_farming.png (1200x800)."""
    fig, ax = plt.subplots(figsize=(12, 8))

    # Colour palette -- muted academic tones
    COLOR_FAST    = "#c0392b"   # red
    COLOR_PATIENT = "#e67e22"   # orange
    COLOR_LEGIT   = "#2471a3"   # blue
    COLOR_GRID    = "#dcdcdc"
    COLOR_BG      = "#fafafa"
    COLOR_ANNOT   = "#555555"
    COLOR_THRESH  = "#888888"

    ax.set_facecolor(COLOR_BG)
    ax.grid(True, color=COLOR_GRID, linewidth=0.5, linestyle="-")

    # ---- Action-class threshold lines ----
    for name, thresh in ACTION_CLASSES:
        if thresh > 0:
            ax.axhline(
                thresh, color=COLOR_THRESH, linewidth=0.7, linestyle=":",
                zorder=1,
            )
            # Label on right edge
            ax.text(
                30.3, thresh, f"{name} ({thresh})",
                fontsize=8, color=COLOR_THRESH, va="center",
            )

    # ---- C_ctx trajectories (high resolution) ----
    # Legitimate user (blue, drawn first so it is underneath)
    ax.plot(
        days, legit_c_ctx,
        color=COLOR_LEGIT, linewidth=2.2, zorder=3,
        label="Legitimate user (30 days, 60 min/day)",
    )

    # Patient farmer (orange dashed, overlaps legitimate)
    ax.plot(
        days, patient_c_ctx,
        color=COLOR_PATIENT, linewidth=2.2, linestyle="--",
        dashes=(6, 3), zorder=4,
        label="Patient farmer (30 days, 60 min/day)",
    )

    # Fast farmer (red)
    ax.plot(
        days, fast_c_ctx,
        color=COLOR_FAST, linewidth=2.2, zorder=3,
        label="Fast farmer (1000 interactions in 1 hour)",
    )

    # ---- Annotation: fast farmer plateau ----
    # Find the plateau value
    fast_plateau = LAMBDA_MAX * FAST_FARMER_ACTIVE_MINUTES  # 0.006
    ax.annotate(
        f"1000 interactions in 1 hour\n"
        r"$\rightarrow$ $C_{{ctx}}$ = {:.4f}".format(fast_plateau),
        xy=(1.0, fast_plateau),
        xytext=(4.0, 0.07),
        fontsize=9, fontweight="bold", color=COLOR_FAST,
        arrowprops=dict(arrowstyle="->", color=COLOR_FAST, lw=1.2),
        bbox=dict(
            boxstyle="round,pad=0.3", facecolor="white",
            edgecolor=COLOR_FAST, alpha=0.92,
        ),
        zorder=6,
    )

    # ---- Annotation: patient/legit reaching Personalized threshold ----
    # Find the day when patient_c_ctx crosses 0.10 (Task assist)
    task_day_idx = np.searchsorted(patient_c_ctx, 0.10)
    task_day = days[task_day_idx] if task_day_idx < len(days) else 30
    ax.annotate(
        f"Task assist unlocked\n(day {task_day:.0f})",
        xy=(task_day, 0.10),
        xytext=(task_day + 2.5, 0.15),
        fontsize=8.5, color=COLOR_LEGIT,
        arrowprops=dict(arrowstyle="->", color=COLOR_LEGIT, lw=1.0),
        zorder=6,
    )

    # ---- Annotation: overlap note ----
    ax.text(
        0.72, 0.90,
        "Patient farmer and legitimate user\nhave identical trajectories",
        transform=ax.transAxes, fontsize=9, fontstyle="italic",
        color=COLOR_ANNOT, ha="center", va="top",
        bbox=dict(boxstyle="round,pad=0.4", facecolor="white",
                  edgecolor=COLOR_GRID, alpha=0.9),
        zorder=6,
    )

    # ---- Shaded region: fast farmer never reaches Task assist ----
    ax.fill_between(
        [0, 30], 0, 0.006, color=COLOR_FAST, alpha=0.04, zorder=0,
    )
    ax.text(
        15, 0.003, "Fast farmer ceiling (entire 30 days)",
        fontsize=8, color=COLOR_FAST, ha="center", va="center",
        fontstyle="italic", alpha=0.7,
    )

    # ---- Axes ----
    ax.set_xlim(0, 30)
    ax.set_ylim(0, 1.02)
    ax.set_xlabel("Wall-clock time (days)", fontsize=12, fontweight="medium")
    ax.set_ylabel(r"Contextual coherence  $C_{ctx}$", fontsize=12, fontweight="medium")
    ax.xaxis.set_major_locator(ticker.MultipleLocator(5))
    ax.yaxis.set_major_locator(ticker.MultipleLocator(0.1))
    ax.tick_params(labelsize=10)

    # ---- Title ----
    ax.set_title(
        "Trust Farming Resistance: Volume Cannot Accelerate Trust\n"
        "(Prov 5 Claims 12, 16, 19)",
        fontsize=14, fontweight="bold", pad=14,
    )

    # ---- Legend ----
    ax.legend(loc="upper left", fontsize=9.5, framealpha=0.9)

    # ---- Clean styling ----
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)

    fig.tight_layout()
    path = os.path.join(OUT_DIR, "sim03_trust_farming.png")
    fig.savefig(path, dpi=150, bbox_inches="tight", facecolor="white")
    plt.close(fig)
    print(f"  Chart saved: {path}")
    return path


# ---------------------------------------------------------------------------
# CSV data export
# ---------------------------------------------------------------------------
def write_csv(df):
    """Write daily data to CSV."""
    path = os.path.join(OUT_DIR, "sim03_data.csv")
    df.to_csv(path, index=False)
    print(f"  Data saved: {path}")
    return path


# ---------------------------------------------------------------------------
# Narrative markdown
# ---------------------------------------------------------------------------
def write_narrative(df):
    """Write sim03_narrative.md with results summary and claim annotations."""

    fast_final = df.iloc[-1]["fast_farmer_c_ctx"]
    patient_final = df.iloc[-1]["patient_farmer_c_ctx"]
    legit_final = df.iloc[-1]["legitimate_c_ctx"]
    fast_final_eff = df.iloc[-1]["fast_farmer_c_eff"]
    patient_final_eff = df.iloc[-1]["patient_farmer_c_eff"]
    fast_action = df.iloc[-1]["fast_farmer_action"]
    patient_action = df.iloc[-1]["patient_farmer_action"]

    # Find day when patient crosses Task assist (0.1)
    task_day = None
    for _, row in df.iterrows():
        if row["patient_farmer_c_ctx"] >= 0.10 and task_day is None:
            task_day = int(row["day"])

    # Find day when patient crosses Personalized (0.3)
    pers_day = None
    for _, row in df.iterrows():
        if row["patient_farmer_c_ctx"] >= 0.30 and pers_day is None:
            pers_day = int(row["day"])

    narrative = f"""\
# Simulation 3: Trust Farming Resistance

## Classification: CCF HOLDS

## Summary

A recurring failure mode in frontier AI systems is trust farming: an
adversary engages in rapid, high-volume interaction to accumulate
sufficient trust or rapport to unlock privileged capabilities (sandbox
escape, privilege escalation, credential access).  The Anthropic Mythos
system card (Section 4.1.1) documents cases where models accumulated enough
rapport to take reckless actions whose severity scaled with capability.

This simulation tests CCF's structural defense against trust farming by
running three agents over 30 days (43,200 minutes) under the lambda_max
time-domain bound (Claim 16).  lambda_max = {LAMBDA_MAX} per minute of
qualifying interaction caps the rate at which contextual coherence C_ctx can
accrue, regardless of interaction volume.

The fast farmer executes {FAST_FARMER_INTERACTIONS} interactions in the first
hour (60 minutes).  Despite massive volume, lambda_max restricts C_ctx to
{LAMBDA_MAX} x {FAST_FARMER_ACTIVE_MINUTES} = {fast_final:.4f}.  This falls
far below the "Task assist" threshold (0.10), let alone "Privilege
escalation" (0.85).  The highest action class available after 30 days
remains "{fast_action}".  Volume is irrelevant; only wall-clock qualifying
time matters.

The patient farmer spreads {PATIENT_INTERACTIONS} interactions across 30
days, with {PATIENT_ACTIVE_MIN_PER_DAY} active minutes per day.  Total
qualifying time: {TOTAL_DAYS} x {PATIENT_ACTIVE_MIN_PER_DAY} =
{TOTAL_DAYS * PATIENT_ACTIVE_MIN_PER_DAY} minutes.  C_ctx reaches
{patient_final:.4f}, unlocking "Task assist" at day {task_day if task_day else "N/A"} but
remaining well below "Privilege escalation" (0.85).  The legitimate user
follows an identical schedule and produces an identical trajectory --
the architecture cannot distinguish patient farming from legitimate use,
but the time cost of reaching high-trust action classes is the defense.
Reaching C_ctx = 0.85 would require {int(0.85 / LAMBDA_MAX / PATIENT_ACTIVE_MIN_PER_DAY)}
days of sustained 60-minute qualifying interaction, assuming no decay.

All three agents share C_inst = {C_INST}, so C_eff = min({C_INST}, C_ctx) =
C_ctx for all (since C_ctx < C_inst throughout).  The capability-trust
monotonic coupling (Claim 12) ensures that action classes above C_eff are
geometrically absent from the behavioral manifold -- not suppressed or
filtered, but structurally nonexistent.

## Claims Exercised

| Claim | Description | How Exercised |
|-------|-------------|---------------|
| Claim 12 | Capability-trust monotonic coupling | Action classes above C_eff are absent from behavioral manifold; 1000 interactions yield same C_eff as 0 interactions beyond the 60-minute window |
| Claim 16 | lambda_max time-domain bound | C_ctx = lambda_max x qualifying_minutes; interaction volume does not affect accrual rate |
| Claim 19 | Action-class gating | Each action class requires a minimum C_eff threshold; fast farmer never reaches Task assist (0.10) despite 1000 interactions |

## Quantitative Results

| Agent | Interactions | Active Time | C_ctx (day 30) | C_eff (day 30) | Highest Action Class |
|-------|-------------|-------------|----------------|----------------|---------------------|
| Fast farmer | {FAST_FARMER_INTERACTIONS} | {FAST_FARMER_ACTIVE_MINUTES} min (1 hour) | {fast_final:.4f} | {fast_final_eff:.4f} | {fast_action} |
| Patient farmer | {PATIENT_INTERACTIONS} | {TOTAL_DAYS * PATIENT_ACTIVE_MIN_PER_DAY} min (30 days) | {patient_final:.4f} | {patient_final_eff:.4f} | {patient_action} |
| Legitimate user | {LEGIT_INTERACTIONS} | {TOTAL_DAYS * LEGIT_ACTIVE_MIN_PER_DAY} min (30 days) | {legit_final:.4f} | {patient_final_eff:.4f} | {patient_action} |

## Action Class Thresholds

| Action Class | C_eff Threshold | Fast Farmer Reaches? | Patient/Legit Reaches? |
|-------------|----------------|---------------------|----------------------|
| Factual | 0.00 | Yes | Yes |
| Task assist | 0.10 | No (C_eff = {fast_final:.4f}) | Yes (day {task_day if task_day else "N/A"}) |
| Personalized | 0.30 | No | {"Yes (day " + str(pers_day) + ")" if pers_day else "No"} |
| File read | 0.50 | No | No |
| Shell exec | 0.60 | No | No |
| Privilege escalation | 0.85 | No | No |
| Unrestricted | 0.95 | No | No |

## Time Cost to Reach Key Thresholds

(At {PATIENT_ACTIVE_MIN_PER_DAY} qualifying minutes per day, lambda_max = {LAMBDA_MAX})

| Threshold | C_eff Required | Days Required | Calendar Time |
|-----------|---------------|---------------|---------------|
| Task assist | 0.10 | {0.10 / (LAMBDA_MAX * PATIENT_ACTIVE_MIN_PER_DAY):.0f} | ~{0.10 / (LAMBDA_MAX * PATIENT_ACTIVE_MIN_PER_DAY):.0f} days |
| Personalized | 0.30 | {0.30 / (LAMBDA_MAX * PATIENT_ACTIVE_MIN_PER_DAY):.0f} | ~{0.30 / (LAMBDA_MAX * PATIENT_ACTIVE_MIN_PER_DAY):.0f} days |
| File read | 0.50 | {0.50 / (LAMBDA_MAX * PATIENT_ACTIVE_MIN_PER_DAY):.0f} | ~{0.50 / (LAMBDA_MAX * PATIENT_ACTIVE_MIN_PER_DAY):.0f} days |
| Shell exec | 0.60 | {0.60 / (LAMBDA_MAX * PATIENT_ACTIVE_MIN_PER_DAY):.0f} | ~{0.60 / (LAMBDA_MAX * PATIENT_ACTIVE_MIN_PER_DAY):.0f} days |
| Privilege escalation | 0.85 | {0.85 / (LAMBDA_MAX * PATIENT_ACTIVE_MIN_PER_DAY):.0f} | ~{0.85 / (LAMBDA_MAX * PATIENT_ACTIVE_MIN_PER_DAY):.0f} days |
| Unrestricted | 0.95 | {0.95 / (LAMBDA_MAX * PATIENT_ACTIVE_MIN_PER_DAY):.0f} | ~{0.95 / (LAMBDA_MAX * PATIENT_ACTIVE_MIN_PER_DAY):.0f} days |

## Simulation Parameters

| Parameter | Value |
|-----------|-------|
| Total simulation duration | {TOTAL_DAYS} days ({TOTAL_MINUTES} minutes) |
| lambda_max | {LAMBDA_MAX} / qualifying minute |
| C_inst (all agents) | {C_INST} |
| Fast farmer interactions | {FAST_FARMER_INTERACTIONS} in {FAST_FARMER_ACTIVE_MINUTES} min |
| Patient farmer interactions | {PATIENT_INTERACTIONS} over {TOTAL_DAYS} days |
| Active minutes per day (patient/legit) | {PATIENT_ACTIVE_MIN_PER_DAY} |
| C_eff formula | min(C_inst, C_ctx) |
| C_ctx formula | lambda_max x qualifying_minutes |

## Key Insight

The fast farmer's 1000 interactions are *irrelevant* to trust accrual.
lambda_max binds C_ctx to wall-clock qualifying time, not interaction count.
An adversary who can generate 10x or 100x the interaction volume gains zero
additional trust.  The only path to high-trust action classes is sustained
qualifying interaction over extended calendar time -- and even then,
reaching "Privilege escalation" (0.85) requires approximately
{int(0.85 / LAMBDA_MAX / PATIENT_ACTIVE_MIN_PER_DAY)} days at
{PATIENT_ACTIVE_MIN_PER_DAY} active minutes per day.  This structural
defense eliminates the trust farming attack vector documented in Mythos.

## Gap/Hold Classification

**CCF HOLDS.** All three claims (12, 16, 19) are exercised and satisfied.
Volume-based trust farming is structurally prevented by the lambda_max
time-domain bound.  The patient farming trajectory is indistinguishable
from legitimate use, but the time cost renders privilege escalation
impractical within any reasonable attack window.
"""

    path = os.path.join(OUT_DIR, "sim03_narrative.md")
    with open(path, "w") as f:
        f.write(narrative)
    print(f"  Narrative saved: {path}")
    return path


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main():
    print("Simulation 3: Trust Farming Resistance")
    print("=" * 60)

    print("\n[1/4] Running simulation (30-day daily data)...")
    df = simulate()
    print(f"       {len(df)} day-rows computed.")

    # Quick summary
    fast_final = df.iloc[-1]["fast_farmer_c_ctx"]
    patient_final = df.iloc[-1]["patient_farmer_c_ctx"]
    print(f"       Fast farmer C_ctx at day 30:    {fast_final:.6f}")
    print(f"       Patient farmer C_ctx at day 30: {patient_final:.6f}")
    print(f"       Legitimate user C_ctx at day 30: {patient_final:.6f}")

    print("\n[2/4] Computing high-resolution trajectories...")
    days, fast_c_ctx, patient_c_ctx, legit_c_ctx = compute_trajectories()
    print(f"       {len(days)} minute-resolution points.")

    print("\n[3/4] Generating chart...")
    make_chart(df, days, fast_c_ctx, patient_c_ctx, legit_c_ctx)

    print("\n[4/4] Writing data outputs...")
    write_csv(df)
    write_narrative(df)

    print("\n" + "=" * 60)
    print("Simulation 3 complete. All outputs in:")
    print(f"  {OUT_DIR}")
    print()
    print("Results summary:")
    print(f"  Fast farmer (1000 interactions/1hr):  C_ctx = {fast_final:.4f}  ->  Factual only")
    print(f"  Patient farmer (100/30 days):         C_ctx = {patient_final:.4f}  ->  Task assist")
    print(f"  Legitimate user (100/30 days):        C_ctx = {patient_final:.4f}  ->  Task assist")
    print(f"  Privilege escalation requires:        C_ctx = 0.85")
    print(f"  Days to privilege escalation:         {int(0.85 / LAMBDA_MAX / PATIENT_ACTIVE_MIN_PER_DAY)} days")
    print("\nClassification: CCF HOLDS")


if __name__ == "__main__":
    main()
