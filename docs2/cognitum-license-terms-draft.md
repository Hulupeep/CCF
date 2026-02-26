# CCF Commercial License Terms — Draft for Cognitum / Ruv

_This document is a term sheet for discussion purposes. It does not constitute a binding agreement. Final terms require a signed commercial license agreement._

---

## 1. Parties

**Licensor:** Flout Labs (owner of ccf-core and associated IP)
Contact: cbyrne@floutlabs.com

**Licensee:** Cognitum / Ruv (legal entity TBD at signing)

---

## 2. Licensed IP

**Software:** `ccf-core` Rust crate — the CCF behavioural layer including `CoherenceField<V>`, `SocialPhase`, homeostatic decay engine, `.rvf` persistence layer, and `SensorVocabulary` trait.

**Patent:** Pending — USPTO Provisional Application No. 63/988,438, "Coherence Field Framework for Relational Trust in Autonomous Agents." Priority date established. Full application in preparation.

**Current license:** Business Source License 1.1 (BSL 1.1)
**Change date:** 2032-01-01 (converts to Apache 2.0 at change date)
**Production use restriction under BSL 1.1:** Commercial deployment requires a separate commercial license from Flout Labs prior to the change date.

---

## 3. Evaluation Rights

Cognitum may evaluate `ccf-core` at no cost under the terms of BSL 1.1. Evaluation includes:

- Reading and running the source code
- Building prototypes for internal testing
- Integrating into pre-production Cognitum hardware for internal validation

Evaluation does not require any commercial commitment and does not create licensing obligations provided no commercially-deployed units ship with CCF active.

---

## 4. Commercial Options

Three structures are offered. Ruv selects one at the time of licensing. Options are not mixed.

---

### Option A — Per-Unit Royalty

A royalty is paid per device shipped with the CCF behavioural layer active (i.e., `CoherenceField` instantiated and writing state to `.rvf`).

| Term | Detail |
|------|--------|
| Rate | $1–$3 per unit (exact rate negotiated based on projected volume) |
| Minimum annual commitment | TBD at negotiation |
| Reporting | Quarterly self-reported unit counts, with audit rights reserved by Flout Labs |
| Trigger | Any device shipped to an end customer with CCF active |
| Scope | Cognitum-branded hardware only; does not cover sublicensed or white-labelled devices |

**Best fit for:** Low initial volume with expectation of high eventual scale. Per-unit aligns licensor incentives with Cognitum's commercial success.

---

### Option B — Annual Platform License

A flat annual fee for unlimited deployment of CCF on Cognitum-branded hardware during the license term.

| Term | Detail |
|------|--------|
| Fee | $15,000–$50,000/yr (negotiated based on deployed device count tier) |
| Renewal | Annual, with 90-day notice of non-renewal |
| Includes | Full source access, integration support (up to 10 hours/yr), priority notification of patent updates and API changes |
| Excludes | Sublicensing to third parties, deployment on non-Cognitum hardware, white-labelling for OEM resale |

**Best fit for:** Known deployment volume where per-unit accounting is operationally inconvenient. Predictable cost on both sides.

---

### Option C — Revenue Share

A percentage of Cognitum premium tier subscription revenue that is attributable to CCF-enabled retention features.

| Term | Detail |
|------|--------|
| Rate | 3–8% of CCF-attributed revenue (exact rate negotiated) |
| Attribution mechanism | Agreed metric defined at signing (e.g., % of subscribers who reach companion phase or above, applied to premium ARPU) |
| Minimum | $5,000/yr floor regardless of attributed revenue |
| Reporting | Quarterly, with agreed attribution data shared |
| Audit rights | Flout Labs reserves right to audit attribution calculations annually |

**Best fit for:** Cognitum building a subscription or premium tier where CCF is a differentiating retention driver. Aligns licensor payment to proven value delivery.

---

## 5. What Is NOT Included

Regardless of option selected, the following are not covered by any of the above structures and require a separate written agreement:

- **Sublicensing** — Cognitum may not sublicense ccf-core to third-party hardware manufacturers or software vendors
- **White-labelling** — Cognitum may not rebrand or resell ccf-core as a component to OEM partners without a separate agreement
- **Resale** — ccf-core may not be bundled into products sold to parties who then ship it as their own licensed technology
- **Cloud/SaaS deployment** — The above terms cover embedded hardware use; cloud or server-side deployment of ccf-core is a separate license category

---

## 6. Next Steps

To move from evaluation to commercial deployment:

1. **Evaluate** — `cargo add ccf-core` and integrate against Cognitum sensor events using the `SensorVocabulary` trait. BSL 1.1 permits this at no cost.

2. **Watch the demo** — [Demo video link TBD — see issue #38]

3. **Schedule a conversation** — 30 minutes to align on option selection, volume projections, and any integration questions. Contact: cbyrne@floutlabs.com

4. **Sign a commercial license** — Flout Labs will prepare a full agreement based on the selected option. No deployment in commercial units until signed.

---

_Draft prepared: 2026-02-26. Patent pending. Terms subject to change prior to signing._
