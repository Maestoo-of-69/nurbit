// The only place the interface talks to Rust. Each function wraps one Tauri
// command (an in-process call, not HTTP). Types mirror the Rust structs and
// are kept by hand for the beta (Rust is the source of truth).
import { invoke } from "@tauri-apps/api/core";

export interface AppInfo {
  name: string;
  version: string;
  notice: string;
}

export interface DailyQuestion {
  id: string;
  text: string;
  low_anchor: string;
  high_anchor: string;
}

export type HrpCategory = "human_physiology" | "behavioral_performance" | "space_radiation";

export type IndicatorClass =
  | "directly_measured"
  | "self_reported"
  | "derived"
  | "risk_estimate"
  | "specialist_finding";
export type Collection = "automatic" | "self_collected" | "medical_test" | "laboratory" | "calculated";
export type Frequency = "continuous" | "daily" | "periodic" | "pre_and_post_flight" | "symptom_based";
export type BetaSource =
  | "daily_questionnaire"
  | "follow_up_questionnaire"
  | "manual_entry"
  | "simulated_device"
  | "not_in_beta";

export interface Indicator {
  name: string;
  class: IndicatorClass;
  collection: Collection;
  frequency: Frequency;
  available_in_flight: boolean;
  baseline_comparable: boolean;
  equipment: string | null;
  beta_source: BetaSource;
}

export interface RiskModuleInfo {
  id: string;
  hrp_title: string;
  short_title: string;
  category: HrpCategory;
  monitored: string;
  indicators: Indicator[];
  reference_limits: { description: string; source_id: string; applies: string }[];
  questionnaire_triggers: string[];
  specialist_package: string[];
  limitations: string[];
  reference_ids: string[];
  status: "approved_not_active" | "active";
}

export type CrewRole = "astronaut" | "specialist";

export interface CrewMember {
  id: string;
  display_name: string;
  role: CrewRole;
  position: string;
  is_synthetic: boolean;
}

export type AnswerValue =
  | { kind: "scale"; value: number }
  | { kind: "yes_no"; value: boolean }
  | { kind: "number"; value: number }
  | { kind: "choice"; value: string };

export type QuestionKind =
  | { type: "scale"; low_anchor: string; high_anchor: string }
  | { type: "yes_no" }
  | { type: "number"; unit: string; min: number; max: number }
  | { type: "choice"; options: [string, string][] };

export type Domain = "sleep" | "body" | "mood";

export interface FollowUpQuestion {
  id: string;
  domain: Domain;
  text: string;
  kind: QuestionKind;
  is_safety: boolean;
}

export interface TriggeredRule {
  rule_id: string;
  domain: Domain;
  reason: string;
  basis: string;
}

export interface FollowUpPlan {
  triggered: TriggeredRule[];
  questions: { question: FollowUpQuestion; rule_id: string }[];
}

export interface DailyScores {
  rested: number;
  physical: number;
  mood_energy: number;
}

export interface SubmittedAnswer {
  question_id: string;
  value: AnswerValue;
}

export type Flag = "urgent_symptoms" | "contact_requested" | "vision_change" | "fall_or_impact";

export interface SaveResult {
  checkin_id: string;
  duration_ms: number;
  outcome: {
    questions_asked: number;
    follow_ups_asked: number;
    triggered: TriggeredRule[];
    flags: { flag: Flag; message: string }[];
  };
}

export interface AnswerRecord {
  question_id: string;
  value: AnswerValue;
  triggered_by_rule: string | null;
}

export type DataOrigin = "self_report" | "manual_entry" | "device" | "lab" | "simulated";

export interface CheckInRecord {
  id: string;
  crew_member_id: string;
  started_at: number;
  completed_at: number;
  device_id: string;
  origin: DataOrigin;
  answers: AnswerRecord[];
}

export interface SyncReport {
  at: number;
  outcome: "not_configured" | "nothing_to_send" | "ok" | "failed";
  accepted: number;
  duplicates: number;
  conflicts: number;
  invalid: number;
  error: string | null;
}

export interface SyncStatus {
  pending: number;
  sent: number;
  conflict: number;
  server_configured: boolean;
  server_url: string | null;
  last: SyncReport | null;
}

export const saveSyncSettings = (serverUrl: string, token: string) =>
  invoke<void>("save_sync_settings", { serverUrl, token });
export const syncNow = () => invoke<SyncReport>("sync_now");

// ---------- trends and assessments (stages 7-8) ----------

export interface Baseline {
  mean: number;
  low: number;
  high: number;
  values_used: number;
}

export type BaselineStatus =
  | { state: "insufficient_data" }
  | { state: "within_baseline" }
  | { state: "temporary_deviation"; direction: "below" | "above" }
  | { state: "persistent_change"; direction: "below" | "above"; count: number };

export interface IndicatorTrend {
  baseline: Baseline | null;
  status: BaselineStatus;
  latest: number | null;
  has_reference_range: boolean;
  basis: string;
}

export interface TrendPoint {
  at: number;
  value: number;
  simulated: boolean;
}

export interface IndicatorSeries {
  question_id: string;
  label: string;
  points: TrendPoint[];
  trend: IndicatorTrend;
  baseline_since: number | null;
}

export type Level = "no_data" | "nothing_to_report" | "attention" | "specialist_review";

export interface ModuleAssessment {
  module_id: string;
  level: Level;
  findings: { kind: string; level: Level; message: string; basis: string }[];
  gaps: { indicator: string; note: string }[];
  uses_simulated_data: boolean;
}

export type EvidenceLevel =
  | "observed_correlation"
  | "possible_association"
  | "established_relationship"
  | "unverified_hypothesis";

export interface CrossRiskPattern {
  first: string;
  second: string;
  r: number | null;
  pairs: number;
  in_your_data: EvidenceLevel;
  literature: EvidenceLevel;
  literature_note: string;
  literature_source: string | null;
}

export type Finding = ModuleAssessment["findings"][number];

export interface Assessments {
  modules: ModuleAssessment[];
  patterns: CrossRiskPattern[];
  /** Red-flag answers, outside the five modules. */
  safety: Finding[];
}

// ---------- specialist mode (stage 9) ----------

export type DecisionKind = "follow_up_exam" | "clear_for_duty" | "duty_restriction" | "other";
export type DecisionStatus = "recorded" | "pending_second_review" | "confirmed" | "rejected";
export type Verdict = "confirm" | "reject";
export type BaselineDecision = "approve_new_baseline" | "keep_current_baseline";

export interface CrewSummary {
  member: CrewMember;
  overall: Level;
  last_checkin_at: number | null;
  checked_in_today: boolean;
  flags: string[];
  pending_second_reviews: number;
}

export interface Decision {
  id: string;
  crew_member_id: string;
  author_id: string;
  created_at: number;
  kind: DecisionKind;
  text: string;
}

export interface DecisionReview {
  id: string;
  decision_id: string;
  reviewer_id: string;
  created_at: number;
  verdict: Verdict;
  comment: string;
}

export interface SpecialistNote {
  id: string;
  author_id: string;
  created_at: number;
  text: string;
}

export interface Annotation {
  id: string;
  checkin_id: string;
  question_id: string;
  author_id: string;
  created_at: number;
  corrected_value: string;
  reason: string;
}

export interface BaselineReview {
  id: string;
  indicator: string;
  decision: BaselineDecision;
  decided_by: string;
  decided_at: number;
  note: string;
}

export interface MemberRecord {
  member: CrewMember;
  overall: Level;
  safety: Finding[];
  assessments: ModuleAssessment[];
  patterns: CrossRiskPattern[];
  trends: IndicatorSeries[];
  checkins: CheckInRecord[];
  annotations: Annotation[];
  notes: SpecialistNote[];
  decisions: { decision: Decision; status: DecisionStatus; reviews: DecisionReview[] }[];
  baseline_reviews: BaselineReview[];
  assistant_log: AssistantLogEntry[];
  names: Record<string, string>;
}

export interface PendingReview {
  decision: Decision;
  member_name: string;
  author_name: string;
  can_review: boolean;
}

export const crewOverview = (actorId: string) => invoke<CrewSummary[]>("crew_overview", { actorId });
export const memberRecord = (actorId: string, memberId: string) =>
  invoke<MemberRecord>("member_record", { actorId, memberId });
export const addNote = (actorId: string, memberId: string, text: string) =>
  invoke<void>("add_note", { actorId, memberId, text });
export const annotateAnswer = (
  actorId: string,
  checkinId: string,
  questionId: string,
  correctedValue: string,
  reason: string,
) => invoke<void>("annotate_answer", { actorId, checkinId, questionId, correctedValue, reason });
export const reviewBaseline = (
  actorId: string,
  memberId: string,
  indicator: string,
  decision: BaselineDecision,
  note: string,
) => invoke<void>("review_baseline", { actorId, memberId, indicator, decision, note });
export const recordDecision = (actorId: string, memberId: string, kind: DecisionKind, text: string) =>
  invoke<void>("record_decision", { actorId, memberId, kind, text });
export const reviewDecision = (actorId: string, decisionId: string, verdict: Verdict, comment: string) =>
  invoke<void>("review_decision", { actorId, decisionId, verdict, comment });
export const pendingReviews = (actorId: string) => invoke<PendingReview[]>("pending_reviews", { actorId });

// ---------- assistant (stage 11) ----------

export interface KbEntry {
  id: string;
  title: string;
  summary: string;
  source_id: string;
  source_name: string;
  document: string;
  published: string;
  section: string;
  module: string | null;
}

export interface AssistantAnswer {
  provider: string;
  matches: { entry: KbEntry; score: number; matched_terms: string[] }[];
  related_modules: string[];
  urgent_message: string | null;
  medication_notice: string | null;
  missing_info: ("body_area" | "severity" | "onset")[];
  no_reliable_source: boolean;
  uncertainty: string;
}

export interface ProviderInfo {
  id: string;
  label: string;
  state: "active" | "not_installed" | "unavailable";
  note: string;
}

export interface AssistantLogEntry {
  id: string;
  created_at: number;
  provider: string;
  question: string;
  body_area: string | null;
  severity: number | null;
  matched_entries: string[];
  urgent: boolean;
}

export interface AssistantQuery {
  text: string;
  body_area: string | null;
  severity: number | null;
  onset: string | null;
}

export const assistantProviders = () => invoke<ProviderInfo[]>("assistant_providers");
export const askAssistant = (memberId: string, query: AssistantQuery) =>
  invoke<AssistantAnswer>("ask_assistant", { memberId, query });

export const getMemberTrends = (memberId: string) => invoke<IndicatorSeries[]>("member_trends", { memberId });
export const getRiskAssessments = (memberId: string) => invoke<Assessments>("risk_assessments", { memberId });

export const getAppInfo = () => invoke<AppInfo>("app_info");
export const getDailyQuestions = () => invoke<DailyQuestion[]>("daily_questions");
export const getRiskModules = () => invoke<RiskModuleInfo[]>("risk_modules");
export const listCrew = () => invoke<CrewMember[]>("list_crew");
export const getSyncStatus = () => invoke<SyncStatus>("sync_status");

// Tauri maps camelCase JS argument names to snake_case Rust parameter names.
export const planFollowUps = (memberId: string, scores: DailyScores) =>
  invoke<FollowUpPlan>("plan_follow_ups", {
    memberId,
    rested: scores.rested,
    physical: scores.physical,
    moodEnergy: scores.mood_energy,
  });

// `submission` is one struct on the Rust side, so its fields stay snake_case.
export const saveCheckin = (
  memberId: string,
  startedAt: number,
  scores: DailyScores,
  followUps: SubmittedAnswer[],
) =>
  invoke<SaveResult>("save_checkin", {
    memberId,
    startedAt,
    submission: { ...scores, follow_ups: followUps },
  });

export const listCheckins = (memberId: string) => invoke<CheckInRecord[]>("list_checkins", { memberId });

/** Score of one daily question in a stored check-in, if present. */
export function dailyScore(record: CheckInRecord, questionId: string): number | null {
  const answer = record.answers.find((a) => a.question_id === questionId);
  return answer?.value.kind === "scale" ? answer.value.value : null;
}
