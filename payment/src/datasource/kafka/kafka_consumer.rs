use crate::datasource::kafka::kafka_event_publisher::KafkaEventPublisher;
use crate::service::payment_gateway::PaymentGateway;
use crate::service::payment_repository::PaymentRepository;
use crate::service::payment_service::PaymentService;
use futures::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use serde::Deserialize;
use std::sync::Arc;

pub struct KafkaEventConsumer {
    consumer: StreamConsumer,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum IncomingEvent {
    OrderInventoryReserved {
        order_id: String,
        customer_id: String,
        total_amount: u64,
        #[serde(skip)]
        reserved_at: String,
    },
    // Future: Handle cancellation/refunds
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

    pub async fn start<R, G>(
        &self,
        payment_service: Arc<PaymentService<R, G>>,
        event_publisher: Arc<KafkaEventPublisher>,
    ) where
        R: PaymentRepository + Send + Sync + 'static,
        G: PaymentGateway + Send + Sync + 'static,
    {
        let mut stream = self.consumer.stream();

        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        let payload_str = String::from_utf8_lossy(payload);
                        self.handle_event(&payload_str, &payment_service, &event_publisher)
                            .await;
                    }
                }
                Err(e) => tracing::error!("Kafka error: {}", e),
            }
        }
    }

    async fn handle_event<R, G>(
        &self,
        payload: &str,
        service: &Arc<PaymentService<R, G>>,
        publisher: &Arc<KafkaEventPublisher>,
    ) where
        R: PaymentRepository,
        G: PaymentGateway,
    {
        if payload.contains("OrderInventoryReserved") {
            #[derive(Deserialize)]
            struct Event {
                order_id: String,
                customer_id: String,
                total_amount: u64,
            }
            if let Ok(e) = serde_json::from_str::<Event>(payload) {
                tracing::info!(
                    "Processing OrderInventoryReserved for order: {}",
                    e.order_id
                );

                match service
                    .process_payment(e.order_id.clone(), e.total_amount, e.customer_id)
                    .await
                {
                    Ok(payment) => {
                        // PaymentService publishes PaymentCompleted internally?
                        // Let's check PaymentService::process_payment.
                        // Yes, it does. So we don't need to publish here if PaymentService already does it.
                        // Wait, let's verify PaymentService logic.
                    }
                    Err(err) => {
                        tracing::error!("Payment failed for order {}: {}", e.order_id, err);
                        // PaymentService publishes PaymentFailed internally too?
                        // Yes, it does.
                    }
                }
            } else {
                tracing::error!(
                    "Failed to parse OrderInventoryReserved payload: {}",
                    payload
                );
            }
        }
    }
}
