use crate::service::payment_gateway::PaymentGateway;
use crate::service::payment_repository::PaymentRepository;
use crate::service::payment_service::PaymentService;
use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ProcessPaymentRequest {
    pub order_id: String,
    pub amount: u64,
    pub customer_id: String,
}

pub async fn process_payment<R: PaymentRepository + 'static, G: PaymentGateway + 'static>(
    service: web::Data<Arc<PaymentService<R, G>>>,
    body: web::Json<ProcessPaymentRequest>,
) -> Result<HttpResponse> {
    match service
        .process_payment(body.order_id.clone(), body.amount, body.customer_id.clone())
        .await
    {
        Ok(payment) => Ok(HttpResponse::Ok().json(serde_json::json!({
                "payment_id": payment.id.to_string(),
                "status": format!("{:?}", payment.status),
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": e.to_string(),
        }))),
    }
}
