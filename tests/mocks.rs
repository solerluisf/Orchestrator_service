use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use orchestrator_service::core::domain::errors::OrchestratorError;
use orchestrator_service::core::domain::events::SystemEvent;
use orchestrator_service::core::ports::event_bus_port::{BusPublishError, IEventBusPort};
use orchestrator_service::core::ports::service_command_port::{IServiceCommandPort, ServiceAck, ServiceCommand};

pub struct MockEventBus {
    published: Mutex<Vec<(String, SystemEvent)>>,
}

impl MockEventBus {
    pub fn new() -> Self {
        Self {
            published: Mutex::new(Vec::new()),
        }
    }

    pub fn published_events(&self) -> Vec<(String, SystemEvent)> {
        self.published.lock().unwrap().clone()
    }
}

impl Default for MockEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IEventBusPort for MockEventBus {
    async fn publish(&self, topic: &str, event: &SystemEvent) -> Result<(), BusPublishError> {
        self.published.lock().unwrap().push((topic.to_string(), event.clone()));
        Ok(())
    }

    async fn request_reply(&self, _service: &str, _payload: &[u8]) -> Result<Vec<u8>, BusPublishError> {
        Ok(b"{}".to_vec())
    }

    async fn subscribe(&self, _topic: &str) -> Result<(), BusPublishError> {
        Ok(())
    }

    async fn unsubscribe(&self, _topic: &str) -> Result<(), BusPublishError> {
        Ok(())
    }
}

pub struct MockServiceCommand;

#[async_trait]
impl IServiceCommandPort for MockServiceCommand {
    async fn send_command(&self, _target: &str, _cmd: ServiceCommand) -> Result<ServiceAck, OrchestratorError> {
        Ok(ServiceAck {
            success: true,
            message: "mock ack".to_string(),
        })
    }

    async fn broadcast_command(&self, _cmd: ServiceCommand) -> Vec<(String, Result<ServiceAck, OrchestratorError>)> {
        vec![]
    }
}
