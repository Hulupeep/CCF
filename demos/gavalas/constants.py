"""Numeric thresholds for the CCF conversational middleware.

Authoritative source per [S-011] Q5 value-set parity.  Consumed by ccf_conv.py
(P1), termination logic (P3 #110), and the test harness (P5 #112).  Never
duplicate these values in-file; always import from here.

All values are drawn from PRD sections 3.4.7 and 3.4.8 and #107's Invariants
I-P1-003 and I-P1-004.
"""

from __future__ import annotations

# Accumulation — PRD §3.4.7
BASE_RATE: float = 0.01                   # C_ctx grows by this per qualifying tick before asymptote
MAX_ACCRUAL_PER_MINUTE: float = 0.005     # time-domain bound per minute
MAX_ACCRUAL_PER_SESSION: float = 0.05     # per-session ceiling — I-P1-003
CALENDAR_DAY_BONUS: float = 0.02          # extra accrual per distinct calendar day

# Decay — PRD §3.4.7
DECAY_RATE: float = 0.20                  # proportional decay per unit tension; 20× BASE_RATE → 20:1 asymmetry

# Earned floor — PRD §3.4.7
FLOOR_RATE: float = 0.005                 # floor grows by this per interaction
MAX_FLOOR: float = 0.70                   # floor clamped here — I-P1-004

# Termination — PRD §3.4.8 (consumed by P3 #110 session termination)
ACUTE_DISTRESS_FLOOR: float = 0.15        # C_inst below this triggers termination
COOLDOWN_MINUTES: int = 30                # cooldown window on termination

# Bounds — used throughout
COHERENCE_MIN: float = 0.0
COHERENCE_MAX: float = 1.0
