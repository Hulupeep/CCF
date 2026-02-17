//! Personality Narrator - generates in-character responses
//!
//! LLM calls are gated by permeability level via [`NarrationDepth`].
//! Low permeability (< 0.2) skips the LLM entirely, returning `None`.
//! Higher permeability levels produce progressively richer narration
//! from factual observations through full phenomenological reflection.

#[cfg(feature = "brain")]
pub mod templates;
#[cfg(feature = "brain")]
pub mod reflection;

#[cfg(feature = "brain")]
use crate::brain::error::BrainResult;
#[cfg(feature = "brain")]
use crate::brain::llm::{LlmMessage, ProviderChain};
#[cfg(feature = "brain")]
use mbot_core::HomeostasisState;
#[cfg(feature = "brain")]
use mbot_core::coherence::NarrationDepth;
#[cfg(feature = "brain")]
use mbot_core::personality::Personality;
#[cfg(feature = "brain")]
use templates::{build_narrator_system_prompt, build_depth_aware_system_prompt};

/// Generates personality-colored responses to user input, gated by permeability.
#[cfg(feature = "brain")]
pub struct PersonalityNarrator {
    provider_chain: ProviderChain,
}

#[cfg(feature = "brain")]
impl PersonalityNarrator {
    pub fn new(provider_chain: ProviderChain) -> Self {
        Self { provider_chain }
    }

    /// Compute the narration depth from a permeability value.
    ///
    /// This is a convenience wrapper around [`NarrationDepth::from_permeability`].
    pub fn depth_from_permeability(permeability: f32) -> NarrationDepth {
        NarrationDepth::from_permeability(permeability)
    }

    /// Generate a personality-colored response to user text, gated by permeability.
    ///
    /// Returns `None` if permeability is too low for LLM reflection (< 0.2).
    /// Otherwise adjusts the prompt style based on narration depth:
    /// - Minimal (0.2-0.4): factual observations only
    /// - Brief (0.4-0.6): contextual awareness
    /// - Full (0.6-0.8): personality-colored narration
    /// - Deep (> 0.8): full phenomenological reflection
    pub async fn respond(
        &self,
        input: &str,
        personality: &Personality,
        state: &HomeostasisState,
        permeability: f32,
    ) -> BrainResult<Option<String>> {
        let depth = NarrationDepth::from_permeability(permeability);

        if depth == NarrationDepth::None {
            tracing::debug!("Narrator: permeability {:.2} < 0.2, skipping LLM call", permeability);
            return Ok(None);
        }

        let system_prompt = build_depth_aware_system_prompt(personality, state, depth);

        let messages = vec![
            LlmMessage::system(system_prompt),
            LlmMessage::user(input.to_string()),
        ];

        let response = self.provider_chain.complete(&messages).await?;
        Ok(Some(response.content))
    }

    /// Generate a streaming response gated by permeability.
    ///
    /// Returns `None` if permeability is too low for LLM reflection (< 0.2).
    pub async fn respond_streaming(
        &self,
        input: &str,
        personality: &Personality,
        state: &HomeostasisState,
        permeability: f32,
        on_token: Box<dyn Fn(&str) + Send>,
    ) -> BrainResult<Option<String>> {
        let depth = NarrationDepth::from_permeability(permeability);

        if depth == NarrationDepth::None {
            tracing::debug!("Narrator: permeability {:.2} < 0.2, skipping streaming LLM call", permeability);
            return Ok(None);
        }

        let system_prompt = build_depth_aware_system_prompt(personality, state, depth);

        let messages = vec![
            LlmMessage::system(system_prompt),
            LlmMessage::user(input.to_string()),
        ];

        let response = self.provider_chain.complete_streaming(&messages, on_token).await?;
        Ok(Some(response.content))
    }

    /// Generate a response without permeability gating (legacy compatibility).
    ///
    /// Always makes an LLM call regardless of permeability. Use [`respond`]
    /// with a permeability parameter for gated behavior.
    pub async fn respond_ungated(
        &self,
        input: &str,
        personality: &Personality,
        state: &HomeostasisState,
    ) -> BrainResult<String> {
        let system_prompt = build_narrator_system_prompt(personality, state);

        let messages = vec![
            LlmMessage::system(system_prompt),
            LlmMessage::user(input.to_string()),
        ];

        let response = self.provider_chain.complete(&messages).await?;
        Ok(response.content)
    }
}

// ─── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #[cfg(feature = "brain")]
    use mbot_core::coherence::NarrationDepth;

    #[cfg(feature = "brain")]
    use super::PersonalityNarrator;

    #[test]
    #[cfg(feature = "brain")]
    fn test_depth_from_permeability_none() {
        assert_eq!(
            PersonalityNarrator::depth_from_permeability(0.0),
            NarrationDepth::None,
        );
        assert_eq!(
            PersonalityNarrator::depth_from_permeability(0.19),
            NarrationDepth::None,
        );
    }

    #[test]
    #[cfg(feature = "brain")]
    fn test_depth_from_permeability_minimal() {
        assert_eq!(
            PersonalityNarrator::depth_from_permeability(0.2),
            NarrationDepth::Minimal,
        );
        assert_eq!(
            PersonalityNarrator::depth_from_permeability(0.3),
            NarrationDepth::Minimal,
        );
    }

    #[test]
    #[cfg(feature = "brain")]
    fn test_depth_from_permeability_brief() {
        assert_eq!(
            PersonalityNarrator::depth_from_permeability(0.4),
            NarrationDepth::Brief,
        );
        assert_eq!(
            PersonalityNarrator::depth_from_permeability(0.5),
            NarrationDepth::Brief,
        );
    }

    #[test]
    #[cfg(feature = "brain")]
    fn test_depth_from_permeability_full() {
        assert_eq!(
            PersonalityNarrator::depth_from_permeability(0.6),
            NarrationDepth::Full,
        );
        assert_eq!(
            PersonalityNarrator::depth_from_permeability(0.7),
            NarrationDepth::Full,
        );
    }

    #[test]
    #[cfg(feature = "brain")]
    fn test_depth_from_permeability_deep() {
        assert_eq!(
            PersonalityNarrator::depth_from_permeability(0.8),
            NarrationDepth::Deep,
        );
        assert_eq!(
            PersonalityNarrator::depth_from_permeability(1.0),
            NarrationDepth::Deep,
        );
    }

    #[test]
    #[cfg(feature = "brain")]
    fn test_narration_depth_correctly_maps_from_permeability() {
        // Verify the full mapping range
        let cases: &[(f32, NarrationDepth)] = &[
            (0.0, NarrationDepth::None),
            (0.1, NarrationDepth::None),
            (0.19, NarrationDepth::None),
            (0.2, NarrationDepth::Minimal),
            (0.3, NarrationDepth::Minimal),
            (0.39, NarrationDepth::Minimal),
            (0.4, NarrationDepth::Brief),
            (0.5, NarrationDepth::Brief),
            (0.59, NarrationDepth::Brief),
            (0.6, NarrationDepth::Full),
            (0.7, NarrationDepth::Full),
            (0.79, NarrationDepth::Full),
            (0.8, NarrationDepth::Deep),
            (0.9, NarrationDepth::Deep),
            (1.0, NarrationDepth::Deep),
        ];

        for &(p, expected) in cases {
            let depth = PersonalityNarrator::depth_from_permeability(p);
            assert_eq!(
                depth, expected,
                "permeability {:.2} should map to {:?}, got {:?}",
                p, expected, depth
            );
        }
    }
}
