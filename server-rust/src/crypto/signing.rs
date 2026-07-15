/// Ed25519 signature creation and verification.
/// The signature is base64-encoded; the message is a plain UTF-8 string
/// signed as-is (matching the Python SDK's `message.encode()` contract).
use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::error::{AppError, AppResult};

/// Verify an Ed25519 signature.
/// message: plain UTF-8 message string (not base64-encoded)
/// signature_b64: base64-encoded signature bytes
pub fn verify_signature_b64(
    verifying_key: &VerifyingKey,
    message: &str,
    signature_b64: &str,
) -> AppResult<()> {
    let b64 = base64::engine::general_purpose::STANDARD;

    let message_bytes = message.as_bytes();

    let sig_bytes = b64
        .decode(signature_b64)
        .map_err(|_| AppError::InvalidSignature)?;

    if sig_bytes.len() != 64 {
        return Err(AppError::InvalidSignature);
    }
    let sig_array: [u8; 64] = sig_bytes.try_into().map_err(|_| AppError::InvalidSignature)?;
    let signature = Signature::from_bytes(&sig_array);

    verifying_key
        .verify(message_bytes, &signature)
        .map_err(|_| AppError::InvalidSignature)
}
