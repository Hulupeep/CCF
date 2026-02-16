//! Conversation Memory CRUD
//!
//! Tracks multi-turn conversations with channel attribution.

#[cfg(feature = "brain")]
use rusqlite::params;
#[cfg(feature = "brain")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "brain")]
use super::store::MemoryStore;
#[cfg(feature = "brain")]
use crate::brain::error::{BrainError, BrainResult};

#[cfg(feature = "brain")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub channel_type: String,
    pub user_id: Option<String>,
    pub started_at: String,
    pub last_active_at: String,
}

#[cfg(feature = "brain")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub id: i64,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
    pub token_count: Option<i32>,
}

#[cfg(feature = "brain")]
impl MemoryStore {
    /// Create a new conversation
    pub fn create_conversation(
        &self,
        id: &str,
        channel_type: &str,
        user_id: Option<&str>,
    ) -> BrainResult<Conversation> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO conversations (id, channel_type, user_id) VALUES (?1, ?2, ?3)",
            params![id, channel_type, user_id],
        )
        .map_err(|e| BrainError::MemoryError(format!("Insert conversation failed: {}", e)))?;

        self.get_conversation(id)
    }

    /// Get a conversation by ID
    pub fn get_conversation(&self, id: &str) -> BrainResult<Conversation> {
        let conn = self.connection()?;
        conn.query_row(
            "SELECT id, channel_type, user_id, started_at, last_active_at FROM conversations WHERE id = ?1",
            params![id],
            |row| {
                Ok(Conversation {
                    id: row.get(0)?,
                    channel_type: row.get(1)?,
                    user_id: row.get(2)?,
                    started_at: row.get(3)?,
                    last_active_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| BrainError::MemoryError(format!("Get conversation failed: {}", e)))
    }

    /// Add a turn to a conversation
    pub fn add_turn(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        token_count: Option<i32>,
    ) -> BrainResult<i64> {
        let conn = self.connection()?;

        conn.execute(
            "INSERT INTO conversation_turns (conversation_id, role, content, token_count) VALUES (?1, ?2, ?3, ?4)",
            params![conversation_id, role, content, token_count],
        )
        .map_err(|e| BrainError::MemoryError(format!("Insert turn failed: {}", e)))?;

        let turn_id = conn.last_insert_rowid();

        // Update conversation last_active_at
        conn.execute(
            "UPDATE conversations SET last_active_at = datetime('now') WHERE id = ?1",
            params![conversation_id],
        )
        .map_err(|e| BrainError::MemoryError(format!("Update last_active failed: {}", e)))?;

        Ok(turn_id)
    }

    /// Get recent turns for a conversation (most recent first, limited)
    pub fn get_recent_turns(
        &self,
        conversation_id: &str,
        limit: usize,
    ) -> BrainResult<Vec<ConversationTurn>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, role, content, created_at, token_count
             FROM conversation_turns
             WHERE conversation_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2"
        ).map_err(|e| BrainError::MemoryError(format!("Prepare failed: {}", e)))?;

        let turns = stmt.query_map(params![conversation_id, limit as i64], |row| {
            Ok(ConversationTurn {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                token_count: row.get(5)?,
            })
        })
        .map_err(|e| BrainError::MemoryError(format!("Query failed: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| BrainError::MemoryError(format!("Row parse failed: {}", e)))?;

        // Reverse to get chronological order
        let mut turns = turns;
        turns.reverse();
        Ok(turns)
    }

    /// Get the most recent conversation for a user/channel
    pub fn get_latest_conversation(
        &self,
        channel_type: &str,
        user_id: Option<&str>,
    ) -> BrainResult<Option<Conversation>> {
        let conn = self.connection()?;
        let result = if let Some(uid) = user_id {
            conn.query_row(
                "SELECT id, channel_type, user_id, started_at, last_active_at
                 FROM conversations
                 WHERE channel_type = ?1 AND user_id = ?2
                 ORDER BY last_active_at DESC LIMIT 1",
                params![channel_type, uid],
                |row| {
                    Ok(Conversation {
                        id: row.get(0)?,
                        channel_type: row.get(1)?,
                        user_id: row.get(2)?,
                        started_at: row.get(3)?,
                        last_active_at: row.get(4)?,
                    })
                },
            )
        } else {
            conn.query_row(
                "SELECT id, channel_type, user_id, started_at, last_active_at
                 FROM conversations
                 WHERE channel_type = ?1 AND user_id IS NULL
                 ORDER BY last_active_at DESC LIMIT 1",
                params![channel_type],
                |row| {
                    Ok(Conversation {
                        id: row.get(0)?,
                        channel_type: row.get(1)?,
                        user_id: row.get(2)?,
                        started_at: row.get(3)?,
                        last_active_at: row.get(4)?,
                    })
                },
            )
        };

        match result {
            Ok(conv) => Ok(Some(conv)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BrainError::MemoryError(format!("Query failed: {}", e))),
        }
    }
}
