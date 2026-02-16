//! Daily Activity Tracking
//!
//! Tracks what the robot did each day for follow-up and memory recall.

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
pub struct DailyActivity {
    pub id: i64,
    pub date: String,
    pub activity_type: String,
    pub description: Option<String>,
    pub metadata: Option<String>,
    pub created_at: String,
}

#[cfg(feature = "brain")]
impl MemoryStore {
    /// Log a daily activity
    pub fn log_activity(
        &self,
        activity_type: &str,
        description: Option<&str>,
        metadata: Option<&str>,
    ) -> BrainResult<i64> {
        let conn = self.connection()?;
        conn.execute(
            "INSERT INTO daily_activities (date, activity_type, description, metadata)
             VALUES (date('now'), ?1, ?2, ?3)",
            params![activity_type, description, metadata],
        )
        .map_err(|e| BrainError::MemoryError(format!("Insert activity failed: {}", e)))?;

        Ok(conn.last_insert_rowid())
    }

    /// Get activities for a specific date
    pub fn get_activities_for_date(&self, date: &str) -> BrainResult<Vec<DailyActivity>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, date, activity_type, description, metadata, created_at
             FROM daily_activities
             WHERE date = ?1
             ORDER BY created_at ASC"
        ).map_err(|e| BrainError::MemoryError(format!("Prepare failed: {}", e)))?;

        let activities = stmt.query_map(params![date], |row| {
            Ok(DailyActivity {
                id: row.get(0)?,
                date: row.get(1)?,
                activity_type: row.get(2)?,
                description: row.get(3)?,
                metadata: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| BrainError::MemoryError(format!("Query failed: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| BrainError::MemoryError(format!("Row parse failed: {}", e)))?;

        Ok(activities)
    }

    /// Get today's activities
    pub fn get_today_activities(&self) -> BrainResult<Vec<DailyActivity>> {
        let conn = self.connection()?;
        let today: String = conn.query_row(
            "SELECT date('now')", [], |row| row.get(0)
        ).map_err(|e| BrainError::MemoryError(format!("Date query failed: {}", e)))?;

        self.get_activities_for_date(&today)
    }

    /// Get a summary of recent days' activities (last N days)
    pub fn get_recent_activity_summary(&self, days: u32) -> BrainResult<Vec<(String, usize)>> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(
            "SELECT date, COUNT(*) as count
             FROM daily_activities
             WHERE date >= date('now', ?1)
             GROUP BY date
             ORDER BY date DESC"
        ).map_err(|e| BrainError::MemoryError(format!("Prepare failed: {}", e)))?;

        let summary = stmt.query_map(params![format!("-{} days", days)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
        })
        .map_err(|e| BrainError::MemoryError(format!("Query failed: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| BrainError::MemoryError(format!("Row parse failed: {}", e)))?;

        Ok(summary)
    }
}
