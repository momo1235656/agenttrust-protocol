use async_trait::async_trait;

use crate::error::{AppError, AppResult};
use crate::payment_providers::traits::{PaymentProvider, PaymentResult, RefundResult};

pub struct StripeProvider {
    api_key: String,
    client: reqwest::Client,
}

impl StripeProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .build()
            .expect("Failed to create HTTP client");
        Self {
            api_key: api_key.into(),
            client,
        }
    }

    fn stripe_url(&self, path: &str) -> String {
        format!("https://api.stripe.com/v1{}", path)
    }
}

#[async_trait]
impl PaymentProvider for StripeProvider {
    fn name(&self) -> &str {
        "stripe"
    }

    fn supported_currencies(&self) -> Vec<String> {
        vec!["jpy".to_string(), "usd".to_string(), "eur".to_string()]
    }

    async fn execute(
        &self,
        amount: i64,
        currency: &str,
        description: &str,
        idempotency_key: &str,
    ) -> AppResult<PaymentResult> {
        let params = [
            ("amount", amount.to_string()),
            ("currency", currency.to_string()),
            ("description", description.to_string()),
            ("payment_method", "pm_card_visa".to_string()),
            ("confirm", "true".to_string()),
            ("automatic_payment_methods[enabled]", "true".to_string()),
            (
                "automatic_payment_methods[allow_redirects]",
                "never".to_string(),
            ),
        ];

        let response = self
            .client
            .post(self.stripe_url("/payment_intents"))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .header("Idempotency-Key", idempotency_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::PaymentFailed(format!("Stripe request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_body: serde_json::Value = response
                .json()
                .await
                .unwrap_or_else(|_| serde_json::json!({"error": {"message": "Unknown Stripe error"}}));
            let msg = error_body["error"]["message"]
                .as_str()
                .unwrap_or("Stripe error")
                .to_string();
            return Err(AppError::PaymentFailed(msg));
        }

        let pi: serde_json::Value = response.json().await.map_err(|e| {
            AppError::PaymentFailed(format!("Failed to parse Stripe response: {}", e))
        })?;

        let id = pi["id"]
            .as_str()
            .ok_or_else(|| AppError::PaymentFailed("Missing id in Stripe response".to_string()))?
            .to_string();

        let status = pi["status"].as_str().unwrap_or("unknown").to_string();

        Ok(PaymentResult {
            provider_payment_id: id,
            status,
            amount,
        })
    }

    async fn refund(
        &self,
        provider_payment_id: &str,
        amount: Option<i64>,
        reason: &str,
    ) -> AppResult<RefundResult> {
        let mut params = vec![("payment_intent", provider_payment_id.to_string())];
        if let Some(amt) = amount {
            params.push(("amount", amt.to_string()));
        }
        let stripe_reason = match reason {
            "duplicate" | "fraudulent" => reason.to_string(),
            _ => "requested_by_customer".to_string(),
        };
        params.push(("reason", stripe_reason));

        let response = self
            .client
            .post(self.stripe_url("/refunds"))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::PaymentFailed(format!("Stripe refund failed: {}", e)))?;

        if !response.status().is_success() {
            let error_body: serde_json::Value = response
                .json()
                .await
                .unwrap_or_else(|_| serde_json::json!({"error": {"message": "Unknown error"}}));
            let msg = error_body["error"]["message"]
                .as_str()
                .unwrap_or("Refund failed")
                .to_string();
            return Err(AppError::PaymentFailed(msg));
        }

        let refund: serde_json::Value = response.json().await.map_err(|e| {
            AppError::PaymentFailed(format!("Failed to parse refund response: {}", e))
        })?;

        Ok(RefundResult {
            provider_refund_id: refund["id"].as_str().unwrap_or("unknown").to_string(),
            status: refund["status"].as_str().unwrap_or("unknown").to_string(),
            amount: refund["amount"].as_i64().unwrap_or(0),
        })
    }

    async fn get_status(&self, provider_payment_id: &str) -> AppResult<String> {
        let url = self.stripe_url(&format!("/payment_intents/{}", provider_payment_id));
        let response = self
            .client
            .get(&url)
            .basic_auth(&self.api_key, Option::<&str>::None)
            .send()
            .await
            .map_err(|e| AppError::PaymentFailed(format!("Stripe status request failed: {}", e)))?;

        let pi: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::PaymentFailed(e.to_string()))?;

        Ok(pi["status"].as_str().unwrap_or("unknown").to_string())
    }
}
