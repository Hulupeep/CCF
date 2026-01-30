//! Integration tests for Personality-to-Behavior Mapping
//!
//! These tests verify that personality parameters correctly influence
//! the robot's nervous system and produce observable behavior differences.

use mbot_core::{
    MBotBrain, MBotSensors, HomeostasisState,
    personality::{
        Personality, PersonalityPreset, PersonalityMapper,
        apply_tension_influence, apply_curiosity_influence,
        scale_motor_output, scale_light_output,
    },
};

/// Helper to create sensors with specific values
fn sensors_with_distance(distance_cm: f32) -> MBotSensors {
    MBotSensors {
        ultrasonic_cm: distance_cm,
        sound_level: 0.0,
        light_level: 0.5,
        ..Default::default()
    }
}

#[test]
fn test_different_personalities_produce_different_behaviors() {
    // Scenario: Two robots with different personalities react to same stimulus
    let mellow = PersonalityPreset::Mellow.to_personality();
    let excitable = PersonalityPreset::Excitable.to_personality();

    let mellow_mapper = PersonalityMapper::with_personality(&mellow);
    let excitable_mapper = PersonalityMapper::with_personality(&excitable);

    let mellow_influence = mellow_mapper.current_influence();
    let excitable_influence = excitable_mapper.current_influence();

    // Same stimulus should be processed differently
    let raw_tension = 0.6;
    let current_tension = 0.3;

    let mellow_result = apply_tension_influence(
        raw_tension,
        current_tension,
        mellow_influence,
        0.01,
    );

    let excitable_result = apply_tension_influence(
        raw_tension,
        current_tension,
        excitable_influence,
        0.01,
    );

    // Excitable should have higher tension due to higher startle_sensitivity
    assert!(
        excitable_result > mellow_result,
        "Excitable should react more strongly: excitable={} mellow={}",
        excitable_result,
        mellow_result
    );

    // Check motor output scaling
    let motors = (80, 80);
    let mellow_motors = scale_motor_output(motors.0, motors.1, mellow_influence);
    let excitable_motors = scale_motor_output(motors.0, motors.1, excitable_influence);

    // Excitable has higher movement_expressiveness, so should have stronger output
    assert!(
        excitable_motors.0 > mellow_motors.0,
        "Excitable should move more expressively"
    );
}

#[test]
fn test_personality_affects_baseline_mood() {
    // Scenario: Robot with Nervous Nellie personality (high tension_baseline)
    let mut nervous = Personality::default();
    nervous.set_tension_baseline(0.7).unwrap();

    let mapper = PersonalityMapper::with_personality(&nervous);
    let influence = mapper.current_influence();

    // When idle with no stimulus, tension should gravitate toward 0.7
    let current_tension = 0.3;
    let no_stimulus = 0.0;

    let result = apply_tension_influence(
        no_stimulus,
        current_tension,
        influence,
        0.1, // Strong gravity
    );

    // Should be pulled upward toward 0.7
    assert!(result > current_tension, "Tension should increase toward baseline");
}

#[test]
fn test_high_curiosity_increases_investigation_behavior() {
    // Scenario: High curiosity personality is more interested in stimuli
    let mut curious = Personality::default();
    curious.set_curiosity_drive(0.9).unwrap();

    let mut boring = Personality::default();
    boring.set_curiosity_drive(0.1).unwrap();

    let curious_mapper = PersonalityMapper::with_personality(&curious);
    let boring_mapper = PersonalityMapper::with_personality(&boring);

    let raw_curiosity = 0.5;

    let curious_result = apply_curiosity_influence(
        raw_curiosity,
        curious_mapper.current_influence(),
    );

    let boring_result = apply_curiosity_influence(
        raw_curiosity,
        boring_mapper.current_influence(),
    );

    assert!(
        curious_result > boring_result,
        "High curiosity personality should be more interested in novelty"
    );
}

#[test]
fn test_low_energy_baseline_reduces_activity() {
    // Scenario: Low-energy personality moves more slowly
    let mut low_energy = Personality::default();
    low_energy.set_energy_baseline(0.2).unwrap();
    low_energy.set_movement_expressiveness(0.3).unwrap();

    let mapper = PersonalityMapper::with_personality(&low_energy);
    let influence = mapper.current_influence();

    // Full speed motors
    let motors = (100, 100);
    let scaled = scale_motor_output(motors.0, motors.1, influence);

    // Should be scaled down significantly
    assert!(
        scaled.0 < 50,
        "Low energy personality should move slowly: got {}",
        scaled.0
    );
}

#[test]
fn test_high_expressiveness_amplifies_outputs() {
    // Scenario: Expressive personality shows feelings dramatically
    let mut expressive = Personality::default();
    expressive.set_movement_expressiveness(0.9).unwrap();
    expressive.set_light_expressiveness(0.9).unwrap();

    let mapper = PersonalityMapper::with_personality(&expressive);
    let influence = mapper.current_influence();

    let motors = (50, 50);
    let lights = [100, 150, 200];

    let scaled_motors = scale_motor_output(motors.0, motors.1, influence);
    let scaled_lights = scale_light_output(lights, influence);

    // Should be close to original values
    assert!(scaled_motors.0 > 40, "High expressiveness should preserve movement");
    assert!(scaled_lights[0] > 80, "High expressiveness should preserve lighting");
}

#[test]
fn test_low_expressiveness_dampens_outputs() {
    // Scenario: Subdued personality shows feelings subtly
    let mut subdued = Personality::default();
    subdued.set_movement_expressiveness(0.2).unwrap();
    subdued.set_light_expressiveness(0.2).unwrap();

    let mapper = PersonalityMapper::with_personality(&subdued);
    let influence = mapper.current_influence();

    let motors = (100, 100);
    let lights = [200, 200, 200];

    let scaled_motors = scale_motor_output(motors.0, motors.1, influence);
    let scaled_lights = scale_light_output(lights, influence);

    // Should be significantly reduced
    assert!(scaled_motors.0 < 30, "Low expressiveness should dampen movement");
    assert!(scaled_lights[0] < 50, "Low expressiveness should dampen lighting");
}

#[test]
fn test_smooth_transition_between_personalities() {
    // Scenario: Personality change transitions gradually over time
    let mellow = PersonalityPreset::Mellow.to_personality();
    let excitable = PersonalityPreset::Excitable.to_personality();

    let mut mapper = PersonalityMapper::with_personality(&mellow);

    let start_tension_target = mapper.current_influence().tension_target;

    // Initiate 100-tick transition
    mapper.transition_to(&excitable, 100);

    assert!(mapper.is_transitioning());

    // Advance 50 ticks
    for _ in 0..50 {
        mapper.tick_transition();
    }

    let mid_tension_target = mapper.current_influence().tension_target;

    // Should be partway between mellow and excitable
    assert!(
        mid_tension_target > start_tension_target,
        "Should be moving toward excitable baseline"
    );
    assert!(
        mid_tension_target < excitable.tension_baseline(),
        "Should not have reached excitable baseline yet"
    );

    // Complete transition
    for _ in 0..51 {
        mapper.tick_transition();
    }

    // Should now match excitable personality
    assert!(!mapper.is_transitioning());
    let end_tension_target = mapper.current_influence().tension_target;
    assert!(
        (end_tension_target - excitable.tension_baseline()).abs() < 0.01,
        "Should match excitable baseline exactly"
    );
}

#[test]
fn test_no_sudden_jumps_during_transition() {
    // Scenario: Transitions must be gradual (I-PERS-006)
    let zen = PersonalityPreset::Zen.to_personality();
    let excitable = PersonalityPreset::Excitable.to_personality();

    let mut mapper = PersonalityMapper::with_personality(&zen);
    mapper.transition_to(&excitable, 50);

    let mut prev_tension = mapper.current_influence().tension_target;

    for _ in 0..50 {
        mapper.tick_transition();
        let curr_tension = mapper.current_influence().tension_target;

        let delta = (curr_tension - prev_tension).abs();

        // Each step should be small
        assert!(
            delta < 0.03,
            "Transition jump too large: {} (should be gradual)",
            delta
        );

        prev_tension = curr_tension;
    }
}

#[test]
fn test_personality_influences_not_overrides() {
    // Scenario: Personality influences nervous system but doesn't override it (I-PERS-004)
    let timid = PersonalityPreset::Timid.to_personality();
    let mapper = PersonalityMapper::with_personality(&timid);
    let influence = mapper.current_influence();

    // Even with high baseline tension, a calm environment should allow low tension
    let calm_stimulus = 0.0;
    let current_tension = 0.2;

    let result = apply_tension_influence(
        calm_stimulus,
        current_tension,
        influence,
        0.01, // Weak gravity
    );

    // Tension should be influenced but not forced to baseline immediately
    assert!(
        result > 0.1 && result < 0.9,
        "Personality should influence, not override: result={}",
        result
    );
}

#[test]
fn test_behavior_emerges_from_personality_plus_nervous_system() {
    // Scenario: Behavior is emergent, not scripted (I-PERS-005)
    let curious = PersonalityPreset::Curious.to_personality();
    let mapper = PersonalityMapper::with_personality(&curious);
    let influence = mapper.current_influence();

    // Same personality, different nervous system states = different behaviors
    let low_tension = 0.2;
    let high_tension = 0.8;

    let stimulus = 0.3;

    let result_low = apply_tension_influence(stimulus, low_tension, influence, 0.01);
    let result_high = apply_tension_influence(stimulus, high_tension, influence, 0.01);

    // Results should be different despite same personality
    assert!(
        (result_low - result_high).abs() > 0.1,
        "Same personality should produce different behaviors in different states"
    );
}

#[test]
fn test_kitchen_table_test_different_personalities() {
    // Scenario: Observable behavior differences in Kitchen Table Test
    let mellow = PersonalityPreset::Mellow.to_personality();
    let excitable = PersonalityPreset::Excitable.to_personality();

    let mellow_mapper = PersonalityMapper::with_personality(&mellow);
    let excitable_mapper = PersonalityMapper::with_personality(&excitable);

    // Simulate approach to object
    let sensors = sensors_with_distance(30.0); // Medium distance

    let mellow_influence = mellow_mapper.current_influence();
    let excitable_influence = excitable_mapper.current_influence();

    // Calculate LED brightness based on personality
    let base_color = [255, 100, 0];

    let mellow_leds = scale_light_output(base_color, mellow_influence);
    let excitable_leds = scale_light_output(base_color, excitable_influence);

    // Excitable should be brighter (more expressive)
    assert!(
        excitable_leds[0] > mellow_leds[0],
        "Excitable personality should show brighter LEDs"
    );

    // Calculate motor output
    let base_motors = (60, 60);

    let mellow_motors = scale_motor_output(base_motors.0, base_motors.1, mellow_influence);
    let excitable_motors = scale_motor_output(base_motors.0, base_motors.1, excitable_influence);

    // Excitable should move more energetically
    assert!(
        excitable_motors.0 > mellow_motors.0,
        "Excitable personality should move more energetically"
    );
}
