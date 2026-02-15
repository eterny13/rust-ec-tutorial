use crate::domain::order::{Order, OrderError, OrderId};
use crate::service::order_repository::OrderRepository;
use futures::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum IncomingEvent {
    InventoryReserved {
        order_id: String,
        product_id: String,
    },
    InventoryFailed {
        order_id: String,
        reason: String,
    },
    PaymentCompleted {
        order_id: String,
        payment_id: String,
    },
    PaymentFailed {
        order_id: String,
        reason: String,
    },
}

pub struct OrderEventConsumer {
    consumer: StreamConsumer,
}

impl OrderEventConsumer {
    pub fn new(brokers: &str, group_id: &str) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", brokers)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()
            .expect("Consumer creation failed");

        consumer
            .subscribe(&["inventory-events", "payment-events"])
            .expect("Failed to subscribe");

        Self { consumer }
    }

    pub async fn start<R: OrderRepository>(&self, repository: Arc<R>, publisher: Arc<crate::datasource::kafka::kafka_publisher::KafkaEventPublisher>) {
        let mut stream = self.consumer.stream();

        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        let payload_str = String::from_utf8_lossy(payload);
                        self.handle_event(&payload_str, &repository, &publisher).await;
                    }
                }
                Err(e) => tracing::error!("Kafka error: {}", e),
            }
        }
    }

    async fn handle_event<R: OrderRepository>(
        &self,
        payload: &str,
        repository: &Arc<R>,
        publisher: &Arc<crate::datasource::kafka::kafka_publisher::KafkaEventPublisher>,
    ) {
        #[derive(Deserialize)]
        enum IncomingEvent {
            InventoryReserved {
                order_id: String,
            },
            InventoryFailed {
                order_id: String,
                reason: String,
            },
            PaymentCompleted {
                order_id: String,
            },
            PaymentFailed {
                order_id: String,
                reason: String,
            },
        }

        match serde_json::from_str::<IncomingEvent>(payload) {
            Ok(IncomingEvent::InventoryReserved { order_id }) => {
                tracing::info!("Inventory reserved for order: {}", order_id);
                if let Some(order) = self.update_order_status(repository, &order_id, |order| {
                    order.reserve_inventory()
                }).await {
                    let event = crate::domain::order::event::OrderEvent::OrderInventoryReserved {
                        order_id: order.id.to_string(),
                        customer_id: order.customer_id.to_string(),
                        total_amount: order.total_amount(),
                        reserved_at: chrono::Utc::now(),
                    };
                    if let Err(err) = publisher.publish(&event).await {
                        tracing::error!("Failed to publish OrderInventoryReserved: {}", err);
                    } else {
                        tracing::info!("Published OrderInventoryReserved for order: {}", order.id);
                    }
                }
            }
            Ok(IncomingEvent::InventoryFailed { order_id, reason }) => {
                tracing::warn!("Inventory failed for order: {} - {}", order_id, reason);
                self.update_order_status(repository, &order_id, |order| {
                    order.inventory_failed(reason)
                }).await;
            }
            Ok(IncomingEvent::PaymentCompleted { order_id }) => {
                tracing::info!("Payment completed for order: {}", order_id);
                self.update_order_status(repository, &order_id, |order| {
                    order.complete_payment()
                }).await;
            }
            Ok(IncomingEvent::PaymentFailed { order_id, reason }) => {
                tracing::warn!("Payment failed for order: {} - {}", order_id, reason);
                self.update_order_status(repository, &order_id, |order| {
                    order.fail_payment(reason)
                }).await;
            }
            Err(e) => {
                tracing::warn!("Failed to parse event: {} - Payload: {}", e, payload);
            }
        }
    }

    async fn update_order_status<R: OrderRepository, F>(
        &self,
        repository: &Arc<R>,
        order_id_str: &str,
        transition: F,
    ) -> Option<Order>
    where
        F: FnOnce(&mut Order) -> Result<(), OrderError>,
    {
        let order_id = OrderId::from(order_id_str.to_string());
        match repository.find_by_id(order_id).await {
            Ok(Some(mut order)) => {
                if let Err(err) = transition(&mut order) {
                    tracing::error!("Failed to transition order status: {}", err);
                    return None;
                }
                if let Err(err) = repository.save(&order).await {
                    tracing::error!("Failed to save order: {}", err);
                    return None;
                }
                Some(order)
            }
            Ok(None) => {
                tracing::error!("Order not found: {}", order_id_str);
                None
            },
            Err(err) => {
                tracing::error!("Repository error for order {}: {}", order_id_str, err);
                None
            },
        }
    }
}
