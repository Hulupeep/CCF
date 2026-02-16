//! Follow-Up Question Queue
//!
//! Tracks questions the robot wants to ask the user later,
//! enabling J-BRAIN-MEMORY-RECALL journey.

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
pub struct FollowUpQuestion {
    pub id: i64,
    pub conversation_id: Option<String>,
    pub question: String,
    pub context: Option<String>,
    pub priority: i32,
    pub asked: bool,
    pub created_at: String,
}

#[cfg(feature = "brain")]
impl MemoryStore {
    /// Queue a follow-up question
    pub fn queue_followup(
        &self,
        conversation_id: Option<&str>,
        question: &str,
        context: Option<&str>,
        priority: i32,
    ) -> BrainResult<i64> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO follow_up_questions (conversation_id, question, context, priority)
             VALUES (?1, ?2, ?3, ?4)",
            params![conversation_id, question, context, priority],
        )
        .map_err(|e| BrainError::MemoryError(format!("Insert followup failed: {}", e)))?;

        Ok(conn.last_insert_rowid())
    }

    /// Get the next unasked follow-up question (highest priority first)
    pub fn next_followup(&self) -> BrainResult<Option<FollowUpQuestion>> {
        let conn = self.connection()?;
        let result = conn.query_row(
            "SELECT id, conversation_id, question, context, priority, asked, created_at
             FROM follow_up_questions
             WHERE asked = 0
             ORDER BY priority DESC, created_at ASC
             LIMIT 1",
            [],
            |row| {
                Ok(FollowUpQuestion {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    question: row.get(2)?,
                    context: row.get(3)?,
                    priority: row.get(4)?,
                    asked: row.get(5)?,
                    created_at: row.get(6)?,
                })
            },
        );

        match result {
            Ok(q) => Ok(Some(q)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BrainError::MemoryError(format!("Query failed: {}", e))),
        }
    }

    /// Mark a follow-up question as asked
    pub fn mark_followup_asked(&self, id: i64) -> BrainResult<()> {
        let conn = self.connection()?;
        conn.execute(
            "UPDATE follow_up_questions SET asked = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| BrainError::MemoryError(format!("Update followup failed: {}", e)))?;

        Ok(())
    }

    /// Get count of pending follow-up questions
    pub fn pending_followup_count(&self) -> BrainResult<usize> {
        let conn = self.connection()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM follow_up_questions WHERE asked = 0",
            [],
            |row| row.get(0),
        )
        .map_err(|e| BrainError::MemoryError(format!("Count query failed: {}", e)))?;

        Ok(count as usize)
    }
}
