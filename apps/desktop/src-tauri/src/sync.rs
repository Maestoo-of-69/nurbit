//! Synchronization client: sends queued records to the Nurbit server.
//!
//! Works offline-first: if the server cannot be reached, records stay in the
//! outbox with the reason, and a later attempt (manual or the background
//! timer) sends them. Re-sending is safe because the server is idempotent.

use std::collections::HashMap;
use std::time::Duration;

use nurbit_local_store::sync_store::{OutboxStatus, SETTING_SERVER_URL, SETTING_SYNC_TOKEN};
use nurbit_shared::RecordId;
use nurbit_shared::sync::{AckStatus, MAX_RECORDS_PER_PUSH, PushRequest, PushResponse, RecordKind, make_record};
use serde::Serialize;

use crate::{AppState, now_ms};

/// How often the background task tries to sync.
pub const BACKGROUND_INTERVAL: Duration = Duration::from_secs(30);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    NotConfigured,
    NothingToSend,
    Ok,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncReport {
    pub at: i64,
    pub outcome: Outcome,
    pub accepted: usize,
    pub duplicates: usize,
    pub conflicts: usize,
    pub invalid: usize,
    pub error: Option<String>,
}

impl SyncReport {
    fn new(outcome: Outcome) -> Self {
        Self { at: now_ms(), outcome, accepted: 0, duplicates: 0, conflicts: 0, invalid: 0, error: None }
    }

    fn failed(message: String) -> Self {
        Self { error: Some(message), ..Self::new(Outcome::Failed) }
    }
}

/// One sync attempt. Never panics; problems are reported in the result.
pub async fn run_sync(state: &AppState) -> SyncReport {
    let _guard = state.sync_lock.lock().await; // one attempt at a time
    let report = attempt(state).await;
    if report.outcome != Outcome::NotConfigured
        && let Ok(mut last) = state.last_sync.lock()
    {
        *last = Some(report.clone());
    }
    report
}

async fn attempt(state: &AppState) -> SyncReport {
    let store = &state.store;
    let (url, token) = match (store.get_setting(SETTING_SERVER_URL).await, store.get_setting(SETTING_SYNC_TOKEN).await) {
        (Ok(Some(u)), Ok(Some(t))) if !u.is_empty() && !t.is_empty() => (u, t),
        (Err(e), _) | (_, Err(e)) => return SyncReport::failed(format!("Local storage problem: {e}")),
        _ => return SyncReport::new(Outcome::NotConfigured),
    };

    let crew = match store.list_crew().await {
        Ok(c) => c,
        Err(e) => return SyncReport::failed(format!("Local storage problem: {e}")),
    };
    let room = MAX_RECORDS_PER_PUSH.saturating_sub(crew.len()) as i64;
    let pending = match store.pending_outbox(room).await {
        Ok(p) => p,
        Err(e) => return SyncReport::failed(format!("Local storage problem: {e}")),
    };
    if pending.is_empty() {
        return SyncReport::new(Outcome::NothingToSend);
    }

    // Crew profiles go first so the server knows who each check-in belongs to.
    let mut records = Vec::new();
    for member in &crew {
        if let Ok(r) = make_record(RecordKind::CrewMember, member.id, member) {
            records.push(r);
        }
    }
    let mut checkin_ids: Vec<RecordId> = Vec::new();
    for item in &pending {
        match store.get_checkin(item.record_id).await {
            Ok(Some(c)) => match make_record(RecordKind::CheckIn, c.id, &c) {
                Ok(r) => {
                    records.push(r);
                    checkin_ids.push(c.id);
                }
                Err(e) => {
                    let _ = store.outbox_failed(item.record_id, &format!("cannot encode: {e}")).await;
                }
            },
            _ => {
                let _ = store.outbox_failed(item.record_id, "record not found locally").await;
            }
        }
    }

    let request = PushRequest { device_id: state.device_id.clone(), records };
    let response = match send(&url, &token, &request).await {
        Ok(r) => r,
        Err(message) => {
            for id in &checkin_ids {
                let _ = store.outbox_failed(*id, &message).await;
            }
            return SyncReport::failed(message);
        }
    };

    let acks: HashMap<RecordId, (AckStatus, Option<String>)> =
        response.acks.into_iter().map(|a| (a.id, (a.status, a.message))).collect();
    let mut report = SyncReport::new(Outcome::Ok);
    for id in checkin_ids {
        match acks.get(&id) {
            Some((AckStatus::Accepted, _)) => {
                report.accepted += 1;
                let _ = store.mark_outbox(id, OutboxStatus::Sent).await;
            }
            Some((AckStatus::Duplicate, _)) => {
                report.duplicates += 1;
                let _ = store.mark_outbox(id, OutboxStatus::Sent).await;
            }
            Some((AckStatus::Conflict, _)) => {
                report.conflicts += 1;
                let _ = store.mark_outbox(id, OutboxStatus::Conflict).await;
            }
            Some((AckStatus::Invalid, message)) => {
                report.invalid += 1;
                let reason = message.clone().unwrap_or_else(|| "rejected as invalid".into());
                let _ = store.outbox_failed(id, &reason).await;
            }
            None => {
                let _ = store.outbox_failed(id, "server did not answer for this record").await;
            }
        }
    }
    report
}

async fn send(url: &str, token: &str, request: &PushRequest) -> Result<PushResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| format!("Cannot create HTTP client: {e}"))?;
    let endpoint = format!("{}/api/v1/sync/push", url.trim_end_matches('/'));
    let response = client
        .post(&endpoint)
        .bearer_auth(token)
        .json(request)
        .send()
        .await
        .map_err(|_| "Server unreachable. Records stay queued and will be sent later.".to_string())?;
    match response.status().as_u16() {
        200 => response.json::<PushResponse>().await.map_err(|e| format!("Unexpected server answer: {e}")),
        401 => Err("The server rejected the access token.".into()),
        code => Err(format!("Server error (HTTP {code}). Records stay queued.")),
    }
}

/// Server URL must be plain http(s) without spaces.
pub fn validate_server_url(url: &str) -> Result<String, String> {
    let url = url.trim().trim_end_matches('/').to_string();
    let ok = (url.starts_with("http://") || url.starts_with("https://")) && !url.contains(char::is_whitespace) && url.len() > 8;
    if ok { Ok(url) } else { Err("Enter an address like http://127.0.0.1:8787".into()) }
}

#[cfg(test)]
mod tests {
    use super::validate_server_url;

    #[test]
    fn server_url_must_be_http_or_https() {
        assert_eq!(validate_server_url(" http://127.0.0.1:8787/ ").unwrap(), "http://127.0.0.1:8787");
        assert!(validate_server_url("ftp://x").is_err());
        assert!(validate_server_url("http://a b").is_err());
    }
}
