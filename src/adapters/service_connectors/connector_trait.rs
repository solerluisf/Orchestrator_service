use std::sync::Arc;

use async_trait::async_trait;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::health::ServiceHealth;
use crate::core::patterns::circuit_breaker::CircuitBreaker;
use crate::core::patterns::rate_limiter::RateLimiter;
use crate::core::ports::service_command_port::{ServiceAck, ServiceCommand};

#[async_trait]
pub trait IServiceConnector: Send + Sync {
    fn service_id(&self) -> &str;
    fn service_name(&self) -> &str;

    async fn health_check(&self) -> Result<ServiceHealth, OrchestratorError>;

    async fn send_command_inner(
        &self,
        cmd: ServiceCommand,
    ) -> Result<ServiceAck, OrchestratorError>;

    fn circuit_breaker(&self) -> Option<Arc<CircuitBreaker>> {
        None
    }

    fn rate_limiter(&self) -> Option<Arc<RateLimiter>> {
        None
    }

    async fn send_command(
        &self,
        cmd: ServiceCommand,
    ) -> Result<ServiceAck, OrchestratorError> {
        if let Some(cb) = self.circuit_breaker() {
            if !cb.can_execute().await {
                return Err(OrchestratorError::CircuitBreakerOpen(
                    self.service_id().to_string(),
                ));
            }
        }

        if let Some(rl) = self.rate_limiter() {
            if !rl.check_rate(self.service_id()).await {
                return Err(OrchestratorError::RateLimitExceeded(
                    self.service_id().to_string(),
                ));
            }
        }

        let result = self.send_command_inner(cmd).await;

        if let Some(cb) = self.circuit_breaker() {
            match &result {
                Ok(_) => cb.record_success().await,
                Err(_) => cb.record_failure().await,
            }
        }

        result
    }

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
