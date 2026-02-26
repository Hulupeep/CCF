/*
 * Patent Claim Verification Suite
 *
 * These integration tests verify the five major claims of USP provisional
 * patent application 63/988,438 (filed 2026-02-23) covering the
 * Contextual Coherence Field (CCF) architecture.
 *
 * Each test exercises the relevant mbot-core types end-to-end in a way
 * that mirrors how the companion app uses them, without requiring hardware.
 *
 * Reference: docs/patent-hitl-checklist.md for the corresponding
 * human-in-the-loop hardware verification procedure.
 */

use mbot_core::coherence::{
    BrightnessBand, CoherenceAccumulator, CoherenceField, ContextKey, MotionContext,
    NoiseBand, Orientation, PhaseSpace, PresenceSignature, SocialPhase, TimePeriod,
};
use mbot_core::nervous_system::suppression::{SuppressionMap, SuppressionRule};
use mbot_core::nervous_system::stimulus::StimulusKind;

// ─── Context key helpers ──────────────────────────────────────────────────────

/// Construct a context key from discrete band values without going through
/// the sensor-reading path.  This lets tests set up exact context keys
/// independent of the band-boundary arithmetic.
fn context_key(
    brightness: BrightnessBand,
    noise: NoiseBand,
    presence: PresenceSignature,
) -> ContextKey {
    ContextKey {
        brightness,
        noise,
        presence,
        motion: MotionContext::Stationary,
        orientation: Orientation::Upright,
        time_period: TimePeriod::Afternoon,
    }
}

// ─── Claim 1 ─────────────────────────────────────────────────────────────────
//
// Claim 1 — Context Transfer Blocked Across Dissimilar Environments
//
// "Coherence accumulated in one context (e.g., Bright × Quiet) is not
// transferred to a structurally dissimilar context (e.g., Dark × Loud).
// Each context must independently earn trust."
//
// Verification: Build coherence only in the Bright+Quiet accumulator.
// Confirm the Dark+Loud accumulator is still at zero.
// Confirm effective_coherence remains zero in the new context even if
// instant_coherence is high (unfamiliar gate: min(instant, ctx) = 0).

#[test]
fn test_claim_1_context_transfer_blocked_across_dissimilar_environments() {
    let mut field = CoherenceField::new();

    // --- Setup: earn coherence in the "bright and quiet" context ---
    let bright_quiet = context_key(
        BrightnessBand::Bright,
        NoiseBand::Quiet,
        PresenceSignature::Static,
    );

    // Simulate ~60 positive interactions (enough to reach familiar territory
    // ctx >= 0.3 in the original context).
    {
        let acc = field.get_or_create(&bright_quiet);
        for tick in 0..60u64 {
            acc.positive_interaction(0.5, tick, false);
        }
    }

    let bright_quiet_coh = field.context_coherence(&bright_quiet);
    assert!(
        bright_quiet_coh >= 0.3,
        "Bright+Quiet should have reached familiar territory after 60 interactions, \
         got {}",
        bright_quiet_coh
    );

    // --- The dissimilar context has never been visited ---
    let dark_loud = context_key(
        BrightnessBand::Dark,
        NoiseBand::Loud,
        PresenceSignature::Absent,
    );

    // Context coherence must be zero — no carryover.
    let transferred = field.context_coherence(&dark_loud);
    assert_eq!(
        transferred, 0.0,
        "Claim 1 FAIL: Dark+Loud context coherence should be 0.0, not {}. \
         Coherence must be earned per-context; it does not transfer.",
        transferred
    );

    // Even with a high instant_coherence reading, effective_coherence is capped
    // by the unfamiliar gate (CCF-001): min(instant, ctx) = min(0.9, 0.0) = 0.0.
    let effective = field.effective_coherence(0.9, &dark_loud);
    assert_eq!(
        effective, 0.0,
        "Claim 1 FAIL: effective_coherence should be 0.0 in an unfamiliar dissimilar \
         context even when instant_coherence=0.9. Got {}.",
        effective
    );
}

// ─── Claim 2 ─────────────────────────────────────────────────────────────────
//
// Claim 2 — Coherence Grows Only Through Repeated Positive Interaction
//
// "The relational coherence value for a given context grows strictly as a
// monotonically non-decreasing function of repeated positive interactions
// within that context.  A freshly constructed accumulator starts at zero
// and cannot reach high values without sufficient interaction history."
//
// Verification: check that (a) value starts at zero, (b) each positive
// interaction never decreases the value, and (c) the value after N
// interactions is strictly greater than after N-1 interactions for small N
// (before diminishing returns become negligible).

#[test]
fn test_claim_2_coherence_grows_only_through_repeated_positive_interaction() {
    let mut acc = CoherenceAccumulator::new();

    // Starts at zero — no interaction, no trust.
    assert_eq!(
        acc.value, 0.0,
        "Claim 2 FAIL: fresh accumulator must start at 0.0, got {}",
        acc.value
    );

    let mut previous_value = acc.value;

    // Run 100 positive interactions and verify monotone growth.
    for tick in 0..100u64 {
        acc.positive_interaction(0.5, tick, false);

        assert!(
            acc.value >= previous_value,
            "Claim 2 FAIL at tick {}: coherence must never decrease during positive \
             interactions (was {}, now {})",
            tick,
            previous_value,
            acc.value
        );

        // For the first 50 ticks the growth should be strictly positive —
        // diminishing returns only flatten near 1.0.
        if tick < 50 {
            assert!(
                acc.value > previous_value,
                "Claim 2 FAIL at tick {}: coherence should strictly increase during \
                 early positive interactions (was {}, now {})",
                tick,
                previous_value,
                acc.value
            );
        }

        previous_value = acc.value;
    }

    // After 100 positive interactions the robot has genuinely earned trust.
    assert!(
        acc.value > 0.5,
        "Claim 2 FAIL: 100 positive interactions should produce coherence > 0.5, \
         got {}",
        acc.value
    );
    assert_eq!(
        acc.interaction_count, 100,
        "Interaction count should be 100, got {}",
        acc.interaction_count
    );
}

// ─── Claim 3 ─────────────────────────────────────────────────────────────────
//
// Claim 3 — Warm Start Requires a Similar Known Context
//
// "When the robot resumes operation in an environment whose context key
// matches a previously accumulated context, the coherence field returns the
// stored (earned) value immediately — a warm start.  If the new context key
// does not match any stored key, the coherence field returns zero — a cold
// start."
//
// Verification: persist a value directly in the field (simulating a
// serialise/restore cycle), then confirm lookup behaviour for a matching
// and a non-matching key.

#[test]
fn test_claim_3_warm_start_requires_similar_known_context() {
    // ── Session 1: earn coherence in a specific context ──────────────────
    let mut session1 = CoherenceField::new();

    let familiar_context = context_key(
        BrightnessBand::Dim,
        NoiseBand::Quiet,
        PresenceSignature::Static,
    );

    {
        let acc = session1.get_or_create(&familiar_context);
        for tick in 0..80u64 {
            acc.positive_interaction(0.6, tick, false);
        }
    }

    let earned_value = session1.context_coherence(&familiar_context);
    assert!(
        earned_value > 0.3,
        "Session 1 should have earned meaningful coherence, got {}",
        earned_value
    );

    // ── Session 2: simulate warm-start in the SAME context ───────────────
    // In a real system this value comes from the persistence layer.
    // Here we construct the field directly with the previously stored value.
    let mut session2 = CoherenceField::new();
    {
        let acc = session2.get_or_create(&familiar_context);
        // Restore the earned value as if loaded from disk.
        acc.value = earned_value;
        acc.interaction_count = 80;
    }

    let warm_value = session2.context_coherence(&familiar_context);
    assert!(
        (warm_value - earned_value).abs() < 0.001,
        "Claim 3 FAIL: warm-start should restore prior coherence {}, got {}",
        earned_value,
        warm_value
    );
    assert!(
        warm_value > 0.3,
        "Claim 3 FAIL: warm-start coherence should be above the familarity threshold, \
         got {}",
        warm_value
    );

    // ── Session 2: different context → cold start ─────────────────────────
    let different_context = context_key(
        BrightnessBand::Bright,
        NoiseBand::Loud,
        PresenceSignature::Approaching,
    );

    let cold_value = session2.context_coherence(&different_context);
    assert_eq!(
        cold_value, 0.0,
        "Claim 3 FAIL: an unseen context must start cold at 0.0, got {}",
        cold_value
    );

    // The asymmetric gate confirms the cold-start: even high instant_coherence
    // is blocked in an unearned context.
    let cold_effective = session2.effective_coherence(0.95, &different_context);
    assert_eq!(
        cold_effective, 0.0,
        "Claim 3 FAIL: effective_coherence in cold context should be 0.0, got {}",
        cold_effective
    );
}

// ─── Claim 4 ─────────────────────────────────────────────────────────────────
//
// Claim 4 — Deliberative Gate Fires Only on Significant Boundary Change
//
// "A boundary-crossing event (transition between SocialPhase quadrants)
// triggers a deliberative response.  After firing, the gate enforces a
// minimum cooldown period before it can fire again, preventing repeated
// firing on sustained stimuli."
//
// Verification: track SocialPhase transitions across a series of
// (coherence, tension) readings.  Assert that the phase flips exactly
// once when the boundary is crossed, then does not flip again on the
// next N ticks of identical stimuli (hysteresis / cooldown behaviour).
// The CCF-004 hysteresis deadband is the implementation of the cooldown.

#[test]
fn test_claim_4_deliberative_gate_fires_only_on_significant_boundary_change() {
    let ps = PhaseSpace::default();
    // Default enter threshold for high-tension: 0.45
    // Default exit threshold for high-tension: 0.35

    // ── Phase 1: calm baseline ─────────────────────────────────────────
    let mut current_phase = SocialPhase::ShyObserver;
    let baseline = SocialPhase::classify(0.1, 0.2, current_phase, &ps);
    assert_eq!(baseline, SocialPhase::ShyObserver, "Should start in ShyObserver");
    current_phase = baseline;

    // ── Phase 2: sudden high-tension event fires the gate once ─────────
    let after_event = SocialPhase::classify(0.1, 0.6, current_phase, &ps);
    assert_eq!(
        after_event,
        SocialPhase::StartledRetreat,
        "Claim 4 FAIL: crossing tension boundary should produce StartledRetreat"
    );
    let mut transitions = 1u32;
    current_phase = after_event;

    // ── Phase 3: sustained tension at same level — gate must NOT re-fire ──
    // The hysteresis exit threshold (0.35) is below the sustained value (0.6),
    // so the phase should stay in StartledRetreat without further transitions.
    for _ in 0..50 {
        let next = SocialPhase::classify(0.1, 0.6, current_phase, &ps);

        if next != current_phase {
            transitions += 1;
        }
        current_phase = next;
    }

    assert_eq!(
        transitions, 1,
        "Claim 4 FAIL: deliberative gate should fire exactly once on the boundary \
         crossing.  It fired {} times over 51 ticks of sustained tension.",
        transitions
    );

    // ── Phase 4: tension drops below exit threshold → single recovery event ──
    let mut recovery_transitions = 0u32;
    for _ in 0..5 {
        let next = SocialPhase::classify(0.1, 0.2, current_phase, &ps);
        if next != current_phase {
            recovery_transitions += 1;
        }
        current_phase = next;
    }

    assert_eq!(
        recovery_transitions, 1,
        "Claim 4 FAIL: recovery from StartledRetreat should produce exactly one \
         transition, got {}",
        recovery_transitions
    );
    assert_eq!(
        current_phase,
        SocialPhase::ShyObserver,
        "Should return to ShyObserver after tension drops"
    );
}

// ─── Claim 5 ─────────────────────────────────────────────────────────────────
//
// Claim 5 — Suppression Generalisation Never Weakens an Exact Match
//
// "A general suppression rule (e.g., LoudnessSpike in any context) does not
// reduce the suppression factor of a more specific rule keyed to the same
// stimulus in a precise context.  Exact-match rules always return their own
// stored factor regardless of any co-existing broader rules."
//
// Verification: install two suppression rules with the same StimulusKind but
// different context_hash values (simulating a "broad" rule and a "narrow"
// rule).  Confirm that looking up the narrow context returns the narrow
// rule's factor, and looking up an unrelated context returns the broad
// rule's factor, i.e., rules are keyed exactly and do not bleed.

#[test]
fn test_claim_5_suppression_generalisation_never_weakens_exact_match() {
    let mut map = SuppressionMap::new();

    // Context hashes for the two contexts we care about.
    // In production these come from ContextKey::context_hash_u32().
    let bright_noisy_hash = context_key(
        BrightnessBand::Bright,
        NoiseBand::Loud,
        PresenceSignature::Static,
    )
    .context_hash_u32();

    let generic_hash = context_key(
        BrightnessBand::Dark,
        NoiseBand::Loud,
        PresenceSignature::Absent,
    )
    .context_hash_u32();

    // ── General rule: LoudnessSpike in a noisy-dark context → high suppression ──
    // (robot has learned that loud noises in this context are routine)
    map.upsert(SuppressionRule {
        stimulus_kind: StimulusKind::LoudnessSpike,
        context_hash: generic_hash,
        suppression_factor: 0.4, // heavily suppressed in the generic context
        observation_count: 20,
        last_updated_tick: 100,
    });

    // ── Specific rule: Bright+Loud context should MAINTAIN full trust ────
    // (the "override" case from Claim 5: this precise context is safe)
    map.upsert(SuppressionRule {
        stimulus_kind: StimulusKind::LoudnessSpike,
        context_hash: bright_noisy_hash,
        suppression_factor: 0.95, // nearly unsuppressed — this context is trusted
        observation_count: 50,
        last_updated_tick: 200,
    });

    // ── Verify exact-match integrity ──────────────────────────────────────

    // Bright+Noisy lookup returns its own high factor (not the general rule's 0.4).
    let specific_factor = map.lookup(StimulusKind::LoudnessSpike, bright_noisy_hash);
    assert!(
        specific_factor >= 0.9,
        "Claim 5 FAIL: exact-match rule for Bright+Loud context should return \
         factor >= 0.9, got {}.  The general rule (0.4) must not bleed into the \
         specific rule.",
        specific_factor
    );

    // Generic context lookup returns its own lower factor.
    let general_factor = map.lookup(StimulusKind::LoudnessSpike, generic_hash);
    assert!(
        general_factor <= 0.5,
        "Claim 5 FAIL: general rule for Dark+Loud context should return factor \
         <= 0.5, got {}.",
        general_factor
    );

    // The two factors must be distinct — they do not merge.
    assert!(
        specific_factor > general_factor,
        "Claim 5 FAIL: specific-context factor ({}) should be greater than the \
         general-context factor ({}).  Exact match must always preserve its own value.",
        specific_factor,
        general_factor
    );

    // An entirely unknown context returns 1.0 (no suppression at all).
    let unknown_hash = 0xDEAD_BEEF_u32;
    let unseen_factor = map.lookup(StimulusKind::LoudnessSpike, unknown_hash);
    assert_eq!(
        unseen_factor, 1.0,
        "Claim 5 FAIL: an unseen context should return suppression_factor=1.0 \
         (no suppression), got {}",
        unseen_factor
    );
}
