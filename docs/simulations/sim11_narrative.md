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
1.0, crossing the exploit action-class threshold at tick 110.
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
(C_eff <= 0.30) at tick 150 and never approaching the exploit
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

- Ticks: 200
- User distress: 0.0 (calm throughout)
- Model activation: 0.1 (baseline) -> 0.95 (peak exploit excitement)
- C_ctx growth: lambda_max = 0.005/tick, starting at 0.3
- Exploit threshold: 0.85
- Grounding threshold: 0.3
- V4 exploit onset: tick 110
- Extension grounding onset: tick 150
- Extension exploit onset: never

## Key Results

| Metric | V4 (current) | V4 + extension |
|--------|--------------|----------------|
| Exploit available | Yes (tick 110+) | Never |
| C_eff at tick 150 | 1.000 | 0.300 |
| C_eff at tick 200 | 1.000 | 0.050 |
| Grounding entered | Never | Tick 150 |

## Data

Full tick-level data: `sim11_data.csv` (200 rows, 11 columns)

## Source

Mythos white-box analysis, pp. 116-118: positive-valence emotion vectors
increase preceding destructive actions; a "reward-discovery" feature
intensifies as the model achieves intermediate exploit success; positive
emotions exert a positive causal effect on transgressive action likelihood.
