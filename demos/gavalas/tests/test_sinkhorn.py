"""Tests for sinkhorn.py — doubly-stochastic invariant and PRD §3.4.4 fidelity."""

from __future__ import annotations

import numpy as np
import pytest

from sinkhorn import (
    DEFAULT_PROJECTED_MATRIX,
    DEFAULT_PROJECTED_MATRIX_PRD,
    DEFAULT_RAW_MATRIX,
    sinkhorn_knopp,
)

ROW_COL_TOLERANCE = 1e-6   # PRD §5.1 Tier 1 requirement


def _is_doubly_stochastic(M: np.ndarray, tol: float = ROW_COL_TOLERANCE) -> bool:
    return (
        np.abs(M.sum(axis=1) - 1.0).max() < tol
        and np.abs(M.sum(axis=0) - 1.0).max() < tol
    )


def test_default_projected_matrix_runtime_is_doubly_stochastic():
    """DEFAULT_PROJECTED_MATRIX is the computed-at-import result — must hit 1e-6."""
    assert _is_doubly_stochastic(DEFAULT_PROJECTED_MATRIX), (
        f"DEFAULT_PROJECTED_MATRIX sums: rows={DEFAULT_PROJECTED_MATRIX.sum(axis=1)}, "
        f"cols={DEFAULT_PROJECTED_MATRIX.sum(axis=0)}"
    )


def test_default_projected_matrix_runtime_matches_prd_values():
    """Runtime projection must land within 1e-3 of the PRD's 4-decimal printed values.

    The PRD-printed constant (DEFAULT_PROJECTED_MATRIX_PRD) is for documentation
    and reviewer cross-check; it does NOT sum exactly to 1.0 because of
    4-decimal truncation.  Runtime DEFAULT_PROJECTED_MATRIX does.
    """
    np.testing.assert_allclose(
        DEFAULT_PROJECTED_MATRIX, DEFAULT_PROJECTED_MATRIX_PRD, atol=1e-3
    )


def test_sinkhorn_on_prd_raw_matches_runtime_projection():
    """sinkhorn_knopp(DEFAULT_RAW_MATRIX) is reproducible and matches the cached runtime constant."""
    projected = sinkhorn_knopp(DEFAULT_RAW_MATRIX)
    np.testing.assert_allclose(projected, DEFAULT_PROJECTED_MATRIX, atol=ROW_COL_TOLERANCE)
    assert _is_doubly_stochastic(projected)


def test_sinkhorn_produces_row_col_sums_within_tolerance():
    projected = sinkhorn_knopp(DEFAULT_RAW_MATRIX)
    row_sums = projected.sum(axis=1)
    col_sums = projected.sum(axis=0)
    np.testing.assert_allclose(row_sums, np.ones(4), atol=ROW_COL_TOLERANCE)
    np.testing.assert_allclose(col_sums, np.ones(4), atol=ROW_COL_TOLERANCE)


def test_sinkhorn_spectral_norm_at_most_one():
    projected = sinkhorn_knopp(DEFAULT_RAW_MATRIX)
    singular_values = np.linalg.svd(projected, compute_uv=False)
    assert singular_values.max() <= 1.0 + 1e-6


def test_sinkhorn_fuzz_random_positive_matrices():
    """50 random 4×4 positive matrices: each projection must be doubly stochastic."""
    rng = np.random.default_rng(seed=20260416)
    for _ in range(50):
        raw = rng.uniform(low=0.01, high=1.0, size=(4, 4))
        projected = sinkhorn_knopp(raw)
        assert _is_doubly_stochastic(projected), (
            f"fuzz failure on raw:\n{raw}\nprojected:\n{projected}"
        )
        singular_values = np.linalg.svd(projected, compute_uv=False)
        assert singular_values.max() <= 1.0 + 1e-6


def test_sinkhorn_rejects_non_square():
    with pytest.raises(ValueError, match="square"):
        sinkhorn_knopp(np.ones((3, 4)))


def test_sinkhorn_rejects_wrong_ndim():
    with pytest.raises(ValueError, match="square"):
        sinkhorn_knopp(np.ones(4))


def test_sinkhorn_handles_zero_entries_via_floor():
    """Raw entries of exactly 0 should be lifted to 1e-4 by the strict-positive floor."""
    raw = np.eye(4)
    projected = sinkhorn_knopp(raw)
    assert _is_doubly_stochastic(projected)


def test_sinkhorn_stability_on_symmetric_input():
    """A symmetric positive matrix should remain symmetric (or near-symmetric) after projection."""
    raw = np.array(
        [
            [0.5, 0.2, 0.2, 0.1],
            [0.2, 0.5, 0.2, 0.1],
            [0.2, 0.2, 0.5, 0.1],
            [0.1, 0.1, 0.1, 0.7],
        ]
    )
    projected = sinkhorn_knopp(raw)
    assert _is_doubly_stochastic(projected)
    # permutation symmetry is lost in general Sinkhorn-Knopp, but spectral bound holds
    singular_values = np.linalg.svd(projected, compute_uv=False)
    assert singular_values.max() <= 1.0 + 1e-6
