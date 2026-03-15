/// Ed25519 signature creation and verification.
/// Message and signature are base64-encoded (compatible with Python SDK).
use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::error::{AppError, AppResult};

/// Verify an Ed25519 signature.
/// message: base64-encoded message bytes
/// signature_b64: base64-encoded signature bytes
pub fn verify_signature_b64(
    verifying_key: &VerifyingKey,
    message_b64: &str,
    signature_b64: &str,
) -> AppResult<()> {
    let b64 = base64::engine::general_purpose::STANDARD;

    let message_bytes = b64
        .decode(message_b64)
        .map_err(|_| AppError::InvalidSignature)?;

    let sig_bytes = b64
        .decode(signature_b64)
        .map_err(|_| AppError::InvalidSignature)?;

    if sig_bytes.len() != 64 {
        return Err(AppError::InvalidSignature);
    }
    let sig_array: [u8; 64] = sig_bytes.try_into().map_err(|_| AppError::InvalidSignature)?;
    let signature = Signature::from_bytes(&sig_array);

    verifying_key
        .verify(&message_bytes, &signature)
        .map_err(|_| AppError::InvalidSignature)
}
