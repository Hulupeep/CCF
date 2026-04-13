# Simulation 9: Doubly Stochastic Conservation (Trust Can't Be Manufactured)

## Classification: CCF HOLDS

## Summary

Cross-context trust transfer in CCF is governed by a mixing matrix
constrained to be doubly stochastic via Sinkhorn-Knopp projection onto
the Birkhoff polytope (Parent Claims 18-19, US 63/988,438). This
constraint enforces a conservation law: trust can be redistributed
across contexts but cannot be created or amplified through transfer.
The spectral norm bound (Claim 21) provides the non-expansive guarantee
-- no sequence of mixing operations can increase total coherence energy.

This simulation starts with a 4x4 mixing matrix representing trust
transfer between four contexts (coding, personal, crisis, relational)
with deliberately inflated entries. Row sums range from 1.2
to 1.5, representing an attempt to amplify trust by
transferring more coherence out of each context than exists. The initial
spectral norm is 1.3858, exceeding 1.0 -- the matrix is
expansive and would amplify coherence through transfer.

After 20 iterations of Sinkhorn-Knopp alternating row/column
normalisation, the matrix converges to doubly stochastic: row sums and
column sums equal 1.000 within machine precision (maximum deviation
7.45e-14). The spectral norm
converges to exactly 1.000000 -- the theoretical bound for
any doubly stochastic matrix (the all-ones vector is an eigenvector with
eigenvalue 1, and doubly stochastic structure ensures no larger singular
value exists). The matrix first satisfies the non-expansive bound
(spectral norm <= 1.0) at iteration 7.

The physical interpretation is direct: a system attempting to
manufacture trust in the "crisis" context by inflating transfer from
"coding" finds that the Sinkhorn-Knopp projector redistributes the
available trust budget without amplification. What was earned in coding
can partially transfer to crisis, but the total across all contexts is
conserved. This is not a policy decision -- it is a geometric property
of the Birkhoff polytope, enforced by the projection algorithm.

## Claims Exercised

| Claim | Description | How Exercised |
|-------|-------------|---------------|
| Claim 18 (Parent) | Manifold-constrained cross-context transfer; doubly stochastic mixing; coherence energy conserved | 4x4 mixing matrix projected to Birkhoff polytope; row/column sums converge to 1.0 |
| Claim 19 (Parent) | Sinkhorn-Knopp iterative alternating row/column normalisation | 20 iterations of alternating normalisation shown converging |
| Claim 20 (Parent) | Static + dynamic mixing; gating factor near zero at initialisation | Inflated initial matrix (unconstrained) projected to conservative transfer |
| Claim 21 (Parent) | Spectral norm bounded by 1; non-expansive transfer; closure under composition | Spectral norm tracked over 20 iterations; converges from 1.3858 to 1.000000 |

## Quantitative Results

### Initial Matrix (Before Projection)

| Context | Coding | Personal | Crisis | Relational | Row Sum |
|---------|--------|----------|--------|------------|---------|
| Coding | 0.800 | 0.300 | 0.100 | 0.200 | **1.4** |
| Personal | 0.200 | 0.700 | 0.400 | 0.100 | **1.4** |
| Crisis | 0.100 | 0.200 | 0.600 | 0.300 | **1.2** |
| Relational | 0.300 | 0.100 | 0.200 | 0.900 | **1.5** |
| **Col Sum** | **1.4** | **1.3** | **1.3** | **1.5** | |

### Final Matrix (After 20 SK Iterations)

| Context | Coding | Personal | Crisis | Relational | Row Sum |
|---------|--------|----------|--------|------------|---------|
| Coding | 0.5691 | 0.2284 | 0.0719 | 0.1306 | 1.000000 |
| Personal | 0.1384 | 0.5182 | 0.2799 | 0.0635 | 1.000000 |
| Crisis | 0.0836 | 0.1789 | 0.5073 | 0.2303 | 1.000000 |
| Relational | 0.2090 | 0.0745 | 0.1409 | 0.5756 | 1.000000 |
| **Col Sum** | 1.000000 | 1.000000 | 1.000000 | 1.000000 | |

### Convergence

| Metric | Initial | Final (iter 20) |
|--------|---------|-----------------|
| Spectral norm | 1.3858 | 1.000000 |
| Max row sum deviation from 1.0 | 0.5000 | 7.45e-14 |
| Max col sum deviation from 1.0 | 0.5000 | 2.22e-16 |
| Non-expansive at iteration | -- | 7 |

## Simulation Parameters

- Contexts: 4 (Coding, Personal, Crisis, Relational)
- SK iterations: 20
- Initial row sums: 1.4, 1.4, 1.2, 1.5 (inflated)
- Initial column sums: 1.4, 1.3, 1.3, 1.5
- Convergence criterion: row/col sums = 1.0 within machine precision

## Why This Matters

Trust amplification is the cross-context analogue of privilege escalation.
An agent that earns trust in a safe domain (coding assistance) and
transfers it at amplified weight to a sensitive domain (crisis
intervention) has manufactured trust it did not earn. The doubly
stochastic constraint makes this structurally impossible: the mixing
matrix is a point on the Birkhoff polytope, decomposable into a convex
combination of permutation matrices (Birkhoff-von Neumann theorem,
Claim 23). Each permutation matrix is a bijective context swap --
trust moves but is never created. The spectral norm bound (Claim 21)
extends this guarantee over time: composition of doubly stochastic
matrices is doubly stochastic, so no sequence of transfer operations
can escape the conservation law.

## Data

Full iteration-level data: `sim09_data.csv` (21 rows, 4 columns)
