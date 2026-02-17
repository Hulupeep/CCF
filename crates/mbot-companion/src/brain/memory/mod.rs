//! Memory Service - SQLite-backed persistence
//!
//! Invariants:
//! - I-MEM-001: Conversation retention configurable, default 7 days
//! - I-MEM-002: SQLite DB path configurable
//! - I-MEM-003: User can clear all memory (GDPR)

#[cfg(feature = "brain")]
pub mod store;
#[cfg(feature = "brain")]
pub mod conversation;
#[cfg(feature = "brain")]
pub mod activity;
#[cfg(feature = "brain")]
pub mod followup;
#[cfg(feature = "brain")]
pub mod crystallization;

#[cfg(feature = "brain")]
use crate::brain::error::{BrainError, BrainResult};
#[cfg(feature = "brain")]
use store::MemoryStore;

/// Facade for all memory operations
#[cfg(feature = "brain")]
pub struct MemoryService {
    store: MemoryStore,
    retention_days: u32,
}

#[cfg(feature = "brain")]
impl MemoryService {
    /// Create a new memory service with configurable DB path and retention
    pub async fn new(db_path: &str, retention_days: u32) -> BrainResult<Self> {
        let store = MemoryStore::new(db_path)?;
        store.run_migrations()?;

        let svc = Self {
            store,
            retention_days,
        };

        // Clean up expired data on startup
        svc.cleanup_expired().await?;

        Ok(svc)
    }

    /// Get a reference to the underlying store
    pub fn store(&self) -> &MemoryStore {
        &self.store
    }

    /// Clear ALL stored data (I-MEM-003: GDPR compliance)
    pub async fn clear_all(&mut self) -> BrainResult<()> {
        self.store.clear_all()
    }

    /// Clean up data older than retention period (I-MEM-001)
    pub async fn cleanup_expired(&self) -> BrainResult<()> {
        self.store.cleanup_older_than_days(self.retention_days)
    }

    /// Get the configured retention period in days
    pub fn retention_days(&self) -> u32 {
        self.retention_days
    }
}
