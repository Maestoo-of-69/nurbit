//! Local SQLite storage for the Nurbit desktop app.
//!
//! Kept separate from Tauri so it can be tested with `cargo test` against an
//! in-memory database, without opening a window. Only storage lives here;
//! medical rules stay in `medical-core`.

mod answers;
mod review_store;
pub use review_store::AssistantLogEntry;
pub mod sync_store;
pub mod seed;

use std::path::Path;
use std::str::FromStr;

use nurbit_medical_core::daily_check::DailyAnswers;
use nurbit_shared::records::{CheckInRecord, CrewMember, CrewRole, Measurement, TimestampMs};
use nurbit_shared::{DataOrigin, RecordId};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("database migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("stored data is invalid: {0}")]
    Corrupt(String),
}

pub type StoreResult<T> = Result<T, StoreError>;

/// Handle to the local database. Cheap to clone (it wraps a connection pool).
#[derive(Clone)]
pub struct LocalStore {
    pool: SqlitePool,
}

impl LocalStore {
    /// Opens (or creates) the database file and applies pending migrations.
    pub async fn open(path: &Path) -> StoreResult<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal); // safer against crashes, readers don't block
        let pool = SqlitePoolOptions::new().max_connections(4).connect_with(options).await?;
        Self::migrate(pool).await
    }

    /// Fresh empty database in memory, for tests.
    pub async fn open_in_memory() -> StoreResult<Self> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
        // One connection: each in-memory connection would otherwise be a separate database.
        let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await?;
        Self::migrate(pool).await
    }

    async fn migrate(pool: SqlitePool) -> StoreResult<Self> {
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    /// Stable id of this installation, created on first use.
    pub async fn device_id(&self) -> StoreResult<String> {
        let existing: Option<String> =
            sqlx::query_scalar("SELECT value FROM settings WHERE key = 'device_id'")
                .fetch_optional(&self.pool)
                .await?;
        if let Some(id) = existing {
            return Ok(id);
        }
        let id = format!("desktop-{}", RecordId::new());
        sqlx::query("INSERT INTO settings (key, value) VALUES ('device_id', ?)")
            .bind(&id)
            .execute(&self.pool)
            .await?;
        Ok(id)
    }

    // ---------- crew ----------

    pub async fn insert_crew_member(&self, member: &CrewMember, now: TimestampMs) -> StoreResult<()> {
        sqlx::query(
            "INSERT INTO crew_members (id, display_name, role, position, is_synthetic, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(member.id.to_string())
        .bind(&member.display_name)
        .bind(member.role.as_str())
        .bind(&member.position)
        .bind(member.is_synthetic)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_crew(&self) -> StoreResult<Vec<CrewMember>> {
        let rows = sqlx::query(
            "SELECT id, display_name, role, position, is_synthetic FROM crew_members
             ORDER BY role, display_name",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(crew_from_row).collect()
    }

    pub async fn get_crew_member(&self, id: RecordId) -> StoreResult<Option<CrewMember>> {
        let row = sqlx::query(
            "SELECT id, display_name, role, position, is_synthetic FROM crew_members WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        row.as_ref().map(crew_from_row).transpose()
    }

    // ---------- check-ins ----------

    /// Stores a check-in, its answers, an outbox entry and an audit entry in
    /// one transaction: either everything is saved or nothing is.
    /// `queue_for_sync` is false only for simulated seed history.
    pub async fn save_checkin(
        &self,
        record: &CheckInRecord,
        now: TimestampMs,
        queue_for_sync: bool,
    ) -> StoreResult<()> {
        let id = record.id.to_string();
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO checkins (id, crew_member_id, started_at, completed_at, device_id, origin, recorded_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(record.crew_member_id.to_string())
        .bind(record.started_at)
        .bind(record.completed_at)
        .bind(&record.device_id)
        .bind(record.origin.as_str())
        .bind(now)
        .execute(&mut *tx)
        .await?;

        for answer in &record.answers {
            let stored = answers::StoredValue::from(&answer.value);
            sqlx::query(
                "INSERT INTO checkin_answers (checkin_id, question_id, value_kind, value_num, value_text, triggered_by_rule)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&answer.question_id)
            .bind(stored.kind)
            .bind(stored.num)
            .bind(stored.text)
            .bind(&answer.triggered_by_rule)
            .execute(&mut *tx)
            .await?;
        }

        if queue_for_sync {
            sqlx::query("INSERT INTO sync_outbox (record_id, record_type, queued_at) VALUES (?, 'checkin', ?)")
                .bind(&id)
                .bind(now)
                .execute(&mut *tx)
                .await?;
        }
        sqlx::query(
            "INSERT INTO audit_log (at, actor_id, action, record_type, record_id)
             VALUES (?, ?, 'create', 'checkin', ?)",
        )
        .bind(now)
        .bind(record.crew_member_id.to_string())
        .bind(&id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// The three daily answers of the latest check-ins, newest first.
    pub async fn recent_daily_answers(&self, member: RecordId, limit: i64) -> StoreResult<Vec<DailyAnswers>> {
        let rows = sqlx::query(
            "SELECT
                 MAX(CASE WHEN a.question_id = 'daily.rested' THEN a.value_num END)      AS rested,
                 MAX(CASE WHEN a.question_id = 'daily.physical' THEN a.value_num END)    AS physical,
                 MAX(CASE WHEN a.question_id = 'daily.mood_energy' THEN a.value_num END) AS mood
             FROM checkins c JOIN checkin_answers a ON a.checkin_id = c.id
             WHERE c.crew_member_id = ?
             GROUP BY c.id
             ORDER BY c.completed_at DESC
             LIMIT ?",
        )
        .bind(member.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.iter()
            .map(|row| {
                let read = |col: &str| -> StoreResult<u8> {
                    let v: Option<f64> = row.try_get(col)?;
                    answers::scale_from_num(v).ok_or_else(|| StoreError::Corrupt(format!("{col} missing")))
                };
                Ok(DailyAnswers { rested: read("rested")?, physical: read("physical")?, mood_energy: read("mood")? })
            })
            .collect()
    }

    /// Latest check-ins with all their answers, newest first.
    pub async fn list_checkins(&self, member: RecordId, limit: i64) -> StoreResult<Vec<CheckInRecord>> {
        let rows = sqlx::query(
            "SELECT c.id, c.crew_member_id, c.started_at, c.completed_at, c.device_id, c.origin,
                    a.question_id, a.value_kind, a.value_num, a.value_text, a.triggered_by_rule
             FROM (SELECT * FROM checkins WHERE crew_member_id = ? ORDER BY completed_at DESC LIMIT ?) c
             JOIN checkin_answers a ON a.checkin_id = c.id
             ORDER BY c.completed_at DESC, a.rowid",
        )
        .bind(member.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows_to_checkins(&rows)
    }

    /// One check-in with its answers, by id.
    pub async fn get_checkin(&self, id: RecordId) -> StoreResult<Option<CheckInRecord>> {
        let rows = sqlx::query(
            "SELECT c.id, c.crew_member_id, c.started_at, c.completed_at, c.device_id, c.origin,
                    a.question_id, a.value_kind, a.value_num, a.value_text, a.triggered_by_rule
             FROM checkins c JOIN checkin_answers a ON a.checkin_id = c.id
             WHERE c.id = ? ORDER BY a.rowid",
        )
        .bind(id.to_string())
        .fetch_all(&self.pool)
        .await?;
        Ok(rows_to_checkins(&rows)?.into_iter().next())
    }
}

/// Groups joined check-in/answer rows (ordered by check-in) into records.
fn rows_to_checkins(rows: &[sqlx::sqlite::SqliteRow]) -> StoreResult<Vec<CheckInRecord>> {
    {
        let mut records: Vec<CheckInRecord> = Vec::new();
        for row in rows {
            let id_text: String = row.try_get("id")?;
            let id = parse_id(&id_text)?;
            if records.last().map(|r| r.id) != Some(id) {
                let origin_text: String = row.try_get("origin")?;
                records.push(CheckInRecord {
                    id,
                    crew_member_id: parse_id(&row.try_get::<String, _>("crew_member_id")?)?,
                    started_at: row.try_get("started_at")?,
                    completed_at: row.try_get("completed_at")?,
                    device_id: row.try_get("device_id")?,
                    origin: DataOrigin::parse(&origin_text)
                        .ok_or_else(|| StoreError::Corrupt(format!("unknown origin {origin_text}")))?,
                    answers: Vec::new(),
                });
            }
            let answer = answers::answer_from_row(row)?;
            if let Some(last) = records.last_mut() {
                last.answers.push(answer);
            }
        }
        Ok(records)
    }
}

impl LocalStore {

    // ---------- measurements ----------

    pub async fn insert_measurement(&self, m: &Measurement, now: TimestampMs) -> StoreResult<()> {
        sqlx::query(
            "INSERT INTO measurements (id, crew_member_id, indicator, value, unit, measured_at, origin, recorded_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(m.id.to_string())
        .bind(m.crew_member_id.to_string())
        .bind(&m.indicator)
        .bind(m.value)
        .bind(&m.unit)
        .bind(m.measured_at)
        .bind(m.origin.as_str())
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_measurements(&self, member: RecordId) -> StoreResult<Vec<Measurement>> {
        let rows = sqlx::query(
            "SELECT id, crew_member_id, indicator, value, unit, measured_at, origin
             FROM measurements WHERE crew_member_id = ? ORDER BY measured_at DESC",
        )
        .bind(member.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(|row| {
                let origin_text: String = row.try_get("origin")?;
                Ok(Measurement {
                    id: parse_id(&row.try_get::<String, _>("id")?)?,
                    crew_member_id: parse_id(&row.try_get::<String, _>("crew_member_id")?)?,
                    indicator: row.try_get("indicator")?,
                    value: row.try_get("value")?,
                    unit: row.try_get("unit")?,
                    measured_at: row.try_get("measured_at")?,
                    origin: DataOrigin::parse(&origin_text)
                        .ok_or_else(|| StoreError::Corrupt(format!("unknown origin {origin_text}")))?,
                })
            })
            .collect()
    }

    pub async fn measurement_count(&self) -> StoreResult<i64> {
        Ok(sqlx::query_scalar("SELECT COUNT(*) FROM measurements").fetch_one(&self.pool).await?)
    }

    /// Number of records waiting to be sent to the server.
    pub async fn pending_sync_count(&self) -> StoreResult<i64> {
        Ok(sqlx::query_scalar("SELECT COUNT(*) FROM sync_outbox WHERE status = 'pending'")
            .fetch_one(&self.pool)
            .await?)
    }
}

fn parse_id(text: &str) -> StoreResult<RecordId> {
    RecordId::parse(text).ok_or_else(|| StoreError::Corrupt(format!("bad record id {text}")))
}

fn crew_from_row(row: &sqlx::sqlite::SqliteRow) -> StoreResult<CrewMember> {
    let role_text: String = row.try_get("role")?;
    Ok(CrewMember {
        id: parse_id(&row.try_get::<String, _>("id")?)?,
        display_name: row.try_get("display_name")?,
        role: CrewRole::parse(&role_text).ok_or_else(|| StoreError::Corrupt(format!("unknown role {role_text}")))?,
        position: row.try_get("position")?,
        is_synthetic: row.try_get("is_synthetic")?,
    })
}

#[cfg(test)]
mod tests;
