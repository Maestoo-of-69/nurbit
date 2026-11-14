//! Storage tests against a fresh in-memory database.

use nurbit_shared::records::{AnswerRecord, AnswerValue, CheckInRecord, CrewMember, CrewRole};
use nurbit_shared::{DataOrigin, RecordId};

use crate::LocalStore;
use crate::seed::{SEED_DAYS, seed_demo_crew_if_empty, seed_demo_measurements_if_empty};

const NOW: i64 = 1_800_000_000_000;

async fn store_with_member() -> (LocalStore, CrewMember) {
    let store = LocalStore::open_in_memory().await.unwrap();
    let member = CrewMember {
        id: RecordId::new(),
        display_name: "Test".into(),
        role: CrewRole::Astronaut,
        position: "Tester".into(),
        is_synthetic: true,
    };
    store.insert_crew_member(&member, NOW).await.unwrap();
    (store, member)
}

fn checkin(member: RecordId, completed_at: i64, scores: [u8; 3], extra: Vec<AnswerRecord>) -> CheckInRecord {
    let mut answers: Vec<AnswerRecord> = ["daily.rested", "daily.physical", "daily.mood_energy"]
        .iter()
        .zip(scores)
        .map(|(id, v)| AnswerRecord { question_id: (*id).into(), value: AnswerValue::Scale(v), triggered_by_rule: None })
        .collect();
    answers.extend(extra);
    CheckInRecord {
        id: RecordId::new(),
        crew_member_id: member,
        started_at: completed_at - 60_000,
        completed_at,
        device_id: "test-device".into(),
        origin: DataOrigin::SelfReport,
        answers,
    }
}

#[tokio::test]
async fn saved_checkin_reads_back_identically() {
    let (store, member) = store_with_member().await;
    let extra = vec![
        AnswerRecord { question_id: "fu.sleep_hours".into(), value: AnswerValue::Number(5.5), triggered_by_rule: Some("low.rested".into()) },
        AnswerRecord { question_id: "fu.sleep_interrupted".into(), value: AnswerValue::YesNo(true), triggered_by_rule: Some("low.rested".into()) },
        AnswerRecord { question_id: "fu.symptom_area".into(), value: AnswerValue::Choice("back".into()), triggered_by_rule: Some("low.physical".into()) },
    ];
    let record = checkin(member.id, NOW, [2, 3, 7], extra);
    store.save_checkin(&record, NOW, true).await.unwrap();

    let loaded = store.list_checkins(member.id, 10).await.unwrap();
    assert_eq!(loaded, vec![record]);
}

#[tokio::test]
async fn saving_queues_the_record_for_sync() {
    let (store, member) = store_with_member().await;
    store.save_checkin(&checkin(member.id, NOW, [8, 8, 8], vec![]), NOW, true).await.unwrap();
    assert_eq!(store.pending_sync_count().await.unwrap(), 1);
}

#[tokio::test]
async fn saving_the_same_record_twice_is_rejected_not_duplicated() {
    let (store, member) = store_with_member().await;
    let record = checkin(member.id, NOW, [8, 8, 8], vec![]);
    store.save_checkin(&record, NOW, true).await.unwrap();
    assert!(store.save_checkin(&record, NOW, true).await.is_err());
    assert_eq!(store.list_checkins(member.id, 10).await.unwrap().len(), 1);
    assert_eq!(store.pending_sync_count().await.unwrap(), 1);
}

#[tokio::test]
async fn database_refuses_to_change_or_delete_medical_records() {
    let (store, member) = store_with_member().await;
    store.save_checkin(&checkin(member.id, NOW, [8, 8, 8], vec![]), NOW, true).await.unwrap();
    let update = sqlx::query("UPDATE checkin_answers SET value_num = 1").execute(&store.pool).await;
    let delete = sqlx::query("DELETE FROM checkins").execute(&store.pool).await;
    assert!(update.is_err(), "update must be blocked");
    assert!(delete.is_err(), "delete must be blocked");
}

#[tokio::test]
async fn recent_daily_answers_are_newest_first() {
    let (store, member) = store_with_member().await;
    store.save_checkin(&checkin(member.id, NOW - 1000, [1, 1, 1], vec![]), NOW, false).await.unwrap();
    store.save_checkin(&checkin(member.id, NOW, [9, 9, 9], vec![]), NOW, false).await.unwrap();
    let recent = store.recent_daily_answers(member.id, 7).await.unwrap();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].rested, 9);
}

#[tokio::test]
async fn device_id_is_created_once_and_then_reused() {
    let store = LocalStore::open_in_memory().await.unwrap();
    let first = store.device_id().await.unwrap();
    assert_eq!(store.device_id().await.unwrap(), first);
}

#[tokio::test]
async fn seeding_creates_synthetic_crew_once() {
    let store = LocalStore::open_in_memory().await.unwrap();
    assert!(seed_demo_crew_if_empty(&store, NOW).await.unwrap());
    assert!(!seed_demo_crew_if_empty(&store, NOW).await.unwrap());

    let crew = store.list_crew().await.unwrap();
    assert_eq!(crew.len(), 5);
    assert!(crew.iter().all(|m| m.is_synthetic));

    let astronaut = crew.iter().find(|m| m.role == CrewRole::Astronaut).unwrap();
    let history = store.list_checkins(astronaut.id, 100).await.unwrap();
    assert_eq!(history.len() as i64, SEED_DAYS);
    assert!(history.iter().all(|c| c.origin == DataOrigin::Simulated));
    assert_eq!(store.pending_sync_count().await.unwrap(), 0, "seed data is never sent");
}

#[tokio::test]
async fn simulated_measurements_are_seeded_once_and_marked_simulated() {
    let store = LocalStore::open_in_memory().await.unwrap();
    seed_demo_crew_if_empty(&store, NOW).await.unwrap();
    assert!(seed_demo_measurements_if_empty(&store, NOW).await.unwrap());
    assert!(!seed_demo_measurements_if_empty(&store, NOW).await.unwrap());

    let crew = store.list_crew().await.unwrap();
    let astronaut = crew.iter().find(|m| m.role == CrewRole::Astronaut).unwrap();
    let data = store.list_measurements(astronaut.id).await.unwrap();
    assert!(!data.is_empty());
    assert!(data.iter().all(|m| m.origin == DataOrigin::Simulated));
}

#[tokio::test]
async fn specialist_records_round_trip_and_cannot_be_changed() {
    use nurbit_shared::records::{Decision, DecisionKind, DecisionReview, SpecialistNote, Verdict};
    let (store, member) = store_with_member().await;
    let doctor = CrewMember {
        id: RecordId::new(),
        display_name: "Doc".into(),
        role: CrewRole::Specialist,
        position: "Surgeon".into(),
        is_synthetic: true,
    };
    store.insert_crew_member(&doctor, NOW).await.unwrap();

    let note = SpecialistNote { id: RecordId::new(), crew_member_id: member.id, author_id: doctor.id, created_at: NOW, text: "ok".into() };
    store.insert_note(&note).await.unwrap();
    assert_eq!(store.list_notes(member.id).await.unwrap(), vec![note]);

    let decision = Decision {
        id: RecordId::new(),
        crew_member_id: member.id,
        author_id: doctor.id,
        created_at: NOW,
        kind: DecisionKind::DutyRestriction,
        text: "rest".into(),
    };
    store.insert_decision(&decision).await.unwrap();
    let review = DecisionReview {
        id: RecordId::new(),
        decision_id: decision.id,
        reviewer_id: member.id,
        created_at: NOW,
        verdict: Verdict::Confirm,
        comment: String::new(),
    };
    store.insert_decision_review(&review).await.unwrap();
    assert_eq!(store.list_decisions(Some(member.id)).await.unwrap(), vec![decision]);
    assert_eq!(store.list_decision_reviews().await.unwrap(), vec![review]);

    assert!(sqlx::query("UPDATE decisions SET text = 'x'").execute(&store.pool).await.is_err());
    assert!(sqlx::query("DELETE FROM specialist_notes").execute(&store.pool).await.is_err());
}

#[tokio::test]
async fn second_specialist_is_added_once() {
    use crate::seed::seed_second_specialist_if_missing;
    let store = LocalStore::open_in_memory().await.unwrap();
    seed_demo_crew_if_empty(&store, NOW).await.unwrap();
    assert!(seed_second_specialist_if_missing(&store, NOW).await.unwrap());
    assert!(!seed_second_specialist_if_missing(&store, NOW).await.unwrap());
}

#[tokio::test]
async fn outbox_moves_from_pending_to_sent_or_conflict_and_failures_keep_it_pending() {
    use crate::sync_store::OutboxStatus;
    let (store, member) = store_with_member().await;
    let a = checkin(member.id, NOW - 2, [8, 8, 8], vec![]);
    let b = checkin(member.id, NOW - 1, [7, 7, 7], vec![]);
    store.save_checkin(&a, NOW, true).await.unwrap();
    store.save_checkin(&b, NOW, true).await.unwrap();

    let pending = store.pending_outbox(10).await.unwrap();
    assert_eq!(pending.iter().map(|p| p.record_id).collect::<Vec<_>>(), vec![a.id, b.id]);
    assert_eq!(store.get_checkin(a.id).await.unwrap(), Some(a.clone()));

    store.outbox_failed(a.id, "server unreachable").await.unwrap();
    assert_eq!(store.pending_outbox(10).await.unwrap()[0].attempts, 1);

    store.mark_outbox(a.id, OutboxStatus::Sent).await.unwrap();
    store.mark_outbox(b.id, OutboxStatus::Conflict).await.unwrap();
    let counts = store.outbox_counts().await.unwrap();
    assert_eq!((counts.pending, counts.sent, counts.conflict), (0, 1, 1));
}

#[tokio::test]
async fn assistant_log_round_trips_and_is_append_only() {
    use crate::AssistantLogEntry;
    let (store, member) = store_with_member().await;
    let entry = AssistantLogEntry {
        id: RecordId::new(),
        crew_member_id: member.id,
        created_at: NOW,
        provider: "retrieval_only".into(),
        question: "blurred vision".into(),
        body_area: Some("eyes".into()),
        severity: Some(3),
        matched_entries: vec!["kb-sans-overview".into(), "kb-sans-monitoring".into()],
        urgent: false,
    };
    store.insert_assistant_log(&entry).await.unwrap();
    assert_eq!(store.list_assistant_log(member.id).await.unwrap(), vec![entry]);
    assert!(sqlx::query("DELETE FROM assistant_log").execute(&store.pool).await.is_err());
}

#[tokio::test]
async fn settings_can_be_saved_and_replaced() {
    let store = LocalStore::open_in_memory().await.unwrap();
    assert_eq!(store.get_setting("k").await.unwrap(), None);
    store.set_setting("k", "1").await.unwrap();
    store.set_setting("k", "2").await.unwrap();
    assert_eq!(store.get_setting("k").await.unwrap().as_deref(), Some("2"));
}

#[tokio::test]
async fn measurements_cannot_be_changed() {
    let store = LocalStore::open_in_memory().await.unwrap();
    seed_demo_crew_if_empty(&store, NOW).await.unwrap();
    seed_demo_measurements_if_empty(&store, NOW).await.unwrap();
    assert!(sqlx::query("UPDATE measurements SET value = 0").execute(&store.pool).await.is_err());
}
