/// Ed25519 keypair generation and DID derivation.
/// Compatible with the Python implementation using PyNaCl/base58.
use base64::Engine;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use zeroize::ZeroizeOnDrop;

use crate::error::{AppError, AppResult};

/// Wrapper around Ed25519 signing key with automatic zeroing on drop.
#[derive(ZeroizeOnDrop)]
pub struct AgentKeyPair {
    #[zeroize(skip)]
    pub verifying_key: VerifyingKey,
    pub did: String,
    signing_key_bytes: [u8; 32],
}

impl AgentKeyPair {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let did = derive_did(&verifying_key);
        let signing_key_bytes = signing_key.to_bytes();
        Self {
            verifying_key,
            did,
            signing_key_bytes,
        }
    }

    pub fn sign(&self, message: &[u8]) -> ed25519_dalek::Signature {
        use ed25519_dalek::Signer;
        let signing_key = SigningKey::from_bytes(&self.signing_key_bytes);
        signing_key.sign(message)
    }

    pub fn private_key_bytes(&self) -> &[u8; 32] {
        &self.signing_key_bytes
    }

    pub fn private_key_base64(&self) -> String {
        let b64 = base64::engine::general_purpose::STANDARD;
        b64.encode(self.signing_key_bytes)
    }

    pub fn public_key_base64(&self) -> String {
        let b64 = base64::engine::general_purpose::STANDARD;
        b64.encode(self.verifying_key.as_bytes())
    }

    pub fn public_key_bytes(&self) -> &[u8; 32] {
        self.verifying_key.as_bytes()
    }
}

/// Derive a did:key identifier from an Ed25519 public key.
/// Uses multicodec ed25519-pub prefix (0xed01) and multibase base58btc encoding (z prefix).
/// Compatible with the Python implementation.
pub fn derive_did(verifying_key: &VerifyingKey) -> String {
    let public_key_bytes = verifying_key.as_bytes();
    // multicodec ed25519-pub prefix: 0xed01
    let mut multicodec = vec![0xed, 0x01];
    multicodec.extend_from_slice(public_key_bytes);
    let encoded = bs58::encode(&multicodec).into_string();
    format!("did:key:z{}", encoded)
}

/// Resolve a DID string to an Ed25519 public key.
pub fn resolve_public_key(did: &str) -> AppResult<VerifyingKey> {
    let encoded = did
        .strip_prefix("did:key:z")
        .ok_or(AppError::InvalidDID)?;
    let decoded = bs58::decode(encoded)
        .into_vec()
        .map_err(|_| AppError::InvalidDID)?;
    if decoded.len() < 34 || decoded[0] != 0xed || decoded[1] != 0x01 {
        return Err(AppError::InvalidDID);
    }
    let key_bytes: [u8; 32] = decoded[2..34]
        .try_into()
        .map_err(|_| AppError::InvalidDID)?;
    VerifyingKey::from_bytes(&key_bytes).map_err(|_| AppError::InvalidDID)
}

/// Encode public key bytes to base64 (standard encoding, compatible with Python).
pub fn public_key_to_base64(key_bytes: &[u8]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD;
    b64.encode(key_bytes)
}

/// Decode base64 public key string to bytes.
pub fn base64_to_public_key(b64_key: &str) -> AppResult<Vec<u8>> {
    let b64 = base64::engine::general_purpose::STANDARD;
    b64.decode(b64_key)
        .map_err(|_| AppError::InvalidDID)
}

/// Get the VerifyingKey from base64-encoded public key bytes.
pub fn verifying_key_from_base64(b64_key: &str) -> AppResult<VerifyingKey> {
    let bytes = base64_to_public_key(b64_key)?;
    if bytes.len() != 32 {
        return Err(AppError::InvalidDID);
    }
    let key_bytes: [u8; 32] = bytes.try_into().map_err(|_| AppError::InvalidDID)?;
    VerifyingKey::from_bytes(&key_bytes).map_err(|_| AppError::InvalidDID)
}
