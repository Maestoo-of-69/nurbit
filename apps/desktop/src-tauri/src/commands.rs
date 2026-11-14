//! Commands the interface can call. Each one validates its input, delegates to
//! `medical-core` / `local-store`, and returns either data or a readable error
//! message (Tauri sends `Err(String)` to the UI as a rejected promise).

use nurbit_local_store::{AssistantLogEntry, StoreError};
use nurbit_medical_core::assistant::{AssistantAnswer, AssistantQuery, ProviderInfo, answer, providers};
use nurbit_medical_core::baseline::{IndicatorTrend, assess_trend};
use nurbit_medical_core::checkin::{
    CheckInOutcome, CheckInSubmission, DAILY_MOOD_ID, DAILY_PHYSICAL_ID, DAILY_RESTED_ID, evaluate_submission,
};
use nurbit_medical_core::review::baseline_effective_from;
use nurbit_medical_core::risk_eval::{
    CrossRiskPattern, Finding, ModuleAssessment, PersonData, cross_risk_patterns, evaluate_all, safety_findings,
};
use nurbit_shared::records::AnswerValue;
use nurbit_medical_core::daily_check::{DAILY_QUESTIONS, DailyQuestion, validate_daily_answers};
use nurbit_medical_core::followup::{BASELINE_WINDOW, FollowUpPlan, plan_follow_ups as plan};
use nurbit_medical_core::risk_catalog::{RISK_MODULES, RiskModuleInfo};
use nurbit_shared::records::{CheckInRecord, CrewMember, CrewRole};
use nurbit_shared::{DataOrigin, RecordId};
use serde::Serialize;
use tauri::State;

use crate::sync::{SyncReport, run_sync, validate_server_url};
use crate::{AppState, now_ms};
use nurbit_local_store::sync_store::{SETTING_SERVER_URL, SETTING_SYNC_TOKEN};

pub(crate) type CmdResult<T> = Result<T, String>;

pub(crate) fn storage_error(e: StoreError) -> String {
    format!("Local storage problem: {e}")
}

pub(crate) fn parse_member(id: &str) -> CmdResult<RecordId> {
    RecordId::parse(id).ok_or_else(|| "Unknown crew member id.".to_string())
}

/// Only an existing astronaut may submit a daily check.
async fn require_astronaut(state: &AppState, id: RecordId) -> CmdResult<CrewMember> {
    let member = state
        .store
        .get_crew_member(id)
        .await
        .map_err(storage_error)?
        .ok_or("Crew member not found.")?;
    if member.role != CrewRole::Astronaut {
        return Err("Only astronauts complete daily checks.".into());
    }
    Ok(member)
}

#[derive(Serialize)]
pub struct AppInfo {
    name: &'static str,
    version: &'static str,
    notice: &'static str,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "Nurbit",
        version: env!("CARGO_PKG_VERSION"),
        notice: nurbit_shared::SYNTHETIC_DATA_NOTICE,
    }
}

#[tauri::command]
pub fn daily_questions() -> Vec<DailyQuestion> {
    DAILY_QUESTIONS.to_vec()
}

#[tauri::command]
pub fn risk_modules() -> Vec<RiskModuleInfo> {
    RISK_MODULES.to_vec()
}

#[tauri::command]
pub async fn list_crew(state: State<'_, AppState>) -> CmdResult<Vec<CrewMember>> {
    state.store.list_crew().await.map_err(storage_error)
}

/// Which follow-up questions today's three answers lead to, and why.
#[tauri::command]
pub async fn plan_follow_ups(
    state: State<'_, AppState>,
    member_id: String,
    rested: u8,
    physical: u8,
    mood_energy: u8,
) -> CmdResult<FollowUpPlan> {
    let member = parse_member(&member_id)?;
    require_astronaut(&state, member).await?;
    let today = validate_daily_answers(rested, physical, mood_energy).map_err(|e| e.to_string())?;
    let history = state
        .store
        .recent_daily_answers(member, BASELINE_WINDOW as i64)
        .await
        .map_err(storage_error)?;
    Ok(plan(&today, &history))
}

#[derive(Serialize)]
pub struct SaveResult {
    checkin_id: String,
    duration_ms: i64,
    outcome: CheckInOutcome,
}

#[tauri::command]
pub async fn save_checkin(
    state: State<'_, AppState>,
    member_id: String,
    started_at: i64,
    submission: CheckInSubmission,
) -> CmdResult<SaveResult> {
    let member = parse_member(&member_id)?;
    require_astronaut(&state, member).await?;
    let completed_at = now_ms();
    if started_at > completed_at {
        return Err("Start time is in the future.".into());
    }
    let history = state
        .store
        .recent_daily_answers(member, BASELINE_WINDOW as i64)
        .await
        .map_err(storage_error)?;
    let (answers, outcome) = evaluate_submission(&submission, &history).map_err(|e| e.to_string())?;

    let record = CheckInRecord {
        id: RecordId::new(),
        crew_member_id: member,
        started_at,
        completed_at,
        device_id: state.device_id.clone(),
        origin: DataOrigin::SelfReport,
        answers,
    };
    state.store.save_checkin(&record, completed_at, true).await.map_err(storage_error)?;
    Ok(SaveResult { checkin_id: record.id.to_string(), duration_ms: record.duration_ms(), outcome })
}

#[tauri::command]
pub async fn list_checkins(state: State<'_, AppState>, member_id: String) -> CmdResult<Vec<CheckInRecord>> {
    let member = parse_member(&member_id)?;
    state.store.list_checkins(member, 60).await.map_err(storage_error)
}

/// How many check-ins feed trends and assessments.
const ANALYSIS_LIMIT: i64 = 60;

#[derive(Serialize)]
pub struct TrendPoint {
    at: i64,
    value: f64,
    simulated: bool,
}

#[derive(Serialize)]
pub struct IndicatorSeries {
    question_id: &'static str,
    label: &'static str,
    /// Oldest first, ready for a chart.
    points: Vec<TrendPoint>,
    trend: IndicatorTrend,
    /// Set when a specialist approved a new baseline at this time.
    baseline_since: Option<i64>,
}

/// Daily-rating series with the person's own baseline and deviation status.
#[tauri::command]
pub async fn member_trends(state: State<'_, AppState>, member_id: String) -> CmdResult<Vec<IndicatorSeries>> {
    load_trends(&state, parse_member(&member_id)?).await
}

/// Builds the three trend series. If a specialist approved a new baseline for
/// an indicator, only values recorded after that approval form the baseline.
pub(crate) async fn load_trends(state: &AppState, member: RecordId) -> CmdResult<Vec<IndicatorSeries>> {
    let reviews = state.store.list_baseline_reviews(member).await.map_err(storage_error)?;
    let checkins = state.store.list_checkins(member, ANALYSIS_LIMIT).await.map_err(storage_error)?;
    let series = [
        (DAILY_RESTED_ID, "Rested"),
        (DAILY_PHYSICAL_ID, "Body"),
        (DAILY_MOOD_ID, "Mood and energy"),
    ];
    Ok(series
        .into_iter()
        .map(|(question_id, label)| {
            let newest_first: Vec<TrendPoint> = checkins
                .iter()
                .filter_map(|c| {
                    let a = c.answers.iter().find(|a| a.question_id == question_id)?;
                    match a.value {
                        AnswerValue::Scale(v) => Some(TrendPoint {
                            at: c.completed_at,
                            value: f64::from(v),
                            simulated: c.origin == DataOrigin::Simulated,
                        }),
                        _ => None,
                    }
                })
                .collect();
            let since = baseline_effective_from(&reviews, question_id);
            let values: Vec<f64> = newest_first
                .iter()
                .filter(|p| since.is_none_or(|s| p.at >= s))
                .map(|p| p.value)
                .collect();
            let trend = assess_trend(&values);
            let mut points = newest_first;
            points.reverse();
            IndicatorSeries { question_id, label, points, trend, baseline_since: since }
        })
        .collect())
}

#[derive(Serialize)]
pub struct Assessments {
    pub(crate) modules: Vec<ModuleAssessment>,
    pub(crate) patterns: Vec<CrossRiskPattern>,
    /// Red-flag answers: outside the five modules, always shown to specialists.
    pub(crate) safety: Vec<Finding>,
}

/// Evaluates the five risk modules and cross-risk patterns for one person.
#[tauri::command]
pub async fn risk_assessments(state: State<'_, AppState>, member_id: String) -> CmdResult<Assessments> {
    load_assessments(&state, parse_member(&member_id)?).await
}

pub(crate) async fn load_assessments(state: &AppState, member: RecordId) -> CmdResult<Assessments> {
    let checkins = state.store.list_checkins(member, ANALYSIS_LIMIT).await.map_err(storage_error)?;
    let measurements = state.store.list_measurements(member).await.map_err(storage_error)?;
    let data = PersonData { checkins: &checkins, measurements: &measurements, now: now_ms() };
    Ok(Assessments { modules: evaluate_all(&data), patterns: cross_risk_patterns(&data), safety: safety_findings(&data) })
}

#[derive(Serialize)]
pub struct SyncStatus {
    pending: i64,
    sent: i64,
    conflict: i64,
    server_configured: bool,
    server_url: Option<String>,
    last: Option<SyncReport>,
}

#[tauri::command]
pub async fn sync_status(state: State<'_, AppState>) -> CmdResult<SyncStatus> {
    let counts = state.store.outbox_counts().await.map_err(storage_error)?;
    let server_url = state.store.get_setting(SETTING_SERVER_URL).await.map_err(storage_error)?;
    let has_token = state.store.get_setting(SETTING_SYNC_TOKEN).await.map_err(storage_error)?.is_some();
    let last = state.last_sync.lock().ok().and_then(|l| l.clone());
    Ok(SyncStatus {
        pending: counts.pending,
        sent: counts.sent,
        conflict: counts.conflict,
        server_configured: server_url.is_some() && has_token,
        server_url,
        last,
    })
}

/// Saves the server address; an empty token keeps the stored one.
/// Beta limitation: the token is stored unencrypted in the local database.
#[tauri::command]
pub async fn save_sync_settings(state: State<'_, AppState>, server_url: String, token: String) -> CmdResult<()> {
    let url = validate_server_url(&server_url)?;
    state.store.set_setting(SETTING_SERVER_URL, &url).await.map_err(storage_error)?;
    let token = token.trim();
    if !token.is_empty() {
        state.store.set_setting(SETTING_SYNC_TOKEN, token).await.map_err(storage_error)?;
    }
    Ok(())
}

#[tauri::command]
pub fn assistant_providers() -> Vec<ProviderInfo> {
    providers()
}

/// Answers from the verified local knowledge base and logs the question
/// (automated output, visible to specialists, never a diagnosis).
#[tauri::command]
pub async fn ask_assistant(state: State<'_, AppState>, member_id: String, query: AssistantQuery) -> CmdResult<AssistantAnswer> {
    let member = parse_member(&member_id)?;
    if query.text.trim().is_empty() {
        return Err("Describe the symptom or question first.".into());
    }
    if query.text.chars().count() > 1000 {
        return Err("Please keep the description under 1000 characters.".into());
    }
    if query.severity.is_some_and(|s| s > 10) {
        return Err("Severity must be between 0 and 10.".into());
    }
    let result = answer(&query);
    let entry = AssistantLogEntry {
        id: RecordId::new(),
        crew_member_id: member,
        created_at: now_ms(),
        provider: result.provider.to_string(),
        question: query.text.trim().to_string(),
        body_area: query.body_area.clone(),
        severity: query.severity.map(i64::from),
        matched_entries: result.matches.iter().map(|m| m.entry.id.clone()).collect(),
        urgent: result.urgent_message.is_some(),
    };
    state.store.insert_assistant_log(&entry).await.map_err(storage_error)?;
    Ok(result)
}

#[tauri::command]
pub async fn sync_now(state: State<'_, AppState>) -> CmdResult<SyncReport> {
    Ok(run_sync(&state).await)
}
