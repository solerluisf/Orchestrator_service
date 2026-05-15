use async_trait::async_trait;

use crate::core::domain::errors::OrchestratorError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceCommand {
    pub command_type: String,
    pub payload: serde_json::Value,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceAck {
    pub success: bool,
    pub message: String,
}

#[async_trait]
pub trait IServiceCommandPort: Send + Sync {
    async fn send_command(
        &self,
        target: &str,
        cmd: ServiceCommand,
    ) -> Result<ServiceAck, OrchestratorError>;

    async fn broadcast_command(
        &self,
        cmd: ServiceCommand,
    ) -> Vec<(String, Result<ServiceAck, OrchestratorError>)>;
}
