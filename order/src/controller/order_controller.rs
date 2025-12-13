use actix_web::{web, App, HttpServer};
use sqlx::mysql::MySqlPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 環境変数の読み込みやロガーの初期化
    dotenv::dotenv().ok();
    env_logger::init();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to DB");
    HttpServer::new(move || {
        App::new()
            // DBプールをアプリケーションステートとして登録
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api/v1")
                    .route("/orders", web::post().to(create_order))
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn create_order() -> impl actix_web::Responder {
    "Under Construction"
}
