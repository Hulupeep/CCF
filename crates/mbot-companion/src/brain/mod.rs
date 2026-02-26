//! Higher-Order Brain Layer for mBot2
//!
//! Provides LLM-powered reasoning, memory, chat channels, voice pipeline,
//! and autonomy engine on top of mbot-core's deterministic nervous system.
//!
//! # Contract Compliance
//! - **ARCH-001**: All brain code lives in mbot-companion, not mbot-core
//! - **ARCH-002**: Brain is advisory; mbot-core's deterministic tick() is the safety layer
//! - **ARCH-003**: SafetyFilter enforces Kitchen Table Test on all LLM outputs
//! - **I-BRAIN-001**: No LLM code in mbot-core
//! - **I-BRAIN-006**: User can disable brain + clear all data
//! - **I-BRAIN-008**: API keys from env vars only

#[cfg(feature = "brain")]
pub mod error;
#[cfg(feature = "brain")]
pub mod llm;
#[cfg(feature = "brain")]
pub mod planner;
#[cfg(feature = "brain")]
pub mod memory;
#[cfg(feature = "brain")]
pub mod autonomy;
#[cfg(feature = "brain")]
pub mod narrator;
#[cfg(feature = "brain")]
pub mod stt;
#[cfg(feature = "brain")]
pub mod suppression_learner;
#[cfg(feature = "brain")]
pub mod suppression_sync;
#[cfg(feature = "brain")]
pub mod coherence_persist;
#[cfg(feature = "brain")]
pub mod coherence;
#[cfg(feature = "brain")]
pub mod downward;
#[cfg(feature = "brain")]
pub mod episodes;
#[cfg(feature = "brain")]
pub mod habits;
#[cfg(feature = "brain")]
pub mod relational_graph;
#[cfg(feature = "brain")]
pub mod mincut;
#[cfg(feature = "brain")]
pub mod group_manager;
#[cfg(feature = "brain")]
pub mod recomputation;
#[cfg(feature = "brain")]
pub mod consolidation;

#[cfg(feature = "brain")]
pub mod channels;

#[cfg(feature = "voice")]
pub mod voice;

#[cfg(feature = "voice-api")]
pub mod voice_api;

#[cfg(feature = "brain")]
use error::{BrainError, BrainResult};
#[cfg(feature = "brain")]
use llm::ProviderChain;
#[cfg(feature = "brain")]
use memory::MemoryService;
#[cfg(feature = "brain")]
use planner::{BrainAction, BrainPlanner};
#[cfg(feature = "brain")]
use autonomy::AutonomyEngine;
#[cfg(feature = "brain")]
use narrator::PersonalityNarrator;

#[cfg(feature = "brain")]
use mbot_core::{HomeostasisState, MBotSensors, MotorCommand};
#[cfg(feature = "brain")]
use mbot_core::personality::Personality;

/// Configuration for the brain layer
#[cfg(feature = "brain")]
#[derive(Debug, Clone)]
pub struct BrainConfig {
    /// Whether the brain layer is enabled (I-BRAIN-006)
    pub enabled: bool,
    /// How often to query LLM (seconds between queries)
    pub llm_query_interval_secs: u64,
    /// SQLite database path (I-MEM-002)
    pub db_path: String,
    /// Conversation retention days (I-MEM-001, default 7)
    pub retention_days: u32,
    /// Whether autonomy safe mode is on (I-AUTO-003)
    pub safe_mode: bool,
    /// Max concurrent autonomy actions (I-AUTO-002)
    pub max_concurrent_actions: usize,
}

#[cfg(feature = "brain")]
impl Default for BrainConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            llm_query_interval_secs: 5,
            db_path: "mbot_brain.db".to_string(),
            retention_days: 7,
            safe_mode: true,
            max_concurrent_actions: 5,
        }
    }
}

/// Top-level brain layer orchestrator
///
/// Sits on top of mbot-core's deterministic nervous system and provides
/// LLM-powered reasoning, memory, and proactive behaviors.
#[cfg(feature = "brain")]
pub struct BrainLayer {
    config: BrainConfig,
    planner: Option<BrainPlanner>,
    memory: Option<MemoryService>,
    autonomy: Option<AutonomyEngine>,
    narrator: Option<PersonalityNarrator>,
    last_llm_query: std::time::Instant,
}

#[cfg(feature = "brain")]
impl BrainLayer {
    /// Create a new brain layer with the given config
    pub async fn new(config: BrainConfig) -> BrainResult<Self> {
        let memory = if config.enabled {
            Some(MemoryService::new(&config.db_path, config.retention_days).await?)
        } else {
            None
        };

        Ok(Self {
            config,
            planner: None,
            memory,
            autonomy: None,
            narrator: None,
            last_llm_query: std::time::Instant::now(),
        })
    }

    /// Initialize the brain layer with LLM providers
    pub async fn init(&mut self, provider_chain: ProviderChain) -> BrainResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let narrator = PersonalityNarrator::new(provider_chain.clone());
        let planner = BrainPlanner::new(provider_chain, self.config.llm_query_interval_secs);

        self.planner = Some(planner);
        self.narrator = Some(narrator);

        Ok(())
    }

    /// Initialize the autonomy engine
    pub async fn init_autonomy(&mut self) -> BrainResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let engine = AutonomyEngine::new(
            self.config.safe_mode,
            self.config.max_concurrent_actions,
        ).await?;

        self.autonomy = Some(engine);
        Ok(())
    }

    /// Whether the brain layer is enabled (I-BRAIN-006)
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Disable the brain layer (I-BRAIN-006)
    pub fn disable(&mut self) {
        self.config.enabled = false;
    }

    /// Enable the brain layer
    pub fn enable(&mut self) {
        self.config.enabled = true;
    }

    /// Clear all brain data - memory, conversations, activities (I-BRAIN-006, I-MEM-003)
    pub async fn clear_all_data(&mut self) -> BrainResult<()> {
        if let Some(ref mut memory) = self.memory {
            memory.clear_all().await?;
        }
        Ok(())
    }

    /// Process one tick of the brain layer
    ///
    /// Called from the main loop after mbot-core's deterministic tick().
    /// Returns a list of suggested actions (motor overrides, speech, etc.)
    /// All actions pass through SafetyFilter before being returned.
    pub async fn on_tick(
        &mut self,
        state: &HomeostasisState,
        sensors: &MBotSensors,
        personality: &Personality,
    ) -> BrainResult<Vec<BrainAction>> {
        if !self.config.enabled {
            return Ok(vec![]);
        }

        let mut actions = Vec::new();

        // Check if it's time to query LLM
        let elapsed = self.last_llm_query.elapsed();
        let interval = std::time::Duration::from_secs(self.config.llm_query_interval_secs);

        if elapsed >= interval {
            if let Some(ref mut planner) = self.planner {
                match planner.plan(state, sensors, personality).await {
                    Ok(planned_actions) => {
                        actions.extend(planned_actions);
                        self.last_llm_query = std::time::Instant::now();
                    }
                    Err(BrainError::NoProvidersAvailable) => {
                        // I-BRAIN-007: Graceful degradation
                        tracing::warn!("No LLM providers available, brain operating in degraded mode");
                    }
                    Err(BrainError::LlmTimeout(secs)) => {
                        tracing::warn!("LLM timed out after {}s", secs);
                    }
                    Err(e) => {
                        tracing::error!("Brain planner error: {}", e);
                    }
                }
            }
        }

        // Check autonomy engine for scheduled actions
        if let Some(ref mut autonomy) = self.autonomy {
            match autonomy.check_triggers(state, sensors).await {
                Ok(auto_actions) => actions.extend(auto_actions),
                Err(e) => tracing::warn!("Autonomy engine error: {}", e),
            }
        }

        Ok(actions)
    }

    /// Get a reference to the memory service
    pub fn memory(&self) -> Option<&MemoryService> {
        self.memory.as_ref()
    }

    /// Get a mutable reference to the memory service
    pub fn memory_mut(&mut self) -> Option<&mut MemoryService> {
        self.memory.as_mut()
    }

    /// Generate a personality-colored response to text input, gated by permeability.
    ///
    /// Returns `None` if permeability is below the threshold for LLM reflection.
    /// Pass `permeability = 1.0` for always-on behavior (e.g. direct user chat).
    pub async fn respond(
        &self,
        input: &str,
        personality: &Personality,
        state: &HomeostasisState,
        permeability: f32,
    ) -> BrainResult<Option<String>> {
        if !self.config.enabled {
            return Err(BrainError::ConfigError("Brain layer is disabled".into()));
        }

        match &self.narrator {
            Some(narrator) => narrator.respond(input, personality, state, permeability).await,
            None => Err(BrainError::ConfigError("Narrator not initialized".into())),
        }
    }

    /// Generate a personality-colored response without permeability gating.
    ///
    /// Always makes an LLM call. Use for direct user chat where the user
    /// expects a response regardless of robot's internal state.
    pub async fn respond_ungated(
        &self,
        input: &str,
        personality: &Personality,
        state: &HomeostasisState,
    ) -> BrainResult<String> {
        if !self.config.enabled {
            return Err(BrainError::ConfigError("Brain layer is disabled".into()));
        }

        match &self.narrator {
            Some(narrator) => narrator.respond_ungated(input, personality, state).await,
            None => Err(BrainError::ConfigError("Narrator not initialized".into())),
        }
    }
}
