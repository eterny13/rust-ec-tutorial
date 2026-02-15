use actix_web::{web, App, HttpServer};
use std::sync::Arc;

mod controller;
mod datasource;
mod domain;
mod service;

use crate::datasource::kafka::kafka_publisher::KafkaEventPublisher;
use crate::datasource::kafka::order_event_consumer::OrderEventConsumer;
use controller::order_controller;
use datasource::connection_pool::establish_connection;
use datasource::order::order_repository_db::OrderRepositoryDb;
use service::order_service::OrderService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let kafka_brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());

    let pool = establish_connection(&database_url)
        .await
        .expect("Failed to create pool");

    let repository = Arc::new(OrderRepositoryDb::new(pool));
    let event_publisher = Arc::new(KafkaEventPublisher::new(
        &kafka_brokers,
        "order-events",
    ));
    let service = Arc::new(OrderService::new(repository.clone(), event_publisher.clone()));

    let consumer_repository = repository.clone();
    let consumer = OrderEventConsumer::new(&kafka_brokers, "order-service");
    let consumer_publisher = event_publisher.clone();
    tokio::spawn(async move {
        tracing::info!("Starting Order Event Consumer");
        consumer.start(consumer_repository, consumer_publisher).await;
    });

    tracing::info!("Starting Order Service on 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service.clone()))
            .service(
                web::scope("")
                    .route(
                        "/orders",
                        web::post().to(order_controller::create_order::<OrderRepositoryDb>),
                    )
                    .route(
                        "/orders/{id}",
                        web::get().to(order_controller::get_order::<OrderRepositoryDb>),
                    ),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
