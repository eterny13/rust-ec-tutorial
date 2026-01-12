use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentEvent {
  PaymentCompleted {
    order_id: String,
    payment_id: String,
    amount: u64,
    product_id: String,
    quantity: u32,
  },
  PaymentFailed {
    order_id: String,
    reason: String,
    product_id: String,
    quantity: u32,
  },
}
