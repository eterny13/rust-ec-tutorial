use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed(String),
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentId(pub String);

impl PaymentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for PaymentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum PaymentError {
    #[error("Invalid state transition")]
    InvalidStateTransition,
    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: PaymentId,
    pub order_id: String,
    pub amount: u64,
    pub status: PaymentStatus,
    pub external_transaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Payment {
    pub fn new(order_id: String, amount: u64) -> Self {
        let now = Utc::now();
        Self {
            id: PaymentId::generate(),
            order_id,
            amount,
            status: PaymentStatus::Pending,
            external_transaction_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn mark_as_completed(&mut self, transaction_id: String) -> Result<(), PaymentError> {
        if self.status == PaymentStatus::Pending {
            self.status = PaymentStatus::Completed;
            self.external_transaction_id = Some(transaction_id);
            Ok(())
        } else {
            Err(PaymentError::InvalidStateTransition)
        }
    }

    pub fn mark_as_failed(&mut self, reason: String) -> Result<(), PaymentError> {
        self.status = PaymentStatus::Failed(reason);
        Ok(())
    }
}
