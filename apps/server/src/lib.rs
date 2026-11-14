//! Nurbit medical server (stage 10): HTTP API over a SQLite database.
//!
//! * `GET  /health`                        - liveness, no auth;
//! * `POST /api/v1/sync/push`              - receive a batch of records;
//! * `GET  /api/v1/crew`                   - synchronized crew profiles;
//! * `GET  /api/v1/crew/{id}/checkins`     - synchronized check-ins of one person.
//!
//! Every `/api` route needs `Authorization: Bearer <token>`. Records are never
//! overwritten: a different version of a known id is stored as a conflict.

use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use nurbit_shared::RecordId;
use nurbit_shared::records::{CheckInRecord, CrewMember};
use nurbit_shared::sync::{
    Ack, AckStatus, MAX_RECORDS_PER_PUSH, PushRequest, PushResponse, RecordKind, SyncRecord, content_hash,
};
use serde_json::{Value, json};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

/// Shortest accepted access token (characters).
pub const MIN_TOKEN_LEN: usize = 16;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    token: String,
}

pub async fn open_database(path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);
    let pool = SqlitePoolOptions::new().max_connections(4).connect_with(options).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn open_memory_database() -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub fn build_app(pool: SqlitePool, token: String) -> Router {
    let state = AppState { pool, token };
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/sync/push", post(push))
        .route("/api/v1/crew", get(list_crew))
        .route("/api/v1/crew/{id}/checkins", get(member_checkins))
        .with_state(state)
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

/// Compares in constant time so the response time does not reveal the token.
fn tokens_match(given: &str, expected: &str) -> bool {
    given.len() == expected.len() && given.bytes().zip(expected.bytes()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

/// `Some(response)` when access is denied.
fn authorize(headers: &HeaderMap, state: &AppState) -> Option<Response> {
    let given = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    (!tokens_match(given, &state.token)).then(|| error(StatusCode::UNAUTHORIZED, "missing or wrong access token"))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "server_time": now_ms() }))
}

/// Checks hash and payload shape; returns a reason when the record is invalid.
fn validate(record: &SyncRecord) -> Option<String> {
    if content_hash(&record.payload) != record.content_hash {
        return Some("content hash does not match payload".into());
    }
    let payload_id = match record.kind {
        RecordKind::CrewMember => serde_json::from_value::<CrewMember>(record.payload.clone()).map(|m| m.id),
        RecordKind::CheckIn => serde_json::from_value::<CheckInRecord>(record.payload.clone()).map(|c| c.id),
    };
    match payload_id {
        Err(e) => Some(format!("payload is not a valid {}: {e}", record.kind.as_str())),
        Ok(id) if id != record.id => Some("record id differs from payload id".into()),
        Ok(_) => None,
    }
}

async fn store_record(pool: &SqlitePool, device: &str, record: &SyncRecord) -> Result<AckStatus, sqlx::Error> {
    let id = record.id.to_string();
    let mut tx = pool.begin().await?;
    let existing: Option<String> = sqlx::query_scalar("SELECT content_hash FROM synced_records WHERE id = ?")
        .bind(&id)
        .fetch_optional(&mut *tx)
        .await?;
    let status = match existing {
        None => {
            sqlx::query(
                "INSERT INTO synced_records (id, kind, content_hash, payload, device_id, received_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(record.kind.as_str())
            .bind(&record.content_hash)
            .bind(record.payload.to_string())
            .bind(device)
            .bind(now_ms())
            .execute(&mut *tx)
            .await?;
            AckStatus::Accepted
        }
        Some(hash) if hash == record.content_hash => AckStatus::Duplicate,
        Some(_) => {
            // Keep the other version; INSERT OR IGNORE makes re-sending the same conflict harmless.
            sqlx::query(
                "INSERT OR IGNORE INTO sync_conflicts (record_id, content_hash, payload, device_id, received_at) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&record.content_hash)
            .bind(record.payload.to_string())
            .bind(device)
            .bind(now_ms())
            .execute(&mut *tx)
            .await?;
            AckStatus::Conflict
        }
    };
    tx.commit().await?;
    Ok(status)
}

async fn push(State(state): State<AppState>, headers: HeaderMap, Json(request): Json<PushRequest>) -> Response {
    if let Some(denied) = authorize(&headers, &state) {
        return denied;
    }
    if request.records.len() > MAX_RECORDS_PER_PUSH {
        return error(StatusCode::PAYLOAD_TOO_LARGE, "too many records in one request");
    }
    if request.device_id.trim().is_empty() {
        return error(StatusCode::BAD_REQUEST, "device_id is required");
    }
    let mut acks = Vec::with_capacity(request.records.len());
    for record in &request.records {
        let ack = match validate(record) {
            Some(reason) => Ack { id: record.id, status: AckStatus::Invalid, message: Some(reason) },
            None => match store_record(&state.pool, &request.device_id, record).await {
                Ok(status) => Ack { id: record.id, status, message: None },
                Err(e) => return error(StatusCode::INTERNAL_SERVER_ERROR, &format!("storage error: {e}")),
            },
        };
        acks.push(ack);
    }
    Json(PushResponse { acks }).into_response()
}

async fn payloads(pool: &SqlitePool, sql: &str, bind: Option<String>) -> Result<Vec<Value>, sqlx::Error> {
    let mut query = sqlx::query_scalar::<_, String>(sql);
    if let Some(b) = bind {
        query = query.bind(b);
    }
    let rows = query.fetch_all(pool).await?;
    Ok(rows.into_iter().filter_map(|text| serde_json::from_str(&text).ok()).collect())
}

async fn list_crew(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(denied) = authorize(&headers, &state) {
        return denied;
    }
    match payloads(&state.pool, "SELECT payload FROM synced_records WHERE kind = 'crew_member'", None).await {
        Ok(items) => Json(items).into_response(),
        Err(e) => error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn member_checkins(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Response {
    if let Some(denied) = authorize(&headers, &state) {
        return denied;
    }
    if RecordId::parse(&id).is_none() {
        return error(StatusCode::BAD_REQUEST, "invalid crew member id");
    }
    let sql = "SELECT payload FROM synced_records WHERE kind = 'check_in'
               AND json_extract(payload, '$.crew_member_id') = ?
               ORDER BY json_extract(payload, '$.completed_at') DESC";
    match payloads(&state.pool, sql, Some(id)).await {
        Ok(items) => Json(items).into_response(),
        Err(e) => error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

#[cfg(test)]
mod tests;
