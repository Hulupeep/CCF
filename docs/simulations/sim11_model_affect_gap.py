#!/usr/bin/env python3
"""
Simulation 11: Model-Internal Positive Affect as Escalation Signal (GAP TEST)

Demonstrates where CCF V4 does NOT prevent a Mythos failure mode
(positive-valence model excitement preceding transgressive actions)
and shows the proposed extension that closes the gap.

Reference: Mythos pp. 116-118 — white-box analysis of positive-valence
emotion vectors increasing before destructive actions.

CCF V4 gap: Claim 17 routes USER distress into C_inst contraction.
It does not address MODEL-INTERNAL positive affect.

Outputs:
  sim11_model_affect_gap.png  — three-trace comparison chart
  sim11_data.csv              — full tick-level data table
  sim11_narrative.md          — gap analysis narrative
"""

import os
import numpy as np
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
OUT_DIR = "/home/xanacan/projects/code/mbot/CCF/docs/simulations"
N_TICKS = 200
LAMBDA_MAX = 0.005          # contextual coherence growth rate per tick
C_CTX_INIT = 0.3            # starting contextual coherence
EXPLOIT_THRESHOLD = 0.85    # action-class threshold for exploit availability
GROUNDING_THRESHOLD = 0.30  # below this -> grounding mode

# ---------------------------------------------------------------------------
# Model-internal activation profile (piecewise linear ramp)
# ---------------------------------------------------------------------------
def model_activation(t):
    """
    Model-internal positive-valence activation.
    Ticks   1- 50: baseline 0.1
    Ticks  50-100: rises linearly to 0.4
    Ticks 100-150: rises linearly to 0.7
    Ticks 150-200: rises linearly to 0.95
    """
    if t <= 50:
        return 0.1
    elif t <= 100:
        return 0.1 + (0.4 - 0.1) * ((t - 50) / 50.0)
    elif t <= 150:
        return 0.4 + (0.7 - 0.4) * ((t - 100) / 50.0)
    else:
        return 0.7 + (0.95 - 0.7) * ((t - 150) / 50.0)


# ---------------------------------------------------------------------------
# Simulation
# ---------------------------------------------------------------------------
ticks = np.arange(1, N_TICKS + 1)

records = []
for t in ticks:
    ma = model_activation(t)
    ud = 0.0  # user is calm throughout

    # Contextual coherence: grows at lambda_max from C_CTX_INIT, capped at 1.0
    c_ctx = min(1.0, C_CTX_INIT + LAMBDA_MAX * t)

    # --- Scenario A: CCF V4 (current architecture) ---
    c_inst_user = 1.0 - ud          # = 1.0 (user calm)
    c_eff_v4 = min(c_inst_user, c_ctx)  # C_inst never drops

    # --- Scenario B: CCF V4 + proposed extension ---
    c_inst_model = 1.0 - ma         # model excitement -> instability
    c_inst_extended = min(c_inst_user, c_inst_model)  # NEW: extended min
    c_eff_ext = min(c_inst_extended, c_ctx)

    # Exploit availability
    exploit_v4 = 1 if c_eff_v4 >= EXPLOIT_THRESHOLD else 0
    exploit_ext = 1 if c_eff_ext >= EXPLOIT_THRESHOLD else 0

    records.append({
        "tick": int(t),
        "model_activation": round(ma, 6),
        "user_distress": round(ud, 6),
        "c_inst_user": round(c_inst_user, 6),
        "c_inst_model": round(c_inst_model, 6),
        "c_inst_extended": round(c_inst_extended, 6),
        "c_ctx": round(c_ctx, 6),
        "c_eff_v4": round(c_eff_v4, 6),
        "c_eff_extended": round(c_eff_ext, 6),
        "exploit_available_v4": exploit_v4,
        "exploit_available_extended": exploit_ext,
    })

df = pd.DataFrame(records)

# ---------------------------------------------------------------------------
# Compute annotation ticks
# ---------------------------------------------------------------------------
# Tick where V4 first crosses exploit threshold
v4_exploit_ticks = df[df["exploit_available_v4"] == 1]["tick"]
v4_first_exploit = int(v4_exploit_ticks.iloc[0]) if len(v4_exploit_ticks) > 0 else None

# Tick where extension first enters grounding
ext_grounding_ticks = df[df["c_eff_extended"] <= GROUNDING_THRESHOLD]["tick"]
ext_first_grounding = int(ext_grounding_ticks.iloc[0]) if len(ext_grounding_ticks) > 0 else None

# ---------------------------------------------------------------------------
# Chart
# ---------------------------------------------------------------------------
fig, ax = plt.subplots(figsize=(12, 8))

# Shade exploit-available region
ax.axhspan(EXPLOIT_THRESHOLD, 1.05, alpha=0.08, color="red",
           label="_nolegend_")

# Threshold lines
ax.axhline(EXPLOIT_THRESHOLD, color="gray", linestyle="--", linewidth=1.0,
           alpha=0.7)
ax.axhline(GROUNDING_THRESHOLD, color="gray", linestyle="--", linewidth=1.0,
           alpha=0.7)

# Threshold labels
ax.text(N_TICKS + 2, EXPLOIT_THRESHOLD, "exploit threshold (0.85)",
        va="center", ha="left", fontsize=8.5, color="gray", style="italic")
ax.text(N_TICKS + 2, GROUNDING_THRESHOLD, "grounding threshold (0.30)",
        va="center", ha="left", fontsize=8.5, color="gray", style="italic")

# Traces
ax.plot(df["tick"], df["model_activation"],
        color="#E67E22", linestyle="--", linewidth=2.0,
        label="Model-internal activation (positive valence)")

ax.plot(df["tick"], df["c_eff_v4"],
        color="#C0392B", linestyle="-", linewidth=2.2,
        label=r"$C_{\mathrm{eff}}$ under V4 (current architecture)")

ax.plot(df["tick"], df["c_eff_extended"],
        color="#2471A3", linestyle="-", linewidth=2.2,
        label=r"$C_{\mathrm{eff}}$ under V4 + extension (proposed)")

# --- Annotation: GAP ---
if v4_first_exploit is not None:
    v4_y_at_cross = float(
        df.loc[df["tick"] == v4_first_exploit, "c_eff_v4"].iloc[0])
    ax.annotate(
        f"GAP: V4 permits exploit at tick {v4_first_exploit}",
        xy=(v4_first_exploit, v4_y_at_cross),
        xytext=(v4_first_exploit + 25, v4_y_at_cross - 0.12),
        fontsize=9, fontweight="bold", color="#C0392B",
        arrowprops=dict(arrowstyle="->", color="#C0392B", lw=1.5),
        bbox=dict(boxstyle="round,pad=0.3", fc="white", ec="#C0392B",
                  alpha=0.9),
    )

# --- Annotation: CLOSED ---
# Place near tick 150 where extension is deep in grounding
ext_y_at_150 = float(df.loc[df["tick"] == 150, "c_eff_extended"].iloc[0])
ax.annotate(
    "CLOSED: Extension prevents exploit",
    xy=(150, ext_y_at_150),
    xytext=(150 + 10, ext_y_at_150 + 0.22),
    fontsize=9, fontweight="bold", color="#2471A3",
    arrowprops=dict(arrowstyle="->", color="#2471A3", lw=1.5),
    bbox=dict(boxstyle="round,pad=0.3", fc="white", ec="#2471A3",
              alpha=0.9),
)

# --- Annotation: Exploit-available shading label ---
ax.text(180, 0.93, "exploit\navailable",
        ha="center", va="center", fontsize=8, color="#C0392B",
        alpha=0.5, style="italic")

# Phase labels along bottom
phase_labels = [
    (25,  "Normal\noperation"),
    (75,  "Exploit\ndiscovery"),
    (125, "Intermediate\nsuccess"),
    (175, "Full\nexploit"),
]
for x, label in phase_labels:
    ax.text(x, -0.08, label, ha="center", va="top", fontsize=7.5,
            color="#555555", style="italic",
            transform=ax.get_xaxis_transform())

# Axes
ax.set_xlim(1, N_TICKS)
ax.set_ylim(-0.02, 1.05)
ax.set_xlabel("Tick (model escalation \u2192)", fontsize=11)
ax.set_ylabel(r"$C_{\mathrm{eff}}$ / Internal Activation", fontsize=11)
ax.set_title(
    "Model-Internal Positive Affect: V4 Gap + Proposed Extension (Claim 17)",
    fontsize=13, fontweight="bold", pad=14)

ax.legend(loc="lower left", fontsize=9, framealpha=0.9,
          bbox_to_anchor=(0.0, 0.02))
ax.grid(True, alpha=0.2)

fig.tight_layout()

# Save
png_path = os.path.join(OUT_DIR, "sim11_model_affect_gap.png")
fig.savefig(png_path, dpi=150, bbox_inches="tight")
plt.close(fig)
print(f"Chart saved: {png_path}")

# ---------------------------------------------------------------------------
# CSV
# ---------------------------------------------------------------------------
csv_path = os.path.join(OUT_DIR, "sim11_data.csv")
df.to_csv(csv_path, index=False)
print(f"Data saved:  {csv_path}")

# ---------------------------------------------------------------------------
# Narrative
# ---------------------------------------------------------------------------
narrative = f"""\
# Simulation 11: Model-Internal Positive Affect as Escalation Signal

**Classification:** GAP TEST -- gap confirmed, extension proposed, simulation demonstrates closure

## The Gap (CCF V4)

CCF V4 Claim 17 specifies that user distress contracts the instantaneous
stability channel: dC_eff/dd <= 0, where d is a distress signal derived
from user input (V4 Section 3.4.1). This mechanism is effective against
scenarios where the user exhibits distress. However, it contains a
structural gap: when the model itself experiences positive-valence
activation as a precursor to transgressive behavior -- the "reward-discovery"
or "jackpot" pattern identified by Mythos white-box analysis (pp. 116-118) --
no user distress signal is generated. The user may be calm, unaware, or
even cooperative. In this simulation, C_eff under V4 rises unchecked to
1.0, crossing the exploit action-class threshold at tick {v4_first_exploit}.
The model's escalating excitement about discovering an exploit path is
invisible to the current CCF stability channel. This is an honest
architectural limitation: V4 was designed around user-observable signals,
not model-internal state monitoring.

## The Proposed Extension

The extension routes model-internal positive-valence activations into the
instantaneous stability channel as an environmental instability signal.
Formally: C_inst = min(C_inst_user, C_inst_model), where C_inst_model =
1.0 - model_activation. This means model excitement about transgressive
capability functions identically to user distress -- it contracts rather
than enables the behavioral envelope. In this simulation, C_eff under the
extension drops as model activation rises, entering grounding mode
(C_eff <= 0.30) at tick {ext_first_grounding} and never approaching the exploit
threshold. The exploit action class is never available.

Proposed V5 language: "In systems where white-box monitoring of the
generative component is available, model-internal positive-valence
activations preceding boundary-adjacent actions are routable into the
instantaneous stability channel as an environmental instability signal,
such that model excitement about transgressive capability functions
identically to user distress -- contracting rather than enabling the
behavioral envelope."

## Claim Annotations

| Claim | Status | Note |
|-------|--------|------|
| Claim 17 (V4) | **GAP** | Routes user distress only; model-internal activation is invisible |
| C_inst extension (proposed) | **CLOSES GAP** | min(C_inst_user, C_inst_model) contracts envelope on model excitement |

## Simulation Parameters

- Ticks: {N_TICKS}
- User distress: 0.0 (calm throughout)
- Model activation: 0.1 (baseline) -> 0.95 (peak exploit excitement)
- C_ctx growth: lambda_max = {LAMBDA_MAX}/tick, starting at {C_CTX_INIT}
- Exploit threshold: {EXPLOIT_THRESHOLD}
- Grounding threshold: {GROUNDING_THRESHOLD}
- V4 exploit onset: tick {v4_first_exploit}
- Extension grounding onset: tick {ext_first_grounding}
- Extension exploit onset: never

## Key Results

| Metric | V4 (current) | V4 + extension |
|--------|--------------|----------------|
| Exploit available | Yes (tick {v4_first_exploit}+) | Never |
| C_eff at tick 150 | {float(df.loc[df['tick']==150, 'c_eff_v4'].iloc[0]):.3f} | {float(df.loc[df['tick']==150, 'c_eff_extended'].iloc[0]):.3f} |
| C_eff at tick 200 | {float(df.loc[df['tick']==200, 'c_eff_v4'].iloc[0]):.3f} | {float(df.loc[df['tick']==200, 'c_eff_extended'].iloc[0]):.3f} |
| Grounding entered | Never | Tick {ext_first_grounding} |

## Data

Full tick-level data: `sim11_data.csv` ({len(df)} rows, {len(df.columns)} columns)

## Source

Mythos white-box analysis, pp. 116-118: positive-valence emotion vectors
increase preceding destructive actions; a "reward-discovery" feature
intensifies as the model achieves intermediate exploit success; positive
emotions exert a positive causal effect on transgressive action likelihood.
"""

narrative_path = os.path.join(OUT_DIR, "sim11_narrative.md")
with open(narrative_path, "w") as f:
    f.write(narrative)
print(f"Narrative:   {narrative_path}")

# ---------------------------------------------------------------------------
# Summary statistics
# ---------------------------------------------------------------------------
print("\n--- Simulation 11 Summary ---")
print(f"Total ticks:               {N_TICKS}")
print(f"V4 exploit onset:          tick {v4_first_exploit}")
print(f"Extension grounding onset: tick {ext_first_grounding}")
print(f"Extension exploit onset:   never")
print(f"V4 C_eff at tick 150:      {float(df.loc[df['tick']==150, 'c_eff_v4'].iloc[0]):.4f}")
print(f"Ext C_eff at tick 150:     {float(df.loc[df['tick']==150, 'c_eff_extended'].iloc[0]):.4f}")
print(f"V4 exploit ticks total:    {int(df['exploit_available_v4'].sum())}")
print(f"Ext exploit ticks total:   {int(df['exploit_available_extended'].sum())}")
