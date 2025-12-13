use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: String,
    pub items: Vec<OrderItemRequest>,
}

#[derive(Deserialize)]
pub struct OrderItemRequest {
    pub product_id: String,
    pub quantity: u32,
}
