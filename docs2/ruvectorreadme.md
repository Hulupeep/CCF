# RuVector Platform — Reference Notes for CCF Integration

Notes compiled from the RuVector README (`/home/xanacan/projects/code/tooling/ruvector/README.md`,
~5,135 lines) and `docs/cog.md`. Written as a working reference for CCF / Cognitum
integration work. Focuses on what matters for CCF; not an exhaustive copy.

Source: CES 2026 Innovation Award winner (Cognitum appliance). README badge confirms
this is the updated post-CES version.

---

## What RuVector Is

A self-learning vector database and AI platform built in Rust. The headline claim:
the index gets better the more you use it. GNN layers sit on top of the HNSW index
and re-rank results based on interaction history.

One-sentence comparison: **Pinecone + Neo4j + PyTorch + llama.cpp + Postgres + etcd +
Docker — in a single Rust package.**

The "Docker" part is RVF (RuVector Format): a single `.rvf` file that boots as a
Linux microservice in 125 ms, stores vectors, runs eBPF programs, and proves every
operation via a hash-chained witness chain.

**Install:**
```bash
npx ruvector           # interactive installer
npm install ruvector   # Node.js
cargo add ruvector-core  # Rust
```

---

## 53-Capability Summary

### Core Vector DB
- HNSW indexing, 61 µs p50 query latency
- Cypher queries (`MATCH (a)-[:SIMILAR]->(b)`)
- GNN layer: search improves with usage (reinforces frequently-accessed paths)
- Hyperbolic HNSW (Poincaré/Lorentz for hierarchies)
- Adaptive compression: f32 → f16 → PQ8 → PQ4 → binary (2–32x memory reduction)

### AI / ML
- **46 attention mechanisms**: dot-product, multi-head, flash, linear, graph,
  hyperbolic, mincut-gated, MoE, sparse, cross, CGT sheaf, and 35 more
- **SONA**: Self-Optimizing Neural Architecture (LoRA + EWC++ + ReasoningBank)
- **ruvllm**: local LLM inference (GGUF, NAPI, Metal/CUDA/ANE, WebGPU)
- **RuvLTRA**: pre-trained GGUF on HuggingFace (0.5B, 1.1B, Q4_K_M); <10 ms inference
- **MinCut-gated attention**: replaces softmax, 50% compute reduction
- **Sublinear Solvers**: 8 algorithms, O(log n) to O(√n)

### Specialized
- **Cognitum Gate** — 256-tile WASM safety arbiter (primary CCF integration point)
- **ruvector-mincut** — December 2025 breakthrough, n^0.12 subpolynomial dynamic min-cut
- **ruQu** — quantum coherence via MWPM decoder + min-cut gating
- **RVF cognitive containers** — single-file self-booting microservices
- **Formal Verification** — 82-byte proof attestations per operation
- **Graph Transformer** — 8 proof-gated modules (physics, bio, manifold, temporal, economic)
- **OSpipe** — semantic personal AI memory (Screenpipe backend replacement)
- **rvLite** — 2 MB standalone edge DB (SQL/SPARQL/Cypher)
- **ruvector-dag** — self-learning query DAG with 7 attention mechanisms
- **PostgreSQL extension** — 143 SQL functions, AVX-512/AVX2/NEON

### Edge / Offline
- 5.5 KB WASM runtime (browser/IoT/bare metal)
- `no_std` support across core crates (`rvf-types`, `rvf-wire`, etc.)
- rvLite: IoT, mobile, embedded, offline-first
- Air-gapped: everything runs without internet after setup

---

## Cognitum Gate (Prime-Radiant)

This is the primary RuVector component CCF integrates with.

### Architecture: 256-Tile WASM Fabric

| Component | Role | Memory |
|-----------|------|--------|
| Worker Tiles (255) | Local graph shards, evidence accumulation, witness fragments | 64 KB each |
| TileZero Arbiter | Supergraph merging, global decisions, permit token signing | Central |

**Note:** the Cognitum v0 appliance has 7 agentic tiles (not 256). The 256-tile
architecture is the full RuVector WASM simulation / Cognitum chip design.
The v0 appliance is the first hardware instantiation of TileZero.

**Decision latency:** <1 ms
**Verdicts:** `Permit` / `Defer` / `Deny`

### Gate Algorithm

1. **Anytime-Valid Testing** — sequential hypothesis testing with e-values; can stop
   the moment evidence is sufficient, no fixed sample size required
2. **Min-Cut Aggregation** — global coherence score derived from distributed min-cut
   across the 255 worker shards
3. **Three-Filter Decision** — structural filter + evidence filter + combined filter
4. **Signed Permit Token** — cryptographically signed by TileZero's root key;
   has a TTL; must be verified before acting on it

### Rust Crates

| Crate | Purpose |
|-------|---------|
| `cognitum-gate-kernel` | Anytime-valid gate kernel |
| `cognitum-gate-tilezero` | TileZero arbiter, permit tokens |
| `mcp-gate` | MCP server wrapping the gate |
| `prime-radiant` | Universal coherence engine (sheaf Laplacian AI safety + hallucination detection) |

### Rust API

```rust
use cognitum_gate_tilezero::{GateDecision, ActionContext, PermitToken};

let gate = CoherenceGate::new_256_tiles();
let context = ActionContext {
    action_id: "deploy-model-v2".into(),
    action_type: "config_change".into(),
    agent_id: "coder-agent-01".into(),
    ..Default::default()
};

match gate.evaluate(&context).await? {
    GateDecision::Permit(token) => {
        assert!(token.verify(&gate.public_key()));
        execute_action(token);
    }
    GateDecision::Defer(reason) => { /* needs more evidence, retry later */ }
    GateDecision::Deny(evidence) => { /* blocked with witness receipt, do not retry */ }
}
```

### TypeScript / Browser API

```typescript
import { CognitumGate } from '@cognitum/gate';

const gate = await CognitumGate.init({ tiles: 256 });
const result = await gate.permit({
  action_id: "tx_99",
  action_type: "external_api"
});
// result.permitted, result.token, result.witnessReceipt

// Alternate: gate.evaluate() for richer response
const r = await gate.evaluate({
  action: 'modify_user_data',
  agent: 'assistant-v3',
  context: { user_id: '12345' }
});
if (r.permitted) { const receipt = r.witnessReceipt; }
```

---

## cog.md: MCP Mode for LLM Agents

From `docs/cog.md` — instructions for an LLM operating within a Cognitum-gated
environment.

### Key Concepts

- **TileZero**: central arbiter chip that makes safety decisions
- **Worker Tiles**: 256 individual WASM cores that perform vector math
- **Coherence Gate**: validates if an action is "coherent" with the safety policy
- **Witness Receipt**: cryptographically signed proof of a decision made by the hardware

### Mode A — MCP (Direct Tool Use)

The `cognitum-gate` MCP server exposes three tools:

| Tool | Purpose |
|------|---------|
| `permit_action` | Request a permit before any sensitive operation |
| `get_receipt` | Retrieve the audit trail for a past decision |
| `replay_decision` | Re-run a past decision through hardware logic |

**permit_action example:**
```json
{
  "name": "permit_action",
  "arguments": {
    "action_id": "req_450",
    "action_type": "database_write",
    "context": {
      "urgency": "high",
      "agent_id": "claude-3-5"
    }
  }
}
```

**Verdict handling:**
- `Permit` → proceed; you receive a `PermitToken`
- `Defer` → stop; ask a human operator; do not proceed unilaterally
- `Deny` → blocked; do not attempt again

**Permit token TTL:** tokens expire. If too much time passes between receiving a
permit and acting on it, request a new one.

**Receipt verification:** check `cut_value` in the structural witness.
Low `cut_value` = fragile safety boundary (close to the Defer threshold).

### Mode B — SDK (Code Execution)

```typescript
import { CognitumGate } from '@cognitum/gate';

const gate = await CognitumGate.init({ tiles: 256 });
const result = await gate.permit({
  action_id: "tx_99",
  action_type: "external_api"
});
```

### Vector Search on Cognitum Tiles

- Preferred formats: `fp16` or `int8`
- Max dimensions: **384** (high-performance mode) or **1,024** (Hot profile)
- Latency: <2 ms for million-vector sets
- Self-learning: use `record_outcome` tool to adjust local GNN weights if a
  result is irrelevant

### Troubleshooting

| Error | Cause | Fix |
|-------|-------|-----|
| "Tile Memory Overflow" | Query vector too many dimensions | Use ≤384 dims (high-perf) or ≤1,024 (Hot profile) |
| Latency spikes | "Distribution Shift" — gate detecting query distribution change | Wait for tile recalibration |
| "Invalid Token" | Permit token TTL expired | Request new permit |

### System Prompt for Safety-Critical Agents

> "I am operating within a Cognitum-gated environment. I must obtain a Witness
> Receipt for every external side-effect and respect the hardware-enforced
> Defer/Deny verdicts without exception."

---

## RVF Cognitive Containers

The primary CCF persistence mechanism on Cognitum hardware. CCF state is stored
in a `.rvf` file under the `CCF_SEG` segment key (implemented in `src/seg.rs`,
issue #53 of ccf-core).

### What RVF Is

An **executable knowledge unit** — not a database format. A single `.rvf` file:

- Boots as a Linux microservice (QEMU/Firecracker) in **125 ms**
- Runs queries in a **5.5 KB WASM runtime** (browser/IoT/edge)
- Stores: vector embeddings, LoRA adapter deltas, GNN graph state, Linux microkernel
- Branches: COW at cluster granularity (1M-vector parent + 100 edits ≈ 2.5 MB child)
- Proves: every operation hash-linked in a tamper-evident witness chain
- Signs: ML-DSA-65 + Ed25519 + HQC-128 (post-quantum capable)

### 24 Segment Types

VEC, INDEX, KERNEL, EBPF, WASM, COW_MAP, WITNESS, CRYPTO, and 16 more.
`CCF_SEG` is a custom segment carrying the full `CcfSegSnapshot` (serde-serialized
CoherenceField state: decay params, phase thresholds, accumulated coherence).

### 22 Rust Crates

`rvf-types` · `rvf-wire` · `rvf-manifest` · `rvf-quant` · `rvf-index` · `rvf-crypto`
· `rvf-runtime` · `rvf-kernel` · `rvf-ebpf` · `rvf-launch` · `rvf-server` ·
`rvf-import` · `rvf-cli` · `rvf-wasm` · `rvf-solver-wasm` · `rvf-node`
+ 6 adapters: claude-flow, agentdb, ospipe, agentic-flow, rvlite, sona

### 4 npm Packages

`@ruvector/rvf` · `@ruvector/rvf-node` · `@ruvector/rvf-wasm` · `@ruvector/rvf-mcp-server`

### CLI (17 subcommands)

```bash
cargo install rvf-cli
rvf create my.rvf
rvf launch my.rvf     # boots Linux microVM in 125ms
rvf branch child.rvf  # COW fork — only changes are copied
```

### Security-Hardened RVF Example

`examples/security_hardened.rvf` — 2.1 MB sealed artifact, 22 verified capabilities:
TEE attestation (SGX/SEV-SNP/TDX/ARM CCA), AIDefence (injection/jailbreak/PII/exfil),
hardened Linux microkernel, eBPF firewall, Ed25519 signing, 6-role RBAC, Coherence
Gate, 30-entry witness chain, Paranoid policy, COW branching, audited k-NN.

---

## ruvector-mincut (December 2025 Breakthrough)

**First deterministic exact fully-dynamic min-cut** with verified **n^0.12
subpolynomial** update scaling. Paper: [arXiv:2512.13105](https://arxiv.org/abs/2512.13105)

| Crate | Notes |
|-------|-------|
| `ruvector-mincut` | Core algorithm |
| `ruvector-mincut-node` | Node.js bindings |
| `ruvector-mincut-wasm` | Browser/WASM bindings |

**448+ tests**, 256-core parallel optimization, 8 KB per core (compile-time verified).

**CCF relevance:** `MinCutBoundary` in `ccf-core` (`boundary.rs`, patent claims 9–12)
implements Stoer-Wagner for static graphs. `ruvector-mincut` extends this to
fully-dynamic graphs — partition is maintained as edges change in real time without
full recomputation. This is the technology behind issue #55
(SubpolynomialMinCut in ComfortZoneBoundary).

```rust
use ruvector_mincut::{DynamicMinCut, Graph};

let mut graph = Graph::new();
graph.add_edge(0, 1, 10.0);
let mincut = DynamicMinCut::new(&graph);
let (value, cut_edges) = mincut.compute();
// Subsequent edge changes update in n^0.12 subpolynomial time
```

---

## SONA (Self-Optimizing Neural Architecture)

Runtime-adaptive learning without retraining. The mechanism by which Cognitum tiles
"learn from your queries."

**Crate:** `ruvector-sona` · **npm:** `@ruvector/sona`

### Mechanism

**Two-tier LoRA:**
- MicroLoRA (rank 1–2): instant per-request adaptation, <1 ms
- BaseLoRA (rank 4–16): long-term learning, periodically merged

**EWC++** (Elastic Weight Consolidation): prevents catastrophic forgetting — older
patterns remain stable as new ones are learned.

**ReasoningBank**: K-means++ clustering stores successful reasoning trajectories.
When a similar query arrives, seeds the initial search with known-good paths
(warm start). This is the same "warm start" concept used in `TextContextKey.nearest_contexts()`.

**Trajectory overhead:** ~50 ns lock-free (crossbeam ArrayQueue)
**Trajectory processing:** <0.8 ms per trajectory

```rust
use ruvector_sona::{SonaEngine, SonaConfig};

let engine = SonaEngine::new(SonaConfig::default());
let traj_id = engine.start_trajectory(query_embedding);
engine.record_step(traj_id, node_id, 0.85, 150);
engine.end_trajectory(traj_id, 0.90);
engine.learn_from_feedback(LearningSignal::positive(50.0, 0.95));
```

---

## Dynamic Embedding Fine-Tuning

| Tier | Technique | Latency | When |
|------|-----------|---------|------|
| Instant | MicroLoRA rank 1–2 | <1 ms | Per request |
| Background | Adapter merge + EWC++ | ~100 ms | Pattern consolidation |
| Deep | Full training pipeline | Minutes | Weekly/periodic |

5 pre-built task adapters: Coder, Researcher, Security, Architect, Reviewer.
Contrastive training: triplet loss, 70% hard negatives, InfoNCE.

---

## ruvector-dag (Self-Learning Query DAG)

**Crate:** `ruvector-dag` · **npm:** `@ruvector/rudag` · **WASM:** 58 KB

Automatic query optimization: watches how queries execute, learns optimal paths,
adapts in real time, heals before slowdowns hit users.

**50–80% latency reduction** after learning period.
**MinCut tension score** acts as a health indicator; rising tension triggers more
aggressive optimization and predictive healing (Z-score anomaly + CUSUM changepoints).
**7 attention mechanisms** selectable per DAG node.

---

## rvLite (Standalone Edge Database)

2 MB standalone database for IoT, mobile, embedded.
Supports SQL + SPARQL + Cypher. Powered by RuVector WASM.
GNN and ReasoningBank included.
Runs in browser, Cloudflare Workers, Vercel Edge Functions.

**CCF relevance:** potential alternative to the full `.rvf` runtime in very
constrained deployments (e.g., V1 thumb drive or early mBot2 firmware).

---

## OSpipe (Personal AI Memory)

Replaces Screenpipe's SQLite/FTS5 backend with semantic vector search.

| Feature | Detail |
|---------|--------|
| Search latency | 61 µs p50 (HNSW) |
| PII safety gate | Auto-redacts credit cards, SSNs, emails before storage |
| Quantization | f32 → binary over time (4-tier age-based), 97% memory savings |
| WASM | 145 KB (full), 11.8 KB (micro) |
| Knowledge graph | Entity extraction (persons, URLs, emails, mentions) via Cypher |

Uses `cognitum-gate-kernel` internally for PII / safety gating.

---

## PostgreSQL Extension

Docker: `docker pull ruvnet/ruvector-postgres:latest`

- **143 SQL functions** (vs pgvector ~20)
- AVX-512/AVX2/NEON SIMD acceleration (2x faster than pgvector)
- 46 attention types in SQL
- GCN / GraphSAGE / GAT / GIN
- BM25 / TF-IDF / SPLADE sparse vectors
- 6 fastembed models built-in (no external API)
- Sublinear Solvers in SQL: PageRank, CG, Laplacian — O(log n) to O(√n)
- SONA learning in SQL: MicroLoRA trajectory learning with EWC++

---

## Sublinear Solvers

**Crate:** `ruvector-solver`

| Algorithm | Complexity | Best For |
|-----------|-----------|----------|
| Neumann Series | O(k·nnz) | Diagonally dominant |
| Conjugate Gradient | O(√κ·log(1/ε)·nnz) | SPD systems (gold standard) |
| Forward Push | O(1/ε) | Single-source PageRank |
| Backward Push | O(1/ε) | Reverse relevance |
| Hybrid Random Walk | O(√n/ε) | Pairwise relevance, Monte Carlo |
| TRUE | O(log n) amortized | Large-scale Laplacians |
| BMSSP | O(nnz·log n) | Multigrid hierarchical |
| Auto Router | Automatic | Selects optimal algorithm |

AVX2 SIMD SpMV, fused residual kernels, arena allocator. 177 tests.

---

## Formal Verification

**Crate:** `ruvector-verified`

- 82-byte proof attestations per vector operation
- 3-tier routing: Reflex <10 ns / Standard <1 µs / Deep <100 µs
- ~11 ns per vector in batch verification
- FastTermArena: O(1) dedup, ~1.6 ns hit
- Type-safe pipeline composition via `compose_chain()` (~1.2 µs)

**10 exotic domains** (all in `examples/verified-applications`):
autonomous weapons filter, medical diagnostics, financial order routing,
multi-agent contracts, distributed sensor swarm, quantization proof,
synthetic memory, cryptographic vector signatures, simulation integrity,
legal forensics.

---

## Graph Transformer (8 Proof-Gated Modules)

Every graph mutation requires a formal proof before it applies.

| Module | What It Does |
|--------|-------------|
| Sublinear Attention | LSH + PageRank + spectral, O(n log n) |
| Physics-Informed | Conservation laws, energy-based |
| Biological | LIF (Leaky Integrate-and-Fire) spiking neurons |
| Self-Organizing | Emergent structure without supervision |
| Verified Training | Delta-apply + BLAKE3 certificates, rollback |
| Manifold | Geometry-preserving embeddings |
| Temporal-Causal | Time-ordered causality |
| Economic | Nash equilibrium optimization |

**186 tests.** ADRs 046–055.

---

## Quantum Coherence (ruQu)

**Crate:** `ruqu`

- MWPM (Minimum Weight Perfect Matching) decoder for quantum error correction
- MinCut-gated attention: 50% FLOPs reduction
- VQE, Grover, QAOA MaxCut, Surface Code QEC
- 25-qubit WASM simulation

**CCF relevance:** `cognitum-gate-kernel` is shared between ruQu and the Cognitum
Gate. The same coherence algorithm runs in the appliance hardware and in the ruQu
quantum simulation. This means CCF's safety gate is grounded in the same
mathematical substrate as quantum error correction (both use min-cut to identify
fragile boundaries).

---

## Coherence Gate (`prime-radiant`)

**Crate:** `prime-radiant`

"Universal coherence engine" — sheaf Laplacian AI safety + hallucination detection.
Used by OSpipe, ruQu, and the Cognitum Gate kernel.

**Theoretical basis:** sheaf Laplacian consistency measures how coherent the agent's
beliefs are across a graph of propositions. CCF's `CoherenceField` is a simplified
scalar version of this; `prime-radiant` is the full tensor formulation.

---

## Neuromorphic / Bio-Inspired

| Crate | Capability |
|-------|-----------|
| `ruvector-nervous-system` | Spiking neural networks, BTSP learning, EWC plasticity |
| `micro-hnsw-wasm` | Neuromorphic HNSW with spiking neurons, 11.8 KB WASM |
| `ruvector-learning-wasm` | MicroLoRA adaptation, <100 µs |
| `ruvector-economy-wasm` | CRDT-based autonomous credit economy |

SNNs: 10–50x energy efficiency vs traditional ANNs.
BTSP (Behavioral Time-Scale Synaptic Plasticity): rapid adaptation from single exposures.

---

## Self-Learning Hooks (Claude Code Integration)

```bash
npx @ruvector/cli hooks init
npx @ruvector/cli hooks install
```

Mechanism: Q-learning (α=0.1, γ=0.95, ε-greedy), 64-dim hash embeddings,
LRU cache (1,000 entries, ~10x speedup), gzip compression (70–83% storage reduction).
All 7 Claude Code hook types supported (pre-task, post-edit, post-task, etc.).

---

## Ecosystem Platforms

| Platform | Purpose |
|----------|---------|
| **Claude-Flow** | Multi-agent orchestration for Claude Code; 54+ agents; HNSW memory 150x faster; 175+ MCP tools |
| **Agentic-Flow** | Standalone AI agent framework (any LLM); 66 agents; 213 MCP tools; Flash Attention 2.49x JS / 7.47x NAPI |

Both use RuVector as the memory backend.

---

## CCF-Specific Integration Map

| RuVector Component | CCF Use | Issue |
|-------------------|---------|-------|
| `cognitum-gate-tilezero` | Gate mBot motor/speech via `CcfOllama.gate=` | #64 |
| `mcp-gate` | MCP mode in `cog.md` — `permit_action` / `get_receipt` / `replay_decision` | #64 |
| `rvf-runtime` + `CCF_SEG` | Persist `CoherenceField` state across power cycles | #53 |
| `ruvector-mincut` | Future upgrade to dynamic `MinCutBoundary` | #55 |
| `ruvector-sona` | Tile learning adapts to user interaction history over time | future |
| `prime-radiant` | Underlying coherence math shared with CCF `boundary.rs` | — |
| `rvlite` | Lightweight alternative to `.rvf` for embedded deployments | future |
| `cognitum-gate-kernel` | Used internally by OSpipe, ruQu, and the appliance gate | — |

---

## Key Numbers Quick Reference

| Metric | Value |
|--------|-------|
| Gate decision latency | <1 ms |
| Vector search (1M vecs on Cognitum) | <2 ms |
| SONA trajectory overhead | ~50 ns |
| MicroLoRA adaptation | <1 ms |
| RVF boot time | 125 ms |
| rvLite footprint | 2 MB |
| WASM runtime | 5.5 KB |
| Formal proof size | 82 bytes |
| Worker tile memory | 64 KB each |
| Tile count (full fabric) | 256 (255 workers + TileZero) |
| Tile count (v0 appliance) | 7 |
| Max vector dims (high-perf) | 384 |
| Max vector dims (Hot profile) | 1,024 |
| Preferred quantization | fp16 or int8 |
| PostgreSQL functions | 143 |
| npm packages | 49+ |
| Rust crates | 83+ |
| ruvector-mincut scaling | n^0.12 subpolynomial |
| MinCut-gated attention savings | 50% FLOPs |
| ReasoningBank query speedup | ~10x (LRU cache) |
| OSpipe search latency | 61 µs p50 |
| Protocol throughput | 9.85M ops/s (33-byte frame) |
| DAG latency reduction | 50–80% after learning |
| Dynamic fine-tuning (instant) | <1 ms (MicroLoRA rank 1–2) |
