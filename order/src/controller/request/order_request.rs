use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: String,
    pub product_id: String,
    pub quantity: u32,
}

#[derive(Deserialize)]
pub struct OrderProductRequest {
    pub product_id: String,
    pub quantity: u32,
    pub unit_price: u64,
}
