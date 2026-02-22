use actix_web::{web, App, HttpServer};

use crate::datasource::payment_gateway_impl::PaymentGatewayImpl;
use crate::datasource::{establish_connection, KafkaEventPublisher, PaymentRepositoryDb};
use crate::service::payment_service::PaymentService;
use controller::payment_controller;
use std::sync::Arc;

mod controller;
mod datasource;
mod domain;
mod service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let kafka_brokers =
        std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());

    // create db connection pool
    let pool = establish_connection(&database_url)
        .await
        .expect("Failed to create pool");

    let repository = Arc::new(PaymentRepositoryDb::new(pool));
    // Implementation of PaymentRepository is required for PaymentService
    // We need to clone repository for the service and consumer? 
    // Actually PaymentService takes Arc<R>, so we can clone the Arc<PaymentRepositoryDb> if needed, 
    // or just pass the repository struct if it implements Clone (which it likely doesn't if it holds a pool, but pool is cloneable).
    // PaymentRepositoryDb holds a Pool which is cloneable.
    
    // PaymentGatewayImpl
    let gateway = Arc::new(PaymentGatewayImpl::new());
    
    // Publisher for Payment events
    let publisher = Arc::new(KafkaEventPublisher::new(&kafka_brokers, "payment-events"));
    
    let service = Arc::new(PaymentService::new(repository, gateway, publisher.clone()));

    // Consumer for Order events (Saga orchestration)
    let consumer = crate::datasource::KafkaEventConsumer::new(
        &kafka_brokers,
        "payment-service-group",
        &["order-events"],
    );

    let service_clone = service.clone();
    let publisher_clone = publisher.clone();
    tokio::spawn(async move {
        tracing::info!("Starting Kafka Consumer...");
        consumer.start(service_clone, publisher_clone).await;
    });

    tracing::info!("Starting Payment Service on 0.0.0.0:8081");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(service.clone())) // Use from to avoid double wrapping if service is already Arc
            .service(web::scope("").route(
                "/payments",
                web::post().to(payment_controller::process_payment::<
                    PaymentRepositoryDb,
                    PaymentGatewayImpl,
                >),
            ))
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}
