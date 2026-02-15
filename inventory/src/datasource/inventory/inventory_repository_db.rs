use async_trait::async_trait;
use sqlx::MySqlPool;
use crate::datasource::inventory::inventory_record::InventoryRecord;
use crate::domain::inventory::Inventory;
use crate::domain::inventory::inventory_error::InventoryError;
use crate::domain::product::ProductId;
use crate::service::inventory_repository::InventoryRepository;

#[derive(Debug, Clone)]
pub struct InventoryRepositoryDb {
    pool: MySqlPool,
}

impl InventoryRepositoryDb {
  pub fn new(pool: MySqlPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl InventoryRepository for InventoryRepositoryDb {
  async fn find_by_product_id(&self, product_id: &ProductId) -> Result<Option<Inventory>, InventoryError> {
    let rec = sqlx::query_as::<_, InventoryRecord>(
      r#"
      SELECT id, available_quantity, reserved_quantity, created_at, updated_at
      FROM inventories
      WHERE id = ?
      "#
    )
    .bind(product_id.0.clone())
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch inventory: {:?}", e);
        InventoryError::Infrastructure(format!("Database error: {}", e))
    })?;

    match rec {
      Some(rec) => {
        Ok(Some(
          Inventory::new(
            ProductId(rec.id),
            rec.available_quantity as u32,
            rec.reserved_quantity as u32,
          )
        ))
      }
      None => Ok(None),
    }
  }

  async fn save(&self, inventory: &Inventory) -> Result<(), InventoryError> {
    sqlx::query(
      r#"
      INSERT INTO inventories (id, available_quantity, reserved_quantity, created_at, updated_at)
      VALUES (?, ?, ?, NOW(), NOW())
      ON DUPLICATE KEY UPDATE
        available_quantity = VALUES(available_quantity),
        reserved_quantity = VALUES(reserved_quantity),
        updated_at = NOW()
      "#
    )
    .bind(&inventory.product_id.0)
    .bind(inventory.available_quantity as i32)
    .bind(inventory.reserved_quantity as i32)
    .execute(&self.pool)
    .await
    .map_err(|_| InventoryError::Infrastructure("Failed to save inventory".to_string()))?;
    Ok(())
  }
}
