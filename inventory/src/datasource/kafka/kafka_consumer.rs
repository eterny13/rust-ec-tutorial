use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use futures::StreamExt;
use std::sync::Arc;
use crate::datasource::kafka::kafka_publisher::KafkaEventPublisher;
use crate::domain::order::event::order_event::OrderEvent;
use crate::domain::payment::event::payment_event::PaymentEvent;
use crate::domain::inventory::event::inventory_event::InventoryEvent;
use crate::domain::product::ProductId;
use crate::service::inventory_repository::InventoryRepository;
use crate::service::inventory_service::InventoryService;

pub struct KafkaEventConsumer {
  consumer: StreamConsumer,
}

impl KafkaEventConsumer {
  pub fn new(brokers: &str, group_id: &str, topics: &[&str]) -> Self {
    let consumer: StreamConsumer = ClientConfig::new()
      .set("group.id", group_id)
      .set("bootstrap.servers", brokers)
      .set("enable.auto.commit", "true")
      .set("auto.offset.reset", "earliest")
      .create()
      .expect("Consumer creation failed");

    consumer
      .subscribe(topics)
      .expect("Failed to subscribe to topics");

    Self { consumer }
  }

  pub async fn start<R: InventoryRepository>(
    &self,
    inventory_service: Arc<InventoryService<R>>,
    event_publisher: Arc<KafkaEventPublisher>,
  ) {
    let mut stream = self.consumer.stream();

    while let Some(result) = stream.next().await {
      match result {
        Ok(message) => {
          if let Some(payload) = message.payload() {
            let payload_str = String::from_utf8_lossy(payload);

            match message.topic() {
              "order-events" => {
                self.handle_order_event(
                  &payload_str,
                  &inventory_service,
                  &event_publisher,
                ).await;
              }
              "payment-events" => {
                self.handle_payment_event(
                  &payload_str,
                  &inventory_service,
                  &event_publisher,
                ).await;
              }
              _ => {
                tracing::warn!("Unknown topic: {}", message.topic());
              }
            }
          }
        }
        Err(e) => {
          tracing::error!("Kafka error: {}", e);
        }
      }
    }
  }

  async fn handle_order_event<R: InventoryRepository>(
    &self,
    payload: &str,
    service: &Arc<InventoryService<R>>,
    publisher: &Arc<KafkaEventPublisher>,
  ) {
    match serde_json::from_str::<OrderEvent>(payload) {
      Ok(OrderEvent::OrderCreated { order_id, product_id: product_id_str, quantity, .. }) => {
        tracing::info!("Processing OrderCreated: {}", order_id);

        let product_id = ProductId(product_id_str);
        match service.reserve_inventory(&product_id, quantity).await {
          Ok(_) => {
            let event = InventoryEvent::InventoryReserved {
              order_id,
              product_id: product_id.to_string(),
              quantity,
              reserved_at: chrono::Utc::now(),
            };
            if let Err(e) = publisher.publish(&event).await {
              tracing::error!("Failed to publish InventoryReserved: {}", e);
            }
          }
          Err(e) => {
            let event = InventoryEvent::InventoryFailed {
              order_id,
              product_id: product_id.to_string(),
              reason: e.to_string(),
              failed_at: chrono::Utc::now(),
            };
            if let Err(e) = publisher.publish(&event).await {
              tracing::error!("Failed to publish InventoryFailed: {}", e);
            }
          }
        }
      }
      Ok(OrderEvent::OrderCancelled { order_id, product_id: product_id_str, quantity }) => {
        tracing::info!("Processing OrderCancelled: {}", order_id);
        let product_id = ProductId(product_id_str);
        match service.release_inventory(&product_id, quantity).await {
            Ok(_) => tracing::info!("Inventory released for details in order: {}", order_id),
            Err(e) => tracing::error!("Failed to release inventory: {}", e),
        }
      }
      Err(e) => {
        tracing::error!("Failed to parse OrderEvent: {}", e);
      }
    }
  }

  async fn handle_payment_event<R: InventoryRepository>(
    &self,
    payload: &str,
    service: &Arc<InventoryService<R>>,
    publisher: &Arc<KafkaEventPublisher>,
  ) {
    match serde_json::from_str::<PaymentEvent>(payload) {
      Ok(PaymentEvent::PaymentCompleted { order_id, product_id: product_id_str, quantity, .. }) => {
        tracing::info!("Processing PaymentCompleted: {}", order_id);
        let product_id = ProductId(product_id_str);
         match service.confirm_inventory(&product_id, quantity).await {
             Ok(_) => tracing::info!("Inventory confirmed for order: {}", order_id),
             Err(e) => tracing::error!("Failed to confirm inventory: {}", e),
         }
      }
      Ok(PaymentEvent::PaymentFailed { order_id, reason, product_id: product_id_str, quantity }) => {
        tracing::info!("Processing PaymentFailed: {} - {}", order_id, reason);
        let product_id = ProductId(product_id_str);
        match service.release_inventory(&product_id, quantity).await {
            Ok(_) => tracing::info!("Inventory released for failed payment: {}", order_id),
            Err(e) => tracing::error!("Failed to release inventory: {}", e),
        }
      }
      Err(e) => {
        tracing::error!("Failed to parse PaymentEvent: {}", e);
      }
    }
  }
}
