//! Rules for the specialist workflow (Module H). Pure functions.
//!
//! * Specialists outrank automated assessments: rule output never changes a
//!   decision, and decisions are stored separately from machine output.
//! * A duty-restriction recommendation needs confirmation by a SECOND
//!   specialist; the author can never confirm their own recommendation.
//! * The software itself never declares anyone unfit for duty.

use nurbit_shared::RecordId;
use nurbit_shared::records::{BaselineDecision, BaselineReview, Decision, DecisionKind, DecisionReview, TimestampMs, Verdict};
use serde::Serialize;
use thiserror::Error;

pub const MAX_TEXT_CHARS: usize = 4000;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReviewError {
    #[error("text must not be empty")]
    EmptyText,
    #[error("text is longer than {MAX_TEXT_CHARS} characters")]
    TextTooLong,
    #[error("a decision cannot be reviewed by its own author")]
    OwnDecision,
    #[error("this decision does not need a second review")]
    NoReviewNeeded,
    #[error("this decision has already been reviewed")]
    AlreadyReviewed,
}

/// Trims and checks free text written by a specialist.
pub fn validate_text(text: &str) -> Result<String, ReviewError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(ReviewError::EmptyText);
    }
    if trimmed.chars().count() > MAX_TEXT_CHARS {
        return Err(ReviewError::TextTooLong);
    }
    Ok(trimmed.to_string())
}

pub fn requires_second_review(kind: DecisionKind) -> bool {
    kind == DecisionKind::DutyRestriction
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    /// Recorded by one specialist; no second review needed.
    Recorded,
    PendingSecondReview,
    Confirmed,
    Rejected,
}

/// Status follows from the stored reviews; decisions themselves are never edited.
pub fn decision_status(decision: &Decision, reviews: &[DecisionReview]) -> DecisionStatus {
    if !requires_second_review(decision.kind) {
        return DecisionStatus::Recorded;
    }
    let valid = reviews
        .iter()
        .filter(|r| r.decision_id == decision.id && r.reviewer_id != decision.author_id)
        .min_by_key(|r| r.created_at);
    match valid.map(|r| r.verdict) {
        None => DecisionStatus::PendingSecondReview,
        Some(Verdict::Confirm) => DecisionStatus::Confirmed,
        Some(Verdict::Reject) => DecisionStatus::Rejected,
    }
}

/// May `reviewer` give the second review of `decision` now?
pub fn check_reviewer(decision: &Decision, reviews: &[DecisionReview], reviewer: RecordId) -> Result<(), ReviewError> {
    if !requires_second_review(decision.kind) {
        return Err(ReviewError::NoReviewNeeded);
    }
    if decision.author_id == reviewer {
        return Err(ReviewError::OwnDecision);
    }
    if decision_status(decision, reviews) != DecisionStatus::PendingSecondReview {
        return Err(ReviewError::AlreadyReviewed);
    }
    Ok(())
}

/// Start time of the baseline currently in force for one indicator:
/// the latest approved review, or `None` (use the whole history).
pub fn baseline_effective_from(reviews: &[BaselineReview], indicator: &str) -> Option<TimestampMs> {
    reviews
        .iter()
        .filter(|r| r.indicator == indicator && r.decision == BaselineDecision::ApproveNewBaseline)
        .map(|r| r.decided_at)
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decision(kind: DecisionKind, author: RecordId) -> Decision {
        Decision { id: RecordId::new(), crew_member_id: RecordId::new(), author_id: author, created_at: 1, kind, text: "x".into() }
    }

    fn review(d: &Decision, reviewer: RecordId, verdict: Verdict) -> DecisionReview {
        DecisionReview { id: RecordId::new(), decision_id: d.id, reviewer_id: reviewer, created_at: 2, verdict, comment: String::new() }
    }

    #[test]
    fn empty_or_whitespace_text_is_rejected() {
        assert_eq!(validate_text("   "), Err(ReviewError::EmptyText));
        assert_eq!(validate_text("  ok "), Ok("ok".to_string()));
    }

    #[test]
    fn overly_long_text_is_rejected() {
        assert_eq!(validate_text(&"a".repeat(MAX_TEXT_CHARS + 1)), Err(ReviewError::TextTooLong));
    }

    #[test]
    fn ordinary_decisions_are_recorded_without_second_review() {
        let d = decision(DecisionKind::FollowUpExam, RecordId::new());
        assert_eq!(decision_status(&d, &[]), DecisionStatus::Recorded);
    }

    #[test]
    fn duty_restriction_waits_for_a_second_specialist() {
        let d = decision(DecisionKind::DutyRestriction, RecordId::new());
        assert_eq!(decision_status(&d, &[]), DecisionStatus::PendingSecondReview);
    }

    #[test]
    fn author_cannot_confirm_own_duty_restriction() {
        let author = RecordId::new();
        let d = decision(DecisionKind::DutyRestriction, author);
        assert_eq!(check_reviewer(&d, &[], author), Err(ReviewError::OwnDecision));
        // even if such a review were stored, it would not count
        let self_review = review(&d, author, Verdict::Confirm);
        assert_eq!(decision_status(&d, &[self_review]), DecisionStatus::PendingSecondReview);
    }

    #[test]
    fn second_specialist_confirms_or_rejects() {
        let d = decision(DecisionKind::DutyRestriction, RecordId::new());
        let other = RecordId::new();
        assert_eq!(check_reviewer(&d, &[], other), Ok(()));
        assert_eq!(decision_status(&d, &[review(&d, other, Verdict::Confirm)]), DecisionStatus::Confirmed);
        assert_eq!(decision_status(&d, &[review(&d, other, Verdict::Reject)]), DecisionStatus::Rejected);
    }

    #[test]
    fn a_reviewed_decision_cannot_be_reviewed_again() {
        let d = decision(DecisionKind::DutyRestriction, RecordId::new());
        let first = review(&d, RecordId::new(), Verdict::Confirm);
        assert_eq!(check_reviewer(&d, &[first], RecordId::new()), Err(ReviewError::AlreadyReviewed));
    }

    #[test]
    fn latest_approved_review_sets_the_baseline_start() {
        let mk = |at, decision| BaselineReview {
            id: RecordId::new(),
            crew_member_id: RecordId::new(),
            indicator: "daily.rested".into(),
            decision,
            decided_by: RecordId::new(),
            decided_at: at,
            note: String::new(),
        };
        let reviews = vec![
            mk(10, BaselineDecision::ApproveNewBaseline),
            mk(30, BaselineDecision::KeepCurrentBaseline),
            mk(20, BaselineDecision::ApproveNewBaseline),
        ];
        assert_eq!(baseline_effective_from(&reviews, "daily.rested"), Some(20));
        assert_eq!(baseline_effective_from(&reviews, "daily.physical"), None);
    }
}
