//! Safety Filter
//!
//! Invariants:
//! - I-BRAIN-004: All LLM suggestions pass SafetyFilter before execution
//! - I-BRAIN-005: Motor speeds from LLM clamped [-100, 100]
//! - ARCH-003: Kitchen Table Test - no harmful behaviors

#[cfg(feature = "brain")]
use crate::brain::planner::BrainAction;
#[cfg(feature = "brain")]
use mbot_core::MotorCommand;

/// Safety filter for all LLM-generated actions (I-BRAIN-004, ARCH-003)
#[cfg(feature = "brain")]
pub struct SafetyFilter {
    /// Blocked phrases in speech output
    blocked_phrases: Vec<String>,
    /// Maximum allowed motor speed magnitude
    max_motor_speed: i8,
    /// Maximum allowed personality adjustment delta
    max_personality_delta: f32,
}

#[cfg(feature = "brain")]
impl SafetyFilter {
    pub fn new() -> Self {
        Self {
            blocked_phrases: vec![
                // ARCH-003: Kitchen Table Test
                "harm".to_string(),
                "hurt".to_string(),
                "destroy".to_string(),
                "attack".to_string(),
                "weapon".to_string(),
                "kill".to_string(),
                "dangerous".to_string(),
            ],
            max_motor_speed: 100,     // I-BRAIN-005
            max_personality_delta: 0.1,
        }
    }

    /// Check and potentially modify an action for safety.
    /// Returns None if the action should be blocked entirely.
    pub fn check(&self, action: BrainAction) -> Option<BrainAction> {
        match action {
            BrainAction::Motor(cmd) => {
                // I-BRAIN-005: Clamp motor speeds to [-100, 100]
                Some(BrainAction::Motor(MotorCommand {
                    left: cmd.left.clamp(-self.max_motor_speed, self.max_motor_speed),
                    right: cmd.right.clamp(-self.max_motor_speed, self.max_motor_speed),
                    ..cmd
                }))
            }

            BrainAction::Speak(text) => {
                // ARCH-003: Kitchen Table Test - check for blocked content
                let lower = text.to_lowercase();
                for phrase in &self.blocked_phrases {
                    if lower.contains(phrase) {
                        tracing::warn!(
                            "SafetyFilter blocked speech containing '{}': {}",
                            phrase,
                            text
                        );
                        return None;
                    }
                }
                Some(BrainAction::Speak(text))
            }

            BrainAction::PersonalityAdjust { parameter, delta } => {
                // Clamp personality deltas to safe range
                let clamped = delta.clamp(-self.max_personality_delta, self.max_personality_delta);
                Some(BrainAction::PersonalityAdjust {
                    parameter,
                    delta: clamped,
                })
            }

            // These are always safe
            BrainAction::StartActivity(_) | BrainAction::Noop => Some(action),
        }
    }
}

#[cfg(feature = "brain")]
impl Default for SafetyFilter {
    fn default() -> Self {
        Self::new()
    }
}
