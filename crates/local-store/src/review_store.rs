//! Storage for specialist records: notes, annotations, baseline reviews,
//! decisions and second reviews. Every insert also writes an audit entry in
//! the same transaction. Nothing here is ever updated or deleted.

use nurbit_shared::RecordId;
use nurbit_shared::records::{
    Annotation, BaselineDecision, BaselineReview, Decision, DecisionKind, DecisionReview, SpecialistNote,
    TimestampMs, Verdict,
};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, Sqlite, Transaction};

use crate::{LocalStore, StoreError, StoreResult, parse_id};

fn kind_str(k: DecisionKind) -> &'static str {
    match k {
        DecisionKind::FollowUpExam => "follow_up_exam",
        DecisionKind::ClearForDuty => "clear_for_duty",
        DecisionKind::DutyRestriction => "duty_restriction",
        DecisionKind::Other => "other",
    }
}

fn parse_kind(s: &str) -> StoreResult<DecisionKind> {
    [DecisionKind::FollowUpExam, DecisionKind::ClearForDuty, DecisionKind::DutyRestriction, DecisionKind::Other]
        .into_iter()
        .find(|k| kind_str(*k) == s)
        .ok_or_else(|| StoreError::Corrupt(format!("unknown decision kind {s}")))
}

fn verdict_str(v: Verdict) -> &'static str {
    match v {
        Verdict::Confirm => "confirm",
        Verdict::Reject => "reject",
    }
}

fn baseline_str(d: BaselineDecision) -> &'static str {
    match d {
        BaselineDecision::ApproveNewBaseline => "approve_new_baseline",
        BaselineDecision::KeepCurrentBaseline => "keep_current_baseline",
    }
}

fn id_col(row: &SqliteRow, col: &str) -> StoreResult<RecordId> {
    parse_id(&row.try_get::<String, _>(col)?)
}

async fn audit(
    tx: &mut Transaction<'_, Sqlite>,
    at: TimestampMs,
    actor: RecordId,
    record_type: &str,
    record_id: RecordId,
) -> StoreResult<()> {
    sqlx::query("INSERT INTO audit_log (at, actor_id, action, record_type, record_id) VALUES (?, ?, 'create', ?, ?)")
        .bind(at)
        .bind(actor.to_string())
        .bind(record_type)
        .bind(record_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// One assistant question as stored (automated output, not a decision).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AssistantLogEntry {
    pub id: RecordId,
    pub crew_member_id: RecordId,
    pub created_at: TimestampMs,
    pub provider: String,
    pub question: String,
    pub body_area: Option<String>,
    pub severity: Option<i64>,
    pub matched_entries: Vec<String>,
    pub urgent: bool,
}

impl LocalStore {
    pub async fn insert_assistant_log(&self, e: &AssistantLogEntry) -> StoreResult<()> {
        let matched = e.matched_entries.join(",");
        sqlx::query(
            "INSERT INTO assistant_log (id, crew_member_id, created_at, provider, question, body_area, severity, matched_entries, urgent)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(e.id.to_string())
        .bind(e.crew_member_id.to_string())
        .bind(e.created_at)
        .bind(&e.provider)
        .bind(&e.question)
        .bind(&e.body_area)
        .bind(e.severity)
        .bind(matched)
        .bind(e.urgent)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_assistant_log(&self, member: RecordId) -> StoreResult<Vec<AssistantLogEntry>> {
        let rows = sqlx::query("SELECT * FROM assistant_log WHERE crew_member_id = ? ORDER BY created_at DESC LIMIT 50")
            .bind(member.to_string())
            .fetch_all(&self.pool)
            .await?;
        rows.iter()
            .map(|r| {
                let matched: String = r.try_get("matched_entries")?;
                Ok(AssistantLogEntry {
                    id: id_col(r, "id")?,
                    crew_member_id: id_col(r, "crew_member_id")?,
                    created_at: r.try_get("created_at")?,
                    provider: r.try_get("provider")?,
                    question: r.try_get("question")?,
                    body_area: r.try_get("body_area")?,
                    severity: r.try_get("severity")?,
                    matched_entries: matched.split(',').filter(|s| !s.is_empty()).map(str::to_string).collect(),
                    urgent: r.try_get("urgent")?,
                })
            })
            .collect()
    }

    pub async fn insert_note(&self, n: &SpecialistNote) -> StoreResult<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("INSERT INTO specialist_notes (id, crew_member_id, author_id, created_at, text) VALUES (?, ?, ?, ?, ?)")
            .bind(n.id.to_string())
            .bind(n.crew_member_id.to_string())
            .bind(n.author_id.to_string())
            .bind(n.created_at)
            .bind(&n.text)
            .execute(&mut *tx)
            .await?;
        audit(&mut tx, n.created_at, n.author_id, "specialist_note", n.id).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_notes(&self, member: RecordId) -> StoreResult<Vec<SpecialistNote>> {
        let rows = sqlx::query("SELECT * FROM specialist_notes WHERE crew_member_id = ? ORDER BY created_at DESC")
            .bind(member.to_string())
            .fetch_all(&self.pool)
            .await?;
        rows.iter()
            .map(|r| {
                Ok(SpecialistNote {
                    id: id_col(r, "id")?,
                    crew_member_id: id_col(r, "crew_member_id")?,
                    author_id: id_col(r, "author_id")?,
                    created_at: r.try_get("created_at")?,
                    text: r.try_get("text")?,
                })
            })
            .collect()
    }

    /// Owner of a check-in, used to keep annotations attached to the right person.
    pub async fn checkin_owner(&self, checkin: RecordId) -> StoreResult<Option<RecordId>> {
        let owner: Option<String> = sqlx::query_scalar("SELECT crew_member_id FROM checkins WHERE id = ?")
            .bind(checkin.to_string())
            .fetch_optional(&self.pool)
            .await?;
        owner.map(|o| parse_id(&o)).transpose()
    }

    pub async fn insert_annotation(&self, a: &Annotation) -> StoreResult<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO annotations (id, checkin_id, question_id, author_id, created_at, corrected_value, reason)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(a.id.to_string())
        .bind(a.checkin_id.to_string())
        .bind(&a.question_id)
        .bind(a.author_id.to_string())
        .bind(a.created_at)
        .bind(&a.corrected_value)
        .bind(&a.reason)
        .execute(&mut *tx)
        .await?;
        audit(&mut tx, a.created_at, a.author_id, "annotation", a.id).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_annotations(&self, member: RecordId) -> StoreResult<Vec<Annotation>> {
        let rows = sqlx::query(
            "SELECT a.* FROM annotations a JOIN checkins c ON c.id = a.checkin_id
             WHERE c.crew_member_id = ? ORDER BY a.created_at DESC",
        )
        .bind(member.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(|r| {
                Ok(Annotation {
                    id: id_col(r, "id")?,
                    checkin_id: id_col(r, "checkin_id")?,
                    question_id: r.try_get("question_id")?,
                    author_id: id_col(r, "author_id")?,
                    created_at: r.try_get("created_at")?,
                    corrected_value: r.try_get("corrected_value")?,
                    reason: r.try_get("reason")?,
                })
            })
            .collect()
    }

    pub async fn insert_baseline_review(&self, b: &BaselineReview) -> StoreResult<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO baseline_reviews (id, crew_member_id, indicator, decision, decided_by, decided_at, note)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(b.id.to_string())
        .bind(b.crew_member_id.to_string())
        .bind(&b.indicator)
        .bind(baseline_str(b.decision))
        .bind(b.decided_by.to_string())
        .bind(b.decided_at)
        .bind(&b.note)
        .execute(&mut *tx)
        .await?;
        audit(&mut tx, b.decided_at, b.decided_by, "baseline_review", b.id).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_baseline_reviews(&self, member: RecordId) -> StoreResult<Vec<BaselineReview>> {
        let rows = sqlx::query("SELECT * FROM baseline_reviews WHERE crew_member_id = ? ORDER BY decided_at DESC")
            .bind(member.to_string())
            .fetch_all(&self.pool)
            .await?;
        rows.iter()
            .map(|r| {
                let d: String = r.try_get("decision")?;
                Ok(BaselineReview {
                    id: id_col(r, "id")?,
                    crew_member_id: id_col(r, "crew_member_id")?,
                    indicator: r.try_get("indicator")?,
                    decision: if d == "approve_new_baseline" {
                        BaselineDecision::ApproveNewBaseline
                    } else {
                        BaselineDecision::KeepCurrentBaseline
                    },
                    decided_by: id_col(r, "decided_by")?,
                    decided_at: r.try_get("decided_at")?,
                    note: r.try_get("note")?,
                })
            })
            .collect()
    }

    pub async fn insert_decision(&self, d: &Decision) -> StoreResult<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("INSERT INTO decisions (id, crew_member_id, author_id, created_at, kind, text) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(d.id.to_string())
            .bind(d.crew_member_id.to_string())
            .bind(d.author_id.to_string())
            .bind(d.created_at)
            .bind(kind_str(d.kind))
            .bind(&d.text)
            .execute(&mut *tx)
            .await?;
        audit(&mut tx, d.created_at, d.author_id, "decision", d.id).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Decisions of one person, or of everyone when `member` is `None`.
    pub async fn list_decisions(&self, member: Option<RecordId>) -> StoreResult<Vec<Decision>> {
        let rows = sqlx::query(
            "SELECT * FROM decisions WHERE (?1 IS NULL OR crew_member_id = ?1) ORDER BY created_at DESC",
        )
        .bind(member.map(|m| m.to_string()))
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(|r| {
                Ok(Decision {
                    id: id_col(r, "id")?,
                    crew_member_id: id_col(r, "crew_member_id")?,
                    author_id: id_col(r, "author_id")?,
                    created_at: r.try_get("created_at")?,
                    kind: parse_kind(&r.try_get::<String, _>("kind")?)?,
                    text: r.try_get("text")?,
                })
            })
            .collect()
    }

    pub async fn insert_decision_review(&self, r: &DecisionReview) -> StoreResult<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO decision_reviews (id, decision_id, reviewer_id, created_at, verdict, comment) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(r.id.to_string())
        .bind(r.decision_id.to_string())
        .bind(r.reviewer_id.to_string())
        .bind(r.created_at)
        .bind(verdict_str(r.verdict))
        .bind(&r.comment)
        .execute(&mut *tx)
        .await?;
        audit(&mut tx, r.created_at, r.reviewer_id, "decision_review", r.id).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_decision_reviews(&self) -> StoreResult<Vec<DecisionReview>> {
        let rows = sqlx::query("SELECT * FROM decision_reviews ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?;
        rows.iter()
            .map(|r| {
                let v: String = r.try_get("verdict")?;
                Ok(DecisionReview {
                    id: id_col(r, "id")?,
                    decision_id: id_col(r, "decision_id")?,
                    reviewer_id: id_col(r, "reviewer_id")?,
                    created_at: r.try_get("created_at")?,
                    verdict: if v == "confirm" { Verdict::Confirm } else { Verdict::Reject },
                    comment: r.try_get("comment")?,
                })
            })
            .collect()
    }
}
