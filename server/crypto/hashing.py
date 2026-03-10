"""
SHA-256 hash chain computation and verification for audit logs.
This module is designed to be replaceable with a Rust implementation via PyO3.
"""
import hashlib
import json


def compute_hash(index: int, transaction_id: str, amount: int,
                 status: str, timestamp: str, prev_hash: str) -> str:
    """Compute SHA-256 hash for an audit log entry.

    Args:
        index: Sequential index in the hash chain
        transaction_id: Unique transaction identifier
        amount: Transaction amount in smallest currency unit
        status: Transaction status string
        timestamp: ISO 8601 timestamp string
        prev_hash: Hash of the previous entry ("0"*64 for first entry)

    Returns:
        str: Hash string in format "sha256:{hex_digest}"
    """
    data = json.dumps({
        "index": index,
        "transaction_id": transaction_id,
        "amount": amount,
        "status": status,
        "timestamp": timestamp,
        "prev_hash": prev_hash,
    }, sort_keys=True, separators=(',', ':'))
    return "sha256:" + hashlib.sha256(data.encode()).hexdigest()


def verify_chain(entries: list[dict]) -> bool:
    """Verify the integrity of a hash chain.

    Checks that:
    1. Each entry's hash matches its computed hash
    2. Each entry's prev_hash matches the previous entry's hash

    Args:
        entries: List of audit log entry dicts with keys:
                 index, transaction_id, amount, status, timestamp, prev_hash, hash

    Returns:
        bool: True if chain is valid, False if any integrity check fails
    """
    for i, entry in enumerate(entries):
        expected_hash = compute_hash(
            entry["index"],
            entry["transaction_id"],
            entry["amount"],
            entry["status"],
            entry["timestamp"],
            entry["prev_hash"],
        )
        if expected_hash != entry["hash"]:
            return False
        if i > 0 and entry["prev_hash"] != entries[i - 1]["hash"]:
            return False
    return True
