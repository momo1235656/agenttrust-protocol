use async_trait::async_trait;

use crate::error::AppResult;

pub struct PaymentResult {
    pub provider_payment_id: String,
    pub status: String,
    pub amount: i64,
}

pub struct RefundResult {
    pub provider_refund_id: String,
    pub status: String,
    pub amount: i64,
}

/// Abstraction over payment providers (Stripe, PayPay, etc.)
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    /// Provider identifier string.
    fn name(&self) -> &str;

    /// ISO currency codes supported by this provider.
    fn supported_currencies(&self) -> Vec<String>;

    /// Execute a payment.
    async fn execute(
        &self,
        amount: i64,
        currency: &str,
        description: &str,
        idempotency_key: &str,
    ) -> AppResult<PaymentResult>;

    /// Refund a payment. If amount is None, refund the full amount.
    async fn refund(
        &self,
        provider_payment_id: &str,
        amount: Option<i64>,
        reason: &str,
    ) -> AppResult<RefundResult>;

    /// Get current status of a payment.
    async fn get_status(&self, provider_payment_id: &str) -> AppResult<String>;
}
