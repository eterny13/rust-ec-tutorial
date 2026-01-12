use async_trait::async_trait;
use crate::domain::inventory::Inventory;
use crate::domain::inventory::inventory_error::InventoryError;
use crate::domain::product::ProductId;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InventoryRepository: Send + Sync {
  async fn find_by_product_id(&self, product_id: &ProductId) -> Result<Option<Inventory>, InventoryError>;
  async fn save(&self, inventory: &Inventory) -> Result<(), InventoryError>;
}
