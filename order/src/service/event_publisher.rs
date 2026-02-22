use crate::domain::order::event::OrderEvent;

#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &OrderEvent) -> Result<(), String>;
}
