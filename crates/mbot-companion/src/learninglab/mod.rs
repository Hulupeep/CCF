//! LearningLab - Educational experiments system for mBot2
//!
//! This module provides interactive educational experiments for learning about
//! stimulus-response relationships and AI concepts through hands-on interaction
//! with the robot.
//!
//! # Contracts Enforced
//! - ARCH-LEARN-002: All experiment data must be Serialize/Deserialize
//! - ARCH-LEARN-003: Timing must use monotonic clock (Instant::now())
//! - ARCH-LEARN-004: All visualizations must be explainable to 10-year-olds
//! - LEARN-003: Reflex Lab experiments with step-by-step guidance
//! - I-LEARN-020: Age-appropriate language only
//! - I-LEARN-021: Clear learning objectives required
//! - I-LEARN-022: Observations must correlate with actual robot state
//! - I-LEARN-023: Self-guided with helpful hints

pub mod experiments;

pub use experiments::{
    Experiment, ExperimentSession, ExperimentStep, ExperimentType, Observation,
    ParameterChange, ReflexLabExperiments, TimingResult,
};
