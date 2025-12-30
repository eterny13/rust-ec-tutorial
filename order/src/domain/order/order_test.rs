#[cfg(test)]
mod tests {
  use crate::domain::product::{Product, ProductId};
  use crate::domain::order::{Order, OrderStatus, OrderId, OrderError};
  use crate::domain::customer::CustomerId;

  #[test]
  fn test_add_product_when_pending_payment() {
    let order = Order::new(CustomerId::new("customer-11"));

    assert!(matches!(order.status, OrderStatus::PendingPayment));
    assert!(order.products().is_empty());
  }

  #[test]
  fn test_add_product_success() {
    let mut order = Order { 
      id: OrderId::new("order-1"),
      customer_id: CustomerId::new("customer-1"),
      status: OrderStatus::PendingPayment,
      products: Vec::new(),
    };

    let product = Product {
      id: ProductId::new("product-1"),
      name: "Product 1".to_string(),
      price: 100,
      quantity: 2,
    };

    let actual = order.add_product(product.clone());
    assert!(actual.is_ok());
    assert_eq!(order.products, vec![product]);
    assert_eq!(order.total_amount(), 200);
  }

  #[test]
  fn test_add_product_when_paid() {
    let mut order = Order { 
      id: OrderId::new("order-1"),
      customer_id: CustomerId::new("customer-1"),
      status: OrderStatus::PendingPayment,
      products: Vec::new(),
    };

    order.mark_as_paid().unwrap();

    let product = Product {
      id: ProductId::new("product-1"),
      name: "Product 1".to_string(),
      price: 100,
      quantity: 2,
    };

    let actual = order.add_product(product);
    assert!(actual.is_err());
  }

  #[test]
  fn test_total_amount() {
    let mut order = Order { 
      id: OrderId::new("order-1"),
      customer_id: CustomerId::new("customer-1"),
      status: OrderStatus::PendingPayment,
      products: Vec::new(),
    };

    let product1 = Product::generate("p1", 500, 2);
    let product2 = Product::generate("p2", 300, 3);

    order.add_product(product1).unwrap();
    order.add_product(product2).unwrap();

    let actual = order.total_amount();

    assert_eq!(actual, 1900);
  }

  #[test]
  fn test_invalid_status_transition() {
    let mut order = Order { 
      id: OrderId::new("order-1"),
      customer_id: CustomerId::new("customer-1"),
      status: OrderStatus::Paid,
      products: Vec::new(),
    };

    let actual = order.mark_as_paid();
    assert!(actual.is_err());
    
    // verify error details
    let err = actual.unwrap_err();
    assert!(matches!(err, OrderError::InvalidStatusTransition { current, action } 
      if current == "Paid" && action == "mark_as_paid"));
  }
}
