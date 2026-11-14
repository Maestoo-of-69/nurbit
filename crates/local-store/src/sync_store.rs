//! Client side of synchronization: settings and the outbox.
//!
//! The outbox is bookkeeping, not medical data, so its status column may be
//! updated (pending -> sent / conflict); the medical records it points to
//! stay append-only.

use nurbit_shared::RecordId;
use serde::Serialize;
use sqlx::Row;

use crate::{LocalStore, StoreResult, parse_id};

pub const SETTING_SERVER_URL: &str = "sync_server_url";
pub const SETTING_SYNC_TOKEN: &str = "sync_token";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxItem {
    pub record_id: RecordId,
    pub record_type: String,
    pub attempts: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct OutboxCounts {
    pub pending: i64,
    pub sent: i64,
    pub conflict: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboxStatus {
    Sent,
    Conflict,
}

impl LocalStore {
    pub async fn get_setting(&self, key: &str) -> StoreResult<Option<String>> {
        Ok(sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?)
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> StoreResult<()> {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT (key) DO UPDATE SET value = excluded.value")
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Oldest pending records first.
    pub async fn pending_outbox(&self, limit: i64) -> StoreResult<Vec<OutboxItem>> {
        let rows = sqlx::query(
            "SELECT record_id, record_type, attempts FROM sync_outbox WHERE status = 'pending' ORDER BY queued_at LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(|r| {
                Ok(OutboxItem {
                    record_id: parse_id(&r.try_get::<String, _>("record_id")?)?,
                    record_type: r.try_get("record_type")?,
                    attempts: r.try_get("attempts")?,
                })
            })
            .collect()
    }

    /// Records the server's answer for one record.
    pub async fn mark_outbox(&self, record: RecordId, status: OutboxStatus) -> StoreResult<()> {
        let status = match status {
            OutboxStatus::Sent => "sent",
            OutboxStatus::Conflict => "conflict",
        };
        sqlx::query("UPDATE sync_outbox SET status = ?, last_error = NULL WHERE record_id = ?")
            .bind(status)
            .bind(record.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Keeps the record pending and remembers why the attempt failed.
    pub async fn outbox_failed(&self, record: RecordId, error: &str) -> StoreResult<()> {
        sqlx::query("UPDATE sync_outbox SET attempts = attempts + 1, last_error = ? WHERE record_id = ?")
            .bind(error)
            .bind(record.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn outbox_counts(&self) -> StoreResult<OutboxCounts> {
        let row = sqlx::query(
            "SELECT
                 COALESCE(SUM(status = 'pending'), 0)  AS pending,
                 COALESCE(SUM(status = 'sent'), 0)     AS sent,
                 COALESCE(SUM(status = 'conflict'), 0) AS conflict
             FROM sync_outbox",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(OutboxCounts { pending: row.try_get("pending")?, sent: row.try_get("sent")?, conflict: row.try_get("conflict")? })
    }
}
