/// Integration tests for DID endpoints.
/// These tests require a running PostgreSQL instance.
/// Set TEST_DATABASE_URL to run them.

#[cfg(test)]
mod tests {
    use serde_json::Value;

    fn base_url() -> String {
        std::env::var("TEST_SERVER_URL").unwrap_or_else(|_| "http://localhost:8000".to_string())
    }

    #[tokio::test]
    async fn test_create_did() {
        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/did/create", base_url()))
            .json(&serde_json::json!({
                "display_name": "test-agent",
                "max_transaction_limit": 50000,
                "allowed_categories": ["shopping"]
            }))
            .send()
            .await
            .expect("Request failed");

        assert_eq!(res.status().as_u16(), 201);
        let body: Value = res.json().await.expect("JSON parse failed");
        assert!(body["did"].as_str().unwrap().starts_with("did:key:z"));
        assert!(body["private_key_base64"].as_str().is_some());
        assert!(body["document"]["@context"].is_array());
    }

    #[tokio::test]
    async fn test_resolve_did() {
        let client = reqwest::Client::new();

        // Create a DID first
        let create_res = client
            .post(format!("{}/did/create", base_url()))
            .json(&serde_json::json!({}))
            .send()
            .await
            .expect("Create request failed");
        let create_body: Value = create_res.json().await.unwrap();
        let did = create_body["did"].as_str().unwrap().to_string();

        // Resolve it
        let res = client
            .get(format!("{}/did/resolve/{}", base_url(), did))
            .send()
            .await
            .expect("Resolve request failed");

        assert_eq!(res.status().as_u16(), 200);
        let body: Value = res.json().await.unwrap();
        assert_eq!(body["did"].as_str().unwrap(), did);
        assert!(body["found"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_resolve_nonexistent_did() {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/did/resolve/did:key:zNONEXISTENT", base_url()))
            .send()
            .await
            .expect("Request failed");

        assert_eq!(res.status().as_u16(), 404);
    }
}
