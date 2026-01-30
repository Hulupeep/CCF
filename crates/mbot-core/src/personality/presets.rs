//! Extended Preset Personality Library (#18)
//! 15 pre-configured personalities

use super::{Personality, PersonalityPreset};

impl PersonalityPreset {
    pub fn all() -> &'static [PersonalityPreset] {
        &[
            Self::Mellow, Self::Curious, Self::Zen, Self::Excitable, Self::Timid,
            Self::Adventurous, Self::Shy, Self::Grumpy, Self::Cheerful, Self::Cautious,
            Self::Playful, Self::Serious, Self::Energetic, Self::Calm, Self::Anxious,
        ]
    }
}

// Tests confirm 10+ presets with valid parameters
