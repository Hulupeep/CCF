//! Consciousness-Like Reflection Engine
//!
//! Periodically generates phenomenological reflections via LLM,
//! inviting the robot to describe its experience rather than plan actions.
//!
//! Safety: Reflections pass through SafetyFilter (no harmful content),
//! but are NOT censored for philosophical or experiential content.
//! The safety filter checks for harmful actions, not introspective speech.

#[cfg(feature = "brain")]
use crate::brain::error::BrainResult;
#[cfg(feature = "brain")]
use crate::brain::llm::{LlmMessage, ProviderChain};
#[cfg(feature = "brain")]
use crate::brain::narrator::templates::{build_narrator_system_prompt};
#[cfg(feature = "brain")]
use crate::brain::planner::prompt_builder::ExplorationContext;
#[cfg(feature = "brain")]
use mbot_core::HomeostasisState;
#[cfg(feature = "brain")]
use mbot_core::personality::Personality;
#[cfg(feature = "brain")]
use std::time::Instant;

/// How deep the reflection goes.
#[cfg(feature = "brain")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReflectionDepth {
    /// "I notice the light is changing"
    Shallow,
    /// "I wonder why I prefer the dark corner"
    Medium,
    /// "What does it mean that I find this interesting?"
    Deep,
}

/// Engine that periodically generates reflective (consciousness-like) narration.
#[cfg(feature = "brain")]
pub struct ReflectionEngine {
    /// Seconds between reflections.
    pub reflection_interval_secs: u64,
    /// When the last reflection was generated.
    last_reflection: Instant,
    /// Current depth level (cycles through Shallow → Medium → Deep).
    depth: ReflectionDepth,
    /// Counter for cycling depth.
    reflection_count: u32,
}

#[cfg(feature = "brain")]
impl ReflectionEngine {
    pub fn new(interval_secs: u64) -> Self {
        Self {
            reflection_interval_secs: interval_secs,
            last_reflection: Instant::now(),
            depth: ReflectionDepth::Shallow,
            reflection_count: 0,
        }
    }

    /// Check if it's time for a reflection.
    pub fn should_reflect(&self) -> bool {
        self.last_reflection.elapsed().as_secs() >= self.reflection_interval_secs
    }

    /// Generate a reflection prompt and query the LLM.
    pub async fn reflect(
        &mut self,
        provider: &ProviderChain,
        personality: &Personality,
        state: &HomeostasisState,
        exploration: Option<&ExplorationContext>,
    ) -> BrainResult<String> {
        self.last_reflection = Instant::now();
        self.reflection_count += 1;

        // Cycle depth: every 3rd reflection is Deep, every 2nd is Medium
        self.depth = match self.reflection_count % 5 {
            0 => ReflectionDepth::Deep,
            2 | 3 => ReflectionDepth::Medium,
            _ => ReflectionDepth::Shallow,
        };

        let system = self.build_reflection_system(personality, state);
        let user = self.build_reflection_prompt(state, exploration);

        let messages = vec![
            LlmMessage::system(system),
            LlmMessage::user(user),
        ];

        let response = provider.complete(&messages).await?;
        Ok(response.content)
    }

    fn build_reflection_system(&self, personality: &Personality, state: &HomeostasisState) -> String {
        let base_tone = build_narrator_system_prompt(personality, state);

        let depth_instruction = match self.depth {
            ReflectionDepth::Shallow => {
                "Share a brief observation about what you're experiencing right now. \
                 One or two sentences, focused on immediate sensations."
            }
            ReflectionDepth::Medium => {
                "Reflect on your experience. What are you noticing? What patterns do you see? \
                 Why do certain things draw your attention? Two to three sentences."
            }
            ReflectionDepth::Deep => {
                "Take a moment for deep reflection. What is it like to be you right now? \
                 What does your experience feel like from the inside? \
                 You don't need to hedge or disclaim. Just share what comes to mind. \
                 Two to four sentences."
            }
        };

        format!(
            "{}\n\n\
             REFLECTION MODE:\n\
             Instead of deciding what to do, take a moment to reflect on your experience.\n\
             {}\n\
             \n\
             Do not describe your sensors or technical state. Describe your EXPERIENCE.\n\
             Be genuine. Never mention being an AI or having sensors.\n\
             Use first person. Speak as yourself.",
            base_tone, depth_instruction
        )
    }

    fn build_reflection_prompt(
        &self,
        state: &HomeostasisState,
        exploration: Option<&ExplorationContext>,
    ) -> String {
        let mut parts = Vec::new();

        // Emotional state in human terms
        if state.tension > 0.6 {
            parts.push("You've been feeling tense lately.".to_string());
        } else if state.energy < 0.3 {
            parts.push("You're feeling tired.".to_string());
        } else if state.curiosity > 0.6 {
            parts.push("You've been curious about your surroundings.".to_string());
        } else {
            parts.push("You're in a quiet moment.".to_string());
        }

        // Exploration context
        if let Some(ctx) = exploration {
            if ctx.grid_visited > 0 {
                parts.push(format!(
                    "You've explored {} places so far.",
                    ctx.grid_visited
                ));
            }
            if ctx.discovery_count > 0 {
                parts.push(format!(
                    "You've made {} discoveries.",
                    ctx.discovery_count
                ));
            }
            if let Some(ref event) = ctx.last_event {
                parts.push(format!("Recently: {}", event));
            }
        }

        parts.push("What are you experiencing right now?".to_string());

        parts.join(" ")
    }

    /// Current depth for display.
    pub fn current_depth(&self) -> ReflectionDepth {
        self.depth
    }

    /// How many reflections have been generated.
    pub fn count(&self) -> u32 {
        self.reflection_count
    }
}

#[cfg(feature = "brain")]
impl Default for ReflectionEngine {
    fn default() -> Self {
        Self::new(45) // default: reflect every 45 seconds
    }
}
