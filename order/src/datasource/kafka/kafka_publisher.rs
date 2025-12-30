use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use crate::domain::order::event::OrderEvent;

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

  pub async fn publish(&self, event: &OrderEvent) -> Result<(), String> {
    let payload = serde_json::to_string(event)
      .map_err(|e| e.to_string())?;

    let key = match event {
      OrderEvent::OrderCreated { order_id, .. } => order_id.clone(),
      OrderEvent::OrderPaid { order_id, .. } => order_id.clone(),
    };

    let record = FutureRecord::to(&self.topic)
      .payload(&payload)
      .key(&key);

    self.producer
      .send(record, Duration::from_secs(5))
      .await
      .map_err(|(e, _)| e.to_string())?;

    Ok(())
  }
}
