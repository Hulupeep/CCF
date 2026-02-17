//! Action Translator - parses LLM text into BrainAction enum

#[cfg(feature = "brain")]
use crate::brain::planner::{BrainAction, ExploreCommand};
#[cfg(feature = "brain")]
use mbot_core::MotorCommand;

/// Parses LLM response text into structured BrainActions
#[cfg(feature = "brain")]
pub struct ActionTranslator;

#[cfg(feature = "brain")]
impl ActionTranslator {
    pub fn new() -> Self {
        Self
    }

    /// Parse LLM response text into a list of BrainActions
    pub fn parse(&self, response: &str) -> Vec<BrainAction> {
        let mut actions = Vec::new();

        for line in response.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(action) = self.parse_line(line) {
                actions.push(action);
            }
        }

        // If nothing parsed, return Noop
        if actions.is_empty() {
            actions.push(BrainAction::Noop);
        }

        actions
    }

    fn parse_line(&self, line: &str) -> Option<BrainAction> {
        let upper = line.to_uppercase();

        if upper.starts_with("SPEAK:") {
            let text = line[6..].trim().to_string();
            if !text.is_empty() {
                return Some(BrainAction::Speak(text));
            }
        }

        if upper.starts_with("MOTOR:") {
            let parts: Vec<&str> = line[6..].trim().split_whitespace().collect();
            if parts.len() >= 2 {
                if let (Ok(left), Ok(right)) = (parts[0].parse::<i8>(), parts[1].parse::<i8>()) {
                    return Some(BrainAction::Motor(MotorCommand {
                        left,
                        right,
                        ..Default::default()
                    }));
                }
            }
        }

        if upper.starts_with("ACTIVITY:") {
            let name = line[9..].trim().to_string();
            if !name.is_empty() {
                return Some(BrainAction::StartActivity(name));
            }
        }

        if upper.starts_with("ADJUST:") {
            let parts: Vec<&str> = line[7..].trim().split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(delta) = parts[1].parse::<f32>() {
                    return Some(BrainAction::PersonalityAdjust {
                        parameter: parts[0].to_string(),
                        delta,
                    });
                }
            }
        }

        if upper.starts_with("EXPLORE:") {
            let sub = line[8..].trim().to_uppercase();
            if sub.starts_with("SCAN") {
                return Some(BrainAction::Explore(ExploreCommand::Scan));
            }
            if sub.starts_with("MOVE") {
                // EXPLORE: MOVE <sector>
                let parts: Vec<&str> = sub.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(sector) = parts[1].parse::<usize>() {
                        return Some(BrainAction::Explore(ExploreCommand::MoveToSector(sector)));
                    }
                }
                return Some(BrainAction::Explore(ExploreCommand::Scan));
            }
            if sub.starts_with("PAUSE") || sub.starts_with("REFLECT") {
                return Some(BrainAction::Explore(ExploreCommand::Pause));
            }
            if sub.starts_with("RESUME") {
                return Some(BrainAction::Explore(ExploreCommand::Resume));
            }
        }

        if upper.starts_with("NOOP") {
            return Some(BrainAction::Noop);
        }

        None
    }
}

#[cfg(feature = "brain")]
impl Default for ActionTranslator {
    fn default() -> Self {
        Self::new()
    }
}
