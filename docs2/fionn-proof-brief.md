# Formal Proof Brief — Hierarchical Coherence Mixer

**From:** Colm Carey, Flout Labs
**To:** Fionn
**Date:** February 2026
**Re:** US Provisional Patent Application 63/988,438 — Four Mathematical Proofs
**Confidential**

---

## Cover Note

### What CCF Is

Contextual Coherence Fields is a mathematical architecture that gives an autonomous robot earned, context-sensitive social behaviour — without rules, scripts, or a machine learning model. The robot accumulates a trust score per distinct sensory environment. A robot that trusts the living room starts at zero in the basement. A single bad moment cannot erase earned trust, but trust is never granted in advance.

Three primitives combine: context-keyed accumulators, a minimum gate (both the sensor reading *and* the history must be positive for the robot to be expressive), and a graph min-cut boundary (the robot discovers its own comfort zone from the topology of what it has experienced).

### What We Need From You

Four formal proofs establishing that the **HierarchicalMixer** — the scaling extension that makes CCF practical on production humanoid platforms — preserves every mathematical guarantee we've already claimed for the smaller flat version.

These are not academic decoration. They serve two concrete purposes:

1. **Patent prosecution (deadline: February 2027).** The continuation claims (Appendix C below) rest on proof sketches. Your proofs convert them to verified mathematics. If a patent examiner challenges a guarantee, we need a citable proof, not a sketch.

2. **arXiv submission.** The paper's §3 "Mathematical Foundations" section will contain these proofs in full. We need them in LaTeX.

### Priority Order and Deadlines

| Order | Proof | Target | Why |
|-------|-------|--------|-----|
| **1st** | Proof 1 — Norm Preservation | **2 weeks** | Critical path. Everything else depends on it. |
| **2nd** | Proof 4 — Transition Smoothing | **2 weeks** | Easiest. Good warm-up. Can run in parallel with Proof 1. |
| **3rd** | Proof 2 — Compositional Closure | **3 weeks** | Likely a corollary of Proof 1. |
| **4th** | Proof 3 — Approximation Bound | **5 weeks** | Most novel. Most valuable for arXiv. |

**All four finalised:** 8 weeks.

### ⚠️ Critical Flag Instruction — Proof 1

**If Proof 1 fails for any cluster configuration — including unequal cluster sizes or the singleton cluster edge case — email Colm immediately before doing anything else.**

Do not work around it. Do not continue to Proof 4. Flag it.

A broken guarantee is fixable if caught now (we modify the correction formula and re-implement). It is a patent vulnerability if caught during prosecution.

### Deliverable Format

- LaTeX preferred (for arXiv submission; PDF acceptable for review)
- Each proof in the five-part format described in Appendix D
- For Proof 1: explicit treatment of the singleton cluster case
- For Proof 3: explicit bound as a function of (bleeding edge magnitude, k, within-cluster coherence variance)
- One-page non-technical summary for patent counsel

---

## Section 1 — The Algorithm

### 1.1 What the HierarchicalMixer Does

When the number of active contexts exceeds a threshold (default: 50), the flat `n×n` mixing matrix becomes computationally expensive (O(n²) per tick). The HierarchicalMixer replaces it with a two-level block-diagonal structure:

- **k intra-cluster matrices** Hᵢ (each `nᵢ × nᵢ`, doubly stochastic) governing coherence transfer *within* each cluster
- **1 inter-cluster matrix** G (`k × k`, doubly stochastic) governing coherence transfer *between* clusters

The cluster partition is provided by the min-cut algorithm (applied to the relational graph of accumulated interactions). The HierarchicalMixer does not discover clusters — it uses them.

### 1.2 The Five-Step Algorithm

Given coherence vector `c ∈ [0,1]ⁿ` partitioned into k clusters, where cluster i contains nᵢ contexts with indices `member_indices[i]`:

**Step 1 — Intra-cluster mixing**

For each cluster i, apply the doubly stochastic matrix Hᵢ (nᵢ × nᵢ) to the sub-vector of coherence values in that cluster:

```
c'ⱼ = Σₖ Hᵢ[j,k] · cₖ     for j ∈ cluster i
```

(Matrix-vector multiply, row-major. Output clamped to [0,1].)

**Step 2 — Cluster summary (count-weighted mean)**

For each cluster i, compute the mean coherence weighted by interaction counts:

```
μᵢ = Σⱼ∈cluster_i (c'ⱼ · countⱼ) / Σⱼ∈cluster_i countⱼ
```

If all counts in cluster i are zero, use uniform weights: `μᵢ = (1/nᵢ) Σⱼ c'ⱼ`

**Step 3 — Inter-cluster mixing**

Apply the doubly stochastic matrix G (k × k) to the cluster mean vector:

```
μ' = G · μ     where μ = [μ₁, μ₂, ..., μₖ]ᵀ
```

**Step 4 — Count-weighted correction**

Distribute the inter-cluster adjustment back to individual contexts. For each context j in cluster i:

```
wⱼ = countⱼ / Σₘ∈cluster_i countₘ     (normalised within cluster; uniform if all counts = 0)

c''ⱼ = c'ⱼ + (μ'ᵢ − μᵢ) · wⱼ
```

**Step 5 — Clamp**

```
c''ⱼ = clamp(c''ⱼ, 0, 1)
```

### 1.3 Implementation Reference (Rust, `apply_core`)

The exact implementation. This is the function Fionn's proofs must cover:

```rust
fn apply_core(
    clusters: &[CoherenceCluster],   // cluster definitions (member indices, intra matrices)
    num_clusters: usize,
    inter_mix: &[f32],               // k×k inter-cluster projected matrix, row-major
    coherence_values: &mut [f32],    // in-place; indexed by context index
    interaction_counts: &[u32],      // parallel array; same indexing
) {
    // ── Step 1: intra-cluster mixing ─────────────────────────────────────────
    for cluster in clusters.iter().take(num_clusters) {
        let n = cluster.size;
        let mut c_out = [0.0f32; MAX_CLUSTER_SIZE];
        for i in 0..n {
            let mut sum = 0.0f32;
            for k in 0..n {
                let global_k = cluster.member_indices[k];
                sum += cluster.intra_mix_projected[i * MAX_CLUSTER_SIZE + k]
                    * coherence_values[global_k];
            }
            c_out[i] = sum.clamp(0.0, 1.0);
        }
        for i in 0..n {
            coherence_values[cluster.member_indices[i]] = c_out[i];
        }
    }

    // ── Step 2: cluster summary means ────────────────────────────────────────
    let mut s_bar = [0.0f32; MAX_CLUSTERS];     // μ vector
    for (ci, cluster) in clusters.iter().enumerate().take(num_clusters) {
        let n = cluster.size;
        let mut sum = 0.0f32;
        for j in 0..n {
            sum += coherence_values[cluster.member_indices[j]];
        }
        s_bar[ci] = sum / n as f32;
        // Note: the spec uses count-weighted mean; implementation uses simple mean here.
        // Fionn: the proof should handle the count-weighted variant in §1.2.
    }

    // ── Step 3: inter-cluster mixing ─────────────────────────────────────────
    let mut s_bar_prime = [0.0f32; MAX_CLUSTERS];   // μ' vector
    for i in 0..num_clusters {
        let mut sum = 0.0f32;
        for k in 0..num_clusters {
            sum += inter_mix[i * MAX_CLUSTERS + k] * s_bar[k];
        }
        s_bar_prime[i] = sum;
    }

    // ── Steps 4 & 5: correction + clamp ──────────────────────────────────────
    for (ci, cluster) in clusters.iter().enumerate().take(num_clusters) {
        let n = cluster.size;
        let delta_mean = s_bar_prime[ci] - s_bar[ci];   // μ'ᵢ − μᵢ

        // Weight denominator
        let total_count: u32 = (0..n)
            .map(|j| interaction_counts[cluster.member_indices[j]])
            .sum();

        for j in 0..n {
            let idx = cluster.member_indices[j];
            let w = if total_count > 0 {
                interaction_counts[idx] as f32 / total_count as f32
            } else {
                1.0 / n as f32
            };
            coherence_values[idx] = (coherence_values[idx] + delta_mean * w).clamp(0.0, 1.0);
        }
    }
}
```

**Constants:** `MAX_CLUSTERS = 32`, `MAX_CLUSTER_SIZE = 128`, `MAX_CONTEXTS_PER_CLUSTER = 128`.

---

## Section 2 — Existing Proof Sketches

These are starting points. Fionn is formalising and strengthening them, not starting from scratch.

### 2.1 Norm Preservation (Proof Sketch)

*(From `ccf-hierarchical-mixing.md §3.1`)*

**Theorem:** The hierarchical mixing operation is non-expansive: `‖c''‖₂ ≤ ‖c‖₂`.

**Sketch:**

Step 1 (intra-cluster mixing): Each Hᵢ is doubly stochastic, so `‖c'ᵢ‖₂ ≤ ‖cᵢ‖₂` for each cluster (DS matrices have spectral norm ≤ 1). Therefore `‖c'‖₂ ≤ ‖c‖₂`.

Step 3 (inter-cluster mixing): G is doubly stochastic, so `‖μ'‖₂ ≤ ‖μ‖₂`. The inter-cluster transfer redistributes coherence between clusters without amplification.

Step 4 (correction): The correction adds `(μ'ᵢ − μᵢ)` distributed across contexts in cluster i, weighted by normalised weights `(Σwᵢⱼ = 1)`. The total coherence added to cluster i equals `nᵢ × (μ'ᵢ − μᵢ)`. Since G is doubly stochastic, `Σᵢ nᵢ × μ'ᵢ = Σᵢ nᵢ × μᵢ` when clusters are *equal-sized* (by column-sum constraint on G). Total coherence is conserved or reduced.

**The gap this sketch does not close:** The equal-sized assumption in Step 4 is unproven for the general case. When cluster sizes nᵢ are unequal, the column-sum constraint on G is not sufficient to guarantee `Σᵢ nᵢ × (μ'ᵢ − μᵢ) = 0`. Fionn's Proof 1 must close this gap — or flag it.

### 2.2 Compositional Closure (Proof Sketch)

*(From `ccf-hierarchical-mixing.md §3.2`)*

**Theorem:** The composition of hierarchical mixing operations over multiple ticks retains the non-expansive property.

**Sketch:** Each tick applies a sequence of doubly stochastic operations (intra-cluster, inter-cluster). The composition of doubly stochastic matrices is doubly stochastic. The hierarchical structure decomposes each tick's operation into compositions of smaller doubly stochastic operations. Therefore the composite operation across arbitrary ticks is bounded by 1 in spectral norm.

**What Fionn must formalise:** This sketch elides the correction step, which is not a simple matrix multiply. The formal proof must show the correction term does not escape the bound across T composed operations.

### 2.3 Transition Smoothing (Proof Sketch)

*(From `ccf-hierarchical-mixing.md §4.3`)*

When cluster structure changes, the system blends old and new structures:

```
H_effective(t) = (1 − α(t)) · H_old + α(t) · H_new
```

where α(t) ramps linearly from 0 to 1 over `transition_blend_ticks`.

**Sketch:** Because both H_old and H_new are doubly stochastic, and the set of doubly stochastic matrices is *convex* (Birkhoff polytope is convex), the interpolated matrix is also doubly stochastic for all α ∈ [0, 1]. All guarantees are preserved during transitions.

**What Fionn must formalise:** The formal statement should include: (a) the blended matrix is DS, (b) `‖B_α(c)‖₂ ≤ ‖c‖₂`, (c) `B_α` converges to H_new at α = 1. Parts (b) and (c) follow directly from (a) and the non-expansive property. This is the easiest proof — good warm-up.

---

## Section 3 — The Critical Test Case

This is the empirical evidence that the algorithm works for unequal cluster sizes. Fionn's Proof 1 must explain *why* it works — or determine whether the clamp in Step 5 is doing all the work (which would mean the pre-clamp norm bound is weaker than claimed).

```rust
/// Verify correctness when clusters have very different sizes.
///
/// Tests with clusters of sizes 1, 5, 2 — verifies the computation produces
/// valid [0,1] outputs, the identity-matrix case is a no-op, and non-trivial
/// inter-cluster mixing actually moves values between clusters.
#[test]
fn test_unequal_cluster_sizes() {
    let mut mixer = HierarchicalMixer::new(test_config());

    // Cluster 0: 1 context  (index 0)       ← THE SINGLETON
    // Cluster 1: 5 contexts (indices 1–5)
    // Cluster 2: 2 contexts (indices 6–7)
    let assignments = [0u16, 1, 1, 1, 1, 1, 2, 2];
    mixer.update_clusters(&assignments, 3);

    // ── Identity case (matrices = identity → output must equal input) ─────
    let original = [0.9_f32, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7];
    let mut coherence = original;
    let counts = [5u32, 1, 2, 3, 4, 5, 6, 7];

    mixer.apply(&mut coherence, &counts);

    for (i, (&after, &before)) in coherence.iter().zip(original.iter()).enumerate() {
        assert!((after - before).abs() < 1e-5, "identity: coherence[{}] changed", i);
    }

    // ── Non-trivial inter-cluster mixing ──────────────────────────────────
    // 3×3 inter-cluster matrix: diagonal dominant, small off-diagonal transfer
    let inter_raw = [0.8_f32, 0.1, 0.1,
                     0.1,     0.8, 0.1,
                     0.1,     0.1, 0.8];
    mixer.update_inter_params(&inter_raw);
    mixer.reproject_all();

    // Singleton cluster (index 0, coherence = 0.9) is high.
    // Other clusters are low. Inter-cluster mixing should pull the singleton down.
    let mut coherence2 = [0.9_f32, 0.1, 0.1, 0.1, 0.1, 0.1, 0.8, 0.8];
    let counts2 = [10u32, 1, 1, 1, 1, 1, 5, 5];

    mixer.apply(&mut coherence2, &counts2);

    // All values must remain in [0, 1]
    for (i, &v) in coherence2.iter().enumerate() {
        assert!(v >= 0.0 && v <= 1.0, "coherence2[{}] = {} out of [0,1]", i, v);
    }

    // The singleton (index 0, coherence = 0.9) must have decreased slightly
    // because the other clusters have low coherence and inter-cluster mixing
    // transfers some of their lower means back to the singleton.
    assert!(
        coherence2[0] < 0.95,
        "singleton cluster coherence should decrease due to inter-cluster transfer, got {}",
        coherence2[0]
    );
}
```

**What to look for in the singleton case:**

The singleton cluster (cluster 0, size = 1) has `μ₀ = c'₀ = 0.9` (its only member is itself). After inter-cluster mixing, `μ'₀ = 0.8 × 0.9 + 0.1 × μ₁ + 0.1 × μ₂`. Since μ₁ and μ₂ are low, μ'₀ < 0.9. The correction applied is `Δc₀ = (μ'₀ − μ₀) × w₀` where `w₀ = 1.0` (the singleton has weight 1 since it is the only member of its cluster and all weight normalises to 1). So `c''₀ = c'₀ + (μ'₀ − 0.9)`. For the chosen matrices and inputs, the test passes and the value stays in [0, 1].

**The question for Proof 1:** Is this always guaranteed, or is it only the clamp catching it? Specifically: can `|μ'ᵢ − μᵢ| × w₀` exceed `c'₀` (driving c''₀ below 0) or `1 − c'₀` (driving c''₀ above 1), requiring the clamp to enforce the bound? If yes, the pre-clamp norm is not bounded by ‖c‖₂ — only the post-clamp norm is.

---

## Section 4 — Patent Claims 18–23

*From US Provisional Application 63/988,438.*

These are the claims the proofs must support. Each proof's "Patent relevance" section should cite the specific claim number.

**Claim 18.** A system for manifold-constrained cross-context coherence transfer in an autonomous social robot, comprising: a plurality of context-keyed coherence accumulators organised into a coherence vector of dimension n; a mixing matrix of dimension n-by-n constrained to be doubly stochastic via projection onto the Birkhoff polytope, governing bounded cross-context coherence transfer such that total coherence energy across all context streams is conserved; and a behavioral gating subsystem that computes effective coherence from the coherence vector after application of the mixing matrix.

**Claim 19.** The system of claim 18, wherein the doubly stochastic constraint is enforced by projecting unconstrained mixing parameters onto the Birkhoff polytope via the Sinkhorn-Knopp algorithm comprising iterative alternating row and column normalisation of a positive matrix until convergence to a doubly stochastic matrix.

**Claim 20.** The system of claim 18, wherein the mixing matrix comprises a static component representing baseline cross-context relationships, and a dynamic component computed from a learned projection of current sensor state modulated by a gating factor initialised near zero such that the robot begins operation with minimal cross-context coherence transfer and earns increased transfer capacity through accumulated interaction.

**Claim 21.** The system of claim 18, wherein the spectral norm of the mixing matrix is bounded by 1, ensuring that cross-context coherence transfer is non-expansive and the robot cannot amplify coherence through transfer; and wherein the set of doubly stochastic matrices is closed under composition, ensuring that the composite mixing operation across any number of processing cycles retains the non-expansive property and provides a mathematical guarantee of long-term behavioural stability.

**Claim 22.** The system of claim 18, wherein near-zero entries of the mixing matrix identify context boundaries between coherence domains, and wherein the context boundary topology represented by the mixing matrix structure corresponds to the partition that would be obtained by applying a graph min-cut algorithm to a relational graph constructed from accumulated interaction episodes.

**Claim 23.** The system of claim 18, wherein the mixing matrix admits decomposition as a convex combination of permutation matrices by the Birkhoff-von Neumann theorem, providing geometric interpretability wherein the cross-context coherence transfer at any processing cycle is decomposable into a weighted blend of discrete context-swapping operations.

---

## Appendix A — Continuation Claims A–D

*Draft claim language for the continuation filing (depends on your proofs).*

**Continuation Claim A — Hierarchical Block-Diagonal Structure**

The system of Claim 18, wherein the n-by-n mixing matrix is structured as a hierarchical block-diagonal mixing system comprising:
(a) a plurality of intra-cluster mixing matrices, each of dimension nᵢ-by-nᵢ, each constrained to be doubly stochastic via Sinkhorn-Knopp projection;
(b) an inter-cluster mixing matrix of dimension k-by-k, constrained to be doubly stochastic via Sinkhorn-Knopp projection, governing bounded coherence transfer between clusters;
(c) wherein the cluster partition corresponds to the context boundary partition obtained by the graph min-cut algorithm;
(d) wherein the hierarchical mixing operation preserves the non-expansive property, compositional closure, and boundary correspondence guarantees of the non-hierarchical mixing matrix.

*Requires: Proof 1 (non-expansive), Proof 2 (compositional closure).*

**Continuation Claim B — Adaptive Mode Selection**

The system of Claim A, wherein the system adaptively selects between a flat mixing matrix when the number of active context streams is at or below a threshold, and the hierarchical block-diagonal mixing system when the number exceeds the threshold, the transition preserving all doubly stochastic guarantees.

*Requires: Proof 1 (both modes are non-expansive).*

**Continuation Claim C — Transition Smoothing**

The system of Claim A, wherein structural transitions of the cluster partition are smoothed by interpolating between the previous and updated hierarchical mixing structures over a configurable blending period, the interpolation remaining within the Birkhoff polytope by the convexity of the set of doubly stochastic matrices.

*Requires: Proof 4 (transition smoothing).*

**Continuation Claim D — Approximation Bound**

The system of Claim A, wherein the hierarchical mixing operation approximates the flat mixing operation with error bounded by a function of the min-cut bleeding edge, such that when the cluster partition is sharp (bleeding edge approaches zero), the hierarchical and flat operations converge.

*Requires: Proof 3 (approximation bound).*

---

## Appendix B — Proof 5: Merge Invariant (Non-Blocking, for arXiv Appendix)

*From `ccf-merge-claim-artifact.docx §6`. Not blocking the continuation filing. Should appear in the arXiv appendix.*

### B.1 Setup

When the accumulator map exceeds capacity, two accumulators may be merged. The merge rule is:

- `coherence_merged = min(c_A, c_B)` — conservative: trust cannot be manufactured
- `count_merged = n_A + n_B` — cumulative: relational history cannot be erased

**Why min for coherence, sum for count?** These fields serve different semantic roles. Coherence represents the trust the robot has in a specific context — it must be conservative, otherwise merging could grant familiarity the robot hasn't earned. Interaction count feeds the decay floor (which asymptotes at 0.5 for n → ∞) — it must be cumulative, otherwise merging destroys relational investment. The asymmetry (conservative on trust, cumulative on history) is the unique combination that respects both invariants simultaneously.

### B.2 Theorem Statement (Draft)

Let `G(c, I) = min(I, c)` be the minimum gate (Claim 2).
Let `F(n) = 0.5 × (1 − 1/(1 + n/20))` be the earned floor.
Let `E(c, n) = max(c, F(n))` be effective coherence after floor.
Let `M(A, B) = (min(c_A, c_B), n_A + n_B)` be the merge.

**Claim:** For all accumulators A, B and instantaneous coherence I ∈ [0, 1]:

```
G(E(M(A, B)), I)  ≤  max(G(E(A), I), G(E(B), I)) + ε(n_A, n_B)
```

where `ε(n_A, n_B) = F(n_A + n_B) − F(max(n_A, n_B))`, which is bounded by 0.5 − F(max(n_A, n_B)) and decreases monotonically as either count grows.

### B.3 Proof Sketch

**Case 1: F(n_A + n_B) ≤ min(c_A, c_B)**

The floor does not activate. `E(M(A, B)) = min(c_A, c_B)`.
Since `min(c_A, c_B) ≤ c_A`, we have `G(E(M(A,B)), I) = min(I, min(c_A, c_B)) ≤ min(I, c_A) = G(E(A), I)`. ε = 0. QED.

**Case 2: F(n_A + n_B) > min(c_A, c_B)**

The floor activates. `E(M(A, B)) = F(n_A + n_B)`.

Since F is monotonically increasing: `F(n_A + n_B) > F(max(n_A, n_B))`. The merged floor can exceed either source's floor by at most `F(n_A + n_B) − F(max(n_A, n_B))`.

The maximum relaxation (ε at its worst) occurs when merging two accumulators with n = 1 each: `F(2) − F(1) ≈ 0.024`. For accumulators with n ≥ 20 (the promotion threshold), the relaxation is < 0.01.

This bounded relaxation is **architecturally intentional**: total relational investment (sum of counts) should provide resilience even when individual coherence values were low.

### B.4 Note on N-Way Merge

N-way merge (pairwise min iterated) equals global min (min is associative, commutative). Sum is associative, commutative. The composition of N merges is trivially the global min / global sum. Proof extends to N > 2 without additional work.

---

## Appendix C — Proof Format Requirements

Each of the four proofs must be in this five-part format (for dual use in arXiv and patent filing):

1. **Theorem statement** — precise, self-contained, all conditions explicit. All variables defined before use.
2. **Notation table** — define every symbol: vectors (bold), matrices (uppercase), scalars (lowercase), norms (‖·‖₂ unless specified).
3. **Proof** — rigorous but readable by a mathematically sophisticated non-specialist. Show each step. Flag if any step requires a lemma.
4. **Behavioural interpretation** — what the result means for the robot in plain English (1–3 sentences for the arXiv audience).
5. **Patent relevance** — which claim or continuation claim this proof supports (cite by number).

Target length: 1–2 pages per proof in LaTeX (not counting the notation table).

---

## Appendix D — Reference List

| Reference | Relevance to Proofs |
|-----------|---------------------|
| Birkhoff (1946), von Neumann (1953) — Birkhoff-von Neumann theorem | DS matrices = convex hull of permutation matrices. Convexity of the Birkhoff polytope. Used in Proof 4. |
| Horn & Johnson, *Matrix Analysis* (2nd ed., 2013) | Spectral norm, doubly stochastic matrices, non-expansive operators. Foundation for Proof 1. |
| Stoer & Wagner (1997) — "A simple min-cut algorithm" | The min-cut algorithm that produces the cluster partition. Context for Proof 3. |
| Sinkhorn (1964), Knopp (1967) — Sinkhorn-Knopp algorithm | The projection used to enforce the DS constraint. Referenced in Claims 19, A, B, C. |
| arXiv:2512.24880v2 (Xie et al., 2026) — DeepSeek mHC paper | Prior art: fixed n=4 manifold-constrained mixing for LLM training. Does NOT address dynamic cluster discovery, unequal cluster sizes, or n > 4. Distinguishing this is the job of Proof 3. |
| US Provisional Application 63/988,438 (Flout Labs, 2026) | The filing these proofs support. Claims 18–23 + Continuation Claims A–D. |

---

*Flout Labs | Galway, Ireland*
*Patent Pending: US Provisional 63/988,438, priority date 23 February 2026*
*Confidential — do not distribute*
