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
      status: OrderStatus::AwaitingInventory,
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

  pub fn reserve_inventory(&mut self) -> Result<(), OrderError> {
    match &self.status {
      OrderStatus::AwaitingInventory => {
        self.status = OrderStatus::InventoryReserved;
        Ok(())
      }
      _ => Err(OrderError::InvalidStatusTransition {
        current: format!("{:?}", self.status),
        action: "reserve_inventory".to_string()
      })
    }
  }

  pub fn inventory_failed(&mut self, reason: String) -> Result<(), OrderError> {
    match &self.status {
      OrderStatus::AwaitingInventory => {
        self.status = OrderStatus::InventoryFailed(reason);
        Ok(())
      }
      _ => Err(OrderError::InvalidStatusTransition {
        current: format!("{:?}", self.status),
        action: "inventory_failed".to_string()
      })
    }
  }

  pub fn complete_payment(&mut self) -> Result<(), OrderError> {
    match &self.status {
      OrderStatus::InventoryReserved | OrderStatus::PendingPayment => {
        self.status = OrderStatus::Paid;
        Ok(())
      }
      _ => Err(OrderError::InvalidStatusTransition {
        current: format!("{:?}", self.status),
        action: "complete_payment".to_string()
      })
    }
  }

  pub fn fail_payment(&mut self, reason: String) -> Result<(), OrderError> {
    match &self.status {
      OrderStatus::InventoryReserved | OrderStatus::PendingPayment => {
        self.status = OrderStatus::PaymentFailed(reason);
        Ok(())
      }
      _ => Err(OrderError::InvalidStatusTransition {
        current: format!("{:?}", self.status),
        action: "fail_payment".to_string()
      })
    }
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
