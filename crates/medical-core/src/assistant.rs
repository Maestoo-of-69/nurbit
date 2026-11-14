//! Medical assistant (Module F), beta = retrieval only.
//!
//! The assistant does NOT generate medical text. It searches a small local
//! knowledge base of paraphrased, source-checked excerpts (knowledge-base/
//! entries.json, every entry tied to a row in docs/REFERENCES.md) and returns
//! the best matches with their sources. It never diagnoses, never names
//! medication or doses, and says so when no reliable source matches.
//!
//! Matching is plain keyword overlap: transparent and testable, not "AI".

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

const KB_JSON: &str = include_str!("../../../knowledge-base/entries.json");
/// Minimum score for a match to be shown (prototype assumption).
pub const MIN_SCORE: u32 = 2;
pub const MAX_RESULTS: usize = 3;
/// Severity at or above this asks the crew member to contact the medical
/// officer now (prototype assumption, needs specialist validation).
pub const URGENT_SEVERITY: u8 = 8;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KbEntry {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub source_id: String,
    pub source_name: String,
    pub document: String,
    pub published: String,
    pub section: String,
    pub keywords: Vec<String>,
    pub body_areas: Vec<String>,
    pub module: Option<String>,
}

pub fn knowledge_base() -> &'static [KbEntry] {
    static KB: OnceLock<Vec<KbEntry>> = OnceLock::new();
    KB.get_or_init(|| serde_json::from_str(KB_JSON).expect("knowledge-base/entries.json is checked by tests"))
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AssistantQuery {
    pub text: String,
    /// "eyes", "head", "back", "limbs", "mind", "general".
    pub body_area: Option<String>,
    /// 0 = none, 10 = worst.
    pub severity: Option<u8>,
    /// "sudden", "gradual" or "unsure".
    pub onset: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KbMatch {
    pub entry: KbEntry,
    pub score: u32,
    /// Words that made this entry match, shown as "why this source".
    pub matched_terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssistantAnswer {
    pub provider: &'static str,
    pub matches: Vec<KbMatch>,
    /// Risk modules the matched sources belong to (monitoring context, not a diagnosis).
    pub related_modules: Vec<String>,
    pub urgent_message: Option<&'static str>,
    pub medication_notice: Option<&'static str>,
    pub missing_info: Vec<&'static str>,
    pub no_reliable_source: bool,
    pub uncertainty: &'static str,
}

const RED_FLAGS: &[&str] = &[
    "chest pain",
    "faint",
    "passed out",
    "difficulty breathing",
    "can't breathe",
    "cannot breathe",
    "short of breath",
    "sudden vision loss",
    "lost vision",
    "loss of vision",
    "severe headache",
    "worst headache",
];
const MEDICATION_WORDS: &[&str] = &["dose", "dosage", "mg", "medication", "medicine", "pill", "pills", "drug", "tablet"];

pub const URGENT_MESSAGE: &str = "This may need urgent attention. Contact the crew medical officer or flight surgeon now. This is a safety rule of the prototype, not a diagnosis.";
pub const MEDICATION_NOTICE: &str = "Nurbit does not suggest medications or doses. Please ask the flight surgeon.";
pub const UNCERTAINTY: &str = "These excerpts describe what NASA and published studies say about spaceflight risks in general. They cannot tell what is causing your symptom. Only a medical specialist can assess it.";

fn tokens(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 1)
        .map(str::to_string)
        .collect()
}

fn score(entry: &KbEntry, words: &[String], area: Option<&str>) -> (u32, Vec<String>) {
    let mut matched: Vec<String> = entry.keywords.iter().filter(|k| words.contains(k)).cloned().collect();
    matched.sort();
    matched.dedup();
    let mut total = matched.len() as u32 * 2;
    if let Some(a) = area
        && entry.body_areas.iter().any(|b| b == a)
    {
        total += 1;
        matched.push(format!("area: {a}"));
    }
    (total, matched)
}

/// Answers one question from the knowledge base. Pure function.
pub fn answer(query: &AssistantQuery) -> AssistantAnswer {
    let lower = query.text.to_lowercase();
    let words = tokens(&query.text);

    let mut matches: Vec<KbMatch> = knowledge_base()
        .iter()
        .map(|e| {
            let (s, terms) = score(e, &words, query.body_area.as_deref());
            KbMatch { entry: e.clone(), score: s, matched_terms: terms }
        })
        .filter(|m| m.score >= MIN_SCORE)
        .collect();
    matches.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.entry.id.cmp(&b.entry.id)));
    matches.truncate(MAX_RESULTS);

    let mut related_modules: Vec<String> = matches.iter().filter_map(|m| m.entry.module.clone()).collect();
    related_modules.dedup();

    let urgent = RED_FLAGS.iter().any(|f| lower.contains(f)) || query.severity.is_some_and(|s| s >= URGENT_SEVERITY);
    let asks_medication = words.iter().any(|w| MEDICATION_WORDS.contains(&w.as_str()));

    let mut missing_info = Vec::new();
    if query.body_area.is_none() {
        missing_info.push("body_area");
    }
    if query.severity.is_none() {
        missing_info.push("severity");
    }
    if query.onset.is_none() {
        missing_info.push("onset");
    }

    AssistantAnswer {
        provider: "retrieval_only",
        no_reliable_source: matches.is_empty(),
        matches,
        related_modules,
        urgent_message: urgent.then_some(URGENT_MESSAGE),
        medication_notice: asks_medication.then_some(MEDICATION_NOTICE),
        missing_info,
        uncertainty: UNCERTAINTY,
    }
}

// ---------------- providers ----------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderState {
    Active,
    NotInstalled,
    Unavailable,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub state: ProviderState,
    pub note: &'static str,
}

/// The providers the interface can show. Only retrieval exists in the beta.
pub fn providers() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            id: "retrieval_only",
            label: "Verified-source search",
            state: ProviderState::Active,
            note: "Works offline. Searches a local library of source-checked excerpts. No language model.",
        },
        ProviderInfo {
            id: "local_llm",
            label: "Local language model",
            state: ProviderState::NotInstalled,
            note: "Not integrated in this beta. Planned behind the same interface.",
        },
        ProviderInfo {
            id: "cloud",
            label: "Cloud assistant",
            state: ProviderState::Unavailable,
            note: "Not available offline and not authorized in this beta.",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFERENCES: &str = include_str!("../../../docs/REFERENCES.md");

    fn ask(text: &str, area: Option<&str>) -> AssistantAnswer {
        answer(&AssistantQuery { text: text.into(), body_area: area.map(str::to_string), ..Default::default() })
    }

    #[test]
    fn knowledge_base_loads_and_ids_are_unique() {
        let kb = knowledge_base();
        assert!(kb.len() >= 10);
        let mut ids: Vec<_> = kb.iter().map(|e| e.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), kb.len());
    }

    #[test]
    fn every_entry_cites_a_source_listed_in_references() {
        for e in knowledge_base() {
            assert!(REFERENCES.contains(&format!("| {} |", e.source_id)), "{} cites unknown source {}", e.id, e.source_id);
        }
    }

    #[test]
    fn knowledge_base_never_names_doses() {
        for e in knowledge_base() {
            let t = e.summary.to_lowercase();
            assert!(!t.contains(" mg") && !t.contains("dose of"), "{} mentions a dose", e.id);
        }
    }

    #[test]
    fn blurred_vision_finds_sans_sources() {
        let a = ask("My vision is blurred when reading", Some("eyes"));
        assert!(!a.no_reliable_source);
        assert!(a.matches.iter().all(|m| m.entry.module.as_deref() == Some("sans")));
        assert!(a.related_modules.contains(&"sans".to_string()));
    }

    #[test]
    fn unrelated_question_reports_no_reliable_source() {
        let a = ask("What is the weather on Mars", None);
        assert!(a.no_reliable_source);
        assert!(a.matches.is_empty());
    }

    #[test]
    fn red_flag_words_or_high_severity_give_the_urgent_message() {
        assert!(ask("sudden chest pain", None).urgent_message.is_some());
        let high = answer(&AssistantQuery { text: "back hurts".into(), severity: Some(URGENT_SEVERITY), ..Default::default() });
        assert!(high.urgent_message.is_some());
        assert!(ask("tired today", None).urgent_message.is_none());
    }

    #[test]
    fn medication_questions_get_the_notice_and_no_dose() {
        let a = ask("what dose of medication should I take for sleep", None);
        assert_eq!(a.medication_notice, Some(MEDICATION_NOTICE));
    }

    #[test]
    fn missing_details_are_listed_for_follow_up() {
        let a = ask("tired", None);
        assert_eq!(a.missing_info, vec!["body_area", "severity", "onset"]);
    }

    #[test]
    fn only_the_retrieval_provider_is_active() {
        let p = providers();
        assert_eq!(p.iter().filter(|x| x.state == ProviderState::Active).count(), 1);
        assert_eq!(p[0].id, "retrieval_only");
    }
}
