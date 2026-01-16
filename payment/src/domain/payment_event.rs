use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 決済サービスが発行するイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentEvent {
    PaymentCompleted {
        order_id: String,
        payment_id: String,
        amount: u64,
        transaction_id: String,
        completed_at: DateTime<Utc>,
    },
    PaymentFailed {
        order_id: String,
        payment_id: String,
        reason: String,
        failed_at: DateTime<Utc>,
    },
    PaymentRefunded {
        order_id: String,
        payment_id: String,
        amount: u64,
        refunded_at: DateTime<Utc>,
    },
}
