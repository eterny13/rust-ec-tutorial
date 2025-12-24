use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use crate::domain::order::OrderId;
use crate::service::order_repository::OrderRepository;
use crate::service::order_service::OrderService;
use super::request::order_request::{CreateOrderRequest};
use super::response::order_response::OrderResponse;

pub async fn create_order<R: OrderRepository + 'static>(
    service: web::Data<Arc<OrderService<R>>>,
    body: web::Json<CreateOrderRequest>,
) -> Result<HttpResponse> {
    let customer_id = body.customer_id.clone().into();

    match service.create_order(customer_id).await {
        Ok(order) => {
            let response: OrderResponse = (&order).into();
            Ok(HttpResponse::Created().json(response))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(format!("{}", e)))
        }
    }
}

pub async fn get_order<R: OrderRepository + 'static>(
    service: web::Data<Arc<OrderService<R>>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let order_id = OrderId::new(path.into_inner());

    match service.get_order(order_id).await {
        Ok(Some(order)) => {
            let response: OrderResponse = (&order).into();
            Ok(HttpResponse::Ok().json(response))
        }
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().json(format!("{}", e))),
    }
}
