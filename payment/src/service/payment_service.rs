use crate::datasource::{KafkaEventPublishError, KafkaEventPublisher};
use crate::domain::payment::Payment;
use crate::domain::payment_event::PaymentEvent;
use crate::service::payment_gateway::{PaymentGateway, PaymentGatewayError, PaymentMetadata};
use crate::service::payment_repository::{PaymentRepository, PaymentRepositoryError};
use std::sync::Arc;

pub struct PaymentService<R: PaymentRepository, G: PaymentGateway> {
    repository: Arc<R>,
    gateway: Arc<G>,
    event_publisher: Arc<KafkaEventPublisher>,
}

impl<R: PaymentRepository, G: PaymentGateway> PaymentService<R, G> {
    pub fn new(
        repository: Arc<R>,
        gateway: Arc<G>,
        event_publisher: Arc<KafkaEventPublisher>,
    ) -> Self {
        Self {
            repository,
            gateway,
            event_publisher,
        }
    }

    /// 決済を処理し、結果に応じてイベントを発行
    pub async fn process_payment(
        &self,
        order_id: String,
        amount: u64,
        customer_id: String,
    ) -> Result<Payment, PaymentServiceError> {
        // 1. 決済エンティティを作成
        let mut payment = Payment::new(order_id.clone(), amount);
        self.repository.save(&payment).await?;

        // 2. 外部決済APIを呼び出し
        let metadata = PaymentMetadata {
            order_id: order_id.clone(),
            customer_id,
        };

        match self.gateway.process_payment(amount, "JPY", metadata).await {
            Ok(response) => {
                // 3a. 決済成功
                payment.mark_as_completed(response.transaction_id.clone())?;
                self.repository.save(&payment).await?;

                // 4a. 成功イベントを発行
                let event = PaymentEvent::PaymentCompleted {
                    order_id,
                    payment_id: payment.id.to_string(),
                    amount,
                    transaction_id: response.transaction_id,
                    completed_at: chrono::Utc::now(),
                };

                self.event_publisher
                    .publish(&event)
                    .await
                    .map_err(|e| PaymentServiceError::EventPublish(e))?;

                Ok(payment)
            }
            Err(e) => {
                // 3b. 決済失敗
                payment.mark_as_failed(e.to_string())?;
                self.repository.save(&payment).await?;

                // 4b. 失敗イベントを発行
                let event = PaymentEvent::PaymentFailed {
                    order_id,
                    payment_id: payment.id.to_string(),
                    reason: e.to_string(),
                    failed_at: chrono::Utc::now(),
                };
                self.event_publisher
                    .publish(&event)
                    .await
                    .map_err(|e| PaymentServiceError::EventPublish(e))?;

                Err(PaymentServiceError::Gateway(e))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentServiceError {
    #[error("Payment repository error: {0}")]
    Repository(#[from] PaymentRepositoryError),

    #[error("Payment gateway error: {0}")]
    Gateway(#[from] PaymentGatewayError),

    #[error("Event publish error: {0}")]
    EventPublish(#[from] KafkaEventPublishError),

    #[error("Payment domain error: {0}")]
    Domain(#[from] crate::domain::payment::PaymentError),
}
