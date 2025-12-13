use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    PendingPayment,
    PaymentFailed(String),
    Paid,
    Shipped { tracking_id: String },
    Delivered,
    Cancelled,
}

impl OrderStatus {
  pub fn can_add_product(&self) -> bool {
    matches!(self, OrderStatus::PendingPayment)
  }

  pub fn can_cancel(&self) -> bool {
    matches!(self, OrderStatus::PendingPayment | OrderStatus::Paid)
  }
}
