//! Mood-to-Line-Style Mapping - STORY-ART-002 Implementation
//!
//! Maps personality tension levels to drawing line properties.
//! Implements ART-002 contract: personality tension → line characteristics
//!
//! # Invariants enforced:
//! - I-ART-002: Line properties must be within hardware constraints
//! - ART-002: Style strictly derives from tension baseline
//! - ARCH-001: no_std compatible
//! - ARCH-002: Deterministic (same tension = same style)

#[cfg(feature = "no_std")]
extern crate alloc;

#[cfg(feature = "no_std")]
use alloc::string::String;

#[cfg(not(feature = "no_std"))]
#[allow(unused_imports)]
use std::string::String;

use core::fmt;

/// Error type for style generation failures
#[derive(Debug, Clone, PartialEq)]
pub enum StyleError {
    /// Tension value out of valid range [0.0, 1.0]
    InvalidTension { value: f32 },
    /// Line width out of hardware constraints
    InvalidLineWidth { width: f32 },
    /// Waviness parameter out of bounds
    InvalidWaviness { value: f32 },
}

impl fmt::Display for StyleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StyleError::InvalidTension { value } => {
                write!(f, "tension out of bounds: {} (must be 0.0-1.0)", value)
            }
            StyleError::InvalidLineWidth { width } => {
                write!(f, "line width out of bounds: {} (must be 0.5-2.0mm)", width)
            }
            StyleError::InvalidWaviness { value } => {
                write!(f, "waviness out of bounds: {} (must be 0.0-1.0)", value)
            }
        }
    }
}

// ============================================
// Drawing Style Constants
// ============================================

/// Hardware constraint: minimum pen width in mm
pub const MIN_LINE_WIDTH_MM: f32 = 0.5;

/// Hardware constraint: maximum pen width in mm
pub const MAX_LINE_WIDTH_MM: f32 = 2.0;

/// Minimum draw speed (0-100)
pub const MIN_SPEED: u8 = 10;

/// Maximum draw speed (0-100)
pub const MAX_SPEED: u8 = 100;

/// Minimum waviness amplitude in mm
pub const MIN_WAVINESS_MM: f32 = 0.0;

/// Maximum waviness amplitude in mm
pub const MAX_WAVINESS_MM: f32 = 5.0;

// ============================================
// Line Style Definition
// ============================================

/// Line drawing properties derived from personality tension
///
/// Maps personality mood to visual drawing characteristics:
/// - Calm (tension < 0.3): Smooth, thin lines, slow speed
/// - Active (tension 0.3-0.6): Medium properties, moderate speed
/// - Spike (tension 0.6-0.85): Thick, wavy lines, fast speed
/// - Protect (tension > 0.85): Jagged, very thick, very slow
#[derive(Debug, Clone, PartialEq)]
pub struct LineStyle {
    /// Pen width in millimeters (0.5-2.0mm)
    pub line_width: f32,

    /// Waviness amplitude in millimeters (0.0-5.0mm)
    /// Higher = more erratic/wavy lines
    pub waviness: f32,

    /// Drawing speed (0-100, where 100 is max)
    /// Lower speeds produce more deliberate lines
    pub speed: u8,

    /// Frequency of direction changes in Hz
    /// Higher = more frequent oscillations
    pub oscillation_freq: f32,

    /// Pressure variation (0.0-1.0)
    /// 0.0 = constant pressure, 1.0 = maximum variation
    pub pressure_variation: f32,

    /// Description of the mood/style
    pub mood_label: &'static str,
}

impl LineStyle {
    /// Generates a style from personality tension (0.0-1.0)
    ///
    /// Maps tension levels to emotional styles:
    /// - 0.0-0.3: Calm - Smooth, thin, slow
    /// - 0.3-0.6: Active - Moderate properties
    /// - 0.6-0.85: Spike - Thick, wavy, fast
    /// - 0.85-1.0: Protect - Jagged, very thick, very slow
    ///
    /// # Errors
    /// Returns `StyleError::InvalidTension` if tension not in [0.0, 1.0]
    ///
    /// # Example
    /// ```
    /// use mbot_core::artbot::styles::LineStyle;
    ///
    /// let calm_style = LineStyle::from_tension(0.1).unwrap();
    /// assert!(calm_style.line_width < 1.0);
    /// assert_eq!(calm_style.mood_label, "Calm");
    ///
    /// let active_style = LineStyle::from_tension(0.45).unwrap();
    /// assert_eq!(active_style.mood_label, "Active");
    ///
    /// let spike_style = LineStyle::from_tension(0.75).unwrap();
    /// assert!(spike_style.line_width > 1.5);
    /// assert_eq!(spike_style.mood_label, "Spike");
    /// ```
    pub fn from_tension(tension: f32) -> Result<Self, StyleError> {
        if tension < 0.0 || tension > 1.0 {
            return Err(StyleError::InvalidTension { value: tension });
        }

        // Map tension to mood categories
        let (mood_label, base_width, waviness, speed, osc_freq, pressure_var) = if tension < 0.3 {
            // CALM: Smooth, thin, slow, deliberate
            ("Calm", 0.6, 0.1, 30, 0.5, 0.1)
        } else if tension < 0.6 {
            // ACTIVE: Moderate everything, searching
            ("Active", 1.0, 0.8, 55, 1.5, 0.3)
        } else if tension < 0.85 {
            // SPIKE: Thick, wavy, fast, excited
            ("Spike", 1.6, 2.5, 85, 3.5, 0.6)
        } else {
            // PROTECT: Jagged, defensive, careful
            ("Protect", 1.8, 3.5, 25, 5.0, 0.8)
        };

        Ok(LineStyle {
            line_width: base_width,
            waviness,
            speed,
            oscillation_freq: osc_freq,
            pressure_variation: pressure_var,
            mood_label,
        })
    }

    /// Validates that all line properties are within hardware constraints
    pub fn validate(&self) -> Result<(), StyleError> {
        if self.line_width < MIN_LINE_WIDTH_MM || self.line_width > MAX_LINE_WIDTH_MM {
            return Err(StyleError::InvalidLineWidth {
                width: self.line_width,
            });
        }

        if self.waviness < MIN_WAVINESS_MM || self.waviness > MAX_WAVINESS_MM {
            return Err(StyleError::InvalidWaviness {
                value: self.waviness,
            });
        }

        Ok(())
    }

    /// Returns the pen pressure (0-100) for current style
    /// Higher pressure = darker drawing
    pub fn pressure(&self) -> u8 {
        match self.mood_label {
            "Calm" => 40,      // Light touch
            "Active" => 60,    // Medium pressure
            "Spike" => 80,     // Heavy pressure
            "Protect" => 90,   // Very heavy, defensive
            _ => 50,
        }
    }

    /// Calculates the drawing time multiplier (1.0 = normal speed)
    /// Used to scale expected drawing time based on style
    pub fn time_multiplier(&self) -> f32 {
        (self.speed as f32) / 50.0  // Normalize around speed=50
    }
}

impl Default for LineStyle {
    /// Default style: Calm, neutral properties
    fn default() -> Self {
        Self {
            line_width: 0.8,
            waviness: 0.2,
            speed: 50,
            oscillation_freq: 1.0,
            pressure_variation: 0.2,
            mood_label: "Neutral",
        }
    }
}

// ============================================
// Style Interpolation for Smooth Transitions
// ============================================

/// Smoothly interpolates between two line styles
/// Used when mood changes to create flowing style transitions
///
/// # Example
/// ```
/// use mbot_core::artbot::styles::LineStyle;
///
/// let calm = LineStyle::from_tension(0.2).unwrap();
/// let active = LineStyle::from_tension(0.5).unwrap();
///
/// // 30% of the way from calm to active
/// let transition = interpolate_styles(&calm, &active, 0.3);
/// ```
pub fn interpolate_styles(from: &LineStyle, to: &LineStyle, t: f32) -> LineStyle {
    let t = t.clamp(0.0, 1.0);

    LineStyle {
        line_width: from.line_width + (to.line_width - from.line_width) * t,
        waviness: from.waviness + (to.waviness - from.waviness) * t,
        speed: (from.speed as f32 + (to.speed as f32 - from.speed as f32) * t) as u8,
        oscillation_freq: from.oscillation_freq + (to.oscillation_freq - from.oscillation_freq) * t,
        pressure_variation: from.pressure_variation
            + (to.pressure_variation - from.pressure_variation) * t,
        mood_label: if t < 0.5 { from.mood_label } else { to.mood_label },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === I-ART-002: Style Derivation from Tension ===

    #[test]
    fn test_calm_style_properties() {
        let style = LineStyle::from_tension(0.1).unwrap();

        assert_eq!(style.mood_label, "Calm");
        assert!(style.line_width < 1.0);  // Thin
        assert!(style.waviness < 0.5);    // Smooth
        assert!(style.speed < 50);        // Slow
    }

    #[test]
    fn test_active_style_properties() {
        let style = LineStyle::from_tension(0.45).unwrap();

        assert_eq!(style.mood_label, "Active");
        assert!(style.line_width >= 0.9 && style.line_width <= 1.1);  // Medium
        assert!(style.waviness >= 0.5 && style.waviness <= 1.5);      // Moderate
        assert!(style.speed >= 50 && style.speed <= 60);              // Medium speed
    }

    #[test]
    fn test_spike_style_properties() {
        let style = LineStyle::from_tension(0.75).unwrap();

        assert_eq!(style.mood_label, "Spike");
        assert!(style.line_width > 1.5);  // Thick
        assert!(style.waviness > 2.0);    // Wavy
        assert!(style.speed > 80);        // Fast
    }

    #[test]
    fn test_protect_style_properties() {
        let style = LineStyle::from_tension(0.9).unwrap();

        assert_eq!(style.mood_label, "Protect");
        assert!(style.line_width > 1.7);  // Very thick
        assert!(style.waviness > 3.0);    // Very wavy
        assert!(style.speed < 30);        // Very slow
    }

    #[test]
    fn test_tension_boundaries() {
        // Test exact boundary values
        let calm_upper = LineStyle::from_tension(0.29).unwrap();
        assert_eq!(calm_upper.mood_label, "Calm");

        let active_lower = LineStyle::from_tension(0.30).unwrap();
        assert_eq!(active_lower.mood_label, "Active");

        let active_upper = LineStyle::from_tension(0.59).unwrap();
        assert_eq!(active_upper.mood_label, "Active");

        let spike_lower = LineStyle::from_tension(0.60).unwrap();
        assert_eq!(spike_lower.mood_label, "Spike");
    }

    #[test]
    fn test_invalid_tension_below_zero() {
        let result = LineStyle::from_tension(-0.1);
        assert!(matches!(result, Err(StyleError::InvalidTension { .. })));
    }

    #[test]
    fn test_invalid_tension_above_one() {
        let result = LineStyle::from_tension(1.1);
        assert!(matches!(result, Err(StyleError::InvalidTension { .. })));
    }

    #[test]
    fn test_tension_boundary_values() {
        assert!(LineStyle::from_tension(0.0).is_ok());
        assert!(LineStyle::from_tension(1.0).is_ok());
    }

    #[test]
    fn test_style_validation() {
        let mut style = LineStyle::from_tension(0.5).unwrap();

        // Valid by default
        assert!(style.validate().is_ok());

        // Invalid line width
        style.line_width = 3.0;
        assert!(matches!(
            style.validate(),
            Err(StyleError::InvalidLineWidth { .. })
        ));

        style.line_width = 1.0;
        assert!(style.validate().is_ok());

        // Invalid waviness
        style.waviness = 10.0;
        assert!(matches!(
            style.validate(),
            Err(StyleError::InvalidWaviness { .. })
        ));
    }

    #[test]
    fn test_pressure_correlates_with_tension() {
        let calm = LineStyle::from_tension(0.1).unwrap();
        let active = LineStyle::from_tension(0.45).unwrap();
        let spike = LineStyle::from_tension(0.75).unwrap();

        assert!(calm.pressure() < active.pressure());
        assert!(active.pressure() < spike.pressure());
    }

    #[test]
    fn test_time_multiplier() {
        let style = LineStyle::from_tension(0.5).unwrap();

        let multiplier = style.time_multiplier();
        assert!(multiplier > 0.0);

        // Speed varies by mood, multiplier is normalized around speed 50
        // Active style (tension 0.5) has speed 55, so multiplier should be ~1.1
        assert!(multiplier > 0.5 && multiplier < 2.0);
    }

    #[test]
    fn test_default_style_is_valid() {
        let style = LineStyle::default();
        assert!(style.validate().is_ok());
    }

    #[test]
    fn test_interpolation_at_boundaries() {
        let calm = LineStyle::from_tension(0.2).unwrap();
        let active = LineStyle::from_tension(0.5).unwrap();

        // t=0 should match 'from'
        let start = interpolate_styles(&calm, &active, 0.0);
        assert!((start.line_width - calm.line_width).abs() < 0.001);

        // t=1 should match 'to'
        let end = interpolate_styles(&calm, &active, 1.0);
        assert!((end.line_width - active.line_width).abs() < 0.001);
    }

    #[test]
    fn test_interpolation_midpoint() {
        let calm = LineStyle::from_tension(0.2).unwrap();
        let active = LineStyle::from_tension(0.5).unwrap();

        let mid = interpolate_styles(&calm, &active, 0.5);

        // Should be approximately halfway
        let expected_width = (calm.line_width + active.line_width) / 2.0;
        assert!((mid.line_width - expected_width).abs() < 0.01);
    }

    #[test]
    fn test_interpolation_clamps_t() {
        let style1 = LineStyle::from_tension(0.2).unwrap();
        let style2 = LineStyle::from_tension(0.5).unwrap();

        let result_below = interpolate_styles(&style1, &style2, -0.5);
        let result_at_zero = interpolate_styles(&style1, &style2, 0.0);
        assert!((result_below.line_width - result_at_zero.line_width).abs() < 0.001);

        let result_above = interpolate_styles(&style1, &style2, 1.5);
        let result_at_one = interpolate_styles(&style1, &style2, 1.0);
        assert!((result_above.line_width - result_at_one.line_width).abs() < 0.001);
    }

    #[test]
    fn test_style_determinism() {
        // Same tension should produce identical styles (deterministic per ARCH-002)
        let style1 = LineStyle::from_tension(0.42).unwrap();
        let style2 = LineStyle::from_tension(0.42).unwrap();

        assert_eq!(style1.mood_label, style2.mood_label);
        assert_eq!(style1.line_width, style2.line_width);
        assert_eq!(style1.waviness, style2.waviness);
        assert_eq!(style1.speed, style2.speed);
    }
}
