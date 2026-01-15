use crate::domain::payment::Payment;
use crate::domain::payment::PaymentId;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum PaymentRepositoryError {
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),
    #[error("Not found")]
    NotFound,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn save(&self, payment: &Payment) -> Result<(), PaymentRepositoryError>;
    async fn find_by_id(&self, id: &PaymentId) -> Result<Option<Payment>, PaymentRepositoryError>;
}
