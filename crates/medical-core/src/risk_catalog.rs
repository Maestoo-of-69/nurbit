//! The five approved risk modules (Module D) as configuration data.
//!
//! One shared structure for every module: what is monitored, which indicators
//! exist, how and how often each is collected, whether it is available in
//! flight, and how it is classified. No evaluation logic yet (Stage 8).
//!
//! Rules for this file:
//! * Only sourced facts. `reference_ids` point to rows in docs/REFERENCES.md.
//! * Reference limits are copied only from NASA standards; every other
//!   trigger is marked as needing specialist validation.
//! * Monitoring is not diagnosis (see each module's `limitations`).

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HrpCategory {
    HumanPhysiology,
    BehavioralPerformance,
    SpaceRadiation,
}

/// Scope is approved by the developer; monitoring logic is not active yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleStatus {
    ApprovedNotActive,
    Active,
}

/// What kind of information an indicator is (required distinction).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndicatorClass {
    DirectlyMeasured,
    SelfReported,
    Derived,
    RiskEstimate,
    SpecialistFinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Collection {
    /// Sensor or device records it without crew effort.
    Automatic,
    /// Crew member answers or performs it.
    SelfCollected,
    /// Exam or imaging performed with medical equipment.
    MedicalTest,
    /// Blood or urine analysis.
    Laboratory,
    /// Calculated from other data, usually by specialists.
    Calculated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Frequency {
    Continuous,
    Daily,
    Periodic,
    PreAndPostFlight,
    SymptomBased,
}

/// Where the beta gets the value from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BetaSource {
    DailyQuestionnaire,
    FollowUpQuestionnaire,
    ManualEntry,
    SimulatedDevice,
    NotInBeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct Indicator {
    pub name: &'static str,
    pub class: IndicatorClass,
    pub collection: Collection,
    pub frequency: Frequency,
    pub available_in_flight: bool,
    /// Meaningful to compare with the person's own earlier values.
    pub baseline_comparable: bool,
    pub equipment: Option<&'static str>,
    pub beta_source: BetaSource,
}

/// A limit stated in an official NASA standard (never invented).
#[derive(Debug, Clone, Serialize)]
pub struct ReferenceLimit {
    pub description: &'static str,
    pub source_id: &'static str,
    pub applies: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskModuleInfo {
    pub id: &'static str,
    /// Title as written in the NASA Human Research Roadmap / evidence report.
    pub hrp_title: &'static str,
    pub short_title: &'static str,
    pub category: HrpCategory,
    pub monitored: &'static str,
    pub indicators: &'static [Indicator],
    pub reference_limits: &'static [ReferenceLimit],
    /// Candidate reasons for extra questions; all need specialist validation.
    pub questionnaire_triggers: &'static [&'static str],
    pub specialist_package: &'static [&'static str],
    pub limitations: &'static [&'static str],
    pub reference_ids: &'static [&'static str],
    pub status: ModuleStatus,
}

use BetaSource as B;
use Collection as C;
use Frequency as F;
use IndicatorClass as K;

#[allow(clippy::too_many_arguments)] // a table row: one argument per column
const fn ind(
    name: &'static str,
    class: IndicatorClass,
    collection: Collection,
    frequency: Frequency,
    available_in_flight: bool,
    baseline_comparable: bool,
    equipment: Option<&'static str>,
    beta_source: BetaSource,
) -> Indicator {
    Indicator { name, class, collection, frequency, available_in_flight, baseline_comparable, equipment, beta_source }
}

pub const RISK_MODULES: [RiskModuleInfo; 5] = [
    RiskModuleInfo {
        id: "bone",
        hrp_title: "Risk of Bone Fracture due to Spaceflight-induced Changes to Bone",
        short_title: "Bone changes",
        category: HrpCategory::HumanPhysiology,
        monitored: "Countermeasure adherence and bone-related symptoms in flight; bone density only from ground scans before and after flight.",
        indicators: &[
            ind("Bone mineral density, hip and spine (DXA)", K::DirectlyMeasured, C::MedicalTest, F::PreAndPostFlight, false, true, Some("DXA scanner (ground)"), B::ManualEntry),
            ind("Change from pre-flight bone density", K::Derived, C::Calculated, F::PreAndPostFlight, false, true, None, B::ManualEntry),
            ind("Resistive exercise sessions and load", K::DirectlyMeasured, C::Automatic, F::Daily, true, true, Some("Resistive exercise device (ARED)"), B::SimulatedDevice),
            ind("Back, hip or limb pain; falls or impacts", K::SelfReported, C::SelfCollected, F::SymptomBased, true, false, None, B::FollowUpQuestionnaire),
            ind("Vitamin D and bone turnover markers", K::DirectlyMeasured, C::Laboratory, F::Periodic, false, true, Some("Laboratory assay"), B::NotInBeta),
            ind("Fracture risk estimate (FRAX)", K::RiskEstimate, C::Calculated, F::PreAndPostFlight, false, false, None, B::NotInBeta),
        ],
        reference_limits: &[ReferenceLimit {
            description: "Hip and spine bone density at or above 95% of pre-mission values; femoral neck at or above 90%.",
            source_id: "BONE-MTB",
            applies: "Countermeasure goal checked by post-flight scans; cannot be evaluated in flight.",
        }],
        questionnaire_triggers: &[
            "New or worsening back, hip or limb pain (needs specialist validation)",
            "Fall or impact reported (needs specialist validation)",
            "Several missed resistive exercise sessions (needs specialist validation)",
        ],
        specialist_package: &["Exercise history", "Pain and fall reports", "Pre-flight scan results entered by staff"],
        limitations: &[
            "No method measures bone density in flight; the app cannot know current bone density.",
            "Exercise data shows adherence, not bone health.",
            "In-mission fracture risk cannot be estimated with confidence.",
        ],
        reference_ids: &["HRP-BONE", "BONE-ER", "BONE-MTB", "BONE-SMITH"],
        status: ModuleStatus::ApprovedNotActive,
    },
    RiskModuleInfo {
        id: "sans",
        hrp_title: "Risk of Spaceflight Associated Neuro-ocular Syndrome (SANS)",
        short_title: "Neuro-ocular (SANS)",
        category: HrpCategory::HumanPhysiology,
        monitored: "Vision symptoms reported by the crew member and eye exam results recorded by the medical team.",
        indicators: &[
            ind("Vision change, headache (questionnaire)", K::SelfReported, C::SelfCollected, F::Daily, true, true, None, B::FollowUpQuestionnaire),
            ind("Visual acuity and Amsler grid", K::DirectlyMeasured, C::SelfCollected, F::Periodic, true, true, Some("Acuity chart, Amsler grid"), B::ManualEntry),
            ind("Optic nerve and retina imaging (OCT, fundoscopy)", K::SpecialistFinding, C::MedicalTest, F::Periodic, true, true, Some("OCT, fundoscope"), B::ManualEntry),
            ind("Ocular ultrasound", K::SpecialistFinding, C::MedicalTest, F::Periodic, true, true, Some("Ultrasound with ground guidance"), B::ManualEntry),
        ],
        reference_limits: &[],
        questionnaire_triggers: &["Reported vision change since the last check (needs specialist validation)"],
        specialist_package: &["Vision symptom history", "Acuity and Amsler results", "Imaging results and dates"],
        limitations: &[
            "A questionnaire cannot replace an ophthalmological examination.",
            "Mechanism and long-term outcome of SANS are not fully known.",
        ],
        reference_ids: &["HRP-SANS", "HRR-SANS", "SANS-ER", "SANS-OCT"],
        status: ModuleStatus::ApprovedNotActive,
    },
    RiskModuleInfo {
        id: "behavioral-health",
        hrp_title: "Risk of Adverse Cognitive or Behavioral Changes and Psychiatric Disorders",
        short_title: "Behavioral health",
        category: HrpCategory::BehavioralPerformance,
        monitored: "Self-reported mood, stress and concentration, and results of cognitive self-tests, compared with the person's own history.",
        indicators: &[
            ind("Mood and mental energy rating", K::SelfReported, C::SelfCollected, F::Daily, true, true, None, B::DailyQuestionnaire),
            ind("Stress and concentration ratings", K::SelfReported, C::SelfCollected, F::SymptomBased, true, true, None, B::FollowUpQuestionnaire),
            ind("Cognitive test battery results", K::DirectlyMeasured, C::SelfCollected, F::Periodic, true, true, Some("Laptop test battery (e.g. Cognition)"), B::ManualEntry),
            ind("Request to talk to the medical team", K::SelfReported, C::SelfCollected, F::SymptomBased, true, false, None, B::FollowUpQuestionnaire),
        ],
        reference_limits: &[],
        questionnaire_triggers: &[
            "Low mood rating or a drop from the personal average (prototype rule; needs specialist validation)",
        ],
        specialist_package: &["Mood and stress trends", "Contact requests", "Cognitive test results"],
        limitations: &[
            "Ratings and self-tests do not diagnose a psychiatric disorder.",
            "Validated clinical questionnaires may require licences; none are included.",
        ],
        reference_ids: &["HRP-BHV", "HRR-BHV", "BHV-COG", "BHV-APEX"],
        status: ModuleStatus::ApprovedNotActive,
    },
    RiskModuleInfo {
        id: "sleep-schedules",
        hrp_title: "Risk of Performance Decrements and Adverse Health Outcomes Resulting from Sleep Loss, Circadian Desynchronization, and Work Overload",
        short_title: "Sleep and schedules",
        category: HrpCategory::BehavioralPerformance,
        monitored: "Sleep duration and quality, alertness, and reaction-time performance compared with the person's own history.",
        indicators: &[
            ind("Rested rating", K::SelfReported, C::SelfCollected, F::Daily, true, true, None, B::DailyQuestionnaire),
            ind("Sleep hours and interruptions (sleep log)", K::SelfReported, C::SelfCollected, F::Daily, true, true, None, B::FollowUpQuestionnaire),
            ind("Sleep duration and timing (actigraphy)", K::DirectlyMeasured, C::Automatic, F::Continuous, true, true, Some("Wrist actigraph"), B::SimulatedDevice),
            ind("Reaction time and lapses (PVT self-test)", K::DirectlyMeasured, C::SelfCollected, F::Periodic, true, true, Some("Laptop reaction-time test"), B::NotInBeta),
        ],
        reference_limits: &[],
        questionnaire_triggers: &[
            "Low rested rating or a drop from the personal average (prototype rule; needs specialist validation)",
        ],
        specialist_package: &["Sleep hours and rested trends", "Interruption reports", "Alertness ratings"],
        limitations: &[
            "Self-reported sleep is less precise than actigraphy.",
            "In flight, actigraphy has been used mainly as a research tool.",
        ],
        reference_ids: &["HRP-SLEEP", "HRR-SLEEP", "SLEEP-RST", "SLEEP-BARGER"],
        status: ModuleStatus::ApprovedNotActive,
    },
    RiskModuleInfo {
        id: "radiation",
        hrp_title: "Risk of Radiation Carcinogenesis",
        short_title: "Radiation exposure",
        category: HrpCategory::SpaceRadiation,
        monitored: "Radiation dose received (from dosimeters) and its running total against the NASA career limit. Not cancer.",
        indicators: &[
            ind("Personal absorbed dose and dose rate", K::DirectlyMeasured, C::Automatic, F::Continuous, true, false, Some("Personal active dosimeter"), B::SimulatedDevice),
            ind("Cumulative mission and career effective dose", K::Derived, C::Calculated, F::Periodic, true, false, None, B::SimulatedDevice),
            ind("Shelter time and space-weather alerts", K::DirectlyMeasured, C::Automatic, F::SymptomBased, true, false, Some("Area monitors, space-weather service"), B::ManualEntry),
            ind("Risk of exposure-induced death (REID)", K::RiskEstimate, C::Calculated, F::Periodic, false, false, None, B::NotInBeta),
        ],
        reference_limits: &[ReferenceLimit {
            description: "Career effective dose from spaceflight below 600 mSv (all ages and sexes).",
            source_id: "RAD-STD",
            applies: "Tracked as a running total; effective dose is calculated by radiation specialists, not by the app.",
        }],
        questionnaire_triggers: &[],
        specialist_package: &["Dose time series", "Shelter and EVA log", "Alert events"],
        limitations: &[
            "Dose tracking estimates risk; it cannot detect cancer.",
            "No epidemiological data exist for space radiation; risk estimates are very uncertain.",
            "Dose does not cause symptoms that a questionnaire could detect at these levels.",
        ],
        reference_ids: &["HRP-RAD", "HRR-RAD", "RAD-SRAG", "RAD-STD", "RAD-NASEM"],
        status: ModuleStatus::ApprovedNotActive,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn there_are_exactly_the_five_approved_modules() {
        let ids: HashSet<_> = RISK_MODULES.iter().map(|m| m.id).collect();
        let expected: HashSet<_> =
            ["bone", "sans", "behavioral-health", "sleep-schedules", "radiation"].into_iter().collect();
        assert_eq!(ids, expected);
    }

    #[test]
    fn every_module_cites_sources_and_lists_limitations() {
        for m in &RISK_MODULES {
            assert!(!m.reference_ids.is_empty(), "{} has no sources", m.id);
            assert!(!m.limitations.is_empty(), "{} has no limitations", m.id);
            assert!(!m.indicators.is_empty(), "{} has no indicators", m.id);
        }
    }

    #[test]
    fn no_module_is_active_before_specialist_validation() {
        assert!(RISK_MODULES.iter().all(|m| m.status == ModuleStatus::ApprovedNotActive));
    }

    #[test]
    fn every_reference_limit_points_to_a_listed_source() {
        for m in &RISK_MODULES {
            for limit in m.reference_limits {
                assert!(m.reference_ids.contains(&limit.source_id), "{} limit lacks its source", m.id);
            }
        }
    }

    #[test]
    fn every_questionnaire_trigger_is_marked_for_validation() {
        for m in &RISK_MODULES {
            for trigger in m.questionnaire_triggers {
                assert!(trigger.contains("needs specialist validation"), "{trigger}");
            }
        }
    }

    #[test]
    fn bone_density_is_never_claimed_to_be_measured_in_flight() {
        let bone = RISK_MODULES.iter().find(|m| m.id == "bone").unwrap();
        let dxa = bone.indicators.iter().find(|i| i.name.contains("DXA")).unwrap();
        assert!(!dxa.available_in_flight);
    }
}
