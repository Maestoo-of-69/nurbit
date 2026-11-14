//! Risk-module evaluation (Module D) and cross-risk patterns (Module E).
//!
//! One shared output shape for all five modules. Each module looks only at the
//! data the beta really has, says what is missing, and labels every finding
//! with its basis: a source id from docs/REFERENCES.md or "prototype_assumption".
//! Monitoring is not diagnosis: the strongest level is "specialist review".

use nurbit_shared::records::{AnswerValue, CheckInRecord, Measurement, TimestampMs};
use serde::Serialize;

use crate::baseline::{BaselineStatus, Direction, IndicatorTrend, assess_trend};
use crate::checkin::{DAILY_MOOD_ID, DAILY_PHYSICAL_ID, DAILY_RESTED_ID};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;
/// Self-reported flags are considered for this many days (prototype assumption).
pub const FLAG_WINDOW_DAYS: i64 = 7;
/// NASA-STD-3001 V1 4030 career effective dose limit (source RAD-STD).
pub const CAREER_DOSE_LIMIT_MSV: f64 = 600.0;
/// Minimum paired check-ins before a correlation is shown (prototype assumption).
pub const MIN_PAIRS: usize = 10;
const PROTOTYPE: &str = "prototype_assumption";

pub const EXERCISE_MINUTES: &str = "exercise.resistive_minutes";
pub const DOSE_DAILY: &str = "radiation.effective_dose_msv";
pub const DOSE_PRIOR: &str = "radiation.prior_career_dose_msv";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    NoData,
    NothingToReport,
    Attention,
    SpecialistReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    Observation,
    TemporaryDeviation,
    PersistentChange,
    SelfReportedFlag,
    LimitComparison,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Finding {
    pub kind: FindingKind,
    pub level: Level,
    pub message: String,
    pub basis: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DataGap {
    pub indicator: &'static str,
    pub note: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModuleAssessment {
    pub module_id: &'static str,
    pub level: Level,
    pub findings: Vec<Finding>,
    pub gaps: Vec<DataGap>,
    pub uses_simulated_data: bool,
}

/// Everything the evaluation may look at for one person.
pub struct PersonData<'a> {
    /// Newest first.
    pub checkins: &'a [CheckInRecord],
    /// Any order.
    pub measurements: &'a [Measurement],
    pub now: TimestampMs,
}

impl PersonData<'_> {
    fn daily(&self, question_id: &str) -> Vec<f64> {
        self.checkins
            .iter()
            .filter_map(|c| c.answers.iter().find(|a| a.question_id == question_id))
            .filter_map(|a| match a.value {
                AnswerValue::Scale(v) => Some(f64::from(v)),
                _ => None,
            })
            .collect()
    }

    fn recent_yes(&self, question_id: &str) -> usize {
        let since = self.now - FLAG_WINDOW_DAYS * DAY_MS;
        self.checkins
            .iter()
            .filter(|c| c.completed_at >= since)
            .flat_map(|c| &c.answers)
            .filter(|a| a.question_id == question_id && a.value == AnswerValue::YesNo(true))
            .count()
    }

    fn recent_numbers(&self, question_id: &str) -> Vec<f64> {
        let since = self.now - FLAG_WINDOW_DAYS * DAY_MS;
        self.checkins
            .iter()
            .filter(|c| c.completed_at >= since)
            .flat_map(|c| &c.answers)
            .filter(|a| a.question_id == question_id)
            .filter_map(|a| match a.value {
                AnswerValue::Number(n) => Some(n),
                _ => None,
            })
            .collect()
    }

    fn measurements_of<'b>(&'b self, indicator: &'b str) -> impl Iterator<Item = &'b Measurement> + 'b {
        self.measurements.iter().filter(move |m| m.indicator == indicator)
    }
}

fn trend_finding(name: &str, trend: &IndicatorTrend) -> Option<Finding> {
    let word = |d: Direction| if d == Direction::Below { "below" } else { "above" };
    match trend.status {
        BaselineStatus::InsufficientData | BaselineStatus::WithinBaseline => None,
        BaselineStatus::TemporaryDeviation { direction } => Some(Finding {
            kind: FindingKind::TemporaryDeviation,
            level: Level::Attention,
            message: format!("{name}: latest value is {} your usual range.", word(direction)),
            basis: PROTOTYPE,
        }),
        BaselineStatus::PersistentChange { direction, count } => Some(Finding {
            kind: FindingKind::PersistentChange,
            level: Level::SpecialistReview,
            message: format!(
                "{name}: {count} values in a row {} your usual range. A specialist may review whether your baseline should change; it is not changed automatically.",
                word(direction)
            ),
            basis: PROTOTYPE,
        }),
    }
}

fn flag(count: usize, level: Level, message: &str) -> Option<Finding> {
    (count > 0).then(|| Finding {
        kind: FindingKind::SelfReportedFlag,
        level,
        message: format!("{message} ({count}× in the last {FLAG_WINDOW_DAYS} days)."),
        basis: PROTOTYPE,
    })
}

fn finish(module_id: &'static str, has_data: bool, findings: Vec<Finding>, gaps: Vec<DataGap>, simulated: bool) -> ModuleAssessment {
    let level = if !has_data {
        Level::NoData
    } else {
        findings.iter().map(|f| f.level).max().unwrap_or(Level::NothingToReport).max(Level::NothingToReport)
    };
    ModuleAssessment { module_id, level, findings, gaps, uses_simulated_data: simulated }
}

fn sleep(d: &PersonData) -> ModuleAssessment {
    let rested = d.daily(DAILY_RESTED_ID);
    let mut findings: Vec<Finding> = trend_finding("Rested rating", &assess_trend(&rested)).into_iter().collect();
    let hours = d.recent_numbers("fu.sleep_hours");
    if !hours.is_empty() {
        let avg = hours.iter().sum::<f64>() / hours.len() as f64;
        findings.push(Finding {
            kind: FindingKind::Observation,
            level: Level::NothingToReport,
            message: format!("Reported sleep: {avg:.1} h on average ({} reports this week).", hours.len()),
            basis: PROTOTYPE,
        });
    }
    findings.extend(flag(d.recent_yes("fu.sleep_interrupted"), Level::NothingToReport, "Interrupted sleep reported"));
    let gaps = vec![
        DataGap { indicator: "Actigraphy", note: "No actigraph connected in this beta." },
        DataGap { indicator: "Reaction-time self-test", note: "Not implemented in this beta." },
    ];
    finish("sleep-schedules", !rested.is_empty(), findings, gaps, false)
}

fn behavioral(d: &PersonData) -> ModuleAssessment {
    let mood = d.daily(DAILY_MOOD_ID);
    let mut findings: Vec<Finding> = trend_finding("Mood and energy", &assess_trend(&mood)).into_iter().collect();
    findings.extend(flag(d.recent_yes("fu.contact_request"), Level::SpecialistReview, "Asked to be contacted by the medical team"));
    let gaps = vec![DataGap { indicator: "Cognitive test battery", note: "No results entered." }];
    finish("behavioral-health", !mood.is_empty(), findings, gaps, false)
}

fn sans(d: &PersonData) -> ModuleAssessment {
    let mut findings = Vec::new();
    findings.extend(flag(
        d.recent_yes("fu.vision_change"),
        Level::SpecialistReview,
        "Vision change reported; only an eye examination can assess it",
    ));
    let gaps = vec![
        DataGap { indicator: "OCT / fundoscopy / ultrasound", note: "No exam results entered." },
        DataGap { indicator: "Visual acuity and Amsler grid", note: "No results entered." },
    ];
    finish("sans", !d.checkins.is_empty(), findings, gaps, false)
}

fn bone(d: &PersonData) -> ModuleAssessment {
    let mut findings = Vec::new();
    findings.extend(flag(d.recent_yes("fu.fall_or_impact"), Level::Attention, "Fall or impact reported"));

    let since = d.now - FLAG_WINDOW_DAYS * DAY_MS;
    let before = since - FLAG_WINDOW_DAYS * DAY_MS;
    let exercise: Vec<&Measurement> = d.measurements_of(EXERCISE_MINUTES).collect();
    let minutes = |from: i64, to: i64| -> f64 {
        exercise.iter().filter(|m| m.measured_at >= from && m.measured_at < to).map(|m| m.value).sum()
    };
    if !exercise.is_empty() {
        findings.push(Finding {
            kind: FindingKind::Observation,
            level: Level::NothingToReport,
            message: format!(
                "Resistive exercise: {:.0} min this week, {:.0} min the week before. This shows adherence, not bone health.",
                minutes(since, d.now + 1),
                minutes(before, since)
            ),
            basis: PROTOTYPE,
        });
    }
    let gaps = vec![
        DataGap { indicator: "Bone mineral density", note: "Cannot be measured in flight; scans happen before and after flight." },
        DataGap { indicator: "Bone markers", note: "Laboratory data not available in this beta." },
    ];
    finish("bone", !exercise.is_empty() || !d.checkins.is_empty(), findings, gaps, !exercise.is_empty())
}

fn radiation(d: &PersonData) -> ModuleAssessment {
    let prior: f64 = d.measurements_of(DOSE_PRIOR).map(|m| m.value).sum();
    let mission: f64 = d.measurements_of(DOSE_DAILY).map(|m| m.value).sum();
    let has_data = d.measurements_of(DOSE_DAILY).next().is_some();
    let mut findings = Vec::new();
    if has_data {
        let total = prior + mission;
        let over = total >= CAREER_DOSE_LIMIT_MSV;
        findings.push(Finding {
            kind: FindingKind::LimitComparison,
            level: if over { Level::SpecialistReview } else { Level::NothingToReport },
            message: format!(
                "Career effective dose {total:.1} mSv ({:.1}% of the 600 mSv NASA limit; this mission {mission:.1} mSv). Dose estimates risk; it cannot detect cancer.",
                total / CAREER_DOSE_LIMIT_MSV * 100.0
            ),
            basis: "RAD-STD",
        });
    }
    let gaps = vec![DataGap { indicator: "REID risk estimate", note: "Calculated by radiation specialists, not by this app." }];
    finish("radiation", has_data, findings, gaps, has_data)
}

/// Safety answers that belong to no single risk module (red-flag symptoms)
/// but must reach the medical team. Kept outside the five modules on purpose.
pub fn safety_findings(d: &PersonData) -> Vec<Finding> {
    flag(
        d.recent_yes("fu.red_flags"),
        Level::SpecialistReview,
        "Red-flag symptom reported in a daily check (chest pain, fainting, sudden vision loss, sudden severe headache or difficulty breathing)",
    )
    .into_iter()
    .collect()
}

/// Evaluates all five modules for one person, in catalogue order.
pub fn evaluate_all(d: &PersonData) -> Vec<ModuleAssessment> {
    vec![bone(d), sans(d), behavioral(d), sleep(d), radiation(d)]
}

// ---------------- cross-risk (Module E) ----------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLevel {
    ObservedCorrelation,
    PossibleAssociation,
    EstablishedRelationship,
    UnverifiedHypothesis,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CrossRiskPattern {
    pub first: &'static str,
    pub second: &'static str,
    /// Pearson correlation in this person's data; None if too few pairs or no variation.
    pub r: Option<f64>,
    pub pairs: usize,
    pub in_your_data: EvidenceLevel,
    pub literature: EvidenceLevel,
    pub literature_note: &'static str,
    pub literature_source: Option<&'static str>,
}

/// Pearson correlation; `None` when fewer than `MIN_PAIRS` or a series is constant.
pub fn pearson(xs: &[f64], ys: &[f64]) -> Option<f64> {
    let n = xs.len().min(ys.len());
    if n < MIN_PAIRS {
        return None;
    }
    let (xs, ys) = (&xs[..n], &ys[..n]);
    let mx = xs.iter().sum::<f64>() / n as f64;
    let my = ys.iter().sum::<f64>() / n as f64;
    let cov: f64 = xs.iter().zip(ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let vx: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    let vy: f64 = ys.iter().map(|y| (y - my).powi(2)).sum();
    (vx > 0.0 && vy > 0.0).then(|| cov / (vx.sqrt() * vy.sqrt()))
}

/// Pairs of daily ratings from the same check-ins.
fn paired(d: &PersonData, a: &str, b: &str) -> (Vec<f64>, Vec<f64>) {
    let score = |c: &CheckInRecord, id: &str| {
        c.answers.iter().find(|x| x.question_id == id).and_then(|x| match x.value {
            AnswerValue::Scale(v) => Some(f64::from(v)),
            _ => None,
        })
    };
    d.checkins.iter().filter_map(|c| Some((score(c, a)?, score(c, b)?))).unzip()
}

pub fn cross_risk_patterns(d: &PersonData) -> Vec<CrossRiskPattern> {
    let (rested, mood) = paired(d, DAILY_RESTED_ID, DAILY_MOOD_ID);
    let (rested2, body) = paired(d, DAILY_RESTED_ID, DAILY_PHYSICAL_ID);
    vec![
        CrossRiskPattern {
            first: "Rested rating",
            second: "Mood and energy",
            r: pearson(&rested, &mood),
            pairs: rested.len(),
            in_your_data: EvidenceLevel::ObservedCorrelation,
            literature: EvidenceLevel::EstablishedRelationship,
            literature_note: "Sleep loss affects mood and psychosocial functioning in ground studies; the in-flight association is not verified.",
            literature_source: Some("HRR-SLEEP"),
        },
        CrossRiskPattern {
            first: "Rested rating",
            second: "Body symptoms",
            r: pearson(&rested2, &body),
            pairs: rested2.len(),
            in_your_data: EvidenceLevel::ObservedCorrelation,
            literature: EvidenceLevel::UnverifiedHypothesis,
            literature_note: "No source reviewed for this beta links these two ratings.",
            literature_source: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use nurbit_shared::records::AnswerRecord;
    use nurbit_shared::{DataOrigin, RecordId};

    const NOW: i64 = 1_800_000_000_000;

    fn checkin(days_ago: i64, scores: [u8; 3], extra: &[(&str, AnswerValue)]) -> CheckInRecord {
        let ids = [DAILY_RESTED_ID, DAILY_PHYSICAL_ID, DAILY_MOOD_ID];
        let mut answers: Vec<AnswerRecord> = ids
            .iter()
            .zip(scores)
            .map(|(id, v)| AnswerRecord { question_id: (*id).into(), value: AnswerValue::Scale(v), triggered_by_rule: None })
            .collect();
        answers.extend(extra.iter().map(|(id, v)| AnswerRecord {
            question_id: (*id).into(),
            value: v.clone(),
            triggered_by_rule: Some("test".into()),
        }));
        CheckInRecord {
            id: RecordId::new(),
            crew_member_id: RecordId::new(),
            started_at: NOW - days_ago * DAY_MS - 1000,
            completed_at: NOW - days_ago * DAY_MS,
            device_id: "t".into(),
            origin: DataOrigin::SelfReport,
            answers,
        }
    }

    fn history(recent: &[[u8; 3]], usual: [u8; 3]) -> Vec<CheckInRecord> {
        let mut v: Vec<CheckInRecord> = recent.iter().enumerate().map(|(i, s)| checkin(i as i64, *s, &[])).collect();
        v.extend((recent.len()..recent.len() + 12).map(|i| checkin(i as i64, usual, &[])));
        v
    }

    fn dose(indicator: &str, value: f64) -> Measurement {
        Measurement {
            id: RecordId::new(),
            crew_member_id: RecordId::new(),
            indicator: indicator.into(),
            value,
            unit: "mSv".into(),
            measured_at: NOW,
            origin: DataOrigin::Simulated,
        }
    }

    fn module<'a>(all: &'a [ModuleAssessment], id: &str) -> &'a ModuleAssessment {
        all.iter().find(|m| m.module_id == id).unwrap()
    }

    #[test]
    fn no_data_at_all_gives_no_data_everywhere() {
        let d = PersonData { checkins: &[], measurements: &[], now: NOW };
        assert!(evaluate_all(&d).iter().all(|m| m.level == Level::NoData));
    }

    #[test]
    fn steady_history_has_nothing_to_report() {
        let c = history(&[], [8, 8, 8]);
        let d = PersonData { checkins: &c, measurements: &[], now: NOW };
        assert_eq!(module(&evaluate_all(&d), "sleep-schedules").level, Level::NothingToReport);
    }

    #[test]
    fn persistent_sleep_drop_asks_for_specialist_review() {
        let c = history(&[[3, 8, 8], [4, 8, 8], [3, 8, 8]], [8, 8, 8]);
        let d = PersonData { checkins: &c, measurements: &[], now: NOW };
        let sleep = module(&evaluate_all(&d), "sleep-schedules").clone();
        assert_eq!(sleep.level, Level::SpecialistReview);
        assert_eq!(sleep.findings[0].kind, FindingKind::PersistentChange);
    }

    #[test]
    fn reported_vision_change_flags_sans_for_review() {
        let mut c = history(&[], [8, 8, 8]);
        c.insert(0, checkin(0, [8, 2, 8], &[("fu.vision_change", AnswerValue::YesNo(true))]));
        let d = PersonData { checkins: &c, measurements: &[], now: NOW };
        assert_eq!(module(&evaluate_all(&d), "sans").level, Level::SpecialistReview);
    }

    #[test]
    fn old_flags_outside_the_window_are_ignored() {
        let c = vec![checkin(FLAG_WINDOW_DAYS + 1, [8, 2, 8], &[("fu.vision_change", AnswerValue::YesNo(true))])];
        let d = PersonData { checkins: &c, measurements: &[], now: NOW };
        assert_eq!(module(&evaluate_all(&d), "sans").level, Level::NothingToReport);
    }

    #[test]
    fn radiation_compares_with_the_sourced_600_msv_limit() {
        let m = vec![dose(DOSE_PRIOR, 100.0), dose(DOSE_DAILY, 20.0)];
        let d = PersonData { checkins: &[], measurements: &m, now: NOW };
        let rad = module(&evaluate_all(&d), "radiation").clone();
        assert_eq!(rad.level, Level::NothingToReport);
        assert_eq!(rad.findings[0].basis, "RAD-STD");
        assert!(rad.findings[0].message.contains("20.0% of the 600 mSv"));
    }

    #[test]
    fn exceeding_the_dose_limit_asks_for_specialist_review() {
        let m = vec![dose(DOSE_PRIOR, 590.0), dose(DOSE_DAILY, 15.0)];
        let d = PersonData { checkins: &[], measurements: &m, now: NOW };
        assert_eq!(module(&evaluate_all(&d), "radiation").level, Level::SpecialistReview);
    }

    #[test]
    fn bone_module_never_claims_bone_density() {
        let c = history(&[], [8, 8, 8]);
        let d = PersonData { checkins: &c, measurements: &[], now: NOW };
        let bone = module(&evaluate_all(&d), "bone").clone();
        assert!(bone.gaps.iter().any(|g| g.note.contains("Cannot be measured in flight")));
    }

    #[test]
    fn pearson_needs_enough_pairs_and_variation() {
        assert_eq!(pearson(&[1.0; 5], &[1.0; 5]), None);
        assert_eq!(pearson(&[5.0; 12], &(0..12).map(f64::from).collect::<Vec<_>>()), None);
        let x: Vec<f64> = (0..12).map(f64::from).collect();
        let r = pearson(&x, &x).unwrap();
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cross_risk_patterns_are_never_labelled_as_causes() {
        let c = history(&[], [8, 8, 8]);
        let d = PersonData { checkins: &c, measurements: &[], now: NOW };
        for p in cross_risk_patterns(&d) {
            assert_eq!(p.in_your_data, EvidenceLevel::ObservedCorrelation);
        }
    }
}
