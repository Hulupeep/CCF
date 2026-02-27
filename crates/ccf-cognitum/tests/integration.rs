//! Integration tests for ccf-cognitum.
//!
//! Covers invariants: I-COG-010 through I-COG-015
//! Journey: J-COG-APPLIANCE-BOOT
//! Issue: #65

use ccf_cognitum::{
    frame::{parse_frame, FrameError, SensorType},
    sensors::{Attention, CognitumSensors, LightBand, Presence, SoundBand, TimePeriod, Touch},
    SensorWindow,
};
use ccf_core::vocabulary::{ContextKey, SensorVocabulary};

// ── Helper: build a 33-byte frame ────────────────────────────────────────────

fn make_frame(sensor_type: u8, value: f64) -> [u8; 33] {
    let mut buf = [0u8; 33];
    buf[0] = sensor_type;
    buf[9..17].copy_from_slice(&value.to_le_bytes());
    buf
}

// ── I-COG-012: parse known and unknown sensor types ───────────────────────────

#[test]
fn parse_presence_frame() {
    // [data-testid: cognitum-frame-parse-ok]
    let raw = make_frame(0x01, 2.0); // Approaching
    let frame = parse_frame(&raw).unwrap();
    assert_eq!(frame.sensor_type, SensorType::Presence);
    assert!((frame.value - 2.0).abs() < 1e-9);
    eprintln!("[data-testid: cognitum-frame-parse-ok] Presence frame parsed");
}

#[test]
fn parse_light_frame() {
    let raw = make_frame(0x02, 0.8); // Bright
    let frame = parse_frame(&raw).unwrap();
    assert_eq!(frame.sensor_type, SensorType::Light);
    assert!((frame.value - 0.8).abs() < 1e-9);
}

#[test]
fn parse_sound_frame() {
    let raw = make_frame(0x03, 0.1); // Quiet
    let frame = parse_frame(&raw).unwrap();
    assert_eq!(frame.sensor_type, SensorType::Sound);
}

#[test]
fn parse_touch_frame_contact() {
    let raw = make_frame(0x04, 1.0);
    let frame = parse_frame(&raw).unwrap();
    assert_eq!(frame.sensor_type, SensorType::Touch);
    assert!((frame.value - 1.0).abs() < 1e-9);
}

#[test]
fn parse_attention_frame() {
    let raw = make_frame(0x05, 2.0); // Sustained
    let frame = parse_frame(&raw).unwrap();
    assert_eq!(frame.sensor_type, SensorType::Attention);
}

#[test]
fn parse_time_period_frame() {
    let raw = make_frame(0x06, 14.0); // 14:00 → Day
    let frame = parse_frame(&raw).unwrap();
    assert_eq!(frame.sensor_type, SensorType::TimePeriod);
}

#[test]
fn parse_unknown_sensor_type_returns_error() {
    // [data-testid: cognitum-frame-parse-err] — I-COG-012
    let raw = make_frame(0xFF, 0.0);
    let err = parse_frame(&raw).unwrap_err();
    assert_eq!(err, FrameError::UnknownSensorType(0xFF));
    eprintln!("[data-testid: cognitum-frame-parse-err] Unknown type rejected correctly");
}

// ── I-COG-013 & I-COG-014: SensorWindow baseline and feature vec ──────────────

#[test]
fn empty_window_returns_ambient_baseline() {
    // [data-testid: cognitum-snapshot] — I-COG-013
    let window = SensorWindow::new();
    let snap = window.snapshot();
    assert_eq!(snap.presence,    Presence::Absent);
    assert_eq!(snap.light,       LightBand::Dim);
    assert_eq!(snap.sound,       SoundBand::Quiet);
    assert_eq!(snap.touch,       Touch::None);
    assert_eq!(snap.attention,   Attention::None);
    assert_eq!(snap.time_period, TimePeriod::Day);
    eprintln!("[data-testid: cognitum-snapshot] Empty window baseline OK");
}

#[test]
fn feature_vec_all_values_in_unit_range() {
    // [data-testid: cognitum-feature-vec] — I-COG-014
    let sensors = CognitumSensors {
        presence:    Presence::Approaching,
        light:       LightBand::Bright,
        sound:       SoundBand::Loud,
        touch:       Touch::Contact,
        attention:   Attention::Sustained,
        time_period: TimePeriod::Evening,
    };
    let vec = sensors.to_feature_vec();
    assert_eq!(vec.len(), 6);
    for (i, &v) in vec.iter().enumerate() {
        assert!(v >= 0.0 && v <= 1.0, "component {} = {} out of [0, 1]", i, v);
    }
    eprintln!("[data-testid: cognitum-feature-vec] All 6 components in [0, 1]");
}

#[test]
fn feature_vec_baseline_sensible() {
    let baseline = CognitumSensors::AMBIENT_BASELINE;
    let vec = baseline.to_feature_vec();
    // Absent=0/3=0.0, Dim=1/2=0.5, Quiet=0/2=0.0, None=0/1=0.0, None=0/2=0.0, Day=2/3≈0.667
    assert!((vec[0] - 0.0).abs() < 1e-6, "presence baseline should be 0.0");
    assert!((vec[1] - 0.5).abs() < 1e-6, "light (Dim) baseline should be 0.5");
}

// ── I-COG-015: deterministic hashing ─────────────────────────────────────────

#[test]
fn context_hash_is_deterministic() {
    // [data-testid: cognitum-feature-vec] — I-COG-015
    let sensors_a = CognitumSensors {
        presence:    Presence::Approaching,
        light:       LightBand::Bright,
        sound:       SoundBand::Quiet,
        touch:       Touch::None,
        attention:   Attention::Sustained,
        time_period: TimePeriod::Day,
    };
    let sensors_b = sensors_a.clone();

    let key_a = ContextKey::<CognitumSensors, 6>::new(sensors_a);
    let key_b = ContextKey::<CognitumSensors, 6>::new(sensors_b);

    assert_eq!(key_a.context_hash_u32(), key_b.context_hash_u32(),
        "same sensor state must produce the same hash (I-COG-015)");
}

#[test]
fn different_sensor_states_have_different_hashes() {
    let approaching = CognitumSensors { presence: Presence::Approaching, ..CognitumSensors::AMBIENT_BASELINE };
    let absent      = CognitumSensors { presence: Presence::Absent,      ..CognitumSensors::AMBIENT_BASELINE };

    let key_a = ContextKey::<CognitumSensors, 6>::new(approaching);
    let key_b = ContextKey::<CognitumSensors, 6>::new(absent);

    assert_ne!(key_a.context_hash_u32(), key_b.context_hash_u32());
}

// ── SensorWindow ingest → snapshot round-trip ─────────────────────────────────

#[test]
fn window_ingest_updates_correct_dimension() {
    let mut window = SensorWindow::new();

    // Ingest an Approaching presence frame
    let raw = make_frame(0x01, 2.0);
    window.ingest(parse_frame(&raw).unwrap());

    let snap = window.snapshot();
    assert_eq!(snap.presence, Presence::Approaching);
    // Other dimensions should remain at baseline
    assert_eq!(snap.light, LightBand::Dim);
    assert_eq!(snap.touch, Touch::None);
}

#[test]
fn window_ingest_all_six_dimensions() {
    let mut window = SensorWindow::new();

    window.ingest(parse_frame(&make_frame(0x01, 2.0)).unwrap()); // Approaching
    window.ingest(parse_frame(&make_frame(0x02, 0.9)).unwrap()); // Bright
    window.ingest(parse_frame(&make_frame(0x03, 0.8)).unwrap()); // Loud
    window.ingest(parse_frame(&make_frame(0x04, 1.0)).unwrap()); // Contact
    window.ingest(parse_frame(&make_frame(0x05, 2.0)).unwrap()); // Sustained
    window.ingest(parse_frame(&make_frame(0x06, 19.0)).unwrap()); // 19:00 → Evening

    let snap = window.snapshot();
    assert_eq!(snap.presence,    Presence::Approaching);
    assert_eq!(snap.light,       LightBand::Bright);
    assert_eq!(snap.sound,       SoundBand::Loud);
    assert_eq!(snap.touch,       Touch::Contact);
    assert_eq!(snap.attention,   Attention::Sustained);
    assert_eq!(snap.time_period, TimePeriod::Evening);
}

#[test]
fn window_reset_returns_to_baseline() {
    let mut window = SensorWindow::new();
    window.ingest(parse_frame(&make_frame(0x01, 2.0)).unwrap());
    window.reset();
    let snap = window.snapshot();
    assert_eq!(snap.presence, Presence::Absent);
}

// ── Full pipeline: frames → ContextKey → CoherenceField → SocialPhase ─────────

#[test]
fn full_pipeline_positive_interactions_raise_coherence() {
    // [data-testid: cognitum-pipeline] — J-COG-APPLIANCE-BOOT
    use ccf_core::{
        accumulator::CoherenceField,
        phase::Personality,
    };

    let mut field = CoherenceField::new();
    let personality = Personality::default();

    let mut window = SensorWindow::new();
    // Simulate 10 ticks of approaching + sustained attention
    for tick in 0..10u64 {
        window.ingest(parse_frame(&make_frame(0x01, 2.0)).unwrap()); // Approaching
        window.ingest(parse_frame(&make_frame(0x05, 2.0)).unwrap()); // Sustained
        let snap = window.snapshot();
        let key = ContextKey::<CognitumSensors, 6>::new(snap);
        field.positive_interaction(&key, &personality, tick, false);
    }

    // Grab the key for the interaction context
    let snap = window.snapshot();
    let key = ContextKey::<CognitumSensors, 6>::new(snap);
    let coherence = field.context_coherence(&key);

    assert!(coherence > 0.0,
        "Coherence should be > 0 after 10 positive interactions, got {}", coherence);
    assert!(coherence <= 1.0,
        "Coherence must be <= 1.0, got {}", coherence);

    eprintln!("[data-testid: cognitum-pipeline] Coherence after 10 interactions: {:.4}", coherence);
}

#[test]
fn touch_frame_produces_positive_interaction_context() {
    // Touch = positive interaction signal per CCF social semantics
    let mut window = SensorWindow::new();
    window.ingest(parse_frame(&make_frame(0x04, 1.0)).unwrap()); // Contact
    let snap = window.snapshot();
    assert_eq!(snap.touch, Touch::Contact);
    // The key is valid and can drive CoherenceField
    let key = ContextKey::<CognitumSensors, 6>::new(snap);
    let _ = key.context_hash_u32();
}

// ── Time of day boundary tests ─────────────────────────────────────────────────

#[test]
fn time_period_boundaries() {
    use ccf_cognitum::sensors::TimePeriod;
    assert_eq!(TimePeriod::from_hour(5),  TimePeriod::Night);
    assert_eq!(TimePeriod::from_hour(6),  TimePeriod::Morning);
    assert_eq!(TimePeriod::from_hour(11), TimePeriod::Morning);
    assert_eq!(TimePeriod::from_hour(12), TimePeriod::Day);
    assert_eq!(TimePeriod::from_hour(16), TimePeriod::Day);
    assert_eq!(TimePeriod::from_hour(17), TimePeriod::Evening);
    assert_eq!(TimePeriod::from_hour(20), TimePeriod::Evening);
    assert_eq!(TimePeriod::from_hour(21), TimePeriod::Night);
    assert_eq!(TimePeriod::from_hour(0),  TimePeriod::Night);
}

// ── Cosine similarity: different contexts are dissimilar ──────────────────────

#[test]
fn approaching_and_absent_contexts_are_dissimilar() {
    let approaching = ContextKey::<CognitumSensors, 6>::new(CognitumSensors {
        presence: Presence::Approaching,
        ..CognitumSensors::AMBIENT_BASELINE
    });
    let absent = ContextKey::<CognitumSensors, 6>::new(CognitumSensors::AMBIENT_BASELINE);

    let sim = approaching.cosine_similarity(&absent);
    assert!(sim < 1.0, "Different contexts must not be identical: sim={}", sim);
}

#[test]
fn identical_contexts_have_similarity_one() {
    let sensors = CognitumSensors {
        presence:    Presence::Static,
        light:       LightBand::Bright,
        sound:       SoundBand::Moderate,
        touch:       Touch::None,
        attention:   Attention::Glance,
        time_period: TimePeriod::Morning,
    };
    let key = ContextKey::<CognitumSensors, 6>::new(sensors);
    let sim = key.cosine_similarity(&key);
    assert!((sim - 1.0).abs() < 1e-5, "Self-similarity must be 1.0, got {}", sim);
}
