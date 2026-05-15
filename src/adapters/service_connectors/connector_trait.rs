use async_trait::async_trait;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::health::ServiceHealth;
use crate::core::ports::service_command_port::{ServiceAck, ServiceCommand};

#[async_trait]
pub trait IServiceConnector: Send + Sync {
    fn service_id(&self) -> &str;
    fn service_name(&self) -> &str;

    async fn health_check(&self) -> Result<ServiceHealth, OrchestratorError>;

    async fn send_command(
        &self,
        cmd: ServiceCommand,
    ) -> Result<ServiceAck, OrchestratorError>;

    async fn pause(&self) -> Result<ServiceAck, OrchestratorError> {
        self.send_command(ServiceCommand {
            command_type: "pause".to_string(),
            payload: serde_json::json!({}),
            timeout_secs: Some(5),
        })
        .await
    }

    async fn resume(&self) -> Result<ServiceAck, OrchestratorError> {
        self.send_command(ServiceCommand {
            command_type: "resume".to_string(),
            payload: serde_json::json!({}),
            timeout_secs: Some(5),
        })
        .await
    }
}
