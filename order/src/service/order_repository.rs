use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::domain::order::{Order, OrderId};
use crate::domain::customer::CustomerId;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum OrderRepositoryError {
    #[error("Order not found")]
    NotFound,
    #[error("Repository error: {0}")]
    Other(String),
}

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, OrderRepositoryError>;
    async fn save(&self, order: &Order) -> Result<(), OrderRepositoryError>;
    async fn find_by_customer_id(&self, customer_id: CustomerId) -> Result<Vec<Order>, OrderRepositoryError>;
}
