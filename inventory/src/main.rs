mod controller;
mod datasource;
mod domain;
mod service;

use actix_web::{web, App, HttpServer};
use std::sync::Arc;

use controller::inventory_controller;
use datasource::create_pool;
use datasource::inventory::inventory_repository_db::InventoryRepositoryDb;
use datasource::kafka::kafka_consumer::KafkaEventConsumer;
use datasource::kafka::kafka_publisher::KafkaEventPublisher;
use service::inventory_service::InventoryService;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let kafka_brokers =
        std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());

    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create pool");

    let repository = Arc::new(InventoryRepositoryDb::new(pool));
    let service = Arc::new(InventoryService::new(repository));
    let publisher = Arc::new(KafkaEventPublisher::new(&kafka_brokers, "inventory-events"));

    let consumer_service = service.clone();
    let consumer_publisher = publisher.clone();
    let consumer = KafkaEventConsumer::new(
        &kafka_brokers,
        "inventory-service",
        &["order-events", "payment-events"],
    );

    tokio::spawn(async move {
        tracing::info!("Starting Inventory Service - Kafka Consumer");
        consumer.start(consumer_service, consumer_publisher).await;
    });

    tracing::info!("Starting Inventory Service HTTP Server on 0.0.0.0:8082");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service.clone()))
            .service(
                web::scope("")
                    .route(
                        "/inventory/{product_id}",
                        web::post()
                            .to(inventory_controller::upsert_inventory::<InventoryRepositoryDb>),
                    )
                    .route(
                        "/inventory/{product_id}",
                        web::get().to(inventory_controller::get_inventory::<InventoryRepositoryDb>),
                    ),
            )
    })
    .bind("0.0.0.0:8082")?
    .run()
    .await
}
