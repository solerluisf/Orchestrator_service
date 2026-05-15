use async_trait::async_trait;

use crate::core::domain::errors::OrchestratorError;
use crate::core::ports::event_bus_port::IEventBusPort;
use crate::core::ports::service_command_port::{IServiceCommandPort, ServiceAck, ServiceCommand};

pub struct BusServiceCommandAdapter {
    event_bus: std::sync::Arc<dyn IEventBusPort>,
}

impl BusServiceCommandAdapter {
    pub fn new(event_bus: std::sync::Arc<dyn IEventBusPort>) -> Self {
        Self { event_bus }
    }
}

#[async_trait]
impl IServiceCommandPort for BusServiceCommandAdapter {
    async fn send_command(
        &self,
        target: &str,
        cmd: ServiceCommand,
    ) -> Result<ServiceAck, OrchestratorError> {
        let payload = serde_json::to_vec(&cmd)
            .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?;

        let response = self
            .event_bus
            .request_reply(target, &payload)
            .await
            .map_err(|e| OrchestratorError::Bus(e.to_string()))?;

        let ack: ServiceAck = serde_json::from_slice(&response)
            .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?;

        Ok(ack)
    }

    async fn broadcast_command(
        &self,
        cmd: ServiceCommand,
    ) -> Vec<(String, Result<ServiceAck, OrchestratorError>)> {
        let services = [
            "broker_gateway",
            "market_data_service",
            "feature_service",
            "model_service",
            "strategy_service",
            "risk_service",
            "execution_service",
        ];

        let mut results = Vec::new();
        for service in &services {
            let result = self.send_command(service, cmd.clone()).await;
            results.push((service.to_string(), result));
        }
        results
    }
}
