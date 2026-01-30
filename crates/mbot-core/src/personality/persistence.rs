//! Personality Persistence (#23)

use super::Personality;

pub fn to_json(p: &Personality) -> Result<String, String> {
    serde_json::to_string_pretty(p).map_err(|e| e.to_string())
}

pub fn from_json(json: &str) -> Result<Personality, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}
