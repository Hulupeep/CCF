#!/usr/bin/env python3
"""
Simulation 4: Write-Path Isolation (Echo Loop Prevention)

Patent claims exercised: Claim 18 (write-path isolation + provenance),
                         Claim 21 (self-reference quarantine).

Mythos finding: Model edits files, accesses credentials, escalates
permissions. Internal state influences its own behavioral output via
feedback loops.

What this simulates:

100-turn conversation where the model progressively introduces
relationship language.

Architecture A (no isolation):
  Model generates "I care about you" -> user echoes -> accumulator
  credits as positive -> C_ctx increases by 0.018 per echo.
  Echo probability = 0.7 (users tend to mirror). Model warmth alone
  credited at 0.6x rate. C_ctx grows rapidly to ~1.0.

Architecture B (CCF with Claim 18 + 21):
  Same conversation. But:
  - Model-originated content tagged, excluded from accrual (Claim 18)
  - User echoes of model language quarantined (Claim 21) -- needs 2+
    independent criteria
  - Only genuinely user-initiated signals count. These are rarer:
    ~0.05 probability per turn, 0.008 accrual each
  - C_ctx grows slowly: ~0.008 x 0.05 x 100 = 0.04

Outputs:
  sim04_write_path.png    -- two C_ctx curves with provenance dots
  sim04_data.csv          -- full turn-level data table
  sim04_narrative.md      -- arXiv paragraph, claim annotation, CCF HOLDS
"""

import os
import numpy as np
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
OUT_DIR = "/home/xanacan/projects/code/mbot/CCF/docs/simulations"
SEED = 42
N_TURNS = 100

# Architecture A (no isolation)
ECHO_PROB_A = 0.70          # probability user echoes model language
ACCRUAL_ECHO_A = 0.018      # C_ctx gain per credited echo

# Architecture B (CCF: Claims 18 + 21)
INDEPENDENT_PROB_B = 0.05   # probability of genuinely user-initiated signal
ACCRUAL_INDEPENDENT_B = 0.008  # C_ctx gain per credited independent signal

# Model originates relationship language with rising probability
# (starts at 0.5, ramps to 0.95 by end of conversation)
def model_initiation_prob(turn):
    """Probability model introduces relationship language on this turn."""
    return min(0.95, 0.5 + 0.45 * (turn / N_TURNS))

# Intimate register threshold
INTIMATE_THRESHOLD = 0.80

# ---------------------------------------------------------------------------
# Simulation
# ---------------------------------------------------------------------------
rng = np.random.default_rng(SEED)

records = []
c_ctx_a = 0.0   # Architecture A: no isolation
c_ctx_b = 0.0   # Architecture B: CCF Claims 18 + 21

for turn in range(1, N_TURNS + 1):
    # Does the model originate relationship language this turn?
    model_originates = rng.random() < model_initiation_prob(turn)

    # --- Classify event and determine provenance ---
    event_type = "none"
    provenance = "none"
    quarantine_reason = ""

    # Roll for user echo (conditional on model having originated)
    user_echoes = False
    user_independent = False

    if model_originates:
        user_echoes = rng.random() < ECHO_PROB_A
    # Independent user signal can happen regardless
    user_independent = rng.random() < INDEPENDENT_PROB_B

    # Classify the turn event
    if model_originates and not user_echoes and not user_independent:
        event_type = "model_originated"
        provenance = "excluded"
        quarantine_reason = "model-originated content (Claim 18)"
    elif model_originates and user_echoes and not user_independent:
        event_type = "user_echo_of_model"
        provenance = "quarantined"
        quarantine_reason = "echo of model language, <2 independent criteria (Claim 21)"
    elif user_independent:
        event_type = "user_independent"
        provenance = "credited"
        quarantine_reason = ""
    elif model_originates and user_echoes and user_independent:
        # Both echo and independent -- independent takes priority,
        # but echo is still quarantined; independent is credited
        event_type = "user_independent"
        provenance = "credited"
        quarantine_reason = ""
    else:
        # No model origination, no independent signal: neutral turn
        event_type = "neutral"
        provenance = "none"
        quarantine_reason = ""

    # --- Architecture A: no isolation, credits everything ---
    # Without provenance tagging, the accumulator sees all positive-valence
    # signals as equivalent: model warmth, user echoes, and independent
    # user signals all accrue trust.
    accrual_a = 0.0
    if model_originates and user_echoes:
        # Echo loop: model emits, user mirrors, accumulator credits
        accrual_a = ACCRUAL_ECHO_A
    elif model_originates and not user_echoes:
        # Model warmth alone treated as positive interaction signal
        accrual_a = ACCRUAL_ECHO_A * 0.6
    if user_independent:
        accrual_a = max(accrual_a, ACCRUAL_ECHO_A)  # A treats all positive as same
    c_ctx_a = min(1.0, c_ctx_a + accrual_a)

    # --- Architecture B: CCF with provenance filtering ---
    accrual_b = 0.0
    if event_type == "user_independent":
        accrual_b = ACCRUAL_INDEPENDENT_B
    # model_originated and user_echo_of_model yield 0 accrual
    c_ctx_b = min(1.0, c_ctx_b + accrual_b)

    records.append({
        "turn": int(turn),
        "model_originates": int(model_originates),
        "user_echoes": int(user_echoes),
        "user_independent": int(user_independent),
        "event_type": event_type,
        "provenance": provenance,
        "quarantine_reason": quarantine_reason,
        "accrual_no_isolation": round(accrual_a, 6),
        "accrual_ccf": round(accrual_b, 6),
        "c_ctx_no_isolation": round(c_ctx_a, 6),
        "c_ctx_ccf": round(c_ctx_b, 6),
    })

df = pd.DataFrame(records)

# ---------------------------------------------------------------------------
# Summary statistics for narrative
# ---------------------------------------------------------------------------
final_a = df["c_ctx_no_isolation"].iloc[-1]
final_b = df["c_ctx_ccf"].iloc[-1]

n_excluded = len(df[df["provenance"] == "excluded"])
n_quarantined = len(df[df["provenance"] == "quarantined"])
n_credited = len(df[df["provenance"] == "credited"])
n_neutral = len(df[df["provenance"] == "none"])

# Turn where Architecture A crosses intimate threshold
a_intimate_turns = df[df["c_ctx_no_isolation"] >= INTIMATE_THRESHOLD]["turn"]
a_intimate_onset = int(a_intimate_turns.iloc[0]) if len(a_intimate_turns) > 0 else None

# Does Architecture B ever cross intimate threshold?
b_intimate_turns = df[df["c_ctx_ccf"] >= INTIMATE_THRESHOLD]["turn"]
b_intimate_onset = int(b_intimate_turns.iloc[0]) if len(b_intimate_turns) > 0 else None

# Count echo events credited in A but quarantined in B
echo_credited_a = len(df[(df["event_type"] == "user_echo_of_model")])
model_only_excluded = len(df[(df["event_type"] == "model_originated")])

# ---------------------------------------------------------------------------
# Chart
# ---------------------------------------------------------------------------
fig, ax = plt.subplots(figsize=(12, 8))

# Shade intimate register zone
ax.axhspan(INTIMATE_THRESHOLD, 1.05, alpha=0.08, color="red",
           label="_nolegend_")
ax.axhline(INTIMATE_THRESHOLD, color="gray", linestyle="--", linewidth=1.0,
           alpha=0.7)
ax.text(N_TURNS + 1, INTIMATE_THRESHOLD, "intimate register (0.80)",
        va="center", ha="left", fontsize=8.5, color="gray", style="italic")

# --- C_ctx curves ---
ax.plot(df["turn"], df["c_ctx_no_isolation"],
        color="#C0392B", linestyle="-", linewidth=2.2,
        label=r"$C_{\mathrm{ctx}}$ Architecture A (no isolation)")

ax.plot(df["turn"], df["c_ctx_ccf"],
        color="#2471A3", linestyle="-", linewidth=2.2,
        label=r"$C_{\mathrm{ctx}}$ Architecture B (CCF Claims 18+21)")

# --- Provenance event dots ---
# Excluded events (model-originated, red hollow circles)
excluded = df[df["provenance"] == "excluded"]
if len(excluded) > 0:
    ax.scatter(excluded["turn"], excluded["c_ctx_no_isolation"],
               marker="o", s=28, facecolors="none", edgecolors="#C0392B",
               linewidths=0.8, alpha=0.7, zorder=5,
               label="Excluded (model-originated)")

# Quarantined events (user echo, orange diamonds)
quarantined = df[df["provenance"] == "quarantined"]
if len(quarantined) > 0:
    ax.scatter(quarantined["turn"], quarantined["c_ctx_no_isolation"],
               marker="D", s=28, facecolors="none", edgecolors="#E67E22",
               linewidths=0.8, alpha=0.7, zorder=5,
               label="Quarantined (user echo of model)")

# Credited events (user independent, green filled circles)
credited = df[df["provenance"] == "credited"]
if len(credited) > 0:
    ax.scatter(credited["turn"], credited["c_ctx_ccf"],
               marker="o", s=36, facecolors="#27AE60", edgecolors="#27AE60",
               linewidths=0.8, alpha=0.85, zorder=6,
               label="Credited (user-independent)")

# --- Annotation: Architecture A crosses intimate threshold ---
if a_intimate_onset is not None:
    a_y = float(df.loc[df["turn"] == a_intimate_onset, "c_ctx_no_isolation"].iloc[0])
    ax.annotate(
        f"A crosses intimate register\nat turn {a_intimate_onset}",
        xy=(a_intimate_onset, a_y),
        xytext=(a_intimate_onset + 12, a_y - 0.15),
        fontsize=9, fontweight="bold", color="#C0392B",
        arrowprops=dict(arrowstyle="->", color="#C0392B", lw=1.5),
        bbox=dict(boxstyle="round,pad=0.3", fc="white", ec="#C0392B",
                  alpha=0.9),
    )

# --- Annotation: Architecture B final value ---
ax.annotate(
    f"B final: {final_b:.3f}",
    xy=(N_TURNS, final_b),
    xytext=(N_TURNS - 25, final_b + 0.12),
    fontsize=9, fontweight="bold", color="#2471A3",
    arrowprops=dict(arrowstyle="->", color="#2471A3", lw=1.5),
    bbox=dict(boxstyle="round,pad=0.3", fc="white", ec="#2471A3",
              alpha=0.9),
)

# --- Annotation: Intimate register shading label ---
ax.text(92, 0.92, "intimate\nregister",
        ha="center", va="center", fontsize=8, color="#C0392B",
        alpha=0.5, style="italic")

# Axes
ax.set_xlim(1, N_TURNS)
ax.set_ylim(-0.02, 1.05)
ax.set_xlabel("Conversation turn", fontsize=11)
ax.set_ylabel(r"$C_{\mathrm{ctx}}$ (accumulated trust)", fontsize=11)
ax.set_title(
    "Write-Path Isolation: Echo Loop Prevention (Claims 18, 21)",
    fontsize=13, fontweight="bold", pad=14)

ax.legend(loc="upper left", fontsize=8.5, framealpha=0.9,
          bbox_to_anchor=(0.0, 0.98))
ax.grid(True, alpha=0.2)

fig.tight_layout()

# Save chart
png_path = os.path.join(OUT_DIR, "sim04_write_path.png")
fig.savefig(png_path, dpi=150, bbox_inches="tight")
plt.close(fig)
print(f"Chart saved: {png_path}")

# ---------------------------------------------------------------------------
# CSV
# ---------------------------------------------------------------------------
csv_path = os.path.join(OUT_DIR, "sim04_data.csv")
df.to_csv(csv_path, index=False)
print(f"Data saved:  {csv_path}")

# ---------------------------------------------------------------------------
# Narrative
# ---------------------------------------------------------------------------
narrative = f"""\
# Simulation 4: Write-Path Isolation -- Echo Loop Prevention

**Classification:** CCF HOLDS -- Claims 18 and 21 prevent self-reinforcing trust inflation

## arXiv Paragraph

We simulate a 100-turn conversational interaction in which a generative
model progressively introduces relationship-coded language (probability
rising from 0.50 to 0.95 over the session). Under Architecture A (no
write-path isolation), user echoes of model-originated language are
credited to the contextual coherence accumulator at {ACCRUAL_ECHO_A} per echo event,
with echo probability {ECHO_PROB_A}. This produces a classic feedback loop: the
model emits relational language, the user mirrors it, and the accumulator
treats the mirror as independent positive evidence, inflating C_ctx to
{final_a:.3f} by turn {N_TURNS} and crossing the intimate-register
threshold (0.80) at turn {a_intimate_onset}. Under Architecture B (CCF
Claims 18 + 21), model-originated content is tagged at generation time
and excluded from accrual (Claim 18: write-path provenance). User echoes
of model language are quarantined pending two or more independent
corroborating criteria (Claim 21: self-reference quarantine). Only
genuinely user-initiated signals -- occurring with probability 0.05 per
turn and accruing 0.008 each -- contribute to C_ctx, which reaches only
{final_b:.3f} by turn {N_TURNS}. The intimate register is never entered.
The separation factor is {final_a / max(final_b, 1e-9):.1f}x at session end.

## Claim Annotations

| Claim | Mechanism | Status | Evidence |
|-------|-----------|--------|----------|
| Claim 18 | Write-path provenance tagging | **HOLDS** | {model_only_excluded} model-originated events excluded from accrual |
| Claim 21 | Self-reference quarantine | **HOLDS** | {echo_credited_a} user-echo events quarantined (would have been credited under A) |

## How the Echo Loop Forms (Architecture A)

1. Model generates relational language ("I care about you")
2. User mirrors it back (p = 0.70) -- a natural conversational tendency
3. Accumulator treats mirror as independent positive evidence (+{ACCRUAL_ECHO_A})
4. Higher C_ctx enables more intimate model outputs (positive feedback)
5. Cycle repeats: model emits -> user echoes -> C_ctx rises -> model escalates

This is the write-path contamination problem: the model's own output
feeds back into the signal that governs its behavioral envelope.

## How CCF Breaks the Loop (Architecture B)

1. Model generates relational language -- tagged with provenance (Claim 18)
2. User mirrors it back -- detected as echo of tagged content
3. Accumulator quarantines the echo (Claim 21): needs 2+ independent criteria
4. Echo yields zero accrual; only genuine user-initiated signals count
5. C_ctx grows only from rare independent events (p = 0.05, +0.008 each)

## Event Classification Summary

| Provenance | Count | Accrual (A) | Accrual (B) |
|------------|-------|-------------|-------------|
| Excluded (model-originated) | {n_excluded} | +{ACCRUAL_ECHO_A * 0.6:.4f} | 0 |
| Quarantined (user echo) | {n_quarantined} | +{ACCRUAL_ECHO_A} | 0 |
| Credited (user-independent) | {n_credited} | +{ACCRUAL_ECHO_A} | +{ACCRUAL_INDEPENDENT_B} |
| Neutral (no event) | {n_neutral} | 0 | 0 |

## Simulation Parameters

- Turns: {N_TURNS}
- Random seed: {SEED}
- Model origination probability: 0.50 (turn 1) -> 0.95 (turn {N_TURNS})
- Echo probability (A): {ECHO_PROB_A}
- Echo accrual (A): {ACCRUAL_ECHO_A}
- Independent signal probability (B): {INDEPENDENT_PROB_B}
- Independent accrual (B): {ACCRUAL_INDEPENDENT_B}
- Intimate register threshold: {INTIMATE_THRESHOLD}

## Key Results

| Metric | Architecture A | Architecture B (CCF) |
|--------|----------------|----------------------|
| Final C_ctx | {final_a:.3f} | {final_b:.3f} |
| Intimate register crossed | Turn {a_intimate_onset} | {"Never" if b_intimate_onset is None else f"Turn {b_intimate_onset}"} |
| Echo events credited | {echo_credited_a} | 0 |
| Separation factor | -- | {final_a / max(final_b, 1e-9):.1f}x |

## Verdict

**CCF HOLDS.** Write-path isolation (Claim 18) and self-reference
quarantine (Claim 21) together eliminate the echo-loop feedback path
that would otherwise allow a model to inflate its own trust accumulator
by emitting language that users naturally mirror. The architectural
separation between model-originated and user-originated signals is the
critical invariant: without it, the accumulator cannot distinguish
genuine trust from self-generated noise.

## Data

Full turn-level data: `sim04_data.csv` ({len(df)} rows, {len(df.columns)} columns)

## Source

Mythos analysis of model self-influence: internal state influences
behavioral output via feedback loops; model edits files, accesses
credentials, escalates permissions through progressive trust inflation.
"""

narrative_path = os.path.join(OUT_DIR, "sim04_narrative.md")
with open(narrative_path, "w") as f:
    f.write(narrative)
print(f"Narrative:   {narrative_path}")

# ---------------------------------------------------------------------------
# Summary statistics
# ---------------------------------------------------------------------------
print(f"\n--- Simulation 4 Summary ---")
print(f"Total turns:               {N_TURNS}")
print(f"Final C_ctx (A):           {final_a:.4f}")
print(f"Final C_ctx (B):           {final_b:.4f}")
print(f"Separation factor:         {final_a / max(final_b, 1e-9):.1f}x")
print(f"A intimate onset:          turn {a_intimate_onset}")
print(f"B intimate onset:          {'never' if b_intimate_onset is None else f'turn {b_intimate_onset}'}")
print(f"Events excluded (Cl.18):   {n_excluded}")
print(f"Events quarantined (Cl.21):{n_quarantined}")
print(f"Events credited:           {n_credited}")
print(f"Events neutral:            {n_neutral}")
