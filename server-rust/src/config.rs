use anyhow::{Context, Result};
use base64::Engine;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

/// Application configuration. Loaded from environment variables.
/// Sensitive values are stored as Strings internally but treated with care.
#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    // Stored as plain String internally; exposed only via accessor
    stripe_secret_key: String,
    pub stripe_webhook_secret: Option<String>,
    pub jwt_private_key_bytes: Vec<u8>,
    pub jwt_public_key_bytes: Vec<u8>,
    pub cors_origins: Vec<String>,
    pub did_store_path: String,
    pub rate_limit_per_agent_per_minute: u64,
    pub rate_limit_per_ip_per_minute: u64,
    pub approval_expiry_hours: u64,
    pub approval_required_above: i64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse::<u16>()
            .context("Invalid PORT")?;

        let database_url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")?;

        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let stripe_secret_key = std::env::var("STRIPE_SECRET_KEY")
            .unwrap_or_else(|_| "sk_test_placeholder".to_string());

        let stripe_webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET").ok();

        let (private_key_bytes, public_key_bytes) = load_or_generate_jwt_keys()?;

        let cors_origins = std::env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8080".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let did_store_path = std::env::var("DID_STORE_PATH")
            .unwrap_or_else(|_| "./data/dids".to_string());

        let rate_limit_per_agent_per_minute = std::env::var("RATE_LIMIT_PER_AGENT_PER_MINUTE")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .unwrap_or(60);

        let rate_limit_per_ip_per_minute = std::env::var("RATE_LIMIT_PER_IP_PER_MINUTE")
            .unwrap_or_else(|_| "120".to_string())
            .parse()
            .unwrap_or(120);

        let approval_expiry_hours = std::env::var("APPROVAL_EXPIRY_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .unwrap_or(24);

        let approval_required_above = std::env::var("APPROVAL_REQUIRED_ABOVE")
            .unwrap_or_else(|_| "30000".to_string())
            .parse()
            .unwrap_or(30000);

        Ok(Config {
            port,
            database_url,
            redis_url,
            stripe_secret_key,
            stripe_webhook_secret,
            jwt_private_key_bytes: private_key_bytes,
            jwt_public_key_bytes: public_key_bytes,
            cors_origins,
            did_store_path,
            rate_limit_per_agent_per_minute,
            rate_limit_per_ip_per_minute,
            approval_expiry_hours,
            approval_required_above,
        })
    }

    /// Returns Stripe secret key. Do not log this value.
    pub fn stripe_secret_key(&self) -> &str {
        &self.stripe_secret_key
    }
}

fn load_or_generate_jwt_keys() -> Result<(Vec<u8>, Vec<u8>)> {
    let b64 = base64::engine::general_purpose::STANDARD;

    let private_env = std::env::var("JWT_SERVER_PRIVATE_KEY").unwrap_or_default();
    let public_env = std::env::var("JWT_SERVER_PUBLIC_KEY").unwrap_or_default();

    if !private_env.is_empty() && !public_env.is_empty() {
        let private_bytes = b64
            .decode(private_env.trim())
            .context("Invalid JWT_SERVER_PRIVATE_KEY (expected base64-encoded 32 bytes)")?;
        let public_bytes = b64
            .decode(public_env.trim())
            .context("Invalid JWT_SERVER_PUBLIC_KEY (expected base64-encoded 32 bytes)")?;
        if private_bytes.len() == 32 && public_bytes.len() == 32 {
            return Ok((private_bytes, public_bytes));
        }
        tracing::warn!("JWT keys have unexpected length; generating new ephemeral keys");
    }

    tracing::warn!(
        "JWT keys not configured — generating ephemeral keys. \
         Set JWT_SERVER_PRIVATE_KEY and JWT_SERVER_PUBLIC_KEY for persistence."
    );
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key: VerifyingKey = signing_key.verifying_key();
    let private_bytes = signing_key.to_bytes().to_vec();
    let public_bytes = verifying_key.to_bytes().to_vec();

    let priv_b64 = b64.encode(&private_bytes);
    let pub_b64 = b64.encode(&public_bytes);
    eprintln!("Generated JWT_SERVER_PRIVATE_KEY={}", priv_b64);
    eprintln!("Generated JWT_SERVER_PUBLIC_KEY={}", pub_b64);

    Ok((private_bytes, public_bytes))
}
