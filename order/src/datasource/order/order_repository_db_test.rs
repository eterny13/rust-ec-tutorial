#[cfg(test)]

mod tests {
  use sqlx::MySqlPool;
  use crate::domain::order::{Order, OrderStatus, OrderId};
  use crate::domain::customer::CustomerId;
  use crate::datasource::order_repository_db::OrderRepositoryDb;
  use crate::service::order_repository::OrderRepository;

  async fn get_test_pool() -> MySqlPool {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MySqlPool::connect(&database_url).await.unwrap()
  }

  #[tokio::test]
  async fn test_save_and_find_order() {
    let pool = get_test_pool().await;
    let repo = OrderRepositoryDb::new(pool);

    let order = Order {
      id: OrderId::new("order-123"),
      customer_id: CustomerId::new("customer-456"),
      status: OrderStatus::PendingPayment,
      products: vec![],
    };

    // Save the order
    repo.save(&order).await.unwrap();

    // Retrieve the order
    let fetched_order = repo.find_by_id(order.id.clone()).await.unwrap();
    assert!(fetched_order.is_some());
    let fetched_order = fetched_order.unwrap();
    assert_eq!(fetched_order, order);
  }
}
