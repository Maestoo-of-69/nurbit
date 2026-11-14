//! Synthetic demo crew with 14 days of synthetic daily check-ins.
//!
//! Every profile is marked `is_synthetic` and every seeded check-in has origin
//! `simulated`, so demo data can never be mistaken for real data. Values are
//! deterministic (no randomness) so the demo looks the same on every machine.

use nurbit_medical_core::checkin::{DAILY_MOOD_ID, DAILY_PHYSICAL_ID, DAILY_RESTED_ID};
use nurbit_medical_core::risk_eval::{DOSE_DAILY, DOSE_PRIOR, EXERCISE_MINUTES};
use nurbit_shared::records::{
    AnswerRecord, AnswerValue, CheckInRecord, CrewMember, CrewRole, Measurement, TimestampMs,
};
use nurbit_shared::{DataOrigin, RecordId};

use crate::{LocalStore, StoreResult};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;
pub const SEED_DAYS: i64 = 14;
const SEED_DEVICE: &str = "synthetic-seed";

/// (name, position, typical [rested, physical, mood], pattern)
struct Persona {
    name: &'static str,
    position: &'static str,
    typical: [u8; 3],
    pattern: Pattern,
}

/// Story each synthetic astronaut tells in the demo.
#[derive(Clone, Copy)]
enum Pattern {
    /// Stable, ordinary days.
    Steady,
    /// Sleep ratings fall over the last few days (sleep module demo).
    SleepDecline,
    /// Mood slowly lower in the second week (behavioral module demo).
    MoodDip,
}

const PERSONAS: [Persona; 4] = [
    Persona { name: "Vega", position: "Commander", typical: [8, 8, 8], pattern: Pattern::Steady },
    Persona { name: "Orion", position: "Flight engineer", typical: [8, 7, 7], pattern: Pattern::SleepDecline },
    Persona { name: "Lyra", position: "Mission specialist", typical: [7, 8, 8], pattern: Pattern::MoodDip },
    Persona { name: "Atlas", position: "Payload specialist", typical: [7, 7, 8], pattern: Pattern::Steady },
];

/// Small deterministic wobble of -1, 0 or +1 so the history is not flat.
fn wobble(day: i64, salt: i64) -> i8 {
    [0, 1, 0, -1, 0, 0, 1][((day * 3 + salt * 5) % 7) as usize]
}

fn scores(persona: &Persona, salt: i64, days_ago: i64) -> [u8; 3] {
    let mut s = persona.typical.map(i16::from);
    match persona.pattern {
        Pattern::Steady => {}
        Pattern::SleepDecline if days_ago <= 3 => s[0] -= 4 - days_ago as i16,
        Pattern::MoodDip if days_ago <= 6 => s[2] -= 2,
        _ => {}
    }
    for (i, v) in s.iter_mut().enumerate() {
        *v = (*v + i16::from(wobble(days_ago, salt + i as i64))).clamp(0, 10);
    }
    s.map(|v| v as u8)
}

fn seeded_checkin(member: RecordId, persona: &Persona, salt: i64, days_ago: i64, now: TimestampMs) -> CheckInRecord {
    let completed_at = now - days_ago * DAY_MS;
    let [rested, physical, mood] = scores(persona, salt, days_ago);
    let answer = |id: &str, v: u8| AnswerRecord {
        question_id: id.to_string(),
        value: AnswerValue::Scale(v),
        triggered_by_rule: None,
    };
    CheckInRecord {
        id: RecordId::new(),
        crew_member_id: member,
        started_at: completed_at - 70_000,
        completed_at,
        device_id: SEED_DEVICE.to_string(),
        origin: DataOrigin::Simulated,
        answers: vec![
            answer(DAILY_RESTED_ID, rested),
            answer(DAILY_PHYSICAL_ID, physical),
            answer(DAILY_MOOD_ID, mood),
        ],
    }
}

/// Synthetic prior career dose per persona, mSv. ILLUSTRATIVE VALUES ONLY:
/// they are not real astronaut or ISS dose data.
const PRIOR_DOSE_MSV: [(&str, f64); 4] = [("Vega", 120.0), ("Orion", 45.0), ("Lyra", 0.0), ("Atlas", 80.0)];
/// Synthetic daily effective dose, mSv. Illustrative only, not a real ISS dose rate.
const DAILY_DOSE_MSV: f64 = 0.5;

fn simulated(member: RecordId, indicator: &str, value: f64, unit: &str, at: TimestampMs) -> Measurement {
    Measurement {
        id: RecordId::new(),
        crew_member_id: member,
        indicator: indicator.to_string(),
        value,
        unit: unit.to_string(),
        measured_at: at,
        origin: DataOrigin::Simulated,
    }
}

/// Simulated device data (exercise device log and dosimeter) for each astronaut,
/// only if no measurements exist yet. Returns true when data was created.
pub async fn seed_demo_measurements_if_empty(store: &LocalStore, now: TimestampMs) -> StoreResult<bool> {
    if store.measurement_count().await? > 0 {
        return Ok(false);
    }
    for member in store.list_crew().await?.iter().filter(|m| m.role == CrewRole::Astronaut) {
        let prior = PRIOR_DOSE_MSV.iter().find(|(n, _)| *n == member.display_name).map_or(0.0, |(_, d)| *d);
        let start = now - SEED_DAYS * DAY_MS;
        store.insert_measurement(&simulated(member.id, DOSE_PRIOR, prior, "mSv", start), now).await?;
        for days_ago in (1..=SEED_DAYS).rev() {
            let at = now - days_ago * DAY_MS;
            store.insert_measurement(&simulated(member.id, DOSE_DAILY, DAILY_DOSE_MSV, "mSv", at), now).await?;
            // Atlas skips most resistive sessions in the last week (bone-module demo).
            let skipped = member.display_name == "Atlas" && days_ago <= 7 && days_ago % 3 != 0;
            if !skipped {
                let minutes = 60.0 + f64::from(wobble(days_ago, 9)) * 10.0;
                store.insert_measurement(&simulated(member.id, EXERCISE_MINUTES, minutes, "min", at), now).await?;
            }
        }
    }
    Ok(true)
}

/// Adds a second synthetic flight surgeon (needed to demonstrate the
/// two-specialist review) if the database does not have one yet.
pub async fn seed_second_specialist_if_missing(store: &LocalStore, now: TimestampMs) -> StoreResult<bool> {
    let specialists = store.list_crew().await?.into_iter().filter(|m| m.role == CrewRole::Specialist).count();
    if specialists >= 2 {
        return Ok(false);
    }
    let rigel = CrewMember {
        id: RecordId::new(),
        display_name: "Dr. Rigel".to_string(),
        role: CrewRole::Specialist,
        position: "Flight surgeon (second reviewer)".to_string(),
        is_synthetic: true,
    };
    store.insert_crew_member(&rigel, now).await?;
    Ok(true)
}

/// Creates the synthetic crew and their history, only if the database has no crew yet.
/// Returns true when data was created.
pub async fn seed_demo_crew_if_empty(store: &LocalStore, now: TimestampMs) -> StoreResult<bool> {
    if !store.list_crew().await?.is_empty() {
        return Ok(false);
    }
    for (salt, persona) in PERSONAS.iter().enumerate() {
        let member = CrewMember {
            id: RecordId::new(),
            display_name: persona.name.to_string(),
            role: CrewRole::Astronaut,
            position: persona.position.to_string(),
            is_synthetic: true,
        };
        store.insert_crew_member(&member, now).await?;
        for days_ago in (1..=SEED_DAYS).rev() {
            let record = seeded_checkin(member.id, persona, salt as i64, days_ago, now);
            store.save_checkin(&record, now, false).await?;
        }
    }
    let surgeon = CrewMember {
        id: RecordId::new(),
        display_name: "Dr. Sirius".to_string(),
        role: CrewRole::Specialist,
        position: "Flight surgeon (ground)".to_string(),
        is_synthetic: true,
    };
    store.insert_crew_member(&surgeon, now).await?;
    Ok(true)
}
