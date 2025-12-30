use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
pub enum OrderEvent {
  OrderCreated {
    order_id: String,
    customer_id: String,
    created_at: DateTime<Utc>
  },
  OrderPaid {
    order_id: String,
    total_amount: u64,
    paid_at: DateTime<Utc>,
  }
}
