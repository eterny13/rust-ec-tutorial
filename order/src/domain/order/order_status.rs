use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    AwaitingInventory,
    InventoryReserved,
    InventoryFailed(String),
    PendingPayment,
    PaymentFailed(String),
    Paid,
    Shipped { tracking_id: String },
    Delivered,
    Cancelled,
}

impl OrderStatus {
    pub fn from(status: String) -> Self {
        match status.as_str() {
            "AwaitingInventory" => OrderStatus::AwaitingInventory,
            "InventoryReserved" => OrderStatus::InventoryReserved,
            "InventoryFailed" => OrderStatus::InventoryFailed(String::new()),
            "PendingPayment" => OrderStatus::PendingPayment,
            "PaymentFailed" => OrderStatus::PaymentFailed(String::new()),
            "Paid" => OrderStatus::Paid,
            "Shipped" => OrderStatus::Shipped {
                tracking_id: String::new(),
            },
            "Delivered" => OrderStatus::Delivered,
            "Cancelled" => OrderStatus::Cancelled,
            _ => panic!("Invalid order status: {}", status),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::AwaitingInventory => "AwaitingInventory",
            OrderStatus::InventoryReserved => "InventoryReserved",
            OrderStatus::InventoryFailed(_) => "InventoryFailed",
            OrderStatus::PendingPayment => "PendingPayment",
            OrderStatus::PaymentFailed(_) => "PaymentFailed",
            OrderStatus::Paid => "Paid",
            OrderStatus::Shipped { .. } => "Shipped",
            OrderStatus::Delivered => "Delivered",
            OrderStatus::Cancelled => "Cancelled",
        }
    }

    pub fn can_add_product(&self) -> bool {
        matches!(
            self,
            OrderStatus::AwaitingInventory
                | OrderStatus::InventoryReserved
                | OrderStatus::PendingPayment
        )
    }

    pub fn can_cancel(&self) -> bool {
        matches!(
            self,
            OrderStatus::AwaitingInventory
                | OrderStatus::InventoryReserved
                | OrderStatus::InventoryFailed(_)
                | OrderStatus::PendingPayment
                | OrderStatus::PaymentFailed(_)
                | OrderStatus::Paid
        )
    }
}
