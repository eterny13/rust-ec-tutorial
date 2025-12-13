use serde::{Deserialize, Serialize};
use crate::domain::order::{OrderId, OrderStatus, OrderError};
use crate::domain::customer::CustomerId;
use crate::domain::product::Product;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub customer_id: CustomerId,
    pub status: OrderStatus,
    pub products: Vec<Product>,
}

impl Order {
  pub fn new(customer_id: CustomerId) -> Self {
    Self {
      id: OrderId::generate(),
      customer_id,
      products: Vec::new(),
      status: OrderStatus::PendingPayment,
    }
  }

  pub fn add_product(&mut self, product: Product) -> Result<(), OrderError> {
    if !self.status.can_add_product() {
      return Err(OrderError::InvalidStatusTransition {
        current: format!("{:?}", self.status),
        action: "add_product".to_string()
      })
    }
    self.products.push(product);
    Ok(())
  }

  pub fn total_amount(&self) -> u64 {
    self.products.iter().map(|p| p.subtotal()).sum()
  }

  pub fn mark_as_paid(&mut self) -> Result<(), OrderError> {
    match &self.status {
      OrderStatus::PendingPayment => {
        self.status = OrderStatus::Paid;
      }
      _ => {
        return Err(OrderError::InvalidStatusTransition {
          current: format!("{:?}", self.status),
          action: "mark_as_paid".to_string()
        })
      }
    }
    Ok(())
  }

  pub fn id(&self) -> &OrderId {
    &self.id
  }

  pub fn customer_id(&self) -> &CustomerId {
    &self.customer_id
  }

  pub fn status(&self) -> &OrderStatus {
    &self.status
  }

  pub fn products(&self) -> &Vec<Product> {
    &self.products
  }
}
