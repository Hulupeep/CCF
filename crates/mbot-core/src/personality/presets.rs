//! Extended Preset Personality Library (#18)
//! 15 pre-configured personalities with quirks (#26)

use super::{Personality, PersonalityPreset};

impl PersonalityPreset {
    pub fn all() -> &'static [PersonalityPreset] {
        &[
            Self::Mellow, Self::Curious, Self::Zen, Self::Excitable, Self::Timid,
            Self::Adventurous, Self::Shy, Self::Grumpy, Self::Cheerful, Self::Cautious,
            Self::Playful, Self::Serious, Self::Energetic, Self::Calm, Self::Anxious,
        ]
    }

    /// Returns the default quirks for this preset personality
    pub fn default_quirks(self) -> &'static [&'static str] {
        match self {
            PersonalityPreset::Mellow => &["random_sigh"],
            PersonalityPreset::Curious => &["social_butterfly"],
            PersonalityPreset::Zen => &["random_sigh"],
            PersonalityPreset::Excitable => &["spin_when_happy", "chase_tail"],
            PersonalityPreset::Timid => &["back_up_when_scared", "hermit"],
            PersonalityPreset::Adventurous => &["social_butterfly"],
            PersonalityPreset::Shy => &["hermit", "back_up_when_scared"],
            PersonalityPreset::Grumpy => &["hermit"],
            PersonalityPreset::Cheerful => &["spin_when_happy"],
            PersonalityPreset::Cautious => &["back_up_when_scared"],
            PersonalityPreset::Playful => &["spin_when_happy", "chase_tail", "social_butterfly"],
            PersonalityPreset::Serious => &["collector_instinct"],
            PersonalityPreset::Energetic => &["spin_when_happy", "chase_tail", "early_bird"],
            PersonalityPreset::Calm => &["random_sigh", "night_owl"],
            PersonalityPreset::Anxious => &["back_up_when_scared", "hermit"],
        }
    }
}

// Tests confirm 10+ presets with valid parameters
