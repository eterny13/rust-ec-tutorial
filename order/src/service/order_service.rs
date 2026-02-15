use crate::datasource::kafka::kafka_publisher::KafkaEventPublisher;
use crate::domain::customer::CustomerId;
use crate::domain::order::event::OrderEvent;
use crate::domain::order::{Order, OrderError, OrderId};
use crate::domain::product::Product;
use crate::service::order_repository::{OrderRepository, OrderRepositoryError};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderServiceError {
    #[error("Order domain error: {0}")]
    Domain(#[from] OrderError),

    #[error("Repository error: {0}")]
    Repository(#[from] OrderRepositoryError),

    #[error("Order not found")]
    NotFound,

    #[error("Event publishing error: {0}")]
    EventPublishing(String),
}

pub struct OrderService<R: OrderRepository> {
    repository: Arc<R>,
    event_publisher: Arc<KafkaEventPublisher>,
}

impl<R: OrderRepository> OrderService<R> {
    pub fn new(repository: Arc<R>, event_publisher: Arc<KafkaEventPublisher>) -> Self {
        Self {
            repository,
            event_publisher,
        }
    }

    pub async fn create_order(&self, customer_id: CustomerId) -> Result<Order, OrderServiceError> {
        let order = Order::new(customer_id);
        self.repository.save(&order).await?;
        Ok(order)
    }

    /// 1 order: 1 product
    pub async fn create_order_with_product(
        &self,
        customer_id: CustomerId,
        product_id: crate::domain::product::ProductId,
        quantity: u32,
        unit_price: u64,
    ) -> Result<Order, OrderServiceError> {
        let mut order = Order::new(customer_id);
        // TODO: ProductName from product service
        let product = Product::new(product_id, "Product", unit_price, quantity);
        order.add_product(product)?;
        self.repository.save(&order).await?;

        let event = OrderEvent::OrderCreated {
            order_id: order.id.to_string(),
            customer_id: order.customer_id.to_string(),
            product_id: order.products[0].id.to_string(),
            quantity: order.products[0].quantity,
            created_at: chrono::Utc::now(),
        };

        self.event_publisher
            .publish(&event)
            .await
            .map_err(|e| OrderServiceError::EventPublishing(e))?;

        tracing::info!("Order created: {:?}", order);
        Ok(order)
    }

    pub async fn add_product_to_order(
        &self,
        order_id: OrderId,
        product: Product,
    ) -> Result<(), OrderServiceError> {
        let mut order = self
            .repository
            .find_by_id(order_id)
            .await?
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
