use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;

pub async fn establish_connection(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    Ok(MySqlPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
        .expect("Failed to create pool"))
}
