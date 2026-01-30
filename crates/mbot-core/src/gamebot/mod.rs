//! GameBot Module - Game playing functionality for mBot2
//!
//! Implements physical game interactions including:
//! - Turn detection (tap, voice, timeout)
//! - Game state management
//! - Physical acknowledgments (LED, sound, voice)
//! - Emotional responses to game outcomes
//!
//! # Safety (Kitchen Table Test)
//! All motor commands are bounded for safe operation.
//! No harmful behaviors or terminology.

pub mod turn_detection;
pub mod emotions;

pub use turn_detection::{
    TurnDetectionSystem,
    TurnDetectionConfig,
    TurnSignal,
    TurnSignalType,
    TurnAcknowledgment,
    LedPattern,
    AckSound,
    InputMethod,
    GameTurnState,
    AccelerometerReading,
    VoiceDetectionResult,
};

pub use emotions::{
    GameOutcome,
    GameType,
    GameEmotionContext,
    EmotionBehavior,
    LedSpec,
    MovementType,
    EmotionSound,
    AnimationSpeed,
};
