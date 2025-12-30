use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow)]
pub struct OrderRecord {
  pub id: String,
  pub customer_id: String,
  pub status: String,
  pub total_amount: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct OrderProductRecord {
  pub id: String,
  pub order_id: String,
  pub product_id: String,
  pub quntity: i32,
  pub unit_price: i64,
}
