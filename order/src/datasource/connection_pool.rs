use sqlx::mysql::MySqlPoolOptions;

pub async fn establish_connection(database_url: &str) -> sqlx::MySqlPool {
    MySqlPoolOptions::new()
        .max_connections(20) // 負荷に応じた適切な値に設定
        .connect(database_url)
        .await
        .expect("Failed to create pool")
}
