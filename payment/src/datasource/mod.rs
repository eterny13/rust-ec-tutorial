pub mod kafka;
pub mod payment_gateway_impl;
pub mod payment_record;
pub mod payment_repository_db;

pub use crate::datasource::kafka::kafka_event_publisher::{
    KafkaEventPublishError, KafkaEventPublisher,
};
pub use crate::datasource::payment_repository_db::PaymentRepositoryDb;
pub use kafka::kafka_consumer::KafkaEventConsumer;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

pub async fn establish_connection(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}
