use std::sync::Arc;
use crate::domain::order::{Order, OrderId, OrderError};
use crate::service::order_repository::{OrderRepository, OrderRepositoryError};
use crate::domain::product::Product;
use crate::domain::customer::CustomerId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderServiceError {
    #[error("Order domain error: {0}")]
    Domain(#[from] OrderError),

    #[error("Repository error: {0}")]
    Repository(#[from] OrderRepositoryError),

    #[error("Order not found")]
    NotFound,
}

pub struct OrderService<R: OrderRepository> {
    repository: Arc<R>,
}

impl<R: OrderRepository> OrderService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn create_order(&self, customer_id: CustomerId) -> Result<Order, OrderServiceError> {
        let order = Order::new(customer_id);
        self.repository.save(&order).await?;
        Ok(order)
    }

    pub async fn add_product_to_order(&self, order_id: OrderId, product: Product) -> Result<(), OrderServiceError> {
        let mut order = self.repository
          .find_by_id(order_id).await?
          .ok_or(OrderServiceError::NotFound)?;
        order.add_product(product)?;
        self.repository.save(&order).await?;
        Ok(())
    }

    pub async fn get_order(&self, order_id: OrderId) -> Result<Option<Order>, OrderServiceError> {
        let order = self.repository.find_by_id(order_id).await?;
        Ok(order)
    }
}
