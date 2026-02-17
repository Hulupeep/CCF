//! Brain Planner - Queries LLM and translates responses to actions
//!
//! Invariants:
//! - I-BRAIN-004: LLM suggestions pass SafetyFilter before execution
//! - I-BRAIN-005: Motor speeds from LLM clamped [-100, 100]

#[cfg(feature = "brain")]
pub mod prompt_builder;
#[cfg(feature = "brain")]
pub mod action_translator;
#[cfg(feature = "brain")]
pub mod safety;

#[cfg(feature = "brain")]
use crate::brain::error::{BrainError, BrainResult};
#[cfg(feature = "brain")]
use crate::brain::llm::{LlmMessage, ProviderChain};
#[cfg(feature = "brain")]
use mbot_core::{HomeostasisState, MBotSensors, MotorCommand};
#[cfg(feature = "brain")]
use mbot_core::personality::Personality;

#[cfg(feature = "brain")]
use action_translator::ActionTranslator;
#[cfg(feature = "brain")]
use prompt_builder::PromptBuilder;
#[cfg(feature = "brain")]
use safety::SafetyFilter;

/// Exploration sub-commands the LLM can request.
#[cfg(feature = "brain")]
#[derive(Debug, Clone)]
pub enum ExploreCommand {
    /// Begin scanning surroundings.
    Scan,
    /// Drive toward target sector.
    MoveToSector(usize),
    /// Pause exploration for reflection.
    Pause,
    /// Resume exploration after pause.
    Resume,
}

/// Actions the brain can suggest
#[cfg(feature = "brain")]
#[derive(Debug, Clone)]
pub enum BrainAction {
    /// Override motor command (I-BRAIN-005: clamped to [-100, 100])
    Motor(MotorCommand),
    /// Adjust a personality parameter
    PersonalityAdjust { parameter: String, delta: f32 },
    /// Speak text (route to channels/voice)
    Speak(String),
    /// Start a named activity
    StartActivity(String),
    /// Exploration command
    Explore(ExploreCommand),
    /// No action needed
    Noop,
}

/// Brain planner - coordinates LLM queries and action generation
#[cfg(feature = "brain")]
pub struct BrainPlanner {
    provider_chain: ProviderChain,
    prompt_builder: PromptBuilder,
    action_translator: ActionTranslator,
    safety_filter: SafetyFilter,
    query_interval_secs: u64,
}

#[cfg(feature = "brain")]
impl BrainPlanner {
    pub fn new(provider_chain: ProviderChain, query_interval_secs: u64) -> Self {
        Self {
            provider_chain,
            prompt_builder: PromptBuilder::new(),
            action_translator: ActionTranslator::new(),
            safety_filter: SafetyFilter::new(),
            query_interval_secs,
        }
    }

    /// Plan actions based on current state
    pub async fn plan(
        &mut self,
        state: &HomeostasisState,
        sensors: &MBotSensors,
        personality: &Personality,
    ) -> BrainResult<Vec<BrainAction>> {
        // Build the prompt from current state
        let messages = self.prompt_builder.build(state, sensors, personality);

        // Query LLM
        let response = self.provider_chain.complete(&messages).await?;

        // Parse LLM response into actions
        let raw_actions = self.action_translator.parse(&response.content);

        // I-BRAIN-004: Filter through safety before returning
        let safe_actions = raw_actions
            .into_iter()
            .filter_map(|action| self.safety_filter.check(action))
            .collect();

        Ok(safe_actions)
    }
}
