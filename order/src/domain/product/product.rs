use crate::domain::product::ProductId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Product {
    pub id: ProductId,
    pub name: String,
    pub price: u64,
    pub quantity: u32,
}

impl Product {
    pub fn generate(name: impl Into<String>, price: u64, quantity: u32) -> Self {
        Self {
            id: ProductId::generate(),
            name: name.into(),
            price,
            quantity,
        }
    }

    pub fn new(id: ProductId, name: impl Into<String>, price: u64, quantity: u32) -> Self {
        Self {
            id,
            name: name.into(),
            price,
            quantity,
        }
    }

    pub fn subtotal(&self) -> u64 {
        self.price * self.quantity as u64
    }
}
