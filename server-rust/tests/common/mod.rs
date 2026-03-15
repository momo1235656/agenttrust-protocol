/// Test helpers for integration tests.
/// Integration tests hit a live server at TEST_SERVER_URL (default: http://localhost:8000).

pub fn base_url() -> String {
    std::env::var("TEST_SERVER_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string())
}

pub fn reqwest_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("HTTP client")
}
