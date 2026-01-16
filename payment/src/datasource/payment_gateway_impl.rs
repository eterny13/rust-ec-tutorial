use crate::service::payment_gateway::{
    PaymentGateway, PaymentGatewayError, PaymentGatewayResponse, PaymentMetadata, RefundResponse,
};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct PaymentGatewayImpl {}

impl PaymentGatewayImpl {
    // コンストラクタ
    pub fn new() -> Self {
        PaymentGatewayImpl {}
    }
}

#[async_trait]
impl PaymentGateway for PaymentGatewayImpl {
    async fn process_payment(
        &self,
        amount: u64,
        currency: &str,
        metadata: PaymentMetadata,
    ) -> Result<PaymentGatewayResponse, PaymentGatewayError> {
        Ok(PaymentGatewayResponse {
            transaction_id: uuid::Uuid::new_v4().to_string(),
            status: "success".to_string(),
        })
    }

    async fn refund(
        &self,
        transaction_id: &str,
        amount: u64,
    ) -> Result<RefundResponse, PaymentGatewayError> {
        Ok(RefundResponse {
            refund_id: uuid::Uuid::new_v4().to_string(),
            transaction_id: transaction_id.to_string(),
            status: "refunded".to_string(),
        })
    }
}
