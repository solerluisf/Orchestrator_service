use async_trait::async_trait;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::health::{HealthStatus, ServiceHealth};
use crate::core::ports::event_bus_port::IEventBusPort;
use crate::core::ports::service_command_port::{ServiceAck, ServiceCommand};
use super::connector_trait::IServiceConnector;

pub struct StrategyConnector {
    event_bus: std::sync::Arc<dyn IEventBusPort>,
}

impl StrategyConnector {
    pub fn new(event_bus: std::sync::Arc<dyn IEventBusPort>) -> Self {
        Self { event_bus }
    }
}

#[async_trait]
impl IServiceConnector for StrategyConnector {
    fn service_id(&self) -> &str {
        "strategy_service"
    }

    fn service_name(&self) -> &str {
        "Strategy Service"
    }

    async fn health_check(&self) -> Result<ServiceHealth, OrchestratorError> {
        match self.event_bus.request_reply("strategy_service", b"health").await {
            Ok(_) => Ok(ServiceHealth {
                service_id: "strategy_service".to_string(),
                status: HealthStatus::Healthy,
                last_heartbeat_ns: Some(now_nanos()),
                uptime_secs: None,
                details: None,
            }),
            Err(_) => Ok(ServiceHealth {
                service_id: "strategy_service".to_string(),
                status: HealthStatus::Unhealthy,
                last_heartbeat_ns: None,
                uptime_secs: None,
                details: None,
            }),
        }
    }

    async fn send_command(&self, cmd: ServiceCommand) -> Result<ServiceAck, OrchestratorError> {
        let payload = serde_json::to_vec(&cmd)
            .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?;
        let response = self.event_bus
            .request_reply("strategy_service", &payload)
            .await
            .map_err(|e| OrchestratorError::Bus(e.to_string()))?;
        serde_json::from_slice(&response)
            .map_err(|e| OrchestratorError::SerializationError(e.to_string()))
    }
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
