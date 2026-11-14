//! Demo scenarios from the specification (section 9), run through the real
//! medical-core logic with SYNTHETIC data. They measure questions asked,
//! follow-ups triggered and alerts raised. Passing them shows the prototype
//! behaves as designed; it says nothing about clinical effectiveness.
//!
//! Print the metrics: cargo test -p nurbit-medical-core --test demo_scenarios -- --nocapture

use nurbit_medical_core::assistant::{AssistantQuery, answer};
use nurbit_medical_core::checkin::{CheckInSubmission, Flag, SubmittedAnswer, evaluate_submission};
use nurbit_medical_core::daily_check::DailyAnswers;
use nurbit_medical_core::risk_eval::{Level, PersonData, evaluate_all, safety_findings};
use nurbit_shared::records::{AnswerRecord, AnswerValue, CheckInRecord};
use nurbit_shared::{DataOrigin, RecordId};

const DAY: i64 = 24 * 60 * 60 * 1000;
const NOW: i64 = 1_800_000_000_000;

struct Outcome {
    questions: usize,
    follow_ups: usize,
    flags: Vec<Flag>,
    levels: Vec<(&'static str, Level)>,
    safety: usize,
}

fn record(at: i64, s: [u8; 3], origin: DataOrigin) -> CheckInRecord {
    let answers = ["daily.rested", "daily.physical", "daily.mood_energy"]
        .iter()
        .zip(s)
        .map(|(id, v)| AnswerRecord { question_id: (*id).into(), value: AnswerValue::Scale(v), triggered_by_rule: None })
        .collect();
    CheckInRecord { id: RecordId::new(), crew_member_id: RecordId::new(), started_at: at - 60_000, completed_at: at, device_id: "demo".into(), origin, answers }
}

/// 14 days of synthetic history with the given usual scores, newest first.
fn history(usual: [u8; 3]) -> Vec<CheckInRecord> {
    (1..=14).map(|d| record(NOW - d * DAY, usual, DataOrigin::Simulated)).collect()
}

fn daily(c: &CheckInRecord) -> DailyAnswers {
    let v = |id: &str| match c.answers.iter().find(|a| a.question_id == id).map(|a| &a.value) {
        Some(AnswerValue::Scale(x)) => *x,
        _ => 0,
    };
    DailyAnswers { rested: v("daily.rested"), physical: v("daily.physical"), mood_energy: v("daily.mood_energy") }
}

/// One check-in day: plan + validate the submission, put it on top of the
/// history, then evaluate the five risk modules.
fn run_day(past: &[CheckInRecord], today: [u8; 3], follow_ups: &[(&str, AnswerValue)]) -> Outcome {
    let recent: Vec<DailyAnswers> = past.iter().take(7).map(daily).collect();
    let submission = CheckInSubmission {
        rested: today[0],
        physical: today[1],
        mood_energy: today[2],
        follow_ups: follow_ups.iter().map(|(id, v)| SubmittedAnswer { question_id: (*id).into(), value: v.clone() }).collect(),
    };
    let (answers, outcome) = evaluate_submission(&submission, &recent).expect("demo submission is valid");
    let mut all = vec![CheckInRecord { answers, ..record(NOW, today, DataOrigin::SelfReport) }];
    all.extend(past.iter().cloned());
    let data = PersonData { checkins: &all, measurements: &[], now: NOW };
    let levels = evaluate_all(&data).into_iter().map(|m| (m.module_id, m.level)).collect();
    Outcome {
        safety: safety_findings(&data).len(),
        questions: outcome.questions_asked,
        follow_ups: outcome.follow_ups_asked,
        flags: outcome.flags.iter().map(|f| f.flag).collect(),
        levels,
    }
}

fn level(o: &Outcome, module: &str) -> Level {
    o.levels.iter().find(|(m, _)| *m == module).map(|(_, l)| *l).expect("module exists")
}

fn alerts(o: &Outcome) -> usize {
    o.levels.iter().filter(|(_, l)| *l >= Level::Attention).count()
}

fn report(name: &str, o: &Outcome) {
    println!(
        "{name:<18} questions={:<2} follow-ups={:<2} modules at attention/review={} flags={:?}",
        o.questions,
        o.follow_ups,
        alerts(o),
        o.flags
    );
}

#[test]
fn routine_day_asks_three_questions_and_raises_no_alert() {
    let o = run_day(&history([8, 8, 8]), [8, 8, 7], &[]);
    report("routine day", &o);
    assert_eq!((o.questions, o.follow_ups), (3, 0));
    assert!(o.flags.is_empty());
    assert_eq!(alerts(&o), 0, "no unnecessary alerts on an ordinary day");
}

#[test]
fn sleep_drop_from_own_baseline_triggers_sleep_follow_ups() {
    let fu = [
        ("fu.sleep_hours", AnswerValue::Number(5.0)),
        ("fu.sleep_interrupted", AnswerValue::YesNo(true)),
        ("fu.alertness", AnswerValue::Scale(4)),
    ];
    let o = run_day(&history([8, 8, 8]), [4, 8, 8], &fu);
    report("sleep drop", &o);
    assert_eq!((o.questions, o.follow_ups), (6, 3));
    assert_eq!(level(&o, "sleep-schedules"), Level::Attention);
    assert_eq!(alerts(&o), 1, "only the sleep module reacts");
}

#[test]
fn vision_symptom_flags_sans_for_specialist_review() {
    let fu = [
        ("fu.red_flags", AnswerValue::YesNo(false)),
        ("fu.symptom_area", AnswerValue::Choice("eyes".into())),
        ("fu.fall_or_impact", AnswerValue::YesNo(false)),
        ("fu.vision_change", AnswerValue::YesNo(true)),
    ];
    let o = run_day(&history([8, 8, 8]), [8, 3, 8], &fu);
    report("vision symptom", &o);
    assert_eq!((o.questions, o.follow_ups), (7, 4));
    assert_eq!(o.flags, vec![Flag::VisionChange]);
    assert_eq!(level(&o, "sans"), Level::SpecialistReview);
}

#[test]
fn red_flag_answer_produces_urgent_flag() {
    let fu = [
        ("fu.red_flags", AnswerValue::YesNo(true)),
        ("fu.symptom_area", AnswerValue::Choice("chest".into())),
        ("fu.fall_or_impact", AnswerValue::YesNo(false)),
        ("fu.vision_change", AnswerValue::YesNo(false)),
    ];
    let o = run_day(&history([8, 8, 8]), [8, 1, 8], &fu);
    report("red-flag symptom", &o);
    assert!(o.flags.contains(&Flag::UrgentSymptoms));
    // The red flag belongs to no risk module, so it must reach specialists separately.
    assert_eq!(o.safety, 1, "red-flag answers must create a safety finding for specialists");
}

#[test]
fn assistant_answers_with_sources_or_says_it_has_none() {
    let vision = answer(&AssistantQuery { text: "blurred vision when reading".into(), body_area: Some("eyes".into()), ..Default::default() });
    let unknown = answer(&AssistantQuery { text: "how do I fix the coffee machine".into(), ..Default::default() });
    println!("{:<18} sources={} for a vision question; no_reliable_source={} for an unrelated one", "assistant", vision.matches.len(), unknown.no_reliable_source);
    assert!(!vision.matches.is_empty());
    assert!(unknown.no_reliable_source);
}
