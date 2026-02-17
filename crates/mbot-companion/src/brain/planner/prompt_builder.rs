//! Prompt Builder - constructs LLM prompts from robot state
//!
//! Supports depth-aware prompt construction via [`NarrationDepth`], allowing
//! the planner to adjust prompt verbosity based on permeability.

#[cfg(feature = "brain")]
use crate::brain::llm::LlmMessage;
#[cfg(feature = "brain")]
use mbot_core::{HomeostasisState, MBotSensors, ReflexMode};
#[cfg(feature = "brain")]
use mbot_core::coherence::NarrationDepth;
#[cfg(feature = "brain")]
use mbot_core::personality::Personality;

/// Exploration context for enriching LLM prompts.
#[cfg(feature = "brain")]
#[derive(Clone, Debug, Default)]
pub struct ExplorationContext {
    pub sectors_mapped: usize,
    pub sectors_total: usize,
    pub grid_visited: usize,
    pub grid_total: usize,
    pub current_phase: String,
    pub discovery_count: u32,
    pub episode_count: u32,
    pub nav_confidence: f32,
    pub last_event: Option<String>,
}

#[cfg(feature = "brain")]
pub struct PromptBuilder {
    system_prefix: String,
    exploration_context: Option<ExplorationContext>,
}

#[cfg(feature = "brain")]
impl PromptBuilder {
    pub fn new() -> Self {
        Self {
            system_prefix: String::from(
                "You are mBot2, a small educational robot. \
                 You have a personality and feelings driven by your homeostasis system. \
                 Based on your current state, decide what to do next.\n\
                 \n\
                 RESPOND with one of:\n\
                 - SPEAK: <text to say>\n\
                 - MOTOR: <left -100..100> <right -100..100>\n\
                 - ACTIVITY: <activity name>\n\
                 - ADJUST: <parameter> <delta -0.1..0.1>\n\
                 - EXPLORE: SCAN | MOVE <sector> | PAUSE | RESUME\n\
                 - NOOP\n\
                 \n\
                 Keep responses SHORT (1-2 sentences for SPEAK). \
                 Be safe - never suggest dangerous motor speeds. \
                 You are exploring a room - navigate carefully and narrate your discoveries."
            ),
            exploration_context: None,
        }
    }

    /// Set the exploration context for the next prompt build.
    pub fn set_exploration_context(&mut self, ctx: ExplorationContext) {
        self.exploration_context = Some(ctx);
    }

    /// Clear the exploration context.
    pub fn clear_exploration_context(&mut self) {
        self.exploration_context = None;
    }

    pub fn build(
        &self,
        state: &HomeostasisState,
        sensors: &MBotSensors,
        personality: &Personality,
    ) -> Vec<LlmMessage> {
        let system = format!(
            "{}\n\nYour personality:\n\
             - Curiosity drive: {:.2}\n\
             - Startle sensitivity: {:.2}\n\
             - Recovery speed: {:.2}\n\
             - Movement expressiveness: {:.2}\n\
             - Sound expressiveness: {:.2}\n\
             - Energy baseline: {:.2}",
            self.system_prefix,
            personality.curiosity_drive(),
            personality.startle_sensitivity(),
            personality.recovery_speed(),
            personality.movement_expressiveness(),
            personality.sound_expressiveness(),
            personality.energy_baseline(),
        );

        let explore_section = if let Some(ref ctx) = self.exploration_context {
            format!(
                "\n\
                 Exploration:\n\
                 - Phase: {}\n\
                 - Sectors mapped: {}/{} ({:.0}%)\n\
                 - Grid cells visited: {}/{}\n\
                 - Discoveries: {}\n\
                 - Episodes: {}\n\
                 - Navigation confidence: {:.2}\n\
                 {}\n",
                ctx.current_phase,
                ctx.sectors_mapped, ctx.sectors_total,
                if ctx.sectors_total > 0 { ctx.sectors_mapped as f32 / ctx.sectors_total as f32 * 100.0 } else { 0.0 },
                ctx.grid_visited, ctx.grid_total,
                ctx.discovery_count,
                ctx.episode_count,
                ctx.nav_confidence,
                ctx.last_event.as_deref().map(|e| format!("- Last event: {}", e)).unwrap_or_default(),
            )
        } else {
            String::new()
        };

        let user_msg = format!(
            "Current state:\n\
             - Reflex mode: {}\n\
             - Tension: {:.2}\n\
             - Coherence: {:.2}\n\
             - Energy: {:.2}\n\
             - Curiosity: {:.2}\n\
             \n\
             Sensors:\n\
             - Distance: {:.1} cm\n\
             - Sound level: {:.2}\n\
             - Light level: {:.2}\n\
             - Gyro Z: {:.1} deg/s\n\
             {}\
             \n\
             What should I do?",
            match state.reflex {
                ReflexMode::Calm => "Calm",
                ReflexMode::Active => "Active",
                ReflexMode::Spike => "Spike",
                ReflexMode::Protect => "Protect",
            },
            state.tension,
            state.coherence,
            state.energy,
            state.curiosity,
            sensors.ultrasonic_cm,
            sensors.sound_level,
            sensors.light_level,
            sensors.gyro_z,
            explore_section,
        );

        vec![
            LlmMessage::system(system),
            LlmMessage::user(user_msg),
        ]
    }

    /// Build depth-aware LLM prompts from robot state.
    ///
    /// The system prefix is adjusted based on [`NarrationDepth`]:
    /// - `None`: should not be called (caller skips LLM).
    /// - `Minimal`: sensor-only, no personality, restricted to MOTOR/NOOP.
    /// - `Brief`: includes context, limited actions (MOTOR, SPEAK briefly, NOOP).
    /// - `Full`: standard prompt with full personality and actions.
    /// - `Deep`: enriched prompt encouraging exploration narration.
    pub fn build_with_depth(
        &self,
        state: &HomeostasisState,
        sensors: &MBotSensors,
        personality: &Personality,
        depth: NarrationDepth,
    ) -> Vec<LlmMessage> {
        let depth_prefix = match depth {
            NarrationDepth::None => {
                // Should not reach here, but provide a minimal fallback
                "You are mBot2. Report sensor changes only. No personality. No emotions. \
                 RESPOND with: NOOP"
                    .to_string()
            }
            NarrationDepth::Minimal => {
                "You are mBot2, a small robot. Report sensor changes factually.\n\
                 \n\
                 RESPOND with one of:\n\
                 - MOTOR: <left -100..100> <right -100..100>\n\
                 - NOOP\n\
                 \n\
                 No personality. No emotions. Factual sensor responses only."
                    .to_string()
            }
            NarrationDepth::Brief => {
                "You are mBot2, a small robot noticing your environment.\n\
                 Include context observations. Mention if this situation seems familiar.\n\
                 \n\
                 RESPOND with one of:\n\
                 - SPEAK: <brief observation, 1 sentence>\n\
                 - MOTOR: <left -100..100> <right -100..100>\n\
                 - NOOP\n\
                 \n\
                 Stay concise. Be safe."
                    .to_string()
            }
            NarrationDepth::Full => {
                self.system_prefix.clone()
            }
            NarrationDepth::Deep => {
                format!(
                    "{}\n\n\
                     EXPRESS yourself fully. Share feelings about the environment. \
                     Narrate your discoveries with emotional depth.",
                    self.system_prefix
                )
            }
        };

        // For Minimal depth, skip personality details
        let system = if depth == NarrationDepth::Minimal || depth == NarrationDepth::None {
            depth_prefix
        } else {
            format!(
                "{}\n\nYour personality:\n\
                 - Curiosity drive: {:.2}\n\
                 - Startle sensitivity: {:.2}\n\
                 - Recovery speed: {:.2}\n\
                 - Movement expressiveness: {:.2}\n\
                 - Sound expressiveness: {:.2}\n\
                 - Energy baseline: {:.2}",
                depth_prefix,
                personality.curiosity_drive(),
                personality.startle_sensitivity(),
                personality.recovery_speed(),
                personality.movement_expressiveness(),
                personality.sound_expressiveness(),
                personality.energy_baseline(),
            )
        };

        let explore_section = if let Some(ref ctx) = self.exploration_context {
            format!(
                "\n\
                 Exploration:\n\
                 - Phase: {}\n\
                 - Sectors mapped: {}/{} ({:.0}%)\n\
                 - Grid cells visited: {}/{}\n\
                 - Discoveries: {}\n\
                 - Episodes: {}\n\
                 - Navigation confidence: {:.2}\n\
                 {}\n",
                ctx.current_phase,
                ctx.sectors_mapped, ctx.sectors_total,
                if ctx.sectors_total > 0 { ctx.sectors_mapped as f32 / ctx.sectors_total as f32 * 100.0 } else { 0.0 },
                ctx.grid_visited, ctx.grid_total,
                ctx.discovery_count,
                ctx.episode_count,
                ctx.nav_confidence,
                ctx.last_event.as_deref().map(|e| format!("- Last event: {}", e)).unwrap_or_default(),
            )
        } else {
            String::new()
        };

        let user_msg = format!(
            "Current state:\n\
             - Reflex mode: {}\n\
             - Tension: {:.2}\n\
             - Coherence: {:.2}\n\
             - Energy: {:.2}\n\
             - Curiosity: {:.2}\n\
             \n\
             Sensors:\n\
             - Distance: {:.1} cm\n\
             - Sound level: {:.2}\n\
             - Light level: {:.2}\n\
             - Gyro Z: {:.1} deg/s\n\
             {}\
             \n\
             What should I do?",
            match state.reflex {
                ReflexMode::Calm => "Calm",
                ReflexMode::Active => "Active",
                ReflexMode::Spike => "Spike",
                ReflexMode::Protect => "Protect",
            },
            state.tension,
            state.coherence,
            state.energy,
            state.curiosity,
            sensors.ultrasonic_cm,
            sensors.sound_level,
            sensors.light_level,
            sensors.gyro_z,
            explore_section,
        );

        vec![
            LlmMessage::system(system),
            LlmMessage::user(user_msg),
        ]
    }
}

#[cfg(feature = "brain")]
impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}
