/// JWT generation and verification using Ed25519 (EdDSA algorithm).
/// Compatible with the Python PyJWT implementation.
use base64::Engine;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

pub const TOKEN_EXPIRY_SECONDS: i64 = 1800; // 30 minutes

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: String,
    pub scopes: Vec<String>,
    pub max_amount: i64,
    pub currency: String,
    pub allowed_categories: Vec<String>,
    pub iat: i64,
    pub exp: i64,
}

/// Build PKCS8 DER bytes for an Ed25519 private key from raw 32 bytes.
/// This is required by jsonwebtoken for EdDSA encoding.
fn private_key_to_pkcs8_der(raw_bytes: &[u8; 32]) -> Vec<u8> {
    // PKCS8 v2 DER encoding for Ed25519 private key
    // See RFC 8410 and RFC 5958
    let mut der = Vec::new();
    // SEQUENCE {
    //   INTEGER 0 (version)
    //   SEQUENCE { OID 1.3.101.112 }  (Ed25519)
    //   OCTET STRING { OCTET STRING { raw_key } }
    // }
    let oid = [0x06, 0x03, 0x2b, 0x65, 0x70]; // OID 1.3.101.112
    let inner_octet = [vec![0x04, 0x20], raw_bytes.to_vec()].concat();
    let outer_octet = [vec![0x04, inner_octet.len() as u8], inner_octet].concat();
    let alg_seq = [vec![0x30, oid.len() as u8], oid.to_vec()].concat();
    let version = [0x02, 0x01, 0x00]; // INTEGER 0

    let content_len = version.len() + alg_seq.len() + outer_octet.len();
    der.push(0x30); // SEQUENCE
    der.push(content_len as u8);
    der.extend_from_slice(&version);
    der.extend_from_slice(&alg_seq);
    der.extend_from_slice(&outer_octet);
    der
}

/// Build SubjectPublicKeyInfo DER bytes for an Ed25519 public key from raw 32 bytes.
fn public_key_to_spki_der(raw_bytes: &[u8; 32]) -> Vec<u8> {
    // SubjectPublicKeyInfo for Ed25519
    let oid = [0x06, 0x03, 0x2b, 0x65, 0x70]; // OID 1.3.101.112
    let alg_seq = [vec![0x30, oid.len() as u8], oid.to_vec()].concat();
    // BIT STRING: 0x00 (no unused bits) + raw key
    let bit_string = {
        let mut bs = vec![0x03, raw_bytes.len() as u8 + 1, 0x00]; // tag, len, unused_bits
        bs.extend_from_slice(raw_bytes);
        bs
    };
    let content_len = alg_seq.len() + bit_string.len();
    let mut der = vec![0x30, content_len as u8];
    der.extend_from_slice(&alg_seq);
    der.extend_from_slice(&bit_string);
    der
}

pub struct JwtKeys {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtKeys {
    pub fn from_bytes(private_bytes: &[u8], public_bytes: &[u8]) -> AppResult<Self> {
        if private_bytes.len() != 32 || public_bytes.len() != 32 {
            return Err(AppError::Internal("JWT key must be 32 bytes".to_string()));
        }
        let priv_arr: [u8; 32] = private_bytes.try_into().unwrap();
        let pub_arr: [u8; 32] = public_bytes.try_into().unwrap();

        let priv_der = private_key_to_pkcs8_der(&priv_arr);
        let pub_der = public_key_to_spki_der(&pub_arr);

        let encoding_key = EncodingKey::from_ed_der(&priv_der);
        let decoding_key = DecodingKey::from_ed_der(&pub_der);

        Ok(JwtKeys {
            encoding_key,
            decoding_key,
        })
    }

    /// Issue a JWT with the given claims.
    pub fn encode(&self, claims: &JwtClaims) -> AppResult<String> {
        let header = Header::new(Algorithm::EdDSA);
        encode(&header, claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(format!("JWT encode error: {e}")))
    }

    /// Verify and decode a JWT.
    pub fn decode(&self, token: &str) -> AppResult<JwtClaims> {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.validate_exp = true;
        // We don't use audience or issuer claims
        validation.validate_aud = false;
        validation.required_spec_claims = {
            let mut s = std::collections::HashSet::new();
            s.insert("exp".to_string());
            s.insert("sub".to_string());
            s
        };

        decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map(|d| d.claims)
            .map_err(|e| {
                use jsonwebtoken::errors::ErrorKind;
                match e.kind() {
                    ErrorKind::ExpiredSignature => AppError::TokenExpired,
                    _ => AppError::TokenInvalid(e.to_string()),
                }
            })
    }

    /// Return the public key as a JWK (JSON Web Key) for the /oauth/jwks endpoint.
    pub fn to_jwk(&self, public_bytes: &[u8]) -> serde_json::Value {
        let b64url = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        serde_json::json!({
            "kty": "OKP",
            "crv": "Ed25519",
            "x": b64url.encode(public_bytes),
            "kid": "agenttrust-signing-key-1",
            "use": "sig",
            "alg": "EdDSA"
        })
    }
}
