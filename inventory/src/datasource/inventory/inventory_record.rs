use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow)]
pub struct InventoryRecord {
  pub id: String,
  pub available_quantity: u32,
  pub reserved_quantity: u32,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}
