//! Synchronization protocol shared by the desktop app (client) and the server.
//!
//! Each record travels with its id and a SHA-256 hash of its content. The
//! server answers per record:
//! * `Accepted`  - new id, stored;
//! * `Duplicate` - same id and same hash already stored (safe re-send);
//! * `Conflict`  - same id but different content: BOTH versions are kept,
//!   nothing is overwritten, a specialist resolves it;
//! * `Invalid`   - hash does not match the content, or the payload is malformed.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::RecordId;

/// Upper bound on records per request, so one request stays small.
pub const MAX_RECORDS_PER_PUSH: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    CrewMember,
    CheckIn,
}

impl RecordKind {
    pub fn as_str(self) -> &'static str {
        match self {
            RecordKind::CrewMember => "crew_member",
            RecordKind::CheckIn => "check_in",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncRecord {
    pub kind: RecordKind,
    pub id: RecordId,
    pub content_hash: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushRequest {
    pub device_id: String,
    pub records: Vec<SyncRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AckStatus {
    Accepted,
    Duplicate,
    Conflict,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ack {
    pub id: RecordId,
    pub status: AckStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushResponse {
    pub acks: Vec<Ack>,
}

/// SHA-256 (hex) of the canonical JSON text of a payload. `serde_json::Value`
/// keeps object keys sorted, so the same content always gives the same hash.
pub fn content_hash(payload: &Value) -> String {
    let text = payload.to_string();
    let digest = Sha256::digest(text.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Builds a record ready to send.
pub fn make_record<T: Serialize>(kind: RecordKind, id: RecordId, payload: &T) -> serde_json::Result<SyncRecord> {
    let payload = serde_json::to_value(payload)?;
    Ok(SyncRecord { kind, id, content_hash: content_hash(&payload), payload })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hash_ignores_key_order() {
        let a = json!({"a": 1, "b": [1, 2]});
        let b: Value = serde_json::from_str(r#"{"b":[1,2],"a":1}"#).unwrap();
        assert_eq!(content_hash(&a), content_hash(&b));
    }

    #[test]
    fn hash_changes_when_content_changes() {
        assert_ne!(content_hash(&json!({"a": 1})), content_hash(&json!({"a": 2})));
    }

    #[test]
    fn hash_is_sha256_hex() {
        let h = content_hash(&json!({}));
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
