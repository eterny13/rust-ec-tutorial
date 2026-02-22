use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::inventory::inventory::Inventory;
use crate::domain::product::ProductId;
use crate::service::inventory_repository::InventoryRepository;
use crate::service::inventory_service::InventoryService;

#[derive(Deserialize)]
pub struct UpsertInventoryRequest {
    pub quantity: u32,
}

#[derive(Serialize)]
pub struct InventoryResponse {
    pub product_id: String,
    pub available_quantity: u32,
    pub reserved_quantity: u32,
}

impl From<Inventory> for InventoryResponse {
    fn from(inventory: Inventory) -> Self {
        Self {
            product_id: inventory.product_id.to_string(),
            available_quantity: inventory.available_quantity,
            reserved_quantity: inventory.reserved_quantity,
        }
    }
}

pub async fn upsert_inventory<R: InventoryRepository>(
    service: web::Data<Arc<InventoryService<R>>>,
    product_id: web::Path<String>,
    req: web::Json<UpsertInventoryRequest>,
) -> impl Responder {
    let product_id = ProductId(product_id.into_inner());

    match service
        .get_ref()
        .repository
        .find_by_product_id(&product_id)
        .await
    {
        Ok(Some(mut inventory)) => {
            inventory.available_quantity += req.quantity;
            match service.get_ref().repository.save(&inventory).await {
                Ok(_) => {
                    tracing::info!("Inventory updated: {:?}", inventory);
                    HttpResponse::Ok().json(InventoryResponse::from(inventory))
                }
                Err(e) => {
                    tracing::error!("Failed to update inventory: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to update inventory: {}", e)
                    }))
                }
            }
        }
        Ok(None) => {
            let inventory = Inventory::new(product_id, req.quantity, 0);
            match service.get_ref().repository.save(&inventory).await {
                Ok(_) => {
                    tracing::info!("Inventory created: {:?}", inventory);
                    HttpResponse::Created().json(InventoryResponse::from(inventory))
                }
                Err(e) => {
                    tracing::error!("Failed to create inventory: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to create inventory: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to find inventory: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to find inventory: {}", e)
            }))
        }
    }
}

pub async fn get_inventory<R: InventoryRepository>(
    service: web::Data<Arc<InventoryService<R>>>,
    product_id: web::Path<String>,
) -> impl Responder {
    let product_id = ProductId(product_id.into_inner());

    match service
        .get_ref()
        .repository
        .find_by_product_id(&product_id)
        .await
    {
        Ok(Some(inventory)) => HttpResponse::Ok().json(InventoryResponse::from(inventory)),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Inventory not found"
        })),
        Err(e) => {
            tracing::error!("Failed to find inventory: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to find inventory: {}", e)
            }))
        }
    }
}
