//! The three mandatory daily questions (Module A) and answer validation.
//!
//! Wording and scale direction are a draft pending specialist review
//! (see docs/PROJECT_SPECIFICATION.md, Module A, and decision D-5).
//! All three questions use 10 = best, so the direction never flips.

use serde::Serialize;
use thiserror::Error;

/// Lowest and highest allowed answer on every daily scale.
pub const SCALE_MIN: u8 = 0;
pub const SCALE_MAX: u8 = 10;

/// Stable identifiers: stored in the database, so never rename them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DailyQuestionId {
    Rested,
    Physical,
    MoodEnergy,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyQuestion {
    pub id: DailyQuestionId,
    pub text: &'static str,
    /// What 0 means, shown under the left end of the scale.
    pub low_anchor: &'static str,
    /// What 10 means, shown under the right end of the scale.
    pub high_anchor: &'static str,
}

pub const DAILY_QUESTIONS: [DailyQuestion; 3] = [
    DailyQuestion {
        id: DailyQuestionId::Rested,
        text: "How rested do you feel after your last sleep period?",
        low_anchor: "Not rested at all",
        high_anchor: "Fully rested",
    },
    DailyQuestion {
        id: DailyQuestionId::Physical,
        text: "How is your body today: pain, discomfort, illness or other physical symptoms?",
        low_anchor: "Worst I can imagine",
        high_anchor: "No symptoms at all",
    },
    DailyQuestion {
        id: DailyQuestionId::MoodEnergy,
        text: "How are your mood and mental energy today?",
        low_anchor: "Very low",
        high_anchor: "Very good",
    },
];

/// A complete set of answers to the three daily questions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DailyAnswers {
    pub rested: u8,
    pub physical: u8,
    pub mood_energy: u8,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DailyAnswerError {
    #[error("answer to {question:?} is {value}, but must be between {SCALE_MIN} and {SCALE_MAX}")]
    OutOfRange { question: DailyQuestionId, value: u8 },
}

/// Accepts the three answers only if every one is on the 0-10 scale.
/// Missing answers cannot happen: the type requires all three.
pub fn validate_daily_answers(
    rested: u8,
    physical: u8,
    mood_energy: u8,
) -> Result<DailyAnswers, DailyAnswerError> {
    let check = |question, value| {
        if (SCALE_MIN..=SCALE_MAX).contains(&value) {
            Ok(value)
        } else {
            Err(DailyAnswerError::OutOfRange { question, value })
        }
    };
    Ok(DailyAnswers {
        rested: check(DailyQuestionId::Rested, rested)?,
        physical: check(DailyQuestionId::Physical, physical)?,
        mood_energy: check(DailyQuestionId::MoodEnergy, mood_energy)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_exactly_three_mandatory_questions_with_unique_ids() {
        let ids: std::collections::HashSet<_> = DAILY_QUESTIONS.iter().map(|q| q.id).collect();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn accepts_both_ends_of_the_scale() {
        assert!(validate_daily_answers(SCALE_MIN, SCALE_MAX, 5).is_ok());
    }

    #[test]
    fn rejects_value_above_ten_and_names_the_question() {
        let err = validate_daily_answers(5, 11, 5).unwrap_err();
        assert_eq!(
            err,
            DailyAnswerError::OutOfRange { question: DailyQuestionId::Physical, value: 11 }
        );
    }
}
