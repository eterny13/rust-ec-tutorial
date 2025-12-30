#[cfg(test)]
mod tests {
    use actix_web::{test, App, web};
    use std::sync::Arc;
    use crate::service::order_repository::{MockOrderRepository, OrderRepositoryError};
    use crate::service::order_service::{OrderService, OrderServiceError};
    use crate::controller::order_controller::{create_order, get_order};
    use crate::domain::order::{Order, OrderId};
    use crate::domain::customer::CustomerId;

    #[actix_web::test]
    async fn test_create_order_endpoint() {
        let mut mock_repo = MockOrderRepository::new();
        
        // モックの挙動を設定
        mock_repo.expect_save()
            .times(1)
            .returning(|_| Ok(()));

        let service = Arc::new(OrderService::new(Arc::new(mock_repo)));
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(service))
                .route("/orders", web::post().to(create_order::<MockOrderRepository>))
        ).await;

        let req = test::TestRequest::post()
            .uri("/orders")
            .set_json(serde_json::json!({
                "customer_id": "customer-1",
                "product_id": "product-1",
                "quantity": 1,
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_create_order_endpoint_fail() {
        let mut mock_repo = MockOrderRepository::new();

        // モックの挙動を設定
        mock_repo.expect_save()
          .times(1)
          .returning(|_| Err(OrderRepositoryError::NotFound));

        let service = Arc::new(OrderService::new(Arc::new(mock_repo)));

        let app = test::init_service(
            App::new()
              .app_data(web::Data::new(service))
              .route("/orders", web::post().to(create_order::<MockOrderRepository>))
        ).await;

        let req = test::TestRequest::post()
          .uri("/orders")
          .set_json(serde_json::json!({
                "customer_id": "customer-1",
                "product_id": "product-1",
                "quantity": 1,
            }))
          .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_get_order_endpoint() {
        let mut mock_repo = MockOrderRepository::new();
        let order_id = OrderId::new("order-1".to_string());
        let customer_id = CustomerId::new("customer-1".to_string());
        let order = Order::new(customer_id);

        let returned_order = order.clone();
        mock_repo.expect_find_by_id()
            .with(mockall::predicate::eq(order_id.clone()))
            .times(1)
            .returning(move |_| Ok(Some(returned_order.clone())));

        let service = Arc::new(OrderService::new(Arc::new(mock_repo)));
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(service))
                .route("/orders/{id}", web::get().to(get_order::<MockOrderRepository>))
        ).await;

        let req = test::TestRequest::get()
            .uri("/orders/order-1")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["customer_id"], "customer-1");
    }

    #[actix_web::test]
    async fn test_get_order_not_found() {
        let mut mock_repo = MockOrderRepository::new();
        let order_id = OrderId::new("order-1".to_string());

        mock_repo.expect_find_by_id()
          .with(mockall::predicate::eq(order_id.clone()))
          .times(1)
          .returning(move |_| Ok(None));

        let service = Arc::new(OrderService::new(Arc::new(mock_repo)));

        let app = test::init_service(
            App::new()
              .app_data(web::Data::new(service))
              .route("/orders/{id}", web::get().to(get_order::<MockOrderRepository>))
        ).await;

        let req = test::TestRequest::get()
          .uri("/orders/order-1")
          .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_client_error());
    }
}
