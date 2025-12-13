use crate::domain::Order;
use crate::domain::OrderId;
use crate::service::OrderRepository;
use crate::service::OrderRepositoryError;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct OrderRepositoryDb {
    pool: sqlx::MySqlPool,
}

#[async_trait]
impl OrderRepository for OrderRepositoryDb {
    async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, OrderRepositoryError> {
      let rec = sqlx::query!(
        r#"
        SELECT id, customer_id, status as "status: String" 
        FROM orders 
        WHERE id = ?
        "#,
        id.0,
      )
      .fetch_optional(&self.pool)
      .await
      .map_err(|_| OrderRepositoryError::Other("Failed to find order".to_string()))?;
    }

    if let Some(rec) = rec {
      Ok(Some(Order {
        id: OrderId(rec.id),
        customer_id: CustomerId(rec.customer_id),
        status: OrderStatus::from(rec.status),
        products: vec![],
      }))
    } else {
      Ok(None)
    }
}
