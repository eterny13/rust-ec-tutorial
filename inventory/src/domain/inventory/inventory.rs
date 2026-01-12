use serde::{Deserialize, Serialize};
use crate::domain::inventory::inventory_error::InventoryError;
use crate::domain::product::ProductId;

#[derive(Serialize, Deserialize)]
pub struct Inventory {
  pub product_id: ProductId,
  pub available_quantity: u32,
  pub reserved_quantity: u32,
}

impl Inventory {
  pub fn new(product_id: ProductId, available_quantity: u32, reserved_quantity: u32) -> Self {
    Self {
      product_id,
      available_quantity,
      reserved_quantity
    }
  }

  pub fn reserve(&mut self, quantity: u32) -> Result<(), InventoryError> {
    if self.available_quantity >= quantity {
      self.reserved_quantity += quantity;
      self.available_quantity -= quantity;
      Ok(())
    } else {
      let product_id = self.product_id.to_string();
      let requested = self.reserved_quantity;
      let available = self.available_quantity;

      Err(InventoryError::InsufficientStock {
        product_id,
        requested,
        available
      })
    }
  }

  pub fn release(&mut self, quantity: u32) -> Result<(), InventoryError> {
    if self.reserved_quantity >= quantity {
      self.available_quantity += quantity; 
      self.reserved_quantity -= quantity;
      Ok(())
    } else {
      let product_id = self.product_id.to_string();
      let requested = self.reserved_quantity;
      let available = self.available_quantity;

      Err(InventoryError::InsufficientStock {
        product_id,
        requested,
        available
      })
    }
  }

  pub fn confirm(&mut self, quantity: u32) -> Result<(), InventoryError> {
    if self.reserved_quantity >= quantity {
      self.reserved_quantity -= quantity;
      Ok(())
    } else {
      let product_id = self.product_id.to_string();
      let requested = self.reserved_quantity;
      let available = self.available_quantity;

      Err(InventoryError::InsufficientStock {
        product_id,
        requested,
        available
      })
    }
  }
}
