//! Prompt Builder - constructs LLM prompts from robot state

#[cfg(feature = "brain")]
use crate::brain::llm::LlmMessage;
#[cfg(feature = "brain")]
use mbot_core::{HomeostasisState, MBotSensors, ReflexMode};
#[cfg(feature = "brain")]
use mbot_core::personality::Personality;

#[cfg(feature = "brain")]
pub struct PromptBuilder {
    system_prefix: String,
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
                 - NOOP\n\
                 \n\
                 Keep responses SHORT (1-2 sentences for SPEAK). \
                 Be safe - never suggest dangerous motor speeds. \
                 You are on a table - don't drive off edges."
            ),
        }
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
