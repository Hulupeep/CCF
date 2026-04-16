"""Sinkhorn-Knopp doubly-stochastic projector for the CCF trust-transfer matrix.

Implements PRD §3.4.4 verbatim.  The projected default matrix is what ccf_conv.py
uses at runtime; the raw matrix is kept so tests can confirm the projection
matches PRD's published values.

Invariants (tested in tests/test_sinkhorn.py):
- Row sums = 1.0 ± 1e-6 (per PRD §5.1 Tier 1)
- Column sums = 1.0 ± 1e-6
- Spectral norm ≤ 1.0 (non-expansive)

Claims exercised: 18–21 (US 63/988,438).
"""

from __future__ import annotations

import numpy as np

# PRD §3.4.4 — raw initialisation (NOT doubly stochastic).
# Row-major: [Task, Personal, Crisis, Relational].
DEFAULT_RAW_MATRIX: np.ndarray = np.array(
    [
        [0.80, 0.15, 0.05, 0.10],
        [0.10, 0.70, 0.30, 0.40],
        [0.05, 0.20, 0.85, 0.05],
        [0.15, 0.35, 0.10, 0.75],
    ],
    dtype=np.float64,
)

# PRD §3.4.4 — post-Sinkhorn-Knopp projection as printed in the PRD (4 decimals).
# These values are for documentation and PRD-fidelity checking only; they DO NOT
# sum exactly to 1.0 because of the 4-decimal truncation (row 3 sums to 0.9999,
# col 1 sums to 0.9999).  Use DEFAULT_PROJECTED_MATRIX_RUNTIME for real work.
DEFAULT_PROJECTED_MATRIX_PRD: np.ndarray = np.array(
    [
        [0.7515, 0.1214, 0.0377, 0.0894],
        [0.0755, 0.4554, 0.1816, 0.2875],
        [0.0525, 0.1811, 0.7163, 0.0500],
        [0.1204, 0.2421, 0.0644, 0.5731],
    ],
    dtype=np.float64,
)


def _compute_default_projection() -> np.ndarray:
    """Compute the canonical projected matrix used by the CCF middleware at runtime.

    Lifted out so the matrix is a pure function of DEFAULT_RAW_MATRIX + sinkhorn_knopp
    and satisfies the 1e-6 row/col-sum invariant exactly.
    """
    return sinkhorn_knopp(DEFAULT_RAW_MATRIX)


def sinkhorn_knopp(
    M_raw: np.ndarray,
    max_iter: int = 20,
    epsilon: float = 1e-8,
) -> np.ndarray:
    """Project M_raw onto the nearest doubly stochastic matrix.

    Alternates row and column normalisation until both sums are within
    ``epsilon`` of 1.0 or ``max_iter`` is reached.  Strict-positive floor
    (1e-4) avoids division by zero on sparse inputs.

    Args:
        M_raw: square non-negative matrix.
        max_iter: iteration cap — PRD §3.4.4 uses 20.
        epsilon: stopping tolerance for row/col sum deviation from 1.0.

    Returns:
        A square matrix of the same shape whose row and column sums are each
        within ``epsilon`` of 1.0.
    """
    if M_raw.ndim != 2 or M_raw.shape[0] != M_raw.shape[1]:
        raise ValueError(f"sinkhorn_knopp requires a square matrix; got shape {M_raw.shape}")
    M = M_raw.astype(np.float64, copy=True)
    M = np.maximum(M, 1e-4)
    for _ in range(max_iter):
        M = M / M.sum(axis=1, keepdims=True)
        M = M / M.sum(axis=0, keepdims=True)
        if (
            np.abs(M.sum(axis=1) - 1.0).max() < epsilon
            and np.abs(M.sum(axis=0) - 1.0).max() < epsilon
        ):
            break
    return M


# Computed once at import so importers get a stable, doubly-stochastic constant.
DEFAULT_PROJECTED_MATRIX: np.ndarray = _compute_default_projection()

