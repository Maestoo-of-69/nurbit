//! Converts answer values to and from their database columns
//! (`value_kind`, `value_num`, `value_text`).

use nurbit_shared::records::{AnswerRecord, AnswerValue};
use sqlx::Row;

use crate::{StoreError, StoreResult};

pub(crate) struct StoredValue {
    pub kind: &'static str,
    pub num: Option<f64>,
    pub text: Option<String>,
}

impl From<&AnswerValue> for StoredValue {
    fn from(value: &AnswerValue) -> Self {
        match value {
            AnswerValue::Scale(v) => Self { kind: "scale", num: Some(f64::from(*v)), text: None },
            AnswerValue::YesNo(b) => Self { kind: "yes_no", num: Some(if *b { 1.0 } else { 0.0 }), text: None },
            AnswerValue::Number(n) => Self { kind: "number", num: Some(*n), text: None },
            AnswerValue::Choice(c) => Self { kind: "choice", num: None, text: Some(c.clone()) },
        }
    }
}

/// A stored 0-10 scale value back as u8; `None` if missing or out of range.
pub(crate) fn scale_from_num(num: Option<f64>) -> Option<u8> {
    let v = num?;
    (v.fract() == 0.0 && (0.0..=10.0).contains(&v)).then_some(v as u8)
}

pub(crate) fn answer_from_row(row: &sqlx::sqlite::SqliteRow) -> StoreResult<AnswerRecord> {
    let kind: String = row.try_get("value_kind")?;
    let num: Option<f64> = row.try_get("value_num")?;
    let text: Option<String> = row.try_get("value_text")?;
    let corrupt = || StoreError::Corrupt(format!("bad {kind} answer"));
    let value = match kind.as_str() {
        "scale" => AnswerValue::Scale(scale_from_num(num).ok_or_else(corrupt)?),
        "yes_no" => AnswerValue::YesNo(num.ok_or_else(corrupt)? != 0.0),
        "number" => AnswerValue::Number(num.ok_or_else(corrupt)?),
        "choice" => AnswerValue::Choice(text.ok_or_else(corrupt)?),
        _ => return Err(corrupt()),
    };
    Ok(AnswerRecord {
        question_id: row.try_get("question_id")?,
        value,
        triggered_by_rule: row.try_get("triggered_by_rule")?,
    })
}
