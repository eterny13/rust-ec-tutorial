use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum OrderEvent {
    OrderCreated {
        order_id: String,
        customer_id: String,
        product_id: String,
        quantity: u32,
        created_at: DateTime<Utc>,
    },
    OrderInventoryReserved {
        order_id: String,
        customer_id: String,
        total_amount: u64,
        reserved_at: DateTime<Utc>,
    },
    OrderPaid {
        order_id: String,
        total_amount: u64,
        paid_at: DateTime<Utc>,
    },
}
