use async_trait::async_trait;

use crate::core::domain::commands::{CommandAck, OrchestratorCommand};
use crate::core::domain::errors::OrchestratorError;

#[async_trait]
pub trait IControlCommandPort: Send + Sync {
    async fn handle_command(
        &self,
        cmd: OrchestratorCommand,
    ) -> Result<CommandAck, OrchestratorError>;
}
