use async_trait::async_trait;
use sqlx::MySqlPool;
use crate::domain::order::{Order, OrderId, OrderStatus};
use crate::domain::customer::CustomerId;
use crate::datasource::order::order_record::OrderRecord;
use crate::service::order_repository::{OrderRepository, OrderRepositoryError};

#[derive(Debug, Clone)]
pub struct OrderRepositoryDb {
    pool: MySqlPool,
}

impl OrderRepositoryDb {
  pub fn new(pool: MySqlPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl OrderRepository for OrderRepositoryDb {
  async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, OrderRepositoryError> {
    let rec = sqlx::query_as::<_, OrderRecord>(
      r#"
      SELECT id, customer_id, status, total_amount, created_at, updated_at
      FROM orders 
      WHERE id = ?
      "#
    )
    .bind(id.0)
    .fetch_optional(&self.pool)
    .await
    .map_err(|_| OrderRepositoryError::Other("Failed to find order".to_string()))?;

    match rec {
      Some(rec) => {
        Ok(Some(Order {
          id: OrderId(rec.id),
          customer_id: CustomerId(rec.customer_id),
          status: OrderStatus::from(rec.status),
          products: vec![],
        }))
      }
      None => Ok(None),
    }
  }

  async fn save(&self, order: &Order) -> Result<(), OrderRepositoryError> {
    sqlx::query!(
      r#"
      INSERT INTO orders (id, customer_id, status, total_amount, created_at, updated_at)
      VALUES (?, ?, ?, ?, NOW(), NOW())
      ON DUPLICATE KEY UPDATE
        customer_id = VALUES(customer_id),
        status = VALUES(status),
        total_amount = VALUES(total_amount),
        updated_at = NOW()
      "#,
      order.id().0.as_str(),
      order.customer_id().0.as_str(),
      order.status.as_str(),
      order.total_amount() as i64
    )
    .execute(&self.pool)
    .await
    .map_err(|_| OrderRepositoryError::Other("Failed to save order".to_string()))?;

    Ok(())
  }

  async fn find_by_customer_id(&self, customer_id: CustomerId) -> Result<Vec<Order>, OrderRepositoryError> {
    let recs = sqlx::query_as::<_, OrderRecord>(
      r#"
      SELECT id, customer_id, status, total_amount, created_at, updated_at
      FROM orders 
      WHERE customer_id = ?
      "#,
    )
    .bind(customer_id.0)
    .fetch_all(&self.pool)
    .await
    .map_err(|_| OrderRepositoryError::Other("Failed to find orders".to_string()))?;

    let orders = recs.into_iter().map(|rec| {
      Order {
        id: OrderId(rec.id),
        customer_id: CustomerId(rec.customer_id),
        status: OrderStatus::from(rec.status),
        products: vec![],
      }
    }).collect();

    Ok(orders)
  }
}
