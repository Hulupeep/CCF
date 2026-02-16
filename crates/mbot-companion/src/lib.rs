//! mBot Companion - Application features built on mbot-core
//!
//! This crate provides high-level functionality for the mBot robot,
//! including games, learning activities, and utility applications.
//!
//! The `brain` feature enables the higher-order brain layer with LLM
//! reasoning, memory persistence, chat channels, voice pipeline,
//! and autonomy engine.

pub mod learninglab;
pub mod protocol;
pub mod sorter;
pub mod tictactoe_logic;
pub mod transport;
pub mod websocket_v2;

#[cfg(feature = "brain")]
pub mod brain;
