use crate::domain::order::Order;
use crate::domain::order::OrderId;
use crate::domain::order::OrderRepositoryError;
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderRepositoryError {
    NotFound,
    Other(String),
}

#[async_trait]
pub trait OrderRepository {
    async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, OrderRepositoryError>;
    async fn save(&self, order: &Order) -> Result<(), OrderRepositoryError>;
    async fn find_by_customer_id(&self, customer_id: CustomerId) -> Result<Vec<Order>, OrderRepositoryError>;
}
