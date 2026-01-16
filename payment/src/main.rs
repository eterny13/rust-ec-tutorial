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
    let gateway = Arc::new(PaymentGatewayImpl::new());
    let publisher = Arc::new(KafkaEventPublisher::new(&kafka_brokers, "inventory-events"));
    let service = Arc::new(PaymentService::new(repository, gateway, publisher));

    tracing::info!("Starting Inventory Service - Kafka Consumer");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service.clone()))
            .service(web::scope("").route(
                "/payments",
                web::post().to(payment_controller::process_payment::<
                    PaymentRepositoryDb,
                    PaymentGatewayImpl,
                >),
            ))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
