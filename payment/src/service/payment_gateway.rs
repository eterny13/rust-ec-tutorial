use async_trait::async_trait;

/// 外部決済サービスとの連携を抽象化
#[async_trait]
pub trait PaymentGateway: Send + Sync {
    async fn process_payment(
        &self,
        amount: u64,
        currency: &str,
        metadata: PaymentMetadata,
    ) -> Result<PaymentGatewayResponse, PaymentGatewayError>;

    async fn refund(
        &self,
        transaction_id: &str,
        amount: u64,
    ) -> Result<RefundResponse, PaymentGatewayError>;
}

#[derive(Debug)]
pub struct PaymentMetadata {
    pub order_id: String,
    pub customer_id: String,
}

#[derive(Debug)]
pub struct PaymentGatewayResponse {
    pub transaction_id: String,
    pub status: String,
}

#[derive(Debug)]
pub struct RefundResponse {
    pub refund_id: String,
    pub transaction_id: String,
    pub status: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentGatewayError {
    #[error("Payment declined: {0}")]
    Declined(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}
