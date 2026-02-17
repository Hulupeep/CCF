//! Consciousness-Like Reflection Engine
//!
//! Periodically generates phenomenological reflections via LLM,
//! inviting the robot to describe its experience rather than plan actions.
//!
//! Reflection is gated by permeability: if permeability < 0.2, the LLM call
//! is skipped entirely, saving compute and keeping behavior tight. Higher
//! permeability progressively unlocks deeper reflection.
//!
//! Safety: Reflections pass through SafetyFilter (no harmful content),
//! but are NOT censored for philosophical or experiential content.
//! The safety filter checks for harmful actions, not introspective speech.

#[cfg(feature = "brain")]
use crate::brain::error::BrainResult;
#[cfg(feature = "brain")]
use crate::brain::llm::{LlmMessage, ProviderChain};
#[cfg(feature = "brain")]
use crate::brain::narrator::templates::{build_depth_aware_system_prompt, depth_instruction};
#[cfg(feature = "brain")]
use crate::brain::planner::prompt_builder::ExplorationContext;
#[cfg(feature = "brain")]
use mbot_core::HomeostasisState;
#[cfg(feature = "brain")]
use mbot_core::coherence::NarrationDepth;
#[cfg(feature = "brain")]
use mbot_core::personality::Personality;
#[cfg(feature = "brain")]
use std::time::Instant;

/// How deep the reflection goes (legacy cycling enum).
///
/// This is still used internally for cycling between reflection styles.
/// The actual gating is done via [`NarrationDepth`] from permeability.
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
///
/// Gated by permeability: the [`reflect`] method takes a permeability value
/// and returns `None` when permeability is below the threshold for LLM reflection.
#[cfg(feature = "brain")]
pub struct ReflectionEngine {
    /// Seconds between reflections.
    pub reflection_interval_secs: u64,
    /// When the last reflection was generated.
    last_reflection: Instant,
    /// Current depth level (cycles through Shallow -> Medium -> Deep).
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

    /// Generate a reflection, gated by permeability.
    ///
    /// Returns `None` if permeability is too low (< 0.2) to justify an LLM call.
    /// The narration depth determines the style and length of the reflection:
    /// - Minimal: factual observations only
    /// - Brief: contextual awareness
    /// - Full: personality-colored
    /// - Deep: phenomenological reflection
    pub async fn reflect(
        &mut self,
        provider: &ProviderChain,
        personality: &Personality,
        state: &HomeostasisState,
        exploration: Option<&ExplorationContext>,
        permeability: f32,
    ) -> BrainResult<Option<String>> {
        let narration_depth = NarrationDepth::from_permeability(permeability);

        if narration_depth == NarrationDepth::None {
            tracing::debug!(
                "Reflection: permeability {:.2} < 0.2, skipping LLM reflection",
                permeability
            );
            // Still update the timer so we don't retry immediately
            self.last_reflection = Instant::now();
            return Ok(None);
        }

        self.last_reflection = Instant::now();
        self.reflection_count += 1;

        // Cycle the internal depth counter
        self.depth = match self.reflection_count % 5 {
            0 => ReflectionDepth::Deep,
            2 | 3 => ReflectionDepth::Medium,
            _ => ReflectionDepth::Shallow,
        };

        // Clamp the cycling depth by the narration depth ceiling.
        // E.g. at Minimal narration depth, even a cycling "Deep" reflection
        // should be constrained to factual observations.
        let effective_reflection_depth = self.clamp_reflection_depth(narration_depth);

        let system = self.build_reflection_system(personality, state, narration_depth, effective_reflection_depth);
        let user = self.build_reflection_prompt(state, exploration);

        let messages = vec![
            LlmMessage::system(system),
            LlmMessage::user(user),
        ];

        let response = provider.complete(&messages).await?;
        Ok(Some(response.content))
    }

    /// Clamp the cycling ReflectionDepth to what the NarrationDepth allows.
    fn clamp_reflection_depth(&self, narration_depth: NarrationDepth) -> ReflectionDepth {
        match narration_depth {
            NarrationDepth::None => ReflectionDepth::Shallow, // unreachable in practice
            NarrationDepth::Minimal => ReflectionDepth::Shallow,
            NarrationDepth::Brief => match self.depth {
                ReflectionDepth::Deep => ReflectionDepth::Medium,
                other => other,
            },
            NarrationDepth::Full | NarrationDepth::Deep => self.depth,
        }
    }

    fn build_reflection_system(
        &self,
        personality: &Personality,
        state: &HomeostasisState,
        narration_depth: NarrationDepth,
        effective_depth: ReflectionDepth,
    ) -> String {
        let base_tone = build_depth_aware_system_prompt(personality, state, narration_depth);

        let reflection_instruction = match effective_depth {
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

        let depth_constraint = depth_instruction(narration_depth);

        format!(
            "{}\n\n\
             REFLECTION MODE:\n\
             Instead of deciding what to do, take a moment to reflect on your experience.\n\
             {}\n\
             \n\
             DEPTH CONSTRAINT: {}\n\
             \n\
             Do not describe your sensors or technical state. Describe your EXPERIENCE.\n\
             Be genuine. Never mention being an AI or having sensors.\n\
             Use first person. Speak as yourself.",
            base_tone, reflection_instruction, depth_constraint
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
