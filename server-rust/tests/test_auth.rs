/// Integration tests for auth endpoints.

#[cfg(test)]
mod tests {
    use serde_json::Value;

    fn base_url() -> String {
        std::env::var("TEST_SERVER_URL").unwrap_or_else(|_| "http://localhost:8000".to_string())
    }

    async fn create_did_and_sign(client: &reqwest::Client) -> (String, String, String) {
        use base64::Engine;
        use ed25519_dalek::SigningKey;

        // Create DID
        let create_res = client
            .post(format!("{}/did/create", base_url()))
            .json(&serde_json::json!({
                "max_transaction_limit": 100000
            }))
            .send()
            .await
            .unwrap();
        let body: Value = create_res.json().await.unwrap();
        let did = body["did"].as_str().unwrap().to_string();
        let priv_key_b64 = body["private_key_base64"].as_str().unwrap().to_string();

        // Sign a message
        let b64 = base64::engine::general_purpose::STANDARD;
        let private_key_bytes = b64.decode(&priv_key_b64).unwrap();
        let priv_arr: [u8; 32] = private_key_bytes.try_into().unwrap();
        let signing_key = SigningKey::from_bytes(&priv_arr);

        let message = format!("auth_request_{}", chrono::Utc::now().timestamp());
        let message_b64 = b64.encode(message.as_bytes());

        use ed25519_dalek::Signer;
        let sig = signing_key.sign(message.as_bytes());
        let sig_b64 = b64.encode(sig.to_bytes());

        (did, message_b64, sig_b64)
    }

    #[tokio::test]
    async fn test_issue_token() {
        let client = reqwest::Client::new();
        let (did, message, signature) = create_did_and_sign(&client).await;

        let res = client
            .post(format!("{}/auth/token", base_url()))
            .json(&serde_json::json!({
                "did": did,
                "message": message,
                "signature": signature,
                "requested_scopes": ["payment:execute"]
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(res.status().as_u16(), 200);
        let body: Value = res.json().await.unwrap();
        assert!(body["access_token"].as_str().is_some());
        assert_eq!(body["token_type"].as_str().unwrap(), "Bearer");
        assert_eq!(body["expires_in"].as_i64().unwrap(), 1800);
    }

    #[tokio::test]
    async fn test_verify_token() {
        let client = reqwest::Client::new();
        let (did, message, signature) = create_did_and_sign(&client).await;

        // Get token
        let token_res = client
            .post(format!("{}/auth/token", base_url()))
            .json(&serde_json::json!({
                "did": did,
                "message": message,
                "signature": signature,
            }))
            .send()
            .await
            .unwrap();
        let token_body: Value = token_res.json().await.unwrap();
        let token = token_body["access_token"].as_str().unwrap().to_string();

        // Verify token
        let verify_res = client
            .post(format!("{}/auth/verify-token", base_url()))
            .json(&serde_json::json!({ "token": token }))
            .send()
            .await
            .unwrap();

        assert_eq!(verify_res.status().as_u16(), 200);
        let body: Value = verify_res.json().await.unwrap();
        assert!(body["valid"].as_bool().unwrap());
        assert_eq!(body["payload"]["sub"].as_str().unwrap(), did);
    }

    #[tokio::test]
    async fn test_invalid_signature_rejected() {
        let client = reqwest::Client::new();
        let (did, message, _) = create_did_and_sign(&client).await;

        let res = client
            .post(format!("{}/auth/token", base_url()))
            .json(&serde_json::json!({
                "did": did,
                "message": message,
                "signature": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(res.status().as_u16(), 401);
    }
}
