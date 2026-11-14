//! HTTP-level tests: requests go through the real router, with an in-memory database.

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use nurbit_shared::records::{AnswerRecord, AnswerValue, CheckInRecord, CrewMember, CrewRole};
use nurbit_shared::sync::{AckStatus, PushRequest, PushResponse, RecordKind, SyncRecord, make_record};
use nurbit_shared::{DataOrigin, RecordId};
use serde_json::Value;
use tower::ServiceExt;

use crate::{build_app, open_memory_database};

const TOKEN: &str = "test-token-0123456789";

async fn app() -> axum::Router {
    build_app(open_memory_database().await.unwrap(), TOKEN.to_string())
}

fn member() -> CrewMember {
    CrewMember { id: RecordId::new(), display_name: "Vega".into(), role: CrewRole::Astronaut, position: "Commander".into(), is_synthetic: true }
}

fn checkin(member: RecordId, rested: u8) -> CheckInRecord {
    CheckInRecord {
        id: RecordId::new(),
        crew_member_id: member,
        started_at: 1_000,
        completed_at: 2_000,
        device_id: "dev".into(),
        origin: DataOrigin::SelfReport,
        answers: vec![AnswerRecord { question_id: "daily.rested".into(), value: AnswerValue::Scale(rested), triggered_by_rule: None }],
    }
}

async fn send(app: &axum::Router, method: &str, uri: &str, token: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {t}"));
    }
    let body = body.map_or_else(Body::empty, |b| Body::from(b.to_string()));
    let response = app.clone().oneshot(builder.body(body).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1 << 20).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn push(app: &axum::Router, records: Vec<SyncRecord>) -> Vec<AckStatus> {
    let request = PushRequest { device_id: "dev".into(), records };
    let (status, body) = send(app, "POST", "/api/v1/sync/push", Some(TOKEN), Some(serde_json::to_value(request).unwrap())).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let response: PushResponse = serde_json::from_value(body).unwrap();
    response.acks.into_iter().map(|a| a.status).collect()
}

#[tokio::test]
async fn health_needs_no_token() {
    let (status, body) = send(&app().await, "GET", "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn api_rejects_missing_or_wrong_token() {
    let app = app().await;
    assert_eq!(send(&app, "GET", "/api/v1/crew", None, None).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(send(&app, "GET", "/api/v1/crew", Some("wrong-token-0000000000"), None).await.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn new_records_are_accepted_and_resending_is_a_duplicate() {
    let app = app().await;
    let m = member();
    let records = vec![
        make_record(RecordKind::CrewMember, m.id, &m).unwrap(),
        make_record(RecordKind::CheckIn, RecordId::new(), &()).unwrap(),
    ];
    // second record is malformed on purpose (payload is not a check-in)
    assert_eq!(push(&app, records.clone()).await, vec![AckStatus::Accepted, AckStatus::Invalid]);
    assert_eq!(push(&app, records[..1].to_vec()).await, vec![AckStatus::Duplicate]);
}

#[tokio::test]
async fn changed_content_for_a_known_id_is_a_conflict_and_the_original_is_kept() {
    let app = app().await;
    let m = member();
    let original = checkin(m.id, 8);
    let mut altered = original.clone();
    altered.answers[0].value = AnswerValue::Scale(2);

    push(&app, vec![make_record(RecordKind::CrewMember, m.id, &m).unwrap()]).await;
    assert_eq!(push(&app, vec![make_record(RecordKind::CheckIn, original.id, &original).unwrap()]).await, vec![AckStatus::Accepted]);
    let conflict = make_record(RecordKind::CheckIn, altered.id, &altered).unwrap();
    assert_eq!(push(&app, vec![conflict.clone()]).await, vec![AckStatus::Conflict]);
    // re-sending the same conflicting version is still reported, and stored once
    assert_eq!(push(&app, vec![conflict]).await, vec![AckStatus::Conflict]);

    let (_, stored) = send(&app, "GET", &format!("/api/v1/crew/{}/checkins", m.id), Some(TOKEN), None).await;
    let stored: Vec<CheckInRecord> = serde_json::from_value(stored).unwrap();
    assert_eq!(stored, vec![original], "the first version must stay unchanged");
}

#[tokio::test]
async fn tampered_hash_is_invalid() {
    let app = app().await;
    let m = member();
    let mut record = make_record(RecordKind::CrewMember, m.id, &m).unwrap();
    record.content_hash = "0".repeat(64);
    assert_eq!(push(&app, vec![record]).await, vec![AckStatus::Invalid]);
}

#[tokio::test]
async fn specialist_can_read_synchronized_crew() {
    let app = app().await;
    let m = member();
    push(&app, vec![make_record(RecordKind::CrewMember, m.id, &m).unwrap()]).await;
    let (status, body) = send(&app, "GET", "/api/v1/crew", Some(TOKEN), None).await;
    assert_eq!(status, StatusCode::OK);
    let crew: Vec<CrewMember> = serde_json::from_value(body).unwrap();
    assert_eq!(crew, vec![m]);
}
