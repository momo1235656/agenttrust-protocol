use async_trait::async_trait;

use crate::error::{AppError, AppResult};
use crate::payment_providers::traits::{PaymentProvider, PaymentResult, RefundResult};

/// PayPay payment provider (stub implementation).
/// Full implementation requires PayPay SDK credentials.
pub struct PayPayProvider {
    api_key: String,
    api_secret: String,
    merchant_id: String,
    is_production: bool,
}

impl PayPayProvider {
    pub fn new(api_key: String, api_secret: String, merchant_id: String, is_production: bool) -> Self {
        Self {
            api_key,
            api_secret,
            merchant_id,
            is_production,
        }
    }

    fn base_url(&self) -> &str {
        if self.is_production {
            "https://api.paypay.ne.jp"
        } else {
            "https://stg.paypay.ne.jp"
        }
    }
}

#[async_trait]
impl PaymentProvider for PayPayProvider {
    fn name(&self) -> &str {
        "paypay"
    }

    fn supported_currencies(&self) -> Vec<String> {
        vec!["jpy".to_string()]
    }

    async fn execute(
        &self,
        amount: i64,
        currency: &str,
        description: &str,
        idempotency_key: &str,
    ) -> AppResult<PaymentResult> {
        if currency != "jpy" {
            return Err(AppError::PaymentFailed(
                "PayPay only supports JPY".to_string(),
            ));
        }

        // TODO: Implement actual PayPay API call
        // Reference: https://developer.paypay.ne.jp/
        // For now, return a stub response for integration testing
        tracing::warn!("PayPay provider is not fully implemented. Using stub response.");

        Ok(PaymentResult {
            provider_payment_id: format!("paypay_{}", idempotency_key),
            status: "succeeded".to_string(),
            amount,
        })
    }

    async fn refund(
        &self,
        provider_payment_id: &str,
        amount: Option<i64>,
        reason: &str,
    ) -> AppResult<RefundResult> {
        // TODO: Implement PayPay refund
        tracing::warn!("PayPay refund is not fully implemented. Using stub response.");
        Ok(RefundResult {
            provider_refund_id: format!("paypay_refund_{}", provider_payment_id),
            status: "succeeded".to_string(),
            amount: amount.unwrap_or(0),
        })
    }

    async fn get_status(&self, provider_payment_id: &str) -> AppResult<String> {
        // TODO: Implement PayPay status check
        Ok("succeeded".to_string())
    }
}
