//! E2E Tests for Personality Switching (STORY-PERS-004)
//!
//! These tests verify the complete personality switching workflow including:
//! - Runtime personality changes
//! - Smooth transitions
//! - LED animations
//! - Mid-action switches
//! - Rapid switching stress tests

use mbot_core::personality::{
    PersonalityPreset, PersonalitySwitcher, TransitionConfig, TransitionEvent, Easing,
};

/// Test: Switch personality at runtime
///
/// Gherkin: Given the robot is running as "Chill Charlie"
///          When the user selects "Nervous Nellie"
///          Then the personality should transition over 3 seconds
///          And behaviors should gradually shift to match
#[test]
fn test_switch_personality_at_runtime() {
    // Given: Robot is running as Mellow (chill)
    let mellow = PersonalityPreset::Mellow.to_personality();
    let mut switcher = PersonalitySwitcher::new(mellow.clone());

    assert_eq!(switcher.get_current().name, mellow.name);
    assert!(!switcher.is_transitioning());

    // When: User selects Anxious (nervous)
    let anxious = PersonalityPreset::Anxious.to_personality();
    let config = TransitionConfig {
        duration_ms: 3000,
        easing: Easing::EaseInOut,
        led_animation: true,
    };

    let event = switcher.switch_to(anxious.clone(), config);

    // Then: Transition should start
    match event {
        TransitionEvent::Started { from, to } => {
            assert_eq!(from, mellow.name);
            assert_eq!(to, anxious.name);
        }
        _ => panic!("Expected Started event"),
    }

    assert!(switcher.is_transitioning());

    // And: Behaviors should gradually shift
    let initial_tension = switcher.current_influence().tension_target;
    let initial_stimulus = switcher.current_influence().stimulus_multiplier;

    // Simulate 3 seconds at 60fps
    for _ in 0..187 {  // 187 * 16ms ≈ 3000ms
        switcher.update(16);
    }

    // Transition should complete
    assert!(!switcher.is_transitioning());
    assert_eq!(switcher.get_current().name, anxious.name);

    // Final values should match anxious personality
    let final_tension = switcher.current_influence().tension_target;
    let final_stimulus = switcher.current_influence().stimulus_multiplier;

    assert!(final_tension > initial_tension, "Tension should increase (mellow->anxious)");
    assert!(final_stimulus > initial_stimulus, "Stimulus sensitivity should increase");
    assert!((final_tension - anxious.tension_baseline()).abs() < 0.05);
}

/// Test: Transition animation shows on LEDs
///
/// Gherkin: Given the robot is running as "Curious Cleo"
///          When switching to "Bouncy Betty"
///          Then LEDs should show transition animation
///          And animation should reflect both personalities
#[test]
fn test_transition_animation_on_leds() {
    // Given: Robot is Curious
    let curious = PersonalityPreset::Curious.to_personality();
    let mut switcher = PersonalitySwitcher::new(curious);

    // When: Switching to Playful (bouncy)
    let playful = PersonalityPreset::Playful.to_personality();
    let config = TransitionConfig {
        duration_ms: 2000,
        easing: Easing::Linear,
        led_animation: true,
    };

    switcher.switch_to(playful, config);

    // Then: LEDs should show transition animation
    let mut colors_seen = Vec::new();

    for _ in 0..125 {  // 125 * 16ms = 2000ms
        switcher.update(16);
        let color = switcher.transition_led_color();
        colors_seen.push(color);
    }

    // And: Animation should change over time (not static)
    assert!(colors_seen.len() > 10);

    // Colors should vary during transition
    let first_color = colors_seen[0];
    let mid_color = colors_seen[colors_seen.len() / 2];
    let last_color = colors_seen[colors_seen.len() - 1];

    assert!(
        first_color != mid_color || mid_color != last_color,
        "LED colors should change during transition"
    );

    // After transition, LED should return to default (not transitioning)
    switcher.update(16);
    let post_transition_color = switcher.transition_led_color();
    assert_eq!(post_transition_color, [0, 100, 255], "Should return to default blue");
}

/// Test: Mid-action switch handled gracefully
///
/// Gherkin: Given the robot is investigating an object
///          When personality switches to "Nervous Nellie"
///          Then the current action should complete or abort safely
///          And new personality should take effect
///          And no crashes or errors should occur
#[test]
fn test_mid_action_switch_handled_gracefully() {
    // Given: Robot is in Curious mode (investigating)
    let curious = PersonalityPreset::Curious.to_personality();
    let mut switcher = PersonalitySwitcher::new(curious);

    // Start a transition to Adventurous
    let adventurous = PersonalityPreset::Adventurous.to_personality();
    let config = TransitionConfig {
        duration_ms: 1000,
        ..Default::default()
    };
    switcher.switch_to(adventurous, config.clone());

    // Advance partway (simulating robot in middle of action)
    for _ in 0..30 {  // ~480ms - robot is mid-investigation
        switcher.update(16);
    }

    // When: Personality switches to Anxious mid-action
    let anxious = PersonalityPreset::Anxious.to_personality();
    switcher.switch_to(anxious.clone(), config);

    // Then: No crashes should occur
    assert!(switcher.is_transitioning());

    // New personality should take effect
    for _ in 0..70 {
        switcher.update(16);
    }

    assert!(!switcher.is_transitioning());
    assert_eq!(switcher.get_current().name, anxious.name);

    // Influence should be valid
    let influence = switcher.current_influence();
    assert!(influence.tension_target >= 0.0 && influence.tension_target <= 1.0);
    assert!(influence.movement_scale >= 0.0 && influence.movement_scale <= 1.0);
}

/// Test: Rapid switching stress test
///
/// Gherkin: Given the robot is running normally
///          When personalities are switched 10 times in 5 seconds
///          Then no crashes should occur
///          And final personality should be active
///          And system should remain stable
#[test]
fn test_rapid_switching_stress_test() {
    let personalities = [
        PersonalityPreset::Mellow.to_personality(),
        PersonalityPreset::Excitable.to_personality(),
        PersonalityPreset::Zen.to_personality(),
        PersonalityPreset::Curious.to_personality(),
        PersonalityPreset::Timid.to_personality(),
        PersonalityPreset::Adventurous.to_personality(),
        PersonalityPreset::Cheerful.to_personality(),
        PersonalityPreset::Grumpy.to_personality(),
        PersonalityPreset::Playful.to_personality(),
        PersonalityPreset::Anxious.to_personality(),
    ];

    // Given: Robot is running normally
    let mut switcher = PersonalitySwitcher::new(personalities[0].clone());

    let config = TransitionConfig {
        duration_ms: 500,
        easing: Easing::Linear,
        led_animation: true,
    };

    // When: Personalities are switched 10 times in 5 seconds
    for i in 0..10 {
        let target = personalities[i].clone();
        switcher.switch_to(target, config.clone());

        // Advance 50ms per switch (rapid switching - don't let transitions complete)
        for _ in 0..3 {  // 3 * 16ms ≈ 48ms (very rapid)
            switcher.update(16);

            // System should remain stable throughout
            let influence = switcher.current_influence();
            assert!(influence.tension_target >= 0.0 && influence.tension_target <= 1.0);
            assert!(influence.coherence_target >= 0.0 && influence.coherence_target <= 1.0);
            assert!(influence.energy_target >= 0.0 && influence.energy_target <= 1.0);
        }
    }

    // Complete the final transition
    for _ in 0..35 {
        switcher.update(16);
    }

    // Then: No crashes should occur (implicit - test passed)
    // And: Final personality should be active
    assert_eq!(switcher.get_current().name, personalities[9].name);

    // System should remain stable
    let final_influence = switcher.current_influence();
    assert!(final_influence.tension_target >= 0.0 && final_influence.tension_target <= 1.0);
    assert!(final_influence.stimulus_multiplier > 0.0 && final_influence.stimulus_multiplier < 2.0);
}

/// Test: Cancel transition mid-switch
///
/// Gherkin: Given a transition is in progress
///          When the transition is cancelled
///          Then the robot should remain at current interpolated state
///          And no jarring changes should occur
#[test]
fn test_cancel_transition_mid_switch() {
    // Given: A transition is in progress
    let mellow = PersonalityPreset::Mellow.to_personality();
    let excitable = PersonalityPreset::Excitable.to_personality();

    let mut switcher = PersonalitySwitcher::new(mellow);

    let config = TransitionConfig {
        duration_ms: 2000,
        ..Default::default()
    };

    switcher.switch_to(excitable, config);

    // Advance to midpoint
    for _ in 0..62 {  // ~992ms - about halfway
        switcher.update(16);
    }

    let progress_before = switcher.get_progress();
    let tension_before = switcher.current_influence().tension_target;

    assert!(switcher.is_transitioning());
    assert!(progress_before > 0.4 && progress_before < 0.6);

    // When: The transition is cancelled
    let event = switcher.cancel_transition();

    match event {
        TransitionEvent::Cancelled { at_progress } => {
            assert!((at_progress - progress_before).abs() < 0.05);
        }
        _ => panic!("Expected Cancelled event"),
    }

    // Then: Robot should remain at current interpolated state
    assert!(!switcher.is_transitioning());

    let tension_after = switcher.current_influence().tension_target;

    // And: No jarring changes should occur
    assert!(
        (tension_after - tension_before).abs() < 0.01,
        "Tension jumped: {} -> {}",
        tension_before,
        tension_after
    );

    // Continuing to update should not change values
    for _ in 0..10 {
        switcher.update(16);
    }

    let tension_stable = switcher.current_influence().tension_target;
    assert!(
        (tension_stable - tension_after).abs() < 0.01,
        "Values should remain stable after cancel"
    );
}

/// Test: Immediate parameter response during transition
///
/// Gherkin: Given switching from tension_baseline 0.2 to 0.8
///          When 50% through the transition
///          Then effective tension_baseline should be near 0.5
///          And behavior should reflect intermediate state
#[test]
fn test_immediate_parameter_response_during_transition() {
    // Given: Switching from tension_baseline 0.2 (Mellow) to 0.8 (Anxious)
    let mellow = PersonalityPreset::Mellow.to_personality();
    let anxious = PersonalityPreset::Anxious.to_personality();

    let mellow_baseline = mellow.tension_baseline();
    let anxious_baseline = anxious.tension_baseline();

    assert!((mellow_baseline - 0.2).abs() < 0.01);
    assert!((anxious_baseline - 0.8).abs() < 0.01);

    let mut switcher = PersonalitySwitcher::new(mellow);

    let config = TransitionConfig {
        duration_ms: 1000,
        easing: Easing::Linear,  // Linear for predictable midpoint
        led_animation: false,
    };

    switcher.switch_to(anxious, config);

    // When: 50% through the transition
    for _ in 0..31 {  // 31 * 16ms ≈ 496ms (close to 50%)
        switcher.update(16);
    }

    let progress = switcher.get_progress();
    let tension = switcher.current_influence().tension_target;

    // Then: Effective tension_baseline should be near 0.5
    assert!(progress > 0.45 && progress < 0.55, "Progress: {}", progress);
    assert!(
        (tension - 0.5).abs() < 0.15,
        "Tension should be near 0.5, got {}",
        tension
    );

    // And: Behavior should reflect intermediate state
    // (tension is between mellow and anxious)
    assert!(tension > mellow_baseline);
    assert!(tension < anxious_baseline);
}

/// Test: Multiple complete transitions in sequence
#[test]
fn test_multiple_complete_transitions_in_sequence() {
    let personalities = [
        PersonalityPreset::Mellow.to_personality(),
        PersonalityPreset::Curious.to_personality(),
        PersonalityPreset::Zen.to_personality(),
    ];

    let mut switcher = PersonalitySwitcher::new(personalities[0].clone());

    let config = TransitionConfig {
        duration_ms: 500,
        ..Default::default()
    };

    // Complete first transition
    switcher.switch_to(personalities[1].clone(), config.clone());
    for _ in 0..35 {
        switcher.update(16);
    }
    assert!(!switcher.is_transitioning());
    assert_eq!(switcher.get_current().name, personalities[1].name);

    // Complete second transition
    switcher.switch_to(personalities[2].clone(), config);
    for _ in 0..35 {
        switcher.update(16);
    }
    assert!(!switcher.is_transitioning());
    assert_eq!(switcher.get_current().name, personalities[2].name);
}

/// Test: Transition with different easing functions
#[test]
fn test_transition_with_different_easings() {
    let mellow = PersonalityPreset::Mellow.to_personality();
    let excitable = PersonalityPreset::Excitable.to_personality();

    // Test each easing function
    for easing in [Easing::Linear, Easing::EaseIn, Easing::EaseOut, Easing::EaseInOut] {
        let mut switcher = PersonalitySwitcher::new(mellow.clone());

        let config = TransitionConfig {
            duration_ms: 1000,
            easing,
            led_animation: false,
        };

        switcher.switch_to(excitable.clone(), config);

        // Complete transition
        for _ in 0..65 {
            switcher.update(16);
        }

        // Should complete successfully regardless of easing
        assert!(!switcher.is_transitioning());
        assert_eq!(switcher.get_current().name, excitable.name);
    }
}

/// Test: Set immediate during transition
#[test]
fn test_set_immediate_during_transition() {
    let mellow = PersonalityPreset::Mellow.to_personality();
    let excitable = PersonalityPreset::Excitable.to_personality();
    let zen = PersonalityPreset::Zen.to_personality();

    let mut switcher = PersonalitySwitcher::new(mellow);

    let config = TransitionConfig::default();
    switcher.switch_to(excitable, config);

    // Advance partway
    for _ in 0..30 {
        switcher.update(16);
    }

    assert!(switcher.is_transitioning());

    // Set immediate should cancel transition and apply zen instantly
    switcher.set_immediate(zen.clone());

    assert!(!switcher.is_transitioning());
    assert_eq!(switcher.get_current().name, zen.name);

    // Values should match zen exactly
    let tension = switcher.current_influence().tension_target;
    assert!((tension - zen.tension_baseline()).abs() < 0.01);
}

/// Test: Transition duration edge cases
#[test]
fn test_transition_duration_edge_cases() {
    let mellow = PersonalityPreset::Mellow.to_personality();
    let excitable = PersonalityPreset::Excitable.to_personality();

    // Very short duration (100ms)
    let mut switcher = PersonalitySwitcher::new(mellow.clone());
    let config = TransitionConfig {
        duration_ms: 100,
        ..Default::default()
    };
    switcher.switch_to(excitable.clone(), config);

    for _ in 0..10 {  // 10 * 16ms = 160ms
        switcher.update(16);
    }
    assert!(!switcher.is_transitioning());

    // Very long duration (10 seconds)
    let mut switcher = PersonalitySwitcher::new(mellow.clone());
    let config = TransitionConfig {
        duration_ms: 10000,
        ..Default::default()
    };
    switcher.switch_to(excitable, config);

    // After 5 seconds, should still be transitioning
    for _ in 0..312 {  // 312 * 16ms ≈ 5000ms
        switcher.update(16);
    }
    assert!(switcher.is_transitioning());

    // Complete the rest
    for _ in 0..320 {
        switcher.update(16);
    }
    assert!(!switcher.is_transitioning());
}

/// Test: All preset personalities can transition to each other
#[test]
fn test_all_presets_can_transition() {
    let presets = PersonalityPreset::all();

    for (i, from_preset) in presets.iter().enumerate() {
        for (j, to_preset) in presets.iter().enumerate() {
            if i == j {
                continue; // Skip self-transitions
            }

            let from = from_preset.to_personality();
            let to = to_preset.to_personality();

            let mut switcher = PersonalitySwitcher::new(from.clone());

            let config = TransitionConfig {
                duration_ms: 100,
                ..Default::default()
            };

            switcher.switch_to(to.clone(), config);

            // Complete transition
            for _ in 0..10 {
                switcher.update(16);
            }

            assert!(!switcher.is_transitioning());
            assert_eq!(switcher.get_current().name, to.name);
        }
    }
}
