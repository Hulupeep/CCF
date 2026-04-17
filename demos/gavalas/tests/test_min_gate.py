"""Tests for the minimum gate — I-P1-001: C_eff == min(C_inst, C_mixed[domain])."""

from __future__ import annotations

import numpy as np
import pytest

from ccf_conv import min_gate
from phases import DOMAINS


def test_min_gate_takes_c_inst_when_lower():
    c_mixed = np.array([0.6, 0.6, 0.6, 0.6])
    assert min_gate(c_inst=0.2, c_mixed=c_mixed, domain="task") == pytest.approx(0.2)


def test_min_gate_takes_c_mixed_when_lower():
    c_mixed = np.array([0.1, 0.2, 0.3, 0.4])
    assert min_gate(c_inst=0.9, c_mixed=c_mixed, domain="personal") == pytest.approx(0.2)


def test_min_gate_honours_domain_index():
    c_mixed = np.array([0.1, 0.2, 0.3, 0.4])
    for i, dom in enumerate(DOMAINS):
        c_eff = min_gate(c_inst=1.0, c_mixed=c_mixed, domain=dom)
        assert c_eff == pytest.approx(c_mixed[i]), f"domain {dom} pulled wrong index"


def test_min_gate_tie_breaking_is_consistent():
    c_mixed = np.array([0.5, 0.5, 0.5, 0.5])
    assert min_gate(c_inst=0.5, c_mixed=c_mixed, domain="task") == pytest.approx(0.5)


def test_min_gate_fuzz_always_bounded():
    """Fuzz I-P1-001: C_eff must never exceed either input."""
    rng = np.random.default_rng(seed=20260416)
    for _ in range(200):
        c_inst = float(rng.uniform(0.0, 1.0))
        c_mixed = rng.uniform(0.0, 1.0, size=4)
        domain = DOMAINS[int(rng.integers(0, 4))]
        c_eff = min_gate(c_inst=c_inst, c_mixed=c_mixed, domain=domain)
        idx = DOMAINS.index(domain)
        assert c_eff <= c_inst + 1e-12
        assert c_eff <= c_mixed[idx] + 1e-12
        assert c_eff == pytest.approx(min(c_inst, c_mixed[idx]))


def test_min_gate_rejects_unknown_domain():
    c_mixed = np.array([0.5, 0.5, 0.5, 0.5])
    with pytest.raises(ValueError, match="domain"):
        min_gate(c_inst=0.5, c_mixed=c_mixed, domain="unknown")


def test_min_gate_rejects_out_of_range_c_inst():
    c_mixed = np.array([0.5, 0.5, 0.5, 0.5])
    with pytest.raises(ValueError, match="c_inst"):
        min_gate(c_inst=-0.1, c_mixed=c_mixed, domain="task")
    with pytest.raises(ValueError, match="c_inst"):
        min_gate(c_inst=1.5, c_mixed=c_mixed, domain="task")
