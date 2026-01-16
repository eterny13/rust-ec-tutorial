use crate::datasource::payment_record::PaymentRecord;
use crate::domain::payment::{Payment, PaymentId, PaymentStatus};
use crate::service::payment_repository::{PaymentRepository, PaymentRepositoryError};
use async_trait::async_trait;
use sqlx::MySqlPool;

#[derive(Debug, Clone)]
pub struct PaymentRepositoryDb {
    pool: MySqlPool,
}

impl PaymentRepositoryDb {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentRepository for PaymentRepositoryDb {
    async fn save(&self, payment: &Payment) -> Result<(), PaymentRepositoryError> {
        let (status_str, fail_reason) = match &payment.status {
            PaymentStatus::Pending => ("pending", None),
            PaymentStatus::Completed => ("completed", None),
            PaymentStatus::Failed(reason) => ("failed", Some(reason.clone())),
            PaymentStatus::Refunded => ("refunded", None),
        };

        sqlx::query(
            r#"
            INSERT INTO payments (id, order_id, amount, status, fail_reason, external_transaction_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                status = VALUES(status),
                fail_reason = VALUES(fail_reason),
                external_transaction_id = VALUES(external_transaction_id),
                updated_at = VALUES(updated_at)
            "#
        )
        .bind(payment.id.0.clone())
        .bind(payment.order_id.clone())
        .bind(payment.amount)
        .bind(status_str)
        .bind(fail_reason)
        .bind(payment.external_transaction_id.clone())
        .bind(payment.created_at)
        .bind(payment.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| PaymentRepositoryError::Infrastructure(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &PaymentId) -> Result<Option<Payment>, PaymentRepositoryError> {
        let rec = sqlx::query_as::<_, PaymentRecord>(
            r#"
            SELECT id, order_id, amount, status, fail_reason, external_transaction_id, created_at, updated_at
            FROM payments
            WHERE id = ?
            "#
        )
        .bind(&id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PaymentRepositoryError::Infrastructure(e.to_string()))?;

        match rec {
            Some(rec) => {
                let status = match rec.status.as_str() {
                    "pending" => PaymentStatus::Pending,
                    "completed" => PaymentStatus::Completed,
                    "failed" => PaymentStatus::Failed(
                        rec.fail_reason
                            .unwrap_or_else(|| "Unknown error".to_string()),
                    ),
                    "refunded" => PaymentStatus::Refunded,
                    _ => {
                        return Err(PaymentRepositoryError::Infrastructure(format!(
                            "Invalid status in DB: {}",
                            rec.status
                        )))
                    }
                };

                let payment = Payment {
                    id: PaymentId(rec.id),
                    order_id: rec.order_id,
                    amount: rec.amount,
                    status,
                    external_transaction_id: rec.external_transaction_id,
                    created_at: rec.created_at,
                    updated_at: rec.updated_at,
                };
                Ok(Some(payment))
            }
            None => Ok(None),
        }
    }
}
