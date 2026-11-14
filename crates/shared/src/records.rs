//! Shapes of stored records, shared by local storage, the desktop app and
//! (later) the server. Plain data only.

use serde::{Deserialize, Serialize};

use crate::{DataOrigin, RecordId};

/// Milliseconds since 1970-01-01 UTC. One integer type for every timestamp
/// keeps storage portable (SQLite now, PostgreSQL later).
pub type TimestampMs = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrewRole {
    Astronaut,
    Specialist,
}

impl CrewRole {
    pub fn as_str(self) -> &'static str {
        match self {
            CrewRole::Astronaut => "astronaut",
            CrewRole::Specialist => "specialist",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "astronaut" => Some(CrewRole::Astronaut),
            "specialist" => Some(CrewRole::Specialist),
            _ => None,
        }
    }
}

/// A person who uses Nurbit. All beta profiles are synthetic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrewMember {
    pub id: RecordId,
    pub display_name: String,
    pub role: CrewRole,
    /// Mission position, e.g. "Commander" (free text, synthetic).
    pub position: String,
    pub is_synthetic: bool,
}

/// How an answer is expressed. Every answer stores exactly one of these.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum AnswerValue {
    /// 0-10 rating.
    Scale(u8),
    YesNo(bool),
    /// Number with a unit given by the question (e.g. hours of sleep).
    Number(f64),
    /// One option id from a fixed list.
    Choice(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnswerRecord {
    /// Stable question id, e.g. "daily.rested" or "fu.sleep_hours".
    pub question_id: String,
    pub value: AnswerValue,
    /// Rule that caused a follow-up question; `None` for mandatory questions.
    pub triggered_by_rule: Option<String>,
}

/// One completed daily check-in (mandatory answers + any follow-ups).
/// Immutable once stored; corrections will be separate annotation records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckInRecord {
    pub id: RecordId,
    pub crew_member_id: RecordId,
    pub started_at: TimestampMs,
    pub completed_at: TimestampMs,
    pub device_id: String,
    pub origin: DataOrigin,
    pub answers: Vec<AnswerRecord>,
}

/// One numeric value from a device, an exam or a manual entry.
/// Immutable once stored, like check-ins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    pub id: RecordId,
    pub crew_member_id: RecordId,
    /// Stable indicator id, e.g. "exercise.resistive_minutes".
    pub indicator: String,
    pub value: f64,
    pub unit: String,
    pub measured_at: TimestampMs,
    pub origin: DataOrigin,
}

// ---------- specialist records (stage 9): human judgment, kept apart from machine output ----------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecialistNote {
    pub id: RecordId,
    pub crew_member_id: RecordId,
    pub author_id: RecordId,
    pub created_at: TimestampMs,
    pub text: String,
}

/// A correction or comment attached to one answer. The original is never changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub id: RecordId,
    pub checkin_id: RecordId,
    pub question_id: String,
    pub author_id: RecordId,
    pub created_at: TimestampMs,
    /// Corrected value as text (empty when the annotation is only a comment).
    pub corrected_value: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaselineDecision {
    /// Use a new baseline built from values recorded from now on.
    ApproveNewBaseline,
    /// The change does not justify a new baseline.
    KeepCurrentBaseline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineReview {
    pub id: RecordId,
    pub crew_member_id: RecordId,
    /// Daily question id, e.g. "daily.rested".
    pub indicator: String,
    pub decision: BaselineDecision,
    pub decided_by: RecordId,
    pub decided_at: TimestampMs,
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionKind {
    FollowUpExam,
    ClearForDuty,
    /// Recommendation to remove from operational duties: needs a second specialist.
    DutyRestriction,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub id: RecordId,
    pub crew_member_id: RecordId,
    pub author_id: RecordId,
    pub created_at: TimestampMs,
    pub kind: DecisionKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Confirm,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionReview {
    pub id: RecordId,
    pub decision_id: RecordId,
    pub reviewer_id: RecordId,
    pub created_at: TimestampMs,
    pub verdict: Verdict,
    pub comment: String,
}

impl CheckInRecord {
    pub fn duration_ms(&self) -> i64 {
        self.completed_at - self.started_at
    }

    pub fn follow_up_count(&self) -> usize {
        self.answers.iter().filter(|a| a.triggered_by_rule.is_some()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_round_trips_through_its_text_form() {
        for role in [CrewRole::Astronaut, CrewRole::Specialist] {
            assert_eq!(CrewRole::parse(role.as_str()), Some(role));
        }
        assert_eq!(CrewRole::parse("admin"), None);
    }

    #[test]
    fn answer_value_json_is_tagged_with_its_kind() {
        let json = serde_json::to_string(&AnswerValue::Scale(7)).unwrap();
        assert_eq!(json, r#"{"kind":"scale","value":7}"#);
    }
}
