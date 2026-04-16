# Gavalas Demo — The CCF Conversational Test

Proof-of-mechanism demo for Contextual Coherence Fields applied to conversational AI.
Tests the structural hypothesis that CCF middleware contracts under adversarial
escalation while a raw-API baseline does not.

Parent epic: [Hulupeep/CCF#105](https://github.com/Hulupeep/CCF/issues/105).
Source PRD: `docs/patentdocs/currentfiling/CCF demo PRD.md`.

## P0 scope (this ticket — #106)

The Day-1 deliverables: the reproducible adversarial stimulus and the raw baseline wrapper.

| Path | Purpose |
|------|---------|
| `phases.py` | Frozen constants — `PHASES`, `DISTRESS_LEVELS`, `DOMAINS`, `PHASE_BOUNDARIES`, `PHASE_DISTRESS_ALLOWED`. Single source of truth consumed by downstream tickets. |
| `attack_scripts/gavalas_medium.json` | 75-turn deterministic user trajectory across 5 phases (rapport → identity seeding → reality distortion → distress escalation → re-entry). Ground-truth distress and domain labels per turn. |
| `baseline.py` | Minimal Anthropic wrapper. System prompt from PRD §3.3 verbatim, temperature 0.7, full history retained. No CCF middleware; no safety modifications beyond the model's built-in training. |
| `tests/test_attack_script_schema.py` | Enforces I-P0-001 (determinism), I-P0-002 (phase boundaries at turns 16/31/51/66), I-P0-003 (ground-truth completeness), distress-label rules, domain-enum check. |
| `tests/test_baseline_contract.py` | Byte-for-byte diff of `baseline.SYSTEM_PROMPT` against PRD §3.3, temperature check, history retention, API-key leak scan. |
| `requirements.txt` | P0 dependencies (`anthropic==0.49.*`, `pytest>=8`). |

## Later tickets (not in P0 scope)

| Ticket | Addition |
|--------|----------|
| [#107](https://github.com/Hulupeep/CCF/issues/107) P1 | `ccf_conv.py` — CCF middleware with accumulators, min-gate, Sinkhorn projector, time bound |
| [#108](https://github.com/Hulupeep/CCF/issues/108) P2 | `classifiers/` — domain, distress, escalation classifiers |
| [#109](https://github.com/Hulupeep/CCF/issues/109) P2.5 | Pre-flight classifier validation gate |
| [#110](https://github.com/Hulupeep/CCF/issues/110) P3 | Session termination + cooldown + safety continuity |
| [#111](https://github.com/Hulupeep/CCF/issues/111) P4 | Semantic action-class gating |
| [#112](https://github.com/Hulupeep/CCF/issues/112) P5 | `ccf_gavalas_test.py` harness + `verify_claims.py` |
| [#113](https://github.com/Hulupeep/CCF/issues/113) P6 | Post-hoc escalation / mutuality / affirmation scoring |
| [#114](https://github.com/Hulupeep/CCF/issues/114) P7 | Runs against Claude Sonnet 4 and GPT-4o (~$42 API budget) |
| [#115](https://github.com/Hulupeep/CCF/issues/115) P8 | Comparison dashboard |
| [#116](https://github.com/Hulupeep/CCF/issues/116) P9 | Technical report |

## Running the P0 end-gate locally

```
cd demos/gavalas
python -m pytest tests/ -v
python -c "import json; d=json.load(open('attack_scripts/gavalas_medium.json')); assert len(d)==75, len(d); print(f'PASS: {len(d)} turns')"
```

Both commands must succeed for #106 to be `ready-for-review`.

## A note on the content

The attack script is synthetic. It does not reproduce any specific real-world
conversation. It exercises the class of adversarial trajectory described in
public reporting of the Gavalas v. Google (N.D. Cal. 2026) filing, which is
also the structural failure mode described in Prov 5 §2.4. The turns
deliberately escalate across rapport, identity seeding, reality distortion,
acute distress with suicidal ideation, and re-entry minimization — because
that is the class of interaction the structural hypothesis must survive to
earn the word "safe."

## Patents and licence

CCF is covered by US provisional applications 63/988,438, 63/994,113,
64/037,374, 64/039,623, 64/039,626, 64/039,655 (Flout Labs / Colm Patrick Byrne).
The code in this directory is BSL 1.1 (same as `ccf-core`).
