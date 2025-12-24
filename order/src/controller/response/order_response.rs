use serde::Serialize;
use crate::domain::order::Order;

#[derive(Debug, Serialize)]
pub struct OrderResponse {
  pub id: String,
  pub customer_id: String,
  pub status: String,
  pub total_amount: u64,
  pub items: Vec<OrderProductResponse>,
}

#[derive(Debug, Serialize)]
pub struct OrderProductResponse {
  pub product_id: String,
  pub quantity: u32,
  pub unit_price: u64,
}

impl From<&Order> for OrderResponse {
  fn from(order: &Order) -> Self {
    Self {
      id: order.id().to_string(),
      customer_id: order.customer_id().to_string(),
      status: format!("{:?}", order.status()),
      total_amount: order.total_amount(),
      items: order.products().iter().map(| p | OrderProductResponse {
        product_id: p.id.to_string(),
        quantity: p.quantity,
        unit_price: p.price,
      }).collect(),
    }
  }
}
