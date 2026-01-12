use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InventoryEvent {
  InventoryReserved {
    order_id: String,
    product_id: String,
    quantity: u32,
    reserved_at: DateTime<Utc>,
  },
  InventoryFailed {
    order_id: String,
    product_id: String,
    reason: String,
    failed_at: DateTime<Utc>,
  },
  InventoryReleased {
    order_id: String,
    product_id: String,
    quantity: u32,
    released_at: DateTime<Utc>,
  },
}


