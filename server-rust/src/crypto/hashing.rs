/// SHA-256 hash chain computation and verification for audit logs.
/// Must produce identical output to the Python implementation for backward compatibility.
///
/// Python reference:
/// ```python
/// data = json.dumps({...}, sort_keys=True, separators=(',', ':'))
/// return "sha256:" + hashlib.sha256(data.encode()).hexdigest()
/// ```
use sha2::{Digest, Sha256};

/// Compute SHA-256 hash for an audit log entry.
/// The JSON keys are sorted alphabetically, matching Python's sort_keys=True:
/// amount, index, prev_hash, status, timestamp, transaction_id
pub fn compute_hash(
    index: i64,
    transaction_id: &str,
    amount: i64,
    status: &str,
    timestamp: &str,
    prev_hash: &str,
) -> String {
    // Produce the same JSON as Python's json.dumps with sort_keys=True, separators=(',',':')
    // Key order (alphabetical): amount, index, prev_hash, status, timestamp, transaction_id
    let json = format!(
        r#"{{"amount":{},"index":{},"prev_hash":"{}","status":"{}","timestamp":"{}","transaction_id":"{}"}}"#,
        amount, index, prev_hash, status, timestamp, transaction_id
    );

    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{}", hex::encode(result))
}

/// Verify the integrity of a hash chain.
pub fn verify_chain(entries: &[AuditEntry]) -> bool {
    for (i, entry) in entries.iter().enumerate() {
        let expected = compute_hash(
            entry.index,
            &entry.transaction_id,
            entry.amount,
            &entry.status,
            &entry.timestamp,
            &entry.prev_hash,
        );
        if expected != entry.hash {
            return false;
        }
        if i > 0 && entry.prev_hash != entries[i - 1].hash {
            return false;
        }
    }
    true
}

pub struct AuditEntry {
    pub index: i64,
    pub transaction_id: String,
    pub amount: i64,
    pub status: String,
    pub timestamp: String,
    pub prev_hash: String,
    pub hash: String,
}

/// The genesis prev_hash value (first entry in the chain).
pub const GENESIS_PREV_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
