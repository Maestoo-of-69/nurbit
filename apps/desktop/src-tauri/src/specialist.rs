//! Specialist mode commands (Module H).
//!
//! Every command first checks in Rust that `actor_id` belongs to a specialist,
//! so the interface alone cannot unlock these actions. Beta limitation: the
//! actor is the profile chosen at start-up (no password yet).

use std::collections::HashMap;

use nurbit_local_store::AssistantLogEntry;
use nurbit_medical_core::review::{DecisionStatus, check_reviewer, decision_status, validate_text};
use nurbit_medical_core::risk_eval::{CrossRiskPattern, Finding, Level, ModuleAssessment};
use nurbit_shared::RecordId;
use nurbit_shared::records::{
    Annotation, BaselineDecision, BaselineReview, CheckInRecord, CrewMember, CrewRole, Decision, DecisionKind,
    DecisionReview, SpecialistNote, Verdict,
};
use serde::Serialize;
use tauri::State;

use crate::commands::{CmdResult, IndicatorSeries, load_assessments, load_trends, parse_member, storage_error};
use crate::{AppState, now_ms};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

async fn require_specialist(state: &AppState, actor_id: &str) -> CmdResult<CrewMember> {
    let id = parse_member(actor_id)?;
    let actor = state.store.get_crew_member(id).await.map_err(storage_error)?.ok_or("Unknown user.")?;
    if actor.role != CrewRole::Specialist {
        return Err("This action is available to medical specialists only.".into());
    }
    Ok(actor)
}

async fn require_astronaut_record(state: &AppState, member_id: &str) -> CmdResult<CrewMember> {
    let member = state
        .store
        .get_crew_member(parse_member(member_id)?)
        .await
        .map_err(storage_error)?
        .ok_or("Crew member not found.")?;
    if member.role != CrewRole::Astronaut {
        return Err("Medical records exist for astronauts only.".into());
    }
    Ok(member)
}

async fn names(state: &AppState) -> CmdResult<HashMap<String, String>> {
    Ok(state
        .store
        .list_crew()
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(|m| (m.id.to_string(), m.display_name))
        .collect())
}

/// Highest level across the five modules and the safety findings.
fn overall(modules: &[ModuleAssessment], safety: &[Finding]) -> Level {
    modules.iter().map(|m| m.level).chain(safety.iter().map(|f| f.level)).max().unwrap_or(Level::NoData)
}

// ---------------- crew overview ----------------

#[derive(Serialize)]
pub struct CrewSummary {
    member: CrewMember,
    overall: Level,
    last_checkin_at: Option<i64>,
    checked_in_today: bool,
    /// Findings at "attention" or "specialist review" level.
    flags: Vec<String>,
    pending_second_reviews: usize,
}

#[tauri::command]
pub async fn crew_overview(state: State<'_, AppState>, actor_id: String) -> CmdResult<Vec<CrewSummary>> {
    require_specialist(&state, &actor_id).await?;
    let decisions = state.store.list_decisions(None).await.map_err(storage_error)?;
    let reviews = state.store.list_decision_reviews().await.map_err(storage_error)?;
    let now = now_ms();
    let mut out = Vec::new();
    for member in state.store.list_crew().await.map_err(storage_error)? {
        if member.role != CrewRole::Astronaut {
            continue;
        }
        let assessments = load_assessments(&state, member.id).await?;
        let latest = state.store.list_checkins(member.id, 1).await.map_err(storage_error)?;
        let last = latest.first().map(|c| c.completed_at);
        let flags = assessments
            .safety
            .iter()
            .chain(assessments.modules.iter().flat_map(|m| &m.findings))
            .filter(|f| f.level >= Level::Attention)
            .map(|f| f.message.clone())
            .collect();
        let pending = decisions
            .iter()
            .filter(|d| d.crew_member_id == member.id)
            .filter(|d| decision_status(d, &reviews) == DecisionStatus::PendingSecondReview)
            .count();
        out.push(CrewSummary {
            overall: overall(&assessments.modules, &assessments.safety),
            last_checkin_at: last,
            checked_in_today: last.is_some_and(|t| now - t < DAY_MS),
            flags,
            pending_second_reviews: pending,
            member,
        });
    }
    Ok(out)
}

// ---------------- one member's record ----------------

#[derive(Serialize)]
pub struct DecisionView {
    decision: Decision,
    status: DecisionStatus,
    reviews: Vec<DecisionReview>,
}

#[derive(Serialize)]
pub struct MemberRecord {
    member: CrewMember,
    overall: Level,
    safety: Vec<Finding>,
    assessments: Vec<ModuleAssessment>,
    patterns: Vec<CrossRiskPattern>,
    trends: Vec<IndicatorSeries>,
    checkins: Vec<CheckInRecord>,
    annotations: Vec<Annotation>,
    notes: Vec<SpecialistNote>,
    decisions: Vec<DecisionView>,
    baseline_reviews: Vec<BaselineReview>,
    /// Questions asked to the assistant (automated, not decisions).
    assistant_log: Vec<AssistantLogEntry>,
    /// Display names by id, for authors and reviewers.
    names: HashMap<String, String>,
}

#[tauri::command]
pub async fn member_record(state: State<'_, AppState>, actor_id: String, member_id: String) -> CmdResult<MemberRecord> {
    require_specialist(&state, &actor_id).await?;
    let member = require_astronaut_record(&state, &member_id).await?;
    let assessments = load_assessments(&state, member.id).await?;
    let all_reviews = state.store.list_decision_reviews().await.map_err(storage_error)?;
    let decisions = state
        .store
        .list_decisions(Some(member.id))
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(|d| DecisionView {
            status: decision_status(&d, &all_reviews),
            reviews: all_reviews.iter().filter(|r| r.decision_id == d.id).cloned().collect(),
            decision: d,
        })
        .collect();
    Ok(MemberRecord {
        overall: overall(&assessments.modules, &assessments.safety),
        safety: assessments.safety,
        trends: load_trends(&state, member.id).await?,
        checkins: state.store.list_checkins(member.id, 60).await.map_err(storage_error)?,
        annotations: state.store.list_annotations(member.id).await.map_err(storage_error)?,
        notes: state.store.list_notes(member.id).await.map_err(storage_error)?,
        baseline_reviews: state.store.list_baseline_reviews(member.id).await.map_err(storage_error)?,
        assistant_log: state.store.list_assistant_log(member.id).await.map_err(storage_error)?,
        names: names(&state).await?,
        assessments: assessments.modules,
        patterns: assessments.patterns,
        decisions,
        member,
    })
}

// ---------------- write actions ----------------

#[tauri::command]
pub async fn add_note(state: State<'_, AppState>, actor_id: String, member_id: String, text: String) -> CmdResult<()> {
    let actor = require_specialist(&state, &actor_id).await?;
    let member = require_astronaut_record(&state, &member_id).await?;
    let note = SpecialistNote {
        id: RecordId::new(),
        crew_member_id: member.id,
        author_id: actor.id,
        created_at: now_ms(),
        text: validate_text(&text).map_err(|e| e.to_string())?,
    };
    state.store.insert_note(&note).await.map_err(storage_error)
}

#[tauri::command]
pub async fn annotate_answer(
    state: State<'_, AppState>,
    actor_id: String,
    checkin_id: String,
    question_id: String,
    corrected_value: String,
    reason: String,
) -> CmdResult<()> {
    let actor = require_specialist(&state, &actor_id).await?;
    let checkin = RecordId::parse(&checkin_id).ok_or("Unknown check-in.")?;
    state.store.checkin_owner(checkin).await.map_err(storage_error)?.ok_or("Check-in not found.")?;
    let annotation = Annotation {
        id: RecordId::new(),
        checkin_id: checkin,
        question_id: validate_text(&question_id).map_err(|e| e.to_string())?,
        author_id: actor.id,
        created_at: now_ms(),
        corrected_value: corrected_value.trim().to_string(),
        reason: validate_text(&reason).map_err(|_| "A reason is required for every annotation.".to_string())?,
    };
    state.store.insert_annotation(&annotation).await.map_err(storage_error)
}

#[tauri::command]
pub async fn review_baseline(
    state: State<'_, AppState>,
    actor_id: String,
    member_id: String,
    indicator: String,
    decision: BaselineDecision,
    note: String,
) -> CmdResult<()> {
    let actor = require_specialist(&state, &actor_id).await?;
    let member = require_astronaut_record(&state, &member_id).await?;
    if !["daily.rested", "daily.physical", "daily.mood_energy"].contains(&indicator.as_str()) {
        return Err("Unknown indicator.".into());
    }
    let review = BaselineReview {
        id: RecordId::new(),
        crew_member_id: member.id,
        indicator,
        decision,
        decided_by: actor.id,
        decided_at: now_ms(),
        note: validate_text(&note).map_err(|_| "Please give a short reason.".to_string())?,
    };
    state.store.insert_baseline_review(&review).await.map_err(storage_error)
}

#[tauri::command]
pub async fn record_decision(
    state: State<'_, AppState>,
    actor_id: String,
    member_id: String,
    kind: DecisionKind,
    text: String,
) -> CmdResult<()> {
    let actor = require_specialist(&state, &actor_id).await?;
    let member = require_astronaut_record(&state, &member_id).await?;
    let decision = Decision {
        id: RecordId::new(),
        crew_member_id: member.id,
        author_id: actor.id,
        created_at: now_ms(),
        kind,
        text: validate_text(&text).map_err(|e| e.to_string())?,
    };
    state.store.insert_decision(&decision).await.map_err(storage_error)
}

#[tauri::command]
pub async fn review_decision(
    state: State<'_, AppState>,
    actor_id: String,
    decision_id: String,
    verdict: Verdict,
    comment: String,
) -> CmdResult<()> {
    let actor = require_specialist(&state, &actor_id).await?;
    let id = RecordId::parse(&decision_id).ok_or("Unknown decision.")?;
    let decision = state
        .store
        .list_decisions(None)
        .await
        .map_err(storage_error)?
        .into_iter()
        .find(|d| d.id == id)
        .ok_or("Decision not found.")?;
    let reviews = state.store.list_decision_reviews().await.map_err(storage_error)?;
    check_reviewer(&decision, &reviews, actor.id).map_err(|e| e.to_string())?;
    let review = DecisionReview {
        id: RecordId::new(),
        decision_id: id,
        reviewer_id: actor.id,
        created_at: now_ms(),
        verdict,
        comment: comment.trim().to_string(),
    };
    state.store.insert_decision_review(&review).await.map_err(storage_error)
}

// ---------------- pending second reviews ----------------

#[derive(Serialize)]
pub struct PendingReview {
    decision: Decision,
    member_name: String,
    author_name: String,
    /// False when the current specialist is the author.
    can_review: bool,
}

#[tauri::command]
pub async fn pending_reviews(state: State<'_, AppState>, actor_id: String) -> CmdResult<Vec<PendingReview>> {
    let actor = require_specialist(&state, &actor_id).await?;
    let names = names(&state).await?;
    let reviews = state.store.list_decision_reviews().await.map_err(storage_error)?;
    let name = |id: RecordId| names.get(&id.to_string()).cloned().unwrap_or_else(|| "Unknown".into());
    Ok(state
        .store
        .list_decisions(None)
        .await
        .map_err(storage_error)?
        .into_iter()
        .filter(|d| decision_status(d, &reviews) == DecisionStatus::PendingSecondReview)
        .map(|d| PendingReview {
            member_name: name(d.crew_member_id),
            author_name: name(d.author_id),
            can_review: d.author_id != actor.id,
            decision: d,
        })
        .collect())
}
