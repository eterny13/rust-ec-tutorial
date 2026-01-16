pub mod kafka_event_publisher;
pub mod payment_gateway_impl;
pub mod payment_record;
pub mod payment_repository_db;

pub use kafka_event_publisher::{KafkaEventPublishError, KafkaEventPublisher};
pub use payment_repository_db::PaymentRepositoryDb;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

pub async fn establish_connection(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}
