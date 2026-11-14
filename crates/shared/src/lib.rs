//! Types shared by every part of Nurbit.
//!
//! This crate holds only data definitions (no database, no network, no medical
//! rules), so the desktop app, the server and `medical-core` can all depend on it
//! without pulling in each other's code.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod records;
pub mod sync;

/// Shown on every screen: the beta never contains real medical data.
pub const SYNTHETIC_DATA_NOTICE: &str =
    "Prototype - synthetic data only - not a medical device";

/// Unique id of any medical record.
///
/// Created on the device that first records the data (even offline), so the
/// same record keeps the same id after synchronization. UUID v7 starts with a
/// timestamp, so ids sort roughly by creation time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RecordId(Uuid);

impl RecordId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Reads an id back from its text form (as stored in the database).
    pub fn parse(text: &str) -> Option<Self> {
        Uuid::parse_str(text).ok().map(Self)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for RecordId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RecordId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Where a value came from. Stored with every measurement so a reviewer can
/// tell a self-report from a device reading or a lab result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataOrigin {
    /// Answer typed or selected by the crew member.
    SelfReport,
    /// Value entered by hand from an instrument or exam.
    ManualEntry,
    /// Value read from a connected device.
    Device,
    /// Laboratory analysis result.
    Lab,
    /// Generated for the demo; never real data.
    Simulated,
}

impl DataOrigin {
    pub fn as_str(self) -> &'static str {
        match self {
            DataOrigin::SelfReport => "self_report",
            DataOrigin::ManualEntry => "manual_entry",
            DataOrigin::Device => "device",
            DataOrigin::Lab => "lab",
            DataOrigin::Simulated => "simulated",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [Self::SelfReport, Self::ManualEntry, Self::Device, Self::Lab, Self::Simulated]
            .into_iter()
            .find(|o| o.as_str() == s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_ids_are_unique() {
        assert_ne!(RecordId::new(), RecordId::new());
    }

    #[test]
    fn record_id_serializes_as_plain_uuid_string() {
        let id = RecordId::new();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, format!("\"{id}\""));
    }

    #[test]
    fn data_origin_uses_snake_case_in_json() {
        let json = serde_json::to_string(&DataOrigin::SelfReport).unwrap();
        assert_eq!(json, "\"self_report\"");
    }
}
