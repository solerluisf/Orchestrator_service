use async_trait::async_trait;

use crate::core::domain::events::SystemEvent;

#[derive(Debug, Clone, thiserror::Error)]
pub enum BusPublishError {
    #[error("Connection lost: {0}")]
    ConnectionLost(String),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Topic not available: {0}")]
    TopicUnavailable(String),
}

#[async_trait]
pub trait IEventBusPort: Send + Sync {
    async fn publish(&self, topic: &str, event: &SystemEvent) -> Result<(), BusPublishError>;

    async fn request_reply(
        &self,
        service: &str,
        payload: &[u8],
    ) -> Result<Vec<u8>, BusPublishError>;

    async fn subscribe(&self, topic: &str) -> Result<(), BusPublishError>;

    async fn unsubscribe(&self, topic: &str) -> Result<(), BusPublishError>;
}
