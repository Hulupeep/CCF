//! Built-in Proactive Actions
//!
//! GoodMorning, InactivityCheck, IdleOffer

#[cfg(feature = "brain")]
use async_trait::async_trait;

#[cfg(feature = "brain")]
use super::context::ContextMonitor;
#[cfg(feature = "brain")]
use crate::brain::error::BrainResult;
#[cfg(feature = "brain")]
use crate::brain::planner::BrainAction;

/// Trait for proactive autonomy actions
#[cfg(feature = "brain")]
#[async_trait]
pub trait ProactiveAction: Send + Sync {
    /// Unique name for this action
    fn name(&self) -> &str;

    /// Check if this action should trigger given current context
    fn should_trigger(&self, context: &ContextMonitor) -> bool;

    /// Execute the action and return a BrainAction
    async fn execute(&self, context: &ContextMonitor) -> BrainResult<BrainAction>;
}

/// Greet the user when robot starts up
#[cfg(feature = "brain")]
struct GoodMorning {
    fired: std::sync::atomic::AtomicBool,
}

#[cfg(feature = "brain")]
#[async_trait]
impl ProactiveAction for GoodMorning {
    fn name(&self) -> &str {
        "good_morning"
    }

    fn should_trigger(&self, context: &ContextMonitor) -> bool {
        // Fire once, within first 30 seconds of startup
        !self.fired.load(std::sync::atomic::Ordering::Relaxed)
            && context.uptime_secs() < 30
    }

    async fn execute(&self, _context: &ContextMonitor) -> BrainResult<BrainAction> {
        self.fired.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(BrainAction::Speak("Hello! I'm awake and ready to play.".into()))
    }
}

/// Check for inactivity and offer to do something
#[cfg(feature = "brain")]
struct InactivityCheck;

#[cfg(feature = "brain")]
#[async_trait]
impl ProactiveAction for InactivityCheck {
    fn name(&self) -> &str {
        "inactivity_check"
    }

    fn should_trigger(&self, context: &ContextMonitor) -> bool {
        // Trigger after 5 minutes of idle
        context.is_idle(300)
    }

    async fn execute(&self, _context: &ContextMonitor) -> BrainResult<BrainAction> {
        Ok(BrainAction::Speak("It's been a while! Want to play a game or draw something?".into()))
    }
}

/// Offer to do something when idle but user is present
#[cfg(feature = "brain")]
struct IdleOffer;

#[cfg(feature = "brain")]
#[async_trait]
impl ProactiveAction for IdleOffer {
    fn name(&self) -> &str {
        "idle_offer"
    }

    fn should_trigger(&self, context: &ContextMonitor) -> bool {
        // Sound detected but idle for 2+ minutes = user present but not interacting
        context.sound_level > 0.1 && context.is_idle(120)
    }

    async fn execute(&self, context: &ContextMonitor) -> BrainResult<BrainAction> {
        if context.energy_level < 0.3 {
            Ok(BrainAction::Speak("I'm feeling a bit tired, but I can still help if you need me!".into()))
        } else {
            Ok(BrainAction::Speak("I noticed you're around. Need anything?".into()))
        }
    }
}

/// Get the default set of proactive actions
#[cfg(feature = "brain")]
pub fn default_actions() -> Vec<Box<dyn ProactiveAction>> {
    vec![
        Box::new(GoodMorning {
            fired: std::sync::atomic::AtomicBool::new(false),
        }),
        Box::new(InactivityCheck),
        Box::new(IdleOffer),
    ]
}
