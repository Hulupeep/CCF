//! SQLite Memory Store
//!
//! Invariants:
//! - I-MEM-002: SQLite DB path configurable
//! - I-MEM-003: clear_all() for GDPR compliance

#[cfg(feature = "brain")]
use rusqlite::{Connection, params};
#[cfg(feature = "brain")]
use std::sync::Mutex;

#[cfg(feature = "brain")]
use crate::brain::error::{BrainError, BrainResult};

/// SQLite-backed memory store
#[cfg(feature = "brain")]
pub struct MemoryStore {
    conn: Mutex<Connection>,
}

#[cfg(feature = "brain")]
impl MemoryStore {
    /// Open or create a SQLite database (I-MEM-002: configurable path)
    pub fn new(db_path: &str) -> BrainResult<Self> {
        let conn = if db_path == ":memory:" {
            Connection::open_in_memory()
        } else {
            Connection::open(db_path)
        }
        .map_err(|e| BrainError::MemoryError(format!("Failed to open database: {}", e)))?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| BrainError::MemoryError(format!("Failed to set pragmas: {}", e)))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Run database migrations to create tables
    pub fn run_migrations(&self) -> BrainResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            BrainError::MemoryError(format!("Lock poisoned: {}", e))
        })?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                channel_type TEXT NOT NULL DEFAULT 'local',
                user_id TEXT,
                started_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_active_at TEXT NOT NULL DEFAULT (datetime('now')),
                metadata TEXT
            );

            CREATE TABLE IF NOT EXISTS conversation_turns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                token_count INTEGER
            );

            CREATE INDEX IF NOT EXISTS idx_turns_conversation
                ON conversation_turns(conversation_id);
            CREATE INDEX IF NOT EXISTS idx_turns_created
                ON conversation_turns(created_at);

            CREATE TABLE IF NOT EXISTS daily_activities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                activity_type TEXT NOT NULL,
                description TEXT,
                metadata TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_activities_date
                ON daily_activities(date);

            CREATE TABLE IF NOT EXISTS follow_up_questions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id TEXT REFERENCES conversations(id) ON DELETE CASCADE,
                question TEXT NOT NULL,
                context TEXT,
                priority INTEGER NOT NULL DEFAULT 0,
                asked BOOLEAN NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_followups_priority
                ON follow_up_questions(priority DESC, asked ASC);
            "
        )
        .map_err(|e| BrainError::MemoryError(format!("Migration failed: {}", e)))?;

        Ok(())
    }

    /// Clear ALL data from all tables (I-MEM-003: GDPR)
    pub fn clear_all(&self) -> BrainResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            BrainError::MemoryError(format!("Lock poisoned: {}", e))
        })?;

        conn.execute_batch(
            "
            DELETE FROM follow_up_questions;
            DELETE FROM conversation_turns;
            DELETE FROM daily_activities;
            DELETE FROM conversations;
            "
        )
        .map_err(|e| BrainError::MemoryError(format!("Failed to clear data: {}", e)))?;

        Ok(())
    }

    /// Delete data older than N days (I-MEM-001)
    pub fn cleanup_older_than_days(&self, days: u32) -> BrainResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            BrainError::MemoryError(format!("Lock poisoned: {}", e))
        })?;

        conn.execute(
            "DELETE FROM conversation_turns WHERE created_at < datetime('now', ?1)",
            params![format!("-{} days", days)],
        )
        .map_err(|e| BrainError::MemoryError(format!("Cleanup failed: {}", e)))?;

        conn.execute(
            "DELETE FROM daily_activities WHERE created_at < datetime('now', ?1)",
            params![format!("-{} days", days)],
        )
        .map_err(|e| BrainError::MemoryError(format!("Cleanup failed: {}", e)))?;

        // Delete conversations with no remaining turns
        conn.execute(
            "DELETE FROM conversations WHERE id NOT IN (SELECT DISTINCT conversation_id FROM conversation_turns)",
            [],
        )
        .map_err(|e| BrainError::MemoryError(format!("Cleanup failed: {}", e)))?;

        Ok(())
    }

    /// Get a locked reference to the connection for complex queries
    pub fn connection(&self) -> BrainResult<std::sync::MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|e| {
            BrainError::MemoryError(format!("Lock poisoned: {}", e))
        })
    }
}
