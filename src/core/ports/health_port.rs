use async_trait::async_trait;
use std::collections::HashMap;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::health::ServiceHealth;

#[async_trait]
pub trait IHealthPort: Send + Sync {
    async fn get_health(&self, service_id: &str) -> Result<ServiceHealth, OrchestratorError>;

    async fn get_all_health(&self) -> Result<HashMap<String, ServiceHealth>, OrchestratorError>;
}
