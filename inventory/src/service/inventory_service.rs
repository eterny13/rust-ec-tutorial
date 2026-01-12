use std::sync::Arc;
use crate::domain::inventory::inventory_error::InventoryError;
use crate::domain::product::ProductId;
use crate::service::inventory_repository::InventoryRepository;

pub struct InventoryService<R: InventoryRepository> {
  repository: Arc<R>,
}

impl<R: InventoryRepository> InventoryService<R> {
  pub fn new(repository: Arc<R>) -> Self {
    Self { repository }
  }

  pub async fn reserve_inventory(
    &self,
    product_id: &ProductId,
    quantity: u32,
  ) -> Result<(), InventoryError> {
    let res = self.repository.find_by_product_id(product_id).await?;
    match res {
      Some(mut inventory) => {
        inventory.reserve(quantity)?;
        self.repository.save(&inventory).await?;
        Ok(())
      }
      None => {
        Err(InventoryError::ProductNotFound(product_id.to_string()))
      }
    }
  }

  pub async fn release_inventory(
    &self,
    product_id: &ProductId,
    quantity: u32,
  ) -> Result<(), InventoryError> {
    let res = self.repository.find_by_product_id(product_id).await?;
    match res {
      Some(mut inventory) => {
        inventory.release(quantity)?;
        self.repository.save(&inventory).await?;
        Ok(())
      }
      None => {
        Err(InventoryError::ProductNotFound(product_id.to_string()))
      }
    }
  }

  pub async fn confirm_inventory(
    &self,
    product_id: &ProductId,
    quantity: u32,
  ) -> Result<(), InventoryError> {
    let res = self.repository.find_by_product_id(product_id).await?;
    match res {
      Some(mut inventory) => {
        inventory.confirm(quantity)?;
        self.repository.save(&inventory).await?;
        Ok(())
      }
      None => {
        Err(InventoryError::ProductNotFound(product_id.to_string()))
      }
    }
  }
}
