use thiserror::Error;

#[derive(Debug, Error)]
pub enum InventoryError {
  #[error("Insufficient stock")]
  InsufficientStock {
    product_id: String,
    requested: u32,
    available: u32,
  },

  #[error("Product not found: {0}")]
  ProductNotFound(String),

  #[error("Infrastructure error: {0}")]
  Infrastructure(String),
}
