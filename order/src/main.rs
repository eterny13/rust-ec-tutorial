use actix_web::{web, App, HttpServer};
use std::sync::Arc;

mod domain;
mod service;
mod datasource;
mod controller;

use datasource::connection_pool::establish_connection;
use datasource::order_repository_db::OrderRepositoryDb;
use service::order_service::OrderService;
use controller::order_controller;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  // 環境変数の読み込み
  dotenv::dotenv().ok();

  // ロガーの初期化
  tracing_subscriber::fmt::init();

  let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

  // Connection Pool の作成
  let pool = establish_connection(&database_url)
    .await
    .expect("Failed to create pool");

  // 依存関係の組み立て（DI）
  let repository = Arc::new(OrderRepositoryDb::new(pool));
  let service = Arc::new(OrderService::new(repository));

  tracing::info!("Starting Order Service on 0.0.0.0:8080");

  HttpServer::new(move || {
    App::new()
      .app_data(web::Data::new(service.clone()))
      .service(
        web::scope("")
          .route("/orders", web::post().to(order_controller::create_order::<OrderRepositoryDb>))
          .route("/orders/{id}", web::get().to(order_controller::get_order::<OrderRepositoryDb>))
      )
  })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
