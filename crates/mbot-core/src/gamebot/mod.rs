//! GameBot Module - Game playing functionality for mBot2
//!
//! Implements physical game interactions including:
//! - Turn detection (tap, voice, timeout)
//! - Game state management
//! - Physical acknowledgments (LED, sound, voice)
//! - Emotional responses to game outcomes
//! - Chase game mechanics
//! - Simon Says memory game
//! - Tic-Tac-Toe physical drawing
//!
//! # Safety (Kitchen Table Test)
//! All motor commands are bounded for safe operation.
//! No harmful behaviors or terminology.

pub mod turn_detection;
pub mod emotions;
pub mod chase;
pub mod simon_says;
pub mod tictactoe_drawing;

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

pub use chase::{
    ChaseState,
    ChaseConfig,
    ChaseMode,
    ChaseStatus,
    EvasionStyle,
    EvasionPattern,
    MovementCommand,
    MovementType as ChaseMovementType,
};

pub use simon_says::{
    SimonState,
    SimonConfig,
    SimonColor,
    SimonStatus,
    PatternDisplayEvent,
    InputResult,
};

pub use tictactoe_drawing::{
    TicTacToeGrid,
    CellPosition,
    GameSymbol,
    DrawMoveCommand,
    GridDrawResult,
    SymbolDrawResult,
    draw_grid,
    draw_x,
    draw_o,
    draw_move,
    calibrate_position,
};
