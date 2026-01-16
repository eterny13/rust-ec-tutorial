use crate::domain::payment_event::PaymentEvent;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
#[error("Kafka publish error: {0}")]
pub struct KafkaEventPublishError(pub String);

pub struct KafkaEventPublisher {
    producer: FutureProducer,
    topic: String,
}

impl KafkaEventPublisher {
    pub fn new(brokers: &str, topic: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        Self {
            producer,
            topic: topic.to_owned(),
        }
    }

    pub async fn publish(&self, event: &PaymentEvent) -> Result<(), KafkaEventPublishError> {
        let payload =
            serde_json::to_string(event).map_err(|e| KafkaEventPublishError(e.to_string()))?;

        let key = match event {
            PaymentEvent::PaymentCompleted { order_id, .. } => order_id.clone(),
            PaymentEvent::PaymentFailed { order_id, .. } => order_id.clone(),
            PaymentEvent::PaymentRefunded { order_id, .. } => order_id.clone(),
        };

        let record = FutureRecord::to(&self.topic).payload(&payload).key(&key);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| KafkaEventPublishError(e.to_string()))?;

        tracing::info!("Published event: {:?}", event);
        Ok(())
    }
}
