use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct PaymentRecord {
    pub id: String,
    pub order_id: String,
    pub amount: u64, // sqlx should handle mapping if db supports it, otherwise i64 might be safer but Payment has u64.
    pub status: String,
    pub fail_reason: Option<String>,
    pub external_transaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
