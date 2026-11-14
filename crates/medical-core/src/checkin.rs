//! Turns what the UI submitted into validated answer records plus an outcome.
//!
//! The UI is never trusted: follow-ups are re-planned here from the same
//! history, and only exactly those questions may be (and must be) answered.

use nurbit_shared::records::{AnswerRecord, AnswerValue};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::daily_check::{DailyAnswerError, DailyAnswers, validate_daily_answers};
use crate::followup::{FollowUpPlan, QuestionKind, TriggeredRule, plan_follow_ups};

pub const DAILY_RESTED_ID: &str = "daily.rested";
pub const DAILY_PHYSICAL_ID: &str = "daily.physical";
pub const DAILY_MOOD_ID: &str = "daily.mood_energy";

#[derive(Debug, Clone, Deserialize)]
pub struct SubmittedAnswer {
    pub question_id: String,
    pub value: AnswerValue,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckInSubmission {
    pub rested: u8,
    pub physical: u8,
    pub mood_energy: u8,
    pub follow_ups: Vec<SubmittedAnswer>,
}

/// Things the crew member and the specialist must see after saving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Flag {
    /// Safety question answered "yes": high priority.
    UrgentSymptoms,
    /// Crew member asked to be contacted by the medical team.
    ContactRequested,
    /// Reported vision change (relevant to the SANS module).
    VisionChange,
    /// Reported fall or impact (relevant to the bone module).
    FallOrImpact,
}

impl Flag {
    pub fn message(self) -> &'static str {
        match self {
            Flag::UrgentSymptoms => "You reported a symptom that may need urgent attention. Contact the crew medical officer or flight surgeon now. This is a safety rule of the prototype, not a diagnosis.",
            Flag::ContactRequested => "Your request to be contacted is recorded and will reach the medical team at the next synchronization.",
            Flag::VisionChange => "A vision change is recorded for specialist review.",
            Flag::FallOrImpact => "A fall or impact is recorded for specialist review.",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FlagNotice {
    pub flag: Flag,
    pub message: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckInOutcome {
    pub questions_asked: usize,
    pub follow_ups_asked: usize,
    pub triggered: Vec<TriggeredRule>,
    pub flags: Vec<FlagNotice>,
}

#[derive(Debug, Error, PartialEq)]
pub enum CheckInError {
    #[error(transparent)]
    Daily(#[from] DailyAnswerError),
    #[error("question {0} was not asked today")]
    UnexpectedQuestion(String),
    #[error("question {0} was asked but not answered")]
    MissingAnswer(String),
    #[error("question {0} was answered twice")]
    DuplicateAnswer(String),
    #[error("answer to {0} has the wrong type or is out of range")]
    InvalidValue(String),
}

fn value_fits(kind: &QuestionKind, value: &AnswerValue) -> bool {
    match (kind, value) {
        (QuestionKind::Scale { .. }, AnswerValue::Scale(v)) => *v <= 10,
        (QuestionKind::YesNo, AnswerValue::YesNo(_)) => true,
        (QuestionKind::Number { min, max, .. }, AnswerValue::Number(v)) => {
            v.is_finite() && *v >= *min && *v <= *max
        }
        (QuestionKind::Choice { options }, AnswerValue::Choice(c)) => {
            options.iter().any(|(id, _)| id == c)
        }
        _ => false,
    }
}

fn flags_for(answers: &[AnswerRecord]) -> Vec<FlagNotice> {
    let yes = |id: &str| {
        answers
            .iter()
            .any(|a| a.question_id == id && a.value == AnswerValue::YesNo(true))
    };
    [
        (yes("fu.red_flags"), Flag::UrgentSymptoms),
        (yes("fu.contact_request"), Flag::ContactRequested),
        (yes("fu.vision_change"), Flag::VisionChange),
        (yes("fu.fall_or_impact"), Flag::FallOrImpact),
    ]
    .into_iter()
    .filter(|(hit, _)| *hit)
    .map(|(_, flag)| FlagNotice { flag, message: flag.message() })
    .collect()
}

fn daily_records(daily: &DailyAnswers) -> Vec<AnswerRecord> {
    [
        (DAILY_RESTED_ID, daily.rested),
        (DAILY_PHYSICAL_ID, daily.physical),
        (DAILY_MOOD_ID, daily.mood_energy),
    ]
    .into_iter()
    .map(|(id, v)| AnswerRecord {
        question_id: id.to_string(),
        value: AnswerValue::Scale(v),
        triggered_by_rule: None,
    })
    .collect()
}

/// Validates a submission against today's follow-up plan.
/// `history` = previous daily answers of the same person, newest first.
pub fn evaluate_submission(
    submission: &CheckInSubmission,
    history: &[DailyAnswers],
) -> Result<(Vec<AnswerRecord>, CheckInOutcome), CheckInError> {
    let daily = validate_daily_answers(submission.rested, submission.physical, submission.mood_energy)?;
    let plan: FollowUpPlan = plan_follow_ups(&daily, history);

    let mut records = daily_records(&daily);
    for answer in &submission.follow_ups {
        let planned = plan
            .questions
            .iter()
            .find(|p| p.question.id == answer.question_id)
            .ok_or_else(|| CheckInError::UnexpectedQuestion(answer.question_id.clone()))?;
        if records.iter().any(|r| r.question_id == answer.question_id) {
            return Err(CheckInError::DuplicateAnswer(answer.question_id.clone()));
        }
        if !value_fits(&planned.question.kind, &answer.value) {
            return Err(CheckInError::InvalidValue(answer.question_id.clone()));
        }
        records.push(AnswerRecord {
            question_id: answer.question_id.clone(),
            value: answer.value.clone(),
            triggered_by_rule: Some(planned.rule_id.clone()),
        });
    }
    if let Some(missing) = plan
        .questions
        .iter()
        .find(|p| !records.iter().any(|r| r.question_id == p.question.id))
    {
        return Err(CheckInError::MissingAnswer(missing.question.id.to_string()));
    }

    let outcome = CheckInOutcome {
        questions_asked: records.len(),
        follow_ups_asked: plan.questions.len(),
        triggered: plan.triggered,
        flags: flags_for(&records),
    };
    Ok((records, outcome))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn submit(rested: u8, physical: u8, mood: u8, follow_ups: Vec<(&str, AnswerValue)>) -> CheckInSubmission {
        CheckInSubmission {
            rested,
            physical,
            mood_energy: mood,
            follow_ups: follow_ups
                .into_iter()
                .map(|(id, value)| SubmittedAnswer { question_id: id.into(), value })
                .collect(),
        }
    }

    fn body_answers(red_flag: bool) -> Vec<(&'static str, AnswerValue)> {
        vec![
            ("fu.red_flags", AnswerValue::YesNo(red_flag)),
            ("fu.symptom_area", AnswerValue::Choice("back".into())),
            ("fu.fall_or_impact", AnswerValue::YesNo(false)),
            ("fu.vision_change", AnswerValue::YesNo(false)),
        ]
    }

    #[test]
    fn routine_check_saves_three_answers_and_no_flags() {
        let (records, outcome) = evaluate_submission(&submit(8, 8, 8, vec![]), &[]).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(outcome.follow_ups_asked, 0);
        assert!(outcome.flags.is_empty());
    }

    #[test]
    fn red_flag_answer_produces_urgent_flag() {
        let (_, outcome) = evaluate_submission(&submit(8, 2, 8, body_answers(true)), &[]).unwrap();
        assert_eq!(outcome.flags[0].flag, Flag::UrgentSymptoms);
    }

    #[test]
    fn follow_up_records_remember_their_rule() {
        let (records, _) = evaluate_submission(&submit(8, 2, 8, body_answers(false)), &[]).unwrap();
        let follow = records.iter().find(|r| r.question_id == "fu.symptom_area").unwrap();
        assert_eq!(follow.triggered_by_rule.as_deref(), Some("low.physical"));
    }

    #[test]
    fn skipping_the_safety_question_is_rejected() {
        let mut answers = body_answers(false);
        answers.retain(|(id, _)| *id != "fu.red_flags");
        let err = evaluate_submission(&submit(8, 2, 8, answers), &[]).unwrap_err();
        assert_eq!(err, CheckInError::MissingAnswer("fu.red_flags".into()));
    }

    #[test]
    fn answering_a_question_that_was_not_asked_is_rejected() {
        let err = evaluate_submission(
            &submit(8, 8, 8, vec![("fu.stress", AnswerValue::Scale(5))]),
            &[],
        )
        .unwrap_err();
        assert_eq!(err, CheckInError::UnexpectedQuestion("fu.stress".into()));
    }

    #[test]
    fn out_of_range_number_is_rejected() {
        let answers = vec![
            ("fu.sleep_hours", AnswerValue::Number(30.0)),
            ("fu.sleep_interrupted", AnswerValue::YesNo(true)),
            ("fu.alertness", AnswerValue::Scale(4)),
        ];
        let err = evaluate_submission(&submit(2, 8, 8, answers), &[]).unwrap_err();
        assert_eq!(err, CheckInError::InvalidValue("fu.sleep_hours".into()));
    }

    #[test]
    fn unknown_choice_option_is_rejected() {
        let mut answers = body_answers(false);
        answers[1].1 = AnswerValue::Choice("tail".into());
        let err = evaluate_submission(&submit(8, 2, 8, answers), &[]).unwrap_err();
        assert_eq!(err, CheckInError::InvalidValue("fu.symptom_area".into()));
    }
}
