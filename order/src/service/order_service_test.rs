#[cfg(test)]
mod tests {
  use mockall::predicate::*;
  use std::sync::Arc;
  use crate::domain::order::{Order, OrderId, OrderStatus};
  use crate::domain::product::Product;
  use crate::service::order_service::OrderService;
  use crate::service::order_repository::{OrderRepository, OrderRepositoryError};
  use crate::domain::customer::CustomerId;

  mockall::mock! {
      pub OrderRepository {}

      #[async_trait::async_trait]
      impl OrderRepository for OrderRepository {
          async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, OrderRepositoryError>;
          async fn save(&self, order: &Order) -> Result<(), OrderRepositoryError>;
          async fn find_by_customer_id(&self, customer_id: CustomerId) -> Result<Vec<Order>, OrderRepositoryError>;
      }
  }

  #[tokio::test]
  async fn test_create_order_success() {
    let mut mock_repo = MockOrderRepository::new();
    let customer_id = CustomerId::new("customer-1");
    let expected_customer_id = customer_id.clone();

    mock_repo
      .expect_save()
      .withf(move |o| o.customer_id == expected_customer_id)
      .times(1)
      .returning(|_| Ok(()));

    let service = OrderService::new(Arc::new(mock_repo));
    let actual = service.create_order(customer_id.clone()).await;

    assert!(actual.is_ok());
    let order = actual.unwrap();
    assert_eq!(order.customer_id, customer_id);
    assert_eq!(order.status, OrderStatus::PendingPayment);
    assert!(order.products.is_empty());
  }

  #[tokio::test]
  async fn test_add_product_to_order_success() {
    let mut mock_repo = MockOrderRepository::new();
    let order_id = OrderId::new("order-1");
    let order = Order {
      id: order_id.clone(),
      customer_id: CustomerId::new("customer-1"),
      status: OrderStatus::PendingPayment,
      products: vec![],
    };
    let product = Product::generate("product-1", 100, 1);
    let expected_product = product.clone();

    mock_repo
      .expect_find_by_id()
      .with(eq(order_id.clone()))
      .times(1)
      .return_once(move |_| {
        Ok(Some(order))
      });

    mock_repo
      .expect_save()
      .withf(move |o| {
        o.products.contains(&expected_product)
      })
      .times(1)
      .returning(|_| Ok(()));

    let service = OrderService::new(Arc::new(mock_repo));
    let actual = service.add_product_to_order(order_id, product).await;

    assert!(actual.is_ok());
  }
}
