//! Personality Narrator - generates in-character responses

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
use mbot_core::personality::Personality;
#[cfg(feature = "brain")]
use templates::build_narrator_system_prompt;

/// Generates personality-colored responses to user input
#[cfg(feature = "brain")]
pub struct PersonalityNarrator {
    provider_chain: ProviderChain,
}

#[cfg(feature = "brain")]
impl PersonalityNarrator {
    pub fn new(provider_chain: ProviderChain) -> Self {
        Self { provider_chain }
    }

    /// Generate a personality-colored response to user text
    pub async fn respond(
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

    /// Generate a streaming response, calling on_token for each chunk
    pub async fn respond_streaming(
        &self,
        input: &str,
        personality: &Personality,
        state: &HomeostasisState,
        on_token: Box<dyn Fn(&str) + Send>,
    ) -> BrainResult<String> {
        let system_prompt = build_narrator_system_prompt(personality, state);

        let messages = vec![
            LlmMessage::system(system_prompt),
            LlmMessage::user(input.to_string()),
        ];

        let response = self.provider_chain.complete_streaming(&messages, on_token).await?;
        Ok(response.content)
    }
}
