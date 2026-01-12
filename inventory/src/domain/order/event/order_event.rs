use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 注文サービスから受信するイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderEvent {
  OrderCreated {
    order_id: String,
    customer_id: String,
    product_id: String,
    quantity: u32,
    created_at: DateTime<Utc>,
  },
  OrderCancelled {
    order_id: String,
    product_id: String,
    quantity: u32,
  },
}
