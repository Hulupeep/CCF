# CCF Critical Path to Revenue — 2026

**Status:** Feb 2026. Patent filed. Website live. Crate published. Repo public. Demo not yet filmed.

---

## What Exists

| Asset | Status | Notes |
|-------|--------|-------|
| theshyrobot.com | ✅ Live | Vercel, private repo Hulupeep/shyrobot |
| ccf-core v0.1.5 | ✅ Published | crates.io, homepage = theshyrobot.com |
| US Prov. Patent 63/988,438 | ✅ Filed | Priority: 23 Feb 2026. Non-provisional due 23 Feb 2027 |
| CCF repo (Hulupeep/CCF) | ✅ Public | mBot2 demo code, patent claim tests |
| ccf-cognitum crate | ✅ Built | SensorVocabulary impl for Cognitum v0 sensor port |
| Demo video | ❌ Not filmed | MOST URGENT. Blocks everything below |
| arXiv paper | ❌ Not started | Blocked on demo results |
| Non-provisional patent | ❌ Not started | Hard deadline Feb 23, 2027 |
| Cognitum reference firmware | ❌ Not started | B2B hardware licensing path |
| ccf-py (LLM relational layer) | 🔶 Specced | docs/strategy/ccfllm.md — PyPI path |

---

## The Critical Path

```
DEMO VIDEO (#37, #38)
    │
    ├──► arXiv paper (#66)          ──► press / HN / AI Twitter pickup
    │         │                                │
    │         └──► grant applications (#69)    └──► inbound developer interest
    │                   │                                   │
    │                   └── SFI / EI runway ──────────────► ccf-py MVP (#57)
    │                                                            │
    └──► Cognitum ref firmware (#68) ──► licensing outreach (#70) ──► first B2B revenue
                │
                └──► Non-provisional patent (#67) ── HARD DEADLINE FEB 23 2027
```

**Every path flows from the demo video. It is the unlock.**

---

## Revenue Paths (in order of likely speed)

### 1. Grant funding — 6–12 months (non-dilutive)
- Enterprise Ireland Commercialisation Fund (~€500k)
- Science Foundation Ireland Frontiers for the Future (~€750k–€2m)
- EU Horizon Cluster 4 Digital & Industrial (consortium, longer lead)
- **Requires:** working prototype (demo video), arXiv preprint, commercialisation plan
- **Best fit for stage:** pre-revenue, Irish company, novel patented IP

### 2. Hardware licensing — Cognitum/RuVector
- CCF as the behavioural layer on the Cognitum v0 appliance
- Drop-in firmware, BSL-1.1 evaluation → commercial license from Flout Labs
- Per-device royalty or upfront license fee
- **Requires:** reference firmware (#68), demo, licensing conversation (#70)
- **Timeline:** 6–18 months to first signed deal

### 3. Developer tools / ccf-py SaaS
- ccf-py (#57) as a PyPI library + hosted API (CCF-as-a-Service for LLM apps)
- Target: Ollama/LM Studio power users, AI app developers, therapy platforms
- Revenue: free tier / pro tier ($20–50/month) or per-API-call
- **Timeline:** 3–6 months to ship, 12–18 months to meaningful MRR
- **Why:** 10× larger audience than Rust ecosystem, taps AI memory market narrative

### 4. Enterprise consulting
- Help companies implement CCF in their robotic or AI systems
- High margin, no scale — but early revenue while product matures
- **Requires:** demo + credibility (paper helps)

### 5. Seed investment
- After demo + paper + early developer traction
- The patent is the anchor for any VC conversation
- **Timeline:** 2026 H2 earliest

---

## Near-Term Action Priority

### This month
1. **Hardware dry run (#37)** — get the mBot2 running all 4 demo sections
2. **Non-provisional patent attorney** — engage now, brief on LLM/software extension
3. **Film demo video (#38)** — 5-minute, clear shy-to-fluent arc

### Next 60 days (after demo)
4. **arXiv paper draft** — structure exists in patent; paper is 80% there
5. **EI Commercialisation Fund application** — straightforward given the IP and prototype
6. **Cognitum outreach** — schedule meeting, bring integration architecture doc

### Next 90 days
7. **ccf-py v0.1.0** (#57) — Python binding, Ollama integration, PyPI publish
8. **Cognitum reference firmware** (#68) — full deployable firmware package
9. **Developer documentation** — mdBook site, more examples, getting-started guide

---

## The Non-Provisional Patent — Hard Deadline Tracker

| Milestone | Target date |
|-----------|-------------|
| Engage attorney | April 2026 |
| Claims draft reviewed | August 2026 |
| Software/LLM claims added | September 2026 |
| Final draft approved | November 2026 |
| Filed with USPTO | January 2027 |
| Hard deadline | **23 February 2027** |

---

## What Makes CCF Real (the three proofs)

1. **It works** — demo video shows a real robot earning trust over real time
2. **It's novel** — patent + arXiv paper establishes it wasn't done before
3. **It matters** — the Milo problem (asymmetric behaviour in powerful agentic systems) is not fictional. The constraint CCF provides is the answer.

All three are within reach in 2026. The demo video is the first domino.

---

## What Else Could Be Built

- **RuVector platform integration** — CCF as the trust layer in the broader RuVector nervous system platform. Every robot running RuVector runs CCF. Potential for platform-level licensing.
- **Cognitum chip** — if the v0 appliance leads to a v1 chip design, CCF could be silicon-embedded. The wasm32-wasip1 and thumbv7em targets are ready.
- **Multi-robot coherence** — Raft-based social endorsement log (nextplan.md Phase 7) — robot swarms that share earned trust. New IP territory, new patent continuation claim.
- **CCF for LLM agents (agentic AI)** — the ccf-py architecture applies to any agent that should not be more confident in a domain than it has earned the right to be. This is an AI safety story, not just a robotics story. Larger addressable market.
