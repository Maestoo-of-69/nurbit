//! Adaptive follow-up questions (Module B).
//!
//! Decides which extra questions to ask after the three daily answers, and why.
//! Two rules per domain (sleep, body, mood):
//!   * low score   - today's answer is at or below LOW_SCORE_THRESHOLD;
//!   * drop        - today's answer is DROP_POINTS or more below the person's own
//!     average of their recent check-ins (only with enough history).
//!
//! Both thresholds are PROTOTYPE ASSUMPTIONS (no published source gives them;
//! see docs/REFERENCES.md) and must be validated by a medical specialist.

use serde::Serialize;

use crate::daily_check::DailyAnswers;

/// Answers at or below this value trigger follow-ups.
pub const LOW_SCORE_THRESHOLD: u8 = 3;
/// A drop of this many points below the personal average triggers follow-ups.
pub const DROP_POINTS: f64 = 3.0;
/// The personal average uses at most this many previous check-ins...
pub const BASELINE_WINDOW: usize = 7;
/// ...and is used only when at least this many exist.
pub const BASELINE_MIN_HISTORY: usize = 5;
/// Shown next to every rule until a specialist validates the numbers.
pub const RULE_BASIS: &str = "prototype_assumption";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Sleep,
    Body,
    Mood,
}

impl Domain {
    fn key(self) -> &'static str {
        match self {
            Domain::Sleep => "rested",
            Domain::Body => "physical",
            Domain::Mood => "mood_energy",
        }
    }

    /// Name of the daily question, used in human-readable reasons.
    fn label(self) -> &'static str {
        match self {
            Domain::Sleep => "Rested",
            Domain::Body => "Body",
            Domain::Mood => "Mood and energy",
        }
    }

    fn score(self, answers: &DailyAnswers) -> u8 {
        match self {
            Domain::Sleep => answers.rested,
            Domain::Body => answers.physical,
            Domain::Mood => answers.mood_energy,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QuestionKind {
    Scale { low_anchor: &'static str, high_anchor: &'static str },
    YesNo,
    Number { unit: &'static str, min: f64, max: f64 },
    Choice { options: &'static [(&'static str, &'static str)] },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FollowUpQuestion {
    pub id: &'static str,
    pub domain: Domain,
    pub text: &'static str,
    pub kind: QuestionKind,
    /// Safety questions are never skipped once their domain is triggered.
    pub is_safety: bool,
}

/// Why a domain was opened. Stored with every follow-up answer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TriggeredRule {
    pub rule_id: String,
    pub domain: Domain,
    pub reason: String,
    pub basis: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlannedQuestion {
    pub question: FollowUpQuestion,
    pub rule_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct FollowUpPlan {
    pub triggered: Vec<TriggeredRule>,
    pub questions: Vec<PlannedQuestion>,
}

const BODY_AREAS: &[(&str, &str)] = &[
    ("head", "Head"),
    ("eyes", "Eyes or vision"),
    ("chest", "Chest"),
    ("abdomen", "Abdomen"),
    ("back", "Back"),
    ("limbs", "Arms or legs"),
    ("skin", "Skin"),
    ("other", "Other or general"),
];

pub const FOLLOW_UPS: &[FollowUpQuestion] = &[
    FollowUpQuestion {
        id: "fu.sleep_hours",
        domain: Domain::Sleep,
        text: "About how many hours did you sleep in your last sleep period?",
        kind: QuestionKind::Number { unit: "hours", min: 0.0, max: 24.0 },
        is_safety: false,
    },
    FollowUpQuestion {
        id: "fu.sleep_interrupted",
        domain: Domain::Sleep,
        text: "Was your sleep interrupted (noise, light, schedule, discomfort)?",
        kind: QuestionKind::YesNo,
        is_safety: false,
    },
    FollowUpQuestion {
        id: "fu.alertness",
        domain: Domain::Sleep,
        text: "How alert do you feel right now?",
        kind: QuestionKind::Scale { low_anchor: "Very sleepy", high_anchor: "Fully alert" },
        is_safety: false,
    },
    FollowUpQuestion {
        id: "fu.red_flags",
        domain: Domain::Body,
        text: "Right now, do you have chest pain, fainting or near-fainting, sudden loss of vision, a sudden severe headache, or difficulty breathing?",
        kind: QuestionKind::YesNo,
        is_safety: true,
    },
    FollowUpQuestion {
        id: "fu.symptom_area",
        domain: Domain::Body,
        text: "Where is the main problem?",
        kind: QuestionKind::Choice { options: BODY_AREAS },
        is_safety: false,
    },
    FollowUpQuestion {
        id: "fu.fall_or_impact",
        domain: Domain::Body,
        text: "Since your last check, have you had a fall, a bump or an impact (for example during exercise or work)?",
        kind: QuestionKind::YesNo,
        is_safety: false,
    },
    FollowUpQuestion {
        id: "fu.vision_change",
        domain: Domain::Body,
        text: "Has your vision changed since your last check (blurred, distorted, harder to read)?",
        kind: QuestionKind::YesNo,
        is_safety: false,
    },
    FollowUpQuestion {
        id: "fu.stress",
        domain: Domain::Mood,
        text: "How stressed do you feel today?",
        kind: QuestionKind::Scale { low_anchor: "Extremely stressed", high_anchor: "Not stressed" },
        is_safety: false,
    },
    FollowUpQuestion {
        id: "fu.concentration",
        domain: Domain::Mood,
        text: "How well can you concentrate on your tasks today?",
        kind: QuestionKind::Scale { low_anchor: "Cannot concentrate", high_anchor: "Fully focused" },
        is_safety: false,
    },
    FollowUpQuestion {
        id: "fu.contact_request",
        domain: Domain::Mood,
        text: "Would you like the medical team to contact you?",
        kind: QuestionKind::YesNo,
        is_safety: false,
    },
];

pub fn find_follow_up(id: &str) -> Option<&'static FollowUpQuestion> {
    FOLLOW_UPS.iter().find(|q| q.id == id)
}

/// Average of the most recent `BASELINE_WINDOW` scores, or `None` when there
/// are fewer than `BASELINE_MIN_HISTORY`. `history` is ordered newest first.
pub fn personal_average(history: &[DailyAnswers], domain: Domain) -> Option<f64> {
    if history.len() < BASELINE_MIN_HISTORY {
        return None;
    }
    let recent = &history[..history.len().min(BASELINE_WINDOW)];
    let sum: u32 = recent.iter().map(|a| u32::from(domain.score(a))).sum();
    Some(f64::from(sum) / recent.len() as f64)
}

fn rule_for(domain: Domain, today: &DailyAnswers, history: &[DailyAnswers]) -> Option<TriggeredRule> {
    let score = domain.score(today);
    if score <= LOW_SCORE_THRESHOLD {
        return Some(TriggeredRule {
            rule_id: format!("low.{}", domain.key()),
            domain,
            reason: format!("{}: today's answer is {score}, at or below {LOW_SCORE_THRESHOLD}.", domain.label()),
            basis: RULE_BASIS,
        });
    }
    let average = personal_average(history, domain)?;
    if average - f64::from(score) >= DROP_POINTS {
        return Some(TriggeredRule {
            rule_id: format!("drop.{}", domain.key()),
            domain,
            reason: format!(
                "{}: today's answer is {score}, {:.1} points below your recent average of {average:.1}.",
                domain.label(),
                average - f64::from(score)
            ),
            basis: RULE_BASIS,
        });
    }
    None
}

/// Chooses follow-up questions for today's answers. `history` = previous
/// check-ins of the same person, newest first (today excluded).
pub fn plan_follow_ups(today: &DailyAnswers, history: &[DailyAnswers]) -> FollowUpPlan {
    let mut plan = FollowUpPlan::default();
    for domain in [Domain::Sleep, Domain::Body, Domain::Mood] {
        let Some(rule) = rule_for(domain, today, history) else { continue };
        plan.questions.extend(
            FOLLOW_UPS
                .iter()
                .filter(|q| q.domain == domain)
                .map(|q| PlannedQuestion { question: q.clone(), rule_id: rule.rule_id.clone() }),
        );
        plan.triggered.push(rule);
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answers(rested: u8, physical: u8, mood_energy: u8) -> DailyAnswers {
        DailyAnswers { rested, physical, mood_energy }
    }

    fn ids(plan: &FollowUpPlan) -> Vec<&str> {
        plan.questions.iter().map(|p| p.question.id).collect()
    }

    #[test]
    fn good_answers_without_history_ask_nothing_extra() {
        let plan = plan_follow_ups(&answers(8, 9, 7), &[]);
        assert!(plan.questions.is_empty());
        assert!(plan.triggered.is_empty());
    }

    #[test]
    fn low_sleep_score_opens_only_sleep_questions() {
        let plan = plan_follow_ups(&answers(LOW_SCORE_THRESHOLD, 9, 8), &[]);
        assert_eq!(plan.triggered.len(), 1);
        assert_eq!(plan.triggered[0].rule_id, "low.rested");
        assert!(plan.questions.iter().all(|p| p.question.domain == Domain::Sleep));
    }

    #[test]
    fn score_just_above_threshold_does_not_trigger() {
        let plan = plan_follow_ups(&answers(LOW_SCORE_THRESHOLD + 1, 9, 8), &[]);
        assert!(plan.questions.is_empty());
    }

    #[test]
    fn body_follow_ups_always_include_the_safety_question() {
        let plan = plan_follow_ups(&answers(8, 2, 8), &[]);
        assert!(ids(&plan).contains(&"fu.red_flags"));
        assert!(plan.questions.iter().any(|p| p.question.is_safety));
    }

    #[test]
    fn drop_from_personal_average_triggers_even_when_score_is_not_low() {
        let history = vec![answers(9, 9, 9); BASELINE_MIN_HISTORY];
        let plan = plan_follow_ups(&answers(9, 9, 5), &history);
        assert_eq!(plan.triggered.len(), 1);
        assert_eq!(plan.triggered[0].rule_id, "drop.mood_energy");
    }

    #[test]
    fn drop_rule_needs_enough_history() {
        let history = vec![answers(9, 9, 9); BASELINE_MIN_HISTORY - 1];
        let plan = plan_follow_ups(&answers(9, 9, 5), &history);
        assert!(plan.questions.is_empty());
    }

    #[test]
    fn personal_average_uses_only_the_recent_window() {
        let mut history = vec![answers(4, 4, 4); BASELINE_WINDOW];
        history.extend(vec![answers(10, 10, 10); 20]); // older, outside the window
        assert_eq!(personal_average(&history, Domain::Sleep), Some(4.0));
    }

    #[test]
    fn every_rule_is_labelled_as_a_prototype_assumption() {
        let plan = plan_follow_ups(&answers(1, 1, 1), &[]);
        assert_eq!(plan.triggered.len(), 3);
        assert!(plan.triggered.iter().all(|r| r.basis == RULE_BASIS));
    }

    #[test]
    fn follow_up_ids_are_unique() {
        let mut seen = std::collections::HashSet::new();
        assert!(FOLLOW_UPS.iter().all(|q| seen.insert(q.id)));
    }
}
