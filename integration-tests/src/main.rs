use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize)]
struct CreateOrderRequest {
    customer_id: String,
    product_id: String,
    quantity: u32,
    unit_price: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderResponse {
    id: String,
    status: String,
}

const ORDER_SERVICE_URL: &str = "http://localhost:8080";

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Integration Tests...");

    let client = reqwest::Client::new();

    // 1. Create Order
    println!("📝 Creating Order...");
    let create_order_req = CreateOrderRequest {
        customer_id: "user_123".to_string(),
        product_id: "prod_456".to_string(), // Ensure this product exists in Inventory DB!
        quantity: 1,
        unit_price: 1000,
    };

    let res = client
        .post(format!("{}/orders", ORDER_SERVICE_URL))
        .json(&create_order_req)
        .send()
        .await
        .context("Failed to send create order request")?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await?;
        println!("❌ Create Order Failed: {} - {}", status, body);
        return Err(anyhow::anyhow!("Create Order Failed"));
    }

    let order_res: OrderResponse = res.json().await?;
    println!("✅ Order Created: ID={}, Status={}", order_res.id, order_res.status);

    let order_id = order_res.id;

    // 2. Poll Status
    println!("⏳ Polling Order Status for completion...");
    let max_retries = 20;
    let mut final_status = String::new();

    for i in 0..max_retries {
        let res = client
            .get(format!("{}/orders/{}", ORDER_SERVICE_URL, order_id))
            .send()
            .await
            .context("Failed to get order details")?;
        
        if res.status().is_success() {
            let order: OrderResponse = res.json().await?;
            println!("   Retry {}: Status={}", i + 1, order.status);
            
            if order.status == "Paid" {
                println!("✅ Order Processed Successfully! Final Status: Paid");
                final_status = order.status;
                break;
            } else if order.status.contains("Failed") {
                println!("❌ Order Processing Failed! Final Status: {}", order.status);
                final_status = order.status;
                break;
            }
        }
        
        sleep(Duration::from_secs(1)).await;
    }

    if final_status == "Paid" {
        println!("🎉 Integration Test Passed!");
        Ok(())
    } else {
        println!("💥 Integration Test Failed. Final Status: {}", final_status);
        Err(anyhow::anyhow!("Test Failed"))
    }
}
