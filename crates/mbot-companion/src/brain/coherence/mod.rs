//! Comfort-zone boundary computation using the context-key similarity graph.
//!
//! # Invariants
//! - **I-BNDRY-001**: Min-cut is on the context-key similarity graph (not the episode graph).
//! - **I-BNDRY-002**: Edge weight = cosine similarity of 6-dim feature vectors.
//! - **I-BNDRY-003**: Edges inserted only above threshold (0.1).
//! - **I-BNDRY-004**: Stoer-Wagner is NOT used. Uses SubpolynomialMinCut (arXiv:2512.13105).
//! - **I-DLIB-001**: Deliberative fires at most once per `DELIBERATIVE_COOLDOWN` ticks.
//! - **I-DLIB-002**: |Δmin_cut| < `DELIBERATIVE_DELTA` never triggers deliberative.

pub mod boundary;
pub mod warm_start;
pub use boundary::ComfortZoneBoundary;
pub use warm_start::ContextIndex;

/// Minimum absolute change in min-cut value that triggers the deliberative layer (I-DLIB-002).
pub const DELIBERATIVE_DELTA: f32 = 0.05;

/// Minimum ticks between deliberative triggers (I-DLIB-001).
pub const DELIBERATIVE_COOLDOWN: u64 = 50;

/// Pure gate function: returns true when the deliberative layer should fire.
///
/// Fires when the topology has changed meaningfully (`|curr - prev| >= delta_threshold`)
/// AND the cooldown has elapsed (`ticks_since_last >= cooldown`).
///
/// I-DLIB-003: this function never blocks; callers must fire the deliberative layer
/// asynchronously so the real-time tick is not held.
///
/// # Returns
/// `("contracted", true)` when min-cut dropped, `("expanded", true)` when it rose.
/// `("", false)` when the gate does not fire.
pub fn should_fire_deliberative(
    curr: f32,
    prev: f32,
    delta_threshold: f32,
    cooldown: u64,
    ticks_since_last: u64,
) -> (bool, &'static str) {
    let delta = curr - prev;
    if delta.abs() >= delta_threshold && ticks_since_last >= cooldown {
        if delta < 0.0 {
            (true, "contracted")
        } else {
            (true, "expanded")
        }
    } else {
        (false, "")
    }
}

#[cfg(test)]
mod gate_tests {
    use super::*;

    #[test]
    fn stable_min_cut_does_not_fire() {
        let (fire, _) = should_fire_deliberative(0.43, 0.42, DELIBERATIVE_DELTA, DELIBERATIVE_COOLDOWN, 100);
        assert!(!fire, "delta 0.01 should not trigger (below 0.05)");
    }

    #[test]
    fn large_drop_fires_contracted() {
        let (fire, reason) = should_fire_deliberative(0.30, 0.42, DELIBERATIVE_DELTA, DELIBERATIVE_COOLDOWN, 100);
        assert!(fire);
        assert_eq!(reason, "contracted");
    }

    #[test]
    fn large_rise_fires_expanded() {
        let (fire, reason) = should_fire_deliberative(0.45, 0.30, DELIBERATIVE_DELTA, DELIBERATIVE_COOLDOWN, 100);
        assert!(fire);
        assert_eq!(reason, "expanded");
    }

    #[test]
    fn within_cooldown_does_not_fire() {
        let (fire, _) = should_fire_deliberative(0.30, 0.42, DELIBERATIVE_DELTA, DELIBERATIVE_COOLDOWN, 10);
        assert!(!fire, "should not fire within cooldown window");
    }

    #[test]
    fn exact_threshold_fires() {
        // 0.25 is exactly representable; 0.30 - 0.25 = 0.05000001... >= 0.05 in f32.
        let (fire, _) = should_fire_deliberative(0.30, 0.25, DELIBERATIVE_DELTA, DELIBERATIVE_COOLDOWN, 50);
        assert!(fire, "exactly at threshold (0.05) should fire");
    }

    #[test]
    fn exact_cooldown_fires() {
        let (fire, _) = should_fire_deliberative(0.30, 0.42, DELIBERATIVE_DELTA, DELIBERATIVE_COOLDOWN, 50);
        assert!(fire, "exactly at cooldown (50 ticks) should fire");
    }
}
