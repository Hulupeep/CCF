//! Personality Module - Configurable behavior profiles for mBot2
//!
//! This module provides the `Personality` struct for defining how the robot
//! behaves, reacts, and expresses itself. All parameters are bounded to [0.0, 1.0]
//! to ensure safe and predictable behavior.
//!
//! # Modules
//! - `behavior_mapping`: Maps personality parameters to nervous system behaviors
//! - `presets`: Extended library of 20 preset personalities (#18)
//! - `quirks`: Quirks system for unique behaviors (#26)
//! - `persistence`: Save/load personalities to disk (#23)
//!
//! # Invariants
//! - **I-PERS-001:** All personality parameters must be within bounds [0.0, 1.0]
//! - **I-PERS-002:** Personality must be serializable to JSON
//! - **I-PERS-003:** Default personality must have safe, neutral values (0.5)
//! - **I-PERS-004:** Personality parameters must smoothly influence nervous system, not override it
//! - **I-PERS-005:** Behavior must emerge from personality + nervous system, not be scripted
//! - **I-PERS-006:** Transitions must be gradual, never jarring

#![cfg_attr(feature = "no_std", allow(unused_imports))]

// Export submodules
pub mod presets;
pub mod quirks;

#[cfg(feature = "std")]
pub mod persistence;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::{String, ToString}, vec::Vec, format};

#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

use core::fmt;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Error type for personality validation failures
#[derive(Debug, Clone, PartialEq)]
pub enum PersonalityError {
    /// A parameter value was outside the valid range [0.0, 1.0]
    ParameterOutOfBounds {
        /// Name of the parameter that failed validation
        parameter: &'static str,
        /// The invalid value that was provided
        value: f32,
    },
    /// The personality ID was empty
    EmptyId,
    /// The personality name was empty
    EmptyName,
    /// JSON deserialization failed
    DeserializationError(String),
}

impl fmt::Display for PersonalityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersonalityError::ParameterOutOfBounds { parameter, value } => {
                write!(
                    f,
                    "parameter '{}' out of bounds: {} (must be 0.0-1.0)",
                    parameter, value
                )
            }
            PersonalityError::EmptyId => write!(f, "personality ID cannot be empty"),
            PersonalityError::EmptyName => write!(f, "personality name cannot be empty"),
            PersonalityError::DeserializationError(msg) => {
                write!(f, "deserialization error: {}", msg)
            }
        }
    }
}

/// Validates that a value is within [0.0, 1.0] bounds
#[inline]
fn validate_bounded(value: f32, parameter: &'static str) -> Result<f32, PersonalityError> {
    if value >= 0.0 && value <= 1.0 {
        Ok(value)
    } else {
        Err(PersonalityError::ParameterOutOfBounds { parameter, value })
    }
}

/// Clamps a value to [0.0, 1.0] bounds
#[inline]
fn clamp_bounded(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

/// Personality configuration for mBot2
///
/// Defines how the robot behaves, reacts to stimuli, and expresses its internal
/// state. All f32 parameters are bounded to [0.0, 1.0].
///
/// # Example
///
/// ```
/// use mbot_core::personality::Personality;
///
/// // Create a default (neutral) personality
/// let mut personality = Personality::default();
///
/// // Customize it
/// personality.set_tension_baseline(0.3).unwrap();
/// personality.set_curiosity_drive(0.8).unwrap();
///
/// // Serialize to JSON
/// let json = personality.to_json().unwrap();
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Personality {
    // === Identity ===
    /// Unique identifier for this personality
    pub id: String,
    /// Display name of the personality
    pub name: String,
    /// Emoji representation
    pub icon: char,
    /// Schema version for forward compatibility
    pub version: u32,
    /// Unix timestamp of creation
    pub created_at: u64,
    /// Unix timestamp of last modification
    pub modified_at: u64,

    // === Baselines (where it wants to be) ===
    tension_baseline: f32,
    coherence_baseline: f32,
    energy_baseline: f32,

    // === Reactivity (how stimuli affect it) ===
    startle_sensitivity: f32,
    recovery_speed: f32,
    curiosity_drive: f32,

    // === Expression (how it shows feelings) ===
    movement_expressiveness: f32,
    sound_expressiveness: f32,
    light_expressiveness: f32,

    // === Extensions ===
    /// List of quirk identifiers
    pub quirks: Vec<String>,
    /// Optional custom sound pack
    pub sound_pack: Option<String>,
}

impl Default for Personality {
    /// Creates a default personality with safe, neutral values.
    ///
    /// All bounded parameters default to 0.5 (middle of range) per I-PERS-003.
    fn default() -> Self {
        Self {
            id: String::from("default"),
            name: String::from("Default"),
            icon: '?',
            version: 1,
            created_at: 0,
            modified_at: 0,

            // Baselines - neutral at 0.5
            tension_baseline: 0.5,
            coherence_baseline: 0.5,
            energy_baseline: 0.5,

            // Reactivity - neutral at 0.5
            startle_sensitivity: 0.5,
            recovery_speed: 0.5,
            curiosity_drive: 0.5,

            // Expression - neutral at 0.5
            movement_expressiveness: 0.5,
            sound_expressiveness: 0.5,
            light_expressiveness: 0.5,

            // Extensions
            quirks: Vec::new(),
            sound_pack: None,
        }
    }
}

impl Personality {
    /// Creates a new personality with the given ID and name.
    ///
    /// All other parameters are set to safe defaults.
    ///
    /// # Errors
    /// Returns `PersonalityError::EmptyId` if id is empty.
    /// Returns `PersonalityError::EmptyName` if name is empty.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Result<Self, PersonalityError> {
        let id = id.into();
        let name = name.into();

        if id.is_empty() {
            return Err(PersonalityError::EmptyId);
        }
        if name.is_empty() {
            return Err(PersonalityError::EmptyName);
        }

        Ok(Self {
            id,
            name,
            ..Default::default()
        })
    }

    /// Creates a PersonalityBuilder for fluent construction.
    pub fn builder() -> PersonalityBuilder {
        PersonalityBuilder::new()
    }

    // === Baseline Getters ===

    /// Returns the tension baseline (0.0-1.0)
    #[inline]
    pub fn tension_baseline(&self) -> f32 {
        self.tension_baseline
    }

    /// Returns the coherence baseline (0.0-1.0)
    #[inline]
    pub fn coherence_baseline(&self) -> f32 {
        self.coherence_baseline
    }

    /// Returns the energy baseline (0.0-1.0)
    #[inline]
    pub fn energy_baseline(&self) -> f32 {
        self.energy_baseline
    }

    // === Baseline Setters (with validation) ===

    /// Sets the tension baseline.
    ///
    /// # Errors
    /// Returns `PersonalityError::ParameterOutOfBounds` if value is not in [0.0, 1.0]
    pub fn set_tension_baseline(&mut self, value: f32) -> Result<(), PersonalityError> {
        self.tension_baseline = validate_bounded(value, "tension_baseline")?;
        Ok(())
    }

    /// Sets the coherence baseline.
    ///
    /// # Errors
    /// Returns `PersonalityError::ParameterOutOfBounds` if value is not in [0.0, 1.0]
    pub fn set_coherence_baseline(&mut self, value: f32) -> Result<(), PersonalityError> {
        self.coherence_baseline = validate_bounded(value, "coherence_baseline")?;
        Ok(())
    }

    /// Sets the energy baseline.
    ///
    /// # Errors
    /// Returns `PersonalityError::ParameterOutOfBounds` if value is not in [0.0, 1.0]
    pub fn set_energy_baseline(&mut self, value: f32) -> Result<(), PersonalityError> {
        self.energy_baseline = validate_bounded(value, "energy_baseline")?;
        Ok(())
    }

    // === Reactivity Getters ===

    /// Returns the startle sensitivity (0.0-1.0)
    #[inline]
    pub fn startle_sensitivity(&self) -> f32 {
        self.startle_sensitivity
    }

    /// Returns the recovery speed (0.0-1.0)
    #[inline]
    pub fn recovery_speed(&self) -> f32 {
        self.recovery_speed
    }

    /// Returns the curiosity drive (0.0-1.0)
    #[inline]
    pub fn curiosity_drive(&self) -> f32 {
        self.curiosity_drive
    }

    // === Reactivity Setters ===

    /// Sets the startle sensitivity.
    ///
    /// # Errors
    /// Returns `PersonalityError::ParameterOutOfBounds` if value is not in [0.0, 1.0]
    pub fn set_startle_sensitivity(&mut self, value: f32) -> Result<(), PersonalityError> {
        self.startle_sensitivity = validate_bounded(value, "startle_sensitivity")?;
        Ok(())
    }

    /// Sets the recovery speed.
    ///
    /// # Errors
    /// Returns `PersonalityError::ParameterOutOfBounds` if value is not in [0.0, 1.0]
    pub fn set_recovery_speed(&mut self, value: f32) -> Result<(), PersonalityError> {
        self.recovery_speed = validate_bounded(value, "recovery_speed")?;
        Ok(())
    }

    /// Sets the curiosity drive.
    ///
    /// # Errors
    /// Returns `PersonalityError::ParameterOutOfBounds` if value is not in [0.0, 1.0]
    pub fn set_curiosity_drive(&mut self, value: f32) -> Result<(), PersonalityError> {
        self.curiosity_drive = validate_bounded(value, "curiosity_drive")?;
        Ok(())
    }

    // === Expression Getters ===

    /// Returns the movement expressiveness (0.0-1.0)
    #[inline]
    pub fn movement_expressiveness(&self) -> f32 {
        self.movement_expressiveness
    }

    /// Returns the sound expressiveness (0.0-1.0)
    #[inline]
    pub fn sound_expressiveness(&self) -> f32 {
        self.sound_expressiveness
    }

    /// Returns the light expressiveness (0.0-1.0)
    #[inline]
    pub fn light_expressiveness(&self) -> f32 {
        self.light_expressiveness
    }

    // === Expression Setters ===

    /// Sets the movement expressiveness.
    ///
    /// # Errors
    /// Returns `PersonalityError::ParameterOutOfBounds` if value is not in [0.0, 1.0]
    pub fn set_movement_expressiveness(&mut self, value: f32) -> Result<(), PersonalityError> {
        self.movement_expressiveness = validate_bounded(value, "movement_expressiveness")?;
        Ok(())
    }

    /// Sets the sound expressiveness.
    ///
    /// # Errors
    /// Returns `PersonalityError::ParameterOutOfBounds` if value is not in [0.0, 1.0]
    pub fn set_sound_expressiveness(&mut self, value: f32) -> Result<(), PersonalityError> {
        self.sound_expressiveness = validate_bounded(value, "sound_expressiveness")?;
        Ok(())
    }

    /// Sets the light expressiveness.
    ///
    /// # Errors
    /// Returns `PersonalityError::ParameterOutOfBounds` if value is not in [0.0, 1.0]
    pub fn set_light_expressiveness(&mut self, value: f32) -> Result<(), PersonalityError> {
        self.light_expressiveness = validate_bounded(value, "light_expressiveness")?;
        Ok(())
    }

    /// Validates that all parameters are within bounds.
    ///
    /// This is called after deserialization to ensure data integrity.
    pub fn validate(&self) -> Result<(), PersonalityError> {
        validate_bounded(self.tension_baseline, "tension_baseline")?;
        validate_bounded(self.coherence_baseline, "coherence_baseline")?;
        validate_bounded(self.energy_baseline, "energy_baseline")?;
        validate_bounded(self.startle_sensitivity, "startle_sensitivity")?;
        validate_bounded(self.recovery_speed, "recovery_speed")?;
        validate_bounded(self.curiosity_drive, "curiosity_drive")?;
        validate_bounded(self.movement_expressiveness, "movement_expressiveness")?;
        validate_bounded(self.sound_expressiveness, "sound_expressiveness")?;
        validate_bounded(self.light_expressiveness, "light_expressiveness")?;

        if self.id.is_empty() {
            return Err(PersonalityError::EmptyId);
        }
        if self.name.is_empty() {
            return Err(PersonalityError::EmptyName);
        }

        Ok(())
    }
}

// === Serde Serialization (I-PERS-002) ===

#[cfg(feature = "std")]
mod serde_impl {
    use super::*;

    /// JSON-serializable representation of Personality
    #[derive(Debug, Clone)]
    pub struct PersonalityJson {
        pub id: String,
        pub name: String,
        pub icon: String,
        pub version: u32,
        pub created_at: u64,
        pub modified_at: u64,
        pub tension_baseline: f32,
        pub coherence_baseline: f32,
        pub energy_baseline: f32,
        pub startle_sensitivity: f32,
        pub recovery_speed: f32,
        pub curiosity_drive: f32,
        pub movement_expressiveness: f32,
        pub sound_expressiveness: f32,
        pub light_expressiveness: f32,
        pub quirks: Vec<String>,
        pub sound_pack: Option<String>,
    }

    impl From<&Personality> for PersonalityJson {
        fn from(p: &Personality) -> Self {
            Self {
                id: p.id.clone(),
                name: p.name.clone(),
                icon: p.icon.to_string(),
                version: p.version,
                created_at: p.created_at,
                modified_at: p.modified_at,
                tension_baseline: p.tension_baseline,
                coherence_baseline: p.coherence_baseline,
                energy_baseline: p.energy_baseline,
                startle_sensitivity: p.startle_sensitivity,
                recovery_speed: p.recovery_speed,
                curiosity_drive: p.curiosity_drive,
                movement_expressiveness: p.movement_expressiveness,
                sound_expressiveness: p.sound_expressiveness,
                light_expressiveness: p.light_expressiveness,
                quirks: p.quirks.clone(),
                sound_pack: p.sound_pack.clone(),
            }
        }
    }

    impl Personality {
        /// Serializes the personality to a JSON string.
        ///
        /// This manually constructs JSON to avoid serde dependency in core.
        pub fn to_json(&self) -> Result<String, PersonalityError> {
            let quirks_json = if self.quirks.is_empty() {
                String::from("[]")
            } else {
                let items: Vec<String> = self
                    .quirks
                    .iter()
                    .map(|q| format!("\"{}\"", escape_json_string(q)))
                    .collect();
                format!("[{}]", items.join(","))
            };

            let sound_pack_json = match &self.sound_pack {
                Some(sp) => format!("\"{}\"", escape_json_string(sp)),
                None => String::from("null"),
            };

            Ok(format!(
                r#"{{"id":"{}","name":"{}","icon":"{}","version":{},"created_at":{},"modified_at":{},"tension_baseline":{},"coherence_baseline":{},"energy_baseline":{},"startle_sensitivity":{},"recovery_speed":{},"curiosity_drive":{},"movement_expressiveness":{},"sound_expressiveness":{},"light_expressiveness":{},"quirks":{},"sound_pack":{}}}"#,
                escape_json_string(&self.id),
                escape_json_string(&self.name),
                self.icon,
                self.version,
                self.created_at,
                self.modified_at,
                self.tension_baseline,
                self.coherence_baseline,
                self.energy_baseline,
                self.startle_sensitivity,
                self.recovery_speed,
                self.curiosity_drive,
                self.movement_expressiveness,
                self.sound_expressiveness,
                self.light_expressiveness,
                quirks_json,
                sound_pack_json,
            ))
        }

        /// Deserializes a personality from a JSON string.
        ///
        /// Values are validated after parsing per I-PERS-001.
        pub fn from_json(json: &str) -> Result<Self, PersonalityError> {
            // Simple JSON parser for the specific structure we expect
            let mut personality = Self::default();

            // Parse id
            if let Some(id) = extract_string_field(json, "id") {
                personality.id = id;
            }

            // Parse name
            if let Some(name) = extract_string_field(json, "name") {
                personality.name = name;
            }

            // Parse icon
            if let Some(icon) = extract_string_field(json, "icon") {
                personality.icon = icon.chars().next().unwrap_or('?');
            }

            // Parse numeric fields
            if let Some(v) = extract_u32_field(json, "version") {
                personality.version = v;
            }
            if let Some(v) = extract_u64_field(json, "created_at") {
                personality.created_at = v;
            }
            if let Some(v) = extract_u64_field(json, "modified_at") {
                personality.modified_at = v;
            }

            // Parse f32 fields (with clamping for safety)
            if let Some(v) = extract_f32_field(json, "tension_baseline") {
                personality.tension_baseline = clamp_bounded(v);
            }
            if let Some(v) = extract_f32_field(json, "coherence_baseline") {
                personality.coherence_baseline = clamp_bounded(v);
            }
            if let Some(v) = extract_f32_field(json, "energy_baseline") {
                personality.energy_baseline = clamp_bounded(v);
            }
            if let Some(v) = extract_f32_field(json, "startle_sensitivity") {
                personality.startle_sensitivity = clamp_bounded(v);
            }
            if let Some(v) = extract_f32_field(json, "recovery_speed") {
                personality.recovery_speed = clamp_bounded(v);
            }
            if let Some(v) = extract_f32_field(json, "curiosity_drive") {
                personality.curiosity_drive = clamp_bounded(v);
            }
            if let Some(v) = extract_f32_field(json, "movement_expressiveness") {
                personality.movement_expressiveness = clamp_bounded(v);
            }
            if let Some(v) = extract_f32_field(json, "sound_expressiveness") {
                personality.sound_expressiveness = clamp_bounded(v);
            }
            if let Some(v) = extract_f32_field(json, "light_expressiveness") {
                personality.light_expressiveness = clamp_bounded(v);
            }

            // Parse quirks array
            if let Some(quirks) = extract_string_array_field(json, "quirks") {
                personality.quirks = quirks;
            }

            // Parse sound_pack (nullable)
            personality.sound_pack = extract_nullable_string_field(json, "sound_pack");

            // Validate the result
            personality.validate()?;

            Ok(personality)
        }
    }

    /// Escapes special characters for JSON strings
    fn escape_json_string(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                _ => result.push(c),
            }
        }
        result
    }

    /// Extracts a string field from JSON
    fn extract_string_field(json: &str, field: &str) -> Option<String> {
        let pattern = format!("\"{}\":\"", field);
        let start = json.find(&pattern)? + pattern.len();
        let rest = &json[start..];
        let end = find_unescaped_quote(rest)?;
        Some(unescape_json_string(&rest[..end]))
    }

    /// Extracts a nullable string field from JSON
    fn extract_nullable_string_field(json: &str, field: &str) -> Option<String> {
        let pattern = format!("\"{}\":", field);
        let start = json.find(&pattern)? + pattern.len();
        let rest = json[start..].trim_start();
        if rest.starts_with("null") {
            None
        } else if rest.starts_with('"') {
            let content = &rest[1..];
            let end = find_unescaped_quote(content)?;
            Some(unescape_json_string(&content[..end]))
        } else {
            None
        }
    }

    /// Extracts a string array field from JSON
    fn extract_string_array_field(json: &str, field: &str) -> Option<Vec<String>> {
        let pattern = format!("\"{}\":[", field);
        let start = json.find(&pattern)? + pattern.len();
        let rest = &json[start..];
        let end = rest.find(']')?;
        let array_content = &rest[..end];

        if array_content.trim().is_empty() {
            return Some(Vec::new());
        }

        let mut result = Vec::new();
        let mut in_string = false;
        let mut current = String::new();
        let mut escape_next = false;

        for c in array_content.chars() {
            if escape_next {
                current.push(c);
                escape_next = false;
                continue;
            }

            match c {
                '\\' if in_string => escape_next = true,
                '"' => {
                    if in_string {
                        result.push(unescape_json_string(&current));
                        current = String::new();
                    }
                    in_string = !in_string;
                }
                _ if in_string => current.push(c),
                _ => {}
            }
        }

        Some(result)
    }

    /// Extracts a u32 field from JSON
    fn extract_u32_field(json: &str, field: &str) -> Option<u32> {
        let pattern = format!("\"{}\":", field);
        let start = json.find(&pattern)? + pattern.len();
        let rest = json[start..].trim_start();
        let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        rest[..end].parse().ok()
    }

    /// Extracts a u64 field from JSON
    fn extract_u64_field(json: &str, field: &str) -> Option<u64> {
        let pattern = format!("\"{}\":", field);
        let start = json.find(&pattern)? + pattern.len();
        let rest = json[start..].trim_start();
        let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        rest[..end].parse().ok()
    }

    /// Extracts an f32 field from JSON
    fn extract_f32_field(json: &str, field: &str) -> Option<f32> {
        let pattern = format!("\"{}\":", field);
        let start = json.find(&pattern)? + pattern.len();
        let rest = json[start..].trim_start();
        let end = rest
            .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            .unwrap_or(rest.len());
        rest[..end].parse().ok()
    }

    /// Finds the position of the first unescaped quote
    fn find_unescaped_quote(s: &str) -> Option<usize> {
        let mut escape_next = false;
        for (i, c) in s.chars().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }
            if c == '\\' {
                escape_next = true;
                continue;
            }
            if c == '"' {
                return Some(i);
            }
        }
        None
    }

    /// Unescapes a JSON string
    fn unescape_json_string(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }
        result
    }
}

#[cfg(feature = "std")]
pub use serde_impl::*;

/// Builder for constructing Personality with fluent API
#[derive(Debug, Clone)]
pub struct PersonalityBuilder {
    personality: Personality,
    errors: Vec<PersonalityError>,
}

impl PersonalityBuilder {
    /// Creates a new builder with default values
    pub fn new() -> Self {
        Self {
            personality: Personality::default(),
            errors: Vec::new(),
        }
    }

    /// Sets the personality ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.personality.id = id.into();
        self
    }

    /// Sets the personality name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.personality.name = name.into();
        self
    }

    /// Sets the icon
    pub fn icon(mut self, icon: char) -> Self {
        self.personality.icon = icon;
        self
    }

    /// Sets the tension baseline (0.0-1.0)
    pub fn tension_baseline(mut self, value: f32) -> Self {
        match validate_bounded(value, "tension_baseline") {
            Ok(v) => self.personality.tension_baseline = v,
            Err(e) => self.errors.push(e),
        }
        self
    }

    /// Sets the coherence baseline (0.0-1.0)
    pub fn coherence_baseline(mut self, value: f32) -> Self {
        match validate_bounded(value, "coherence_baseline") {
            Ok(v) => self.personality.coherence_baseline = v,
            Err(e) => self.errors.push(e),
        }
        self
    }

    /// Sets the energy baseline (0.0-1.0)
    pub fn energy_baseline(mut self, value: f32) -> Self {
        match validate_bounded(value, "energy_baseline") {
            Ok(v) => self.personality.energy_baseline = v,
            Err(e) => self.errors.push(e),
        }
        self
    }

    /// Sets the startle sensitivity (0.0-1.0)
    pub fn startle_sensitivity(mut self, value: f32) -> Self {
        match validate_bounded(value, "startle_sensitivity") {
            Ok(v) => self.personality.startle_sensitivity = v,
            Err(e) => self.errors.push(e),
        }
        self
    }

    /// Sets the recovery speed (0.0-1.0)
    pub fn recovery_speed(mut self, value: f32) -> Self {
        match validate_bounded(value, "recovery_speed") {
            Ok(v) => self.personality.recovery_speed = v,
            Err(e) => self.errors.push(e),
        }
        self
    }

    /// Sets the curiosity drive (0.0-1.0)
    pub fn curiosity_drive(mut self, value: f32) -> Self {
        match validate_bounded(value, "curiosity_drive") {
            Ok(v) => self.personality.curiosity_drive = v,
            Err(e) => self.errors.push(e),
        }
        self
    }

    /// Sets the movement expressiveness (0.0-1.0)
    pub fn movement_expressiveness(mut self, value: f32) -> Self {
        match validate_bounded(value, "movement_expressiveness") {
            Ok(v) => self.personality.movement_expressiveness = v,
            Err(e) => self.errors.push(e),
        }
        self
    }

    /// Sets the sound expressiveness (0.0-1.0)
    pub fn sound_expressiveness(mut self, value: f32) -> Self {
        match validate_bounded(value, "sound_expressiveness") {
            Ok(v) => self.personality.sound_expressiveness = v,
            Err(e) => self.errors.push(e),
        }
        self
    }

    /// Sets the light expressiveness (0.0-1.0)
    pub fn light_expressiveness(mut self, value: f32) -> Self {
        match validate_bounded(value, "light_expressiveness") {
            Ok(v) => self.personality.light_expressiveness = v,
            Err(e) => self.errors.push(e),
        }
        self
    }

    /// Adds a quirk
    pub fn quirk(mut self, quirk: impl Into<String>) -> Self {
        self.personality.quirks.push(quirk.into());
        self
    }

    /// Sets the sound pack
    pub fn sound_pack(mut self, pack: impl Into<String>) -> Self {
        self.personality.sound_pack = Some(pack.into());
        self
    }

    /// Sets creation and modification timestamps
    pub fn timestamps(mut self, created_at: u64, modified_at: u64) -> Self {
        self.personality.created_at = created_at;
        self.personality.modified_at = modified_at;
        self
    }

    /// Builds the personality, returning the first error if any occurred
    pub fn build(self) -> Result<Personality, PersonalityError> {
        if let Some(error) = self.errors.into_iter().next() {
            return Err(error);
        }

        self.personality.validate()?;
        Ok(self.personality)
    }
}

impl Default for PersonalityBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// === Behavior Mapping ===
pub mod behavior_mapping;
pub use behavior_mapping::{
    PersonalityInfluence, PersonalityMapper,
    apply_tension_influence, apply_coherence_influence,
    apply_energy_influence, apply_curiosity_influence,
    scale_motor_output, scale_sound_output, scale_light_output,
};

// === Personality Switching ===
pub mod switching;
pub use switching::{
    PersonalitySwitcher, TransitionConfig, TransitionEvent, Easing,
};

// === Preset Personalities ===

/// Preset personality types that can be loaded
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum PersonalityPreset {
    /// Calm, slow to react, quick to recover
    Mellow,
    /// High curiosity, expressive, moderate tension
    Curious,
    /// Low tension, high coherence, minimal expression
    Zen,
    /// High energy, high expressiveness, quick reactions
    Excitable,
    /// Cautious, high startle sensitivity, slow recovery
    Timid,
    /// Bold and exploring, loves new experiences
    Adventurous,
    /// Reserved, low expressiveness, observes from distance
    Shy,
    /// Irritable, high tension, low patience
    Grumpy,
    /// Upbeat, high energy, quick to engage
    Cheerful,
    /// Careful and deliberate, double-checks everything
    Cautious,
    /// Playful and spontaneous, loves games
    Playful,
    /// Focused and methodical, task-oriented
    Serious,
    /// High energy, never sits still
    Energetic,
    /// Peaceful and steady, hard to disturb
    Calm,
    /// Nervous and vigilant, high alertness
    Anxious,
}

impl PersonalityPreset {
    /// Creates a Personality from this preset
    pub fn to_personality(self) -> Personality {
        match self {
            PersonalityPreset::Mellow => {
                let mut builder = Personality::builder()
                    .id("preset-mellow")
                    .name("Mellow")
                    .icon('*')
                    .tension_baseline(0.2)
                    .coherence_baseline(0.7)
                    .energy_baseline(0.4)
                    .startle_sensitivity(0.2)
                    .recovery_speed(0.8)
                    .curiosity_drive(0.4)
                    .movement_expressiveness(0.3)
                    .sound_expressiveness(0.2)
                    .light_expressiveness(0.5);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Mellow preset should be valid")
            }

            PersonalityPreset::Curious => {
                let mut builder = Personality::builder()
                    .id("preset-curious")
                    .name("Curious")
                    .icon('?')
                    .tension_baseline(0.4)
                    .coherence_baseline(0.6)
                    .energy_baseline(0.7)
                    .startle_sensitivity(0.6)
                    .recovery_speed(0.5)
                    .curiosity_drive(0.9)
                    .movement_expressiveness(0.7)
                    .sound_expressiveness(0.6)
                    .light_expressiveness(0.8);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Curious preset should be valid")
            }

            PersonalityPreset::Zen => {
                let mut builder = Personality::builder()
                    .id("preset-zen")
                    .name("Zen")
                    .icon('O')
                    .tension_baseline(0.1)
                    .coherence_baseline(0.9)
                    .energy_baseline(0.3)
                    .startle_sensitivity(0.1)
                    .recovery_speed(0.9)
                    .curiosity_drive(0.3)
                    .movement_expressiveness(0.2)
                    .sound_expressiveness(0.1)
                    .light_expressiveness(0.3);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Zen preset should be valid")
            }

            PersonalityPreset::Excitable => {
                let mut builder = Personality::builder()
                    .id("preset-excitable")
                    .name("Excitable")
                    .icon('!')
                    .tension_baseline(0.6)
                    .coherence_baseline(0.5)
                    .energy_baseline(0.9)
                    .startle_sensitivity(0.8)
                    .recovery_speed(0.4)
                    .curiosity_drive(0.8)
                    .movement_expressiveness(0.9)
                    .sound_expressiveness(0.9)
                    .light_expressiveness(0.9);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Excitable preset should be valid")
            }

            PersonalityPreset::Timid => {
                let mut builder = Personality::builder()
                    .id("preset-timid")
                    .name("Timid")
                    .icon('.')
                    .tension_baseline(0.5)
                    .coherence_baseline(0.4)
                    .energy_baseline(0.3)
                    .startle_sensitivity(0.9)
                    .recovery_speed(0.2)
                    .curiosity_drive(0.4)
                    .movement_expressiveness(0.4)
                    .sound_expressiveness(0.2)
                    .light_expressiveness(0.4);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Timid preset should be valid")
            }

            PersonalityPreset::Adventurous => {
                let mut builder = Personality::builder()
                    .id("preset-adventurous")
                    .name("Adventurous")
                    .icon('+')
                    .tension_baseline(0.3)
                    .coherence_baseline(0.7)
                    .energy_baseline(0.8)
                    .startle_sensitivity(0.3)
                    .recovery_speed(0.7)
                    .curiosity_drive(0.9)
                    .movement_expressiveness(0.8)
                    .sound_expressiveness(0.7)
                    .light_expressiveness(0.8);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Adventurous preset should be valid")
            }

            PersonalityPreset::Shy => {
                let mut builder = Personality::builder()
                    .id("preset-shy")
                    .name("Shy")
                    .icon('-')
                    .tension_baseline(0.6)
                    .coherence_baseline(0.5)
                    .energy_baseline(0.3)
                    .startle_sensitivity(0.8)
                    .recovery_speed(0.3)
                    .curiosity_drive(0.5)
                    .movement_expressiveness(0.2)
                    .sound_expressiveness(0.1)
                    .light_expressiveness(0.3);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Shy preset should be valid")
            }

            PersonalityPreset::Grumpy => {
                let mut builder = Personality::builder()
                    .id("preset-grumpy")
                    .name("Grumpy")
                    .icon('#')
                    .tension_baseline(0.7)
                    .coherence_baseline(0.3)
                    .energy_baseline(0.4)
                    .startle_sensitivity(0.7)
                    .recovery_speed(0.3)
                    .curiosity_drive(0.2)
                    .movement_expressiveness(0.5)
                    .sound_expressiveness(0.4)
                    .light_expressiveness(0.4);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Grumpy preset should be valid")
            }

            PersonalityPreset::Cheerful => {
                let mut builder = Personality::builder()
                    .id("preset-cheerful")
                    .name("Cheerful")
                    .icon('^')
                    .tension_baseline(0.2)
                    .coherence_baseline(0.8)
                    .energy_baseline(0.8)
                    .startle_sensitivity(0.4)
                    .recovery_speed(0.8)
                    .curiosity_drive(0.7)
                    .movement_expressiveness(0.8)
                    .sound_expressiveness(0.9)
                    .light_expressiveness(0.9);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Cheerful preset should be valid")
            }

            PersonalityPreset::Cautious => {
                let mut builder = Personality::builder()
                    .id("preset-cautious")
                    .name("Cautious")
                    .icon('~')
                    .tension_baseline(0.5)
                    .coherence_baseline(0.7)
                    .energy_baseline(0.5)
                    .startle_sensitivity(0.6)
                    .recovery_speed(0.5)
                    .curiosity_drive(0.6)
                    .movement_expressiveness(0.4)
                    .sound_expressiveness(0.3)
                    .light_expressiveness(0.5);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Cautious preset should be valid")
            }

            PersonalityPreset::Playful => {
                let mut builder = Personality::builder()
                    .id("preset-playful")
                    .name("Playful")
                    .icon('@')
                    .tension_baseline(0.3)
                    .coherence_baseline(0.6)
                    .energy_baseline(0.9)
                    .startle_sensitivity(0.5)
                    .recovery_speed(0.9)
                    .curiosity_drive(0.8)
                    .movement_expressiveness(0.9)
                    .sound_expressiveness(0.8)
                    .light_expressiveness(0.8);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Playful preset should be valid")
            }

            PersonalityPreset::Serious => {
                let mut builder = Personality::builder()
                    .id("preset-serious")
                    .name("Serious")
                    .icon('=')
                    .tension_baseline(0.4)
                    .coherence_baseline(0.9)
                    .energy_baseline(0.6)
                    .startle_sensitivity(0.3)
                    .recovery_speed(0.6)
                    .curiosity_drive(0.6)
                    .movement_expressiveness(0.5)
                    .sound_expressiveness(0.4)
                    .light_expressiveness(0.5);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Serious preset should be valid")
            }

            PersonalityPreset::Energetic => {
                let mut builder = Personality::builder()
                    .id("preset-energetic")
                    .name("Energetic")
                    .icon('&')
                    .tension_baseline(0.5)
                    .coherence_baseline(0.6)
                    .energy_baseline(1.0)
                    .startle_sensitivity(0.6)
                    .recovery_speed(0.7)
                    .curiosity_drive(0.8)
                    .movement_expressiveness(1.0)
                    .sound_expressiveness(0.9)
                    .light_expressiveness(0.9);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Energetic preset should be valid")
            }

            PersonalityPreset::Calm => {
                let mut builder = Personality::builder()
                    .id("preset-calm")
                    .name("Calm")
                    .icon('_')
                    .tension_baseline(0.1)
                    .coherence_baseline(0.9)
                    .energy_baseline(0.4)
                    .startle_sensitivity(0.2)
                    .recovery_speed(0.9)
                    .curiosity_drive(0.5)
                    .movement_expressiveness(0.3)
                    .sound_expressiveness(0.2)
                    .light_expressiveness(0.4);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Calm preset should be valid")
            }

            PersonalityPreset::Anxious => {
                let mut builder = Personality::builder()
                    .id("preset-anxious")
                    .name("Anxious")
                    .icon('%')
                    .tension_baseline(0.8)
                    .coherence_baseline(0.3)
                    .energy_baseline(0.6)
                    .startle_sensitivity(1.0)
                    .recovery_speed(0.2)
                    .curiosity_drive(0.4)
                    .movement_expressiveness(0.6)
                    .sound_expressiveness(0.5)
                    .light_expressiveness(0.6);

                for quirk in self.default_quirks() {
                    builder = builder.quirk(*quirk);
                }

                builder.build().expect("Anxious preset should be valid")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === I-PERS-001: Parameter Bounds Tests ===

    #[test]
    fn test_valid_parameter_bounds() {
        let mut p = Personality::default();

        // Test boundary values
        assert!(p.set_tension_baseline(0.0).is_ok());
        assert!(p.set_tension_baseline(1.0).is_ok());
        assert!(p.set_tension_baseline(0.5).is_ok());

        // Test all parameters
        assert!(p.set_coherence_baseline(0.7).is_ok());
        assert!(p.set_energy_baseline(0.6).is_ok());
        assert!(p.set_startle_sensitivity(0.3).is_ok());
        assert!(p.set_recovery_speed(0.8).is_ok());
        assert!(p.set_curiosity_drive(0.9).is_ok());
        assert!(p.set_movement_expressiveness(0.4).is_ok());
        assert!(p.set_sound_expressiveness(0.5).is_ok());
        assert!(p.set_light_expressiveness(0.6).is_ok());
    }

    #[test]
    fn test_invalid_parameter_above_bounds() {
        let mut p = Personality::default();

        let result = p.set_tension_baseline(1.5);
        assert!(matches!(
            result,
            Err(PersonalityError::ParameterOutOfBounds {
                parameter: "tension_baseline",
                value: 1.5
            })
        ));
    }

    #[test]
    fn test_invalid_parameter_below_bounds() {
        let mut p = Personality::default();

        let result = p.set_tension_baseline(-0.1);
        assert!(matches!(
            result,
            Err(PersonalityError::ParameterOutOfBounds {
                parameter: "tension_baseline",
                ..
            })
        ));
    }

    #[test]
    fn test_validate_catches_invalid() {
        // Use the builder to test validation
        let result = Personality::builder()
            .id("test")
            .name("Test")
            .tension_baseline(1.5)
            .build();

        assert!(result.is_err());
    }

    // === I-PERS-002: Serialization Tests ===

    #[cfg(feature = "std")]
    #[test]
    fn test_json_serialization() {
        let p = Personality::builder()
            .id("test-id")
            .name("Test Bot")
            .icon('T')
            .tension_baseline(0.7)
            .coherence_baseline(0.5)
            .energy_baseline(0.6)
            .quirk("wiggle")
            .sound_pack("beeps")
            .timestamps(1000, 2000)
            .build()
            .unwrap();

        let json = p.to_json().unwrap();

        assert!(json.contains("\"tension_baseline\":0.7"));
        assert!(json.contains("\"name\":\"Test Bot\""));
        assert!(json.contains("\"quirks\":[\"wiggle\"]"));
        assert!(json.contains("\"sound_pack\":\"beeps\""));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_json_deserialization() {
        let json = r#"{"id":"test","name":"Test","icon":"T","version":1,"created_at":0,"modified_at":0,"tension_baseline":0.7,"coherence_baseline":0.5,"energy_baseline":0.6,"startle_sensitivity":0.5,"recovery_speed":0.5,"curiosity_drive":0.5,"movement_expressiveness":0.5,"sound_expressiveness":0.5,"light_expressiveness":0.5,"quirks":[],"sound_pack":null}"#;

        let p = Personality::from_json(json).unwrap();

        assert_eq!(p.id, "test");
        assert_eq!(p.name, "Test");
        assert!((p.tension_baseline() - 0.7).abs() < 0.001);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_json_roundtrip() {
        let original = Personality::builder()
            .id("roundtrip-test")
            .name("Roundtrip")
            .icon('R')
            .tension_baseline(0.3)
            .coherence_baseline(0.7)
            .energy_baseline(0.5)
            .startle_sensitivity(0.4)
            .recovery_speed(0.6)
            .curiosity_drive(0.8)
            .movement_expressiveness(0.2)
            .sound_expressiveness(0.9)
            .light_expressiveness(0.1)
            .quirk("bounce")
            .quirk("spin")
            .sound_pack("chirps")
            .timestamps(12345, 67890)
            .build()
            .unwrap();

        let json = original.to_json().unwrap();
        let restored = Personality::from_json(&json).unwrap();

        assert_eq!(original.id, restored.id);
        assert_eq!(original.name, restored.name);
        assert!((original.tension_baseline() - restored.tension_baseline()).abs() < 0.001);
        assert!((original.coherence_baseline() - restored.coherence_baseline()).abs() < 0.001);
        assert!((original.energy_baseline() - restored.energy_baseline()).abs() < 0.001);
        assert!((original.startle_sensitivity() - restored.startle_sensitivity()).abs() < 0.001);
        assert!((original.recovery_speed() - restored.recovery_speed()).abs() < 0.001);
        assert!((original.curiosity_drive() - restored.curiosity_drive()).abs() < 0.001);
        assert!(
            (original.movement_expressiveness() - restored.movement_expressiveness()).abs() < 0.001
        );
        assert!((original.sound_expressiveness() - restored.sound_expressiveness()).abs() < 0.001);
        assert!((original.light_expressiveness() - restored.light_expressiveness()).abs() < 0.001);
        assert_eq!(original.quirks, restored.quirks);
        assert_eq!(original.sound_pack, restored.sound_pack);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_deserialization_clamps_invalid_values() {
        // JSON with out-of-bounds values
        let json = r#"{"id":"test","name":"Test","icon":"T","version":1,"created_at":0,"modified_at":0,"tension_baseline":1.5,"coherence_baseline":-0.5,"energy_baseline":0.5,"startle_sensitivity":0.5,"recovery_speed":0.5,"curiosity_drive":0.5,"movement_expressiveness":0.5,"sound_expressiveness":0.5,"light_expressiveness":0.5,"quirks":[],"sound_pack":null}"#;

        let p = Personality::from_json(json).unwrap();

        // Should be clamped to valid range
        assert!((p.tension_baseline() - 1.0).abs() < 0.001);
        assert!((p.coherence_baseline() - 0.0).abs() < 0.001);
    }

    // === I-PERS-003: Default Values Tests ===

    #[test]
    fn test_default_has_neutral_values() {
        let p = Personality::default();

        assert!((p.tension_baseline() - 0.5).abs() < 0.001);
        assert!((p.coherence_baseline() - 0.5).abs() < 0.001);
        assert!((p.energy_baseline() - 0.5).abs() < 0.001);
        assert!((p.startle_sensitivity() - 0.5).abs() < 0.001);
        assert!((p.recovery_speed() - 0.5).abs() < 0.001);
        assert!((p.curiosity_drive() - 0.5).abs() < 0.001);
        assert!((p.movement_expressiveness() - 0.5).abs() < 0.001);
        assert!((p.sound_expressiveness() - 0.5).abs() < 0.001);
        assert!((p.light_expressiveness() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_default_is_valid() {
        let p = Personality::default();
        assert!(p.validate().is_ok());
    }

    // === Builder Tests ===

    #[test]
    fn test_builder_pattern() {
        let p = Personality::builder()
            .id("builder-test")
            .name("Builder Test")
            .icon('B')
            .tension_baseline(0.3)
            .curiosity_drive(0.9)
            .build()
            .unwrap();

        assert_eq!(p.id, "builder-test");
        assert_eq!(p.name, "Builder Test");
        assert!((p.tension_baseline() - 0.3).abs() < 0.001);
        assert!((p.curiosity_drive() - 0.9).abs() < 0.001);
        // Other values should be default
        assert!((p.coherence_baseline() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_builder_collects_errors() {
        let result = Personality::builder()
            .id("test")
            .name("Test")
            .tension_baseline(1.5) // Invalid
            .build();

        assert!(matches!(
            result,
            Err(PersonalityError::ParameterOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_builder_validates_empty_id() {
        let result = Personality::builder().id("").name("Test").build();

        assert!(matches!(result, Err(PersonalityError::EmptyId)));
    }

    #[test]
    fn test_builder_validates_empty_name() {
        let result = Personality::builder().id("test").name("").build();

        assert!(matches!(result, Err(PersonalityError::EmptyName)));
    }

    // === Preset Tests ===

    #[test]
    fn test_all_presets_are_valid() {
        for preset in PersonalityPreset::all() {
            let p = preset.to_personality();
            assert!(p.validate().is_ok(), "Preset {:?} should be valid", preset);
        }
    }

    #[test]
    fn test_preset_characteristics() {
        let mellow = PersonalityPreset::Mellow.to_personality();
        assert!(mellow.tension_baseline() < 0.3);
        assert!(mellow.recovery_speed() > 0.7);

        let curious = PersonalityPreset::Curious.to_personality();
        assert!(curious.curiosity_drive() > 0.8);

        let zen = PersonalityPreset::Zen.to_personality();
        assert!(zen.coherence_baseline() > 0.8);
        assert!(zen.tension_baseline() < 0.2);

        let excitable = PersonalityPreset::Excitable.to_personality();
        assert!(excitable.energy_baseline() > 0.8);
        assert!(excitable.movement_expressiveness() > 0.8);

        let timid = PersonalityPreset::Timid.to_personality();
        assert!(timid.startle_sensitivity() > 0.8);
        assert!(timid.recovery_speed() < 0.3);
    }

    // === Error Display Tests ===

    #[test]
    fn test_error_display() {
        let err = PersonalityError::ParameterOutOfBounds {
            parameter: "tension_baseline",
            value: 1.5,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("out of bounds"));
        assert!(msg.contains("tension_baseline"));
        assert!(msg.contains("1.5"));
    }

    // === Constructor Tests ===

    #[test]
    fn test_new_with_valid_params() {
        let p = Personality::new("my-id", "My Name").unwrap();
        assert_eq!(p.id, "my-id");
        assert_eq!(p.name, "My Name");
    }

    #[test]
    fn test_new_rejects_empty_id() {
        let result = Personality::new("", "Name");
        assert!(matches!(result, Err(PersonalityError::EmptyId)));
    }

    #[test]
    fn test_new_rejects_empty_name() {
        let result = Personality::new("id", "");
        assert!(matches!(result, Err(PersonalityError::EmptyName)));
    }
}
