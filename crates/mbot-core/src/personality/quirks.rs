//! Quirks System (#26)

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// A quirk is a unique behavior trigger
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Quirk {
    pub name: String,
    pub trigger_condition: String,
    pub behavior: String,
    pub frequency: f32,
}

impl Quirk {
    pub fn new(name: String, trigger: String, behavior: String, freq: f32) -> Self {
        Self {
            name,
            trigger_condition: trigger,
            behavior,
            frequency: freq.clamp(0.0, 1.0),
        }
    }
}
