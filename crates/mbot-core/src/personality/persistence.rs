//! Personality Persistence (#23)

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(feature = "std")]
use std::string::{String, ToString};

use super::Personality;

pub fn to_json(p: &Personality) -> Result<String, String> {
    serde_json::to_string_pretty(p).map_err(|e| e.to_string())
}

pub fn from_json(json: &str) -> Result<Personality, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}
