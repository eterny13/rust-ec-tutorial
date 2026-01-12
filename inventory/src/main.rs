mod domain;
mod service;
mod datasource;

use std::sync::Arc;

use datasource::create_pool;
use datasource::inventory::inventory_repository_db::InventoryRepositoryDb;
use datasource::kafka::kafka_publisher::KafkaEventPublisher;
use datasource::kafka::kafka_consumer::KafkaEventConsumer;
use service::inventory_service::InventoryService;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
      .expect("DATABASE_URL must be set");
    let kafka_brokers = std::env::var("KAFKA_BROKERS")
      .unwrap_or_else(|_| "localhost:9092".to_string());

    // create db connection pool
    let pool = create_pool(&database_url)
      .await
      .expect("Failed to create pool");

    let repository = Arc::new(InventoryRepositoryDb::new(pool));
    let service = Arc::new(InventoryService::new(repository));
    let publisher = Arc::new(KafkaEventPublisher::new(&kafka_brokers, "inventory-events"));

    let consumer = KafkaEventConsumer::new(
        &kafka_brokers,
        "inventory-service",
        &["order-events", "payment-events"],
    );

    tracing::info!("Starting Inventory Service - Kafka Consumer");

    // start event loop
    consumer.start(service, publisher).await;

    Ok(())
}
