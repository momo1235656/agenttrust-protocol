/// Unit tests for the hash chain computation.
/// Verifies compatibility with the Python implementation.

#[cfg(test)]
mod tests {
    use agenttrust_server::crypto::hashing::{compute_hash, verify_chain, AuditEntry, GENESIS_PREV_HASH};

    #[test]
    fn test_compute_hash_format() {
        let hash = compute_hash(
            0,
            "tx_abc123",
            5000,
            "succeeded",
            "2024-01-01T00:00:00.000000+00:00",
            GENESIS_PREV_HASH,
        );
        assert!(hash.starts_with("sha256:"));
        assert_eq!(hash.len(), 7 + 64); // "sha256:" + 64 hex chars
    }

    #[test]
    fn test_genesis_prev_hash_is_64_zeros() {
        assert_eq!(GENESIS_PREV_HASH.len(), 64);
        assert!(GENESIS_PREV_HASH.chars().all(|c| c == '0'));
    }

    #[test]
    fn test_verify_chain_empty() {
        assert!(verify_chain(&[]));
    }

    #[test]
    fn test_verify_chain_single_entry() {
        let timestamp = "2024-01-01T00:00:00.000000+00:00";
        let hash = compute_hash(0, "tx_001", 1000, "succeeded", timestamp, GENESIS_PREV_HASH);
        let entries = vec![AuditEntry {
            index: 0,
            transaction_id: "tx_001".to_string(),
            amount: 1000,
            status: "succeeded".to_string(),
            timestamp: timestamp.to_string(),
            prev_hash: GENESIS_PREV_HASH.to_string(),
            hash: hash.clone(),
        }];
        assert!(verify_chain(&entries));
    }

    #[test]
    fn test_verify_chain_multi_entry() {
        let ts0 = "2024-01-01T00:00:00.000000+00:00";
        let ts1 = "2024-01-01T00:01:00.000000+00:00";

        let hash0 = compute_hash(0, "tx_001", 1000, "succeeded", ts0, GENESIS_PREV_HASH);
        let hash1 = compute_hash(1, "tx_002", 2000, "succeeded", ts1, &hash0);

        let entries = vec![
            AuditEntry {
                index: 0,
                transaction_id: "tx_001".to_string(),
                amount: 1000,
                status: "succeeded".to_string(),
                timestamp: ts0.to_string(),
                prev_hash: GENESIS_PREV_HASH.to_string(),
                hash: hash0.clone(),
            },
            AuditEntry {
                index: 1,
                transaction_id: "tx_002".to_string(),
                amount: 2000,
                status: "succeeded".to_string(),
                timestamp: ts1.to_string(),
                prev_hash: hash0.clone(),
                hash: hash1,
            },
        ];
        assert!(verify_chain(&entries));
    }

    #[test]
    fn test_verify_chain_tampered() {
        let ts = "2024-01-01T00:00:00.000000+00:00";
        let hash = compute_hash(0, "tx_001", 1000, "succeeded", ts, GENESIS_PREV_HASH);
        let entries = vec![AuditEntry {
            index: 0,
            transaction_id: "tx_001".to_string(),
            amount: 9999, // tampered amount
            status: "succeeded".to_string(),
            timestamp: ts.to_string(),
            prev_hash: GENESIS_PREV_HASH.to_string(),
            hash, // original hash doesn't match tampered data
        }];
        assert!(!verify_chain(&entries));
    }
}
