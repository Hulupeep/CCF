//! # ccf-cognitum
//!
//! CCF `SensorVocabulary` adapter for the Cognitum v0 appliance sensor expansion port.
//!
//! Implements the 6-dimensional `CognitumSensors` vocabulary and a `SensorWindow`
//! that aggregates 33-byte Cognitum protocol frames into `ContextKey` inputs for
//! `ccf_core::CoherenceField`.
//!
//! ## Pipeline
//!
//! ```text
//! Sensor port (33-byte frames)
//!     │
//!     ▼  frame::parse_frame()
//! ParsedFrame
//!     │
//!     ▼  SensorWindow::ingest()
//! SensorWindow (latest value per dimension)
//!     │
//!     ▼  SensorWindow::snapshot()
//! CognitumSensors  (SensorVocabulary<6>)
//!     │
//!     ▼  ContextKey::new()
//! ContextKey<CognitumSensors, 6>
//!     │
//!     ▼  CoherenceField::positive_interaction() / effective_coherence()
//! SocialPhase output to LLM / actuators
//! ```
//!
//! ## Deployment targets
//!
//! - `wasm32-wasip1` — Cognitum v0 appliance tile ABI  (I-COG-010)
//! - `thumbv7em-none-eabihf` — Cognitum V1 chip / embedded  (I-COG-011)
//!
//! ## Issue
//!
//! Story #65: <https://github.com/Hulupeep/CCF/issues/65>

#![no_std]

pub mod frame;
pub mod sensors;
pub mod window;

// Re-export the most commonly used types at the crate root.
pub use frame::{parse_frame, FrameError, ParsedFrame, SensorType};
pub use sensors::{Attention, CognitumSensors, LightBand, Presence, SoundBand, TimePeriod, Touch};
pub use window::SensorWindow;
