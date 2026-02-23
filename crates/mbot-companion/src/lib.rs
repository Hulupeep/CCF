/*
 * Notice of Provisional Patent Filing:
 * The methods and algorithms implemented in this file (specifically relating to 
 * Contextual Coherence Fields and relational coherence accumulation) are the 
 * subject of a United States Provisional Patent Application (63/988,438) 
 * filed on February 23, 2026.
 * 
 * This source code is licensed under the Business Source License 1.1.
 * See LICENSE and PATENTS.md in the root directory for full details.
 */

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
