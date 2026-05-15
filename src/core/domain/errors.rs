use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum OrchestratorError {
    #[error("Journal error: {0}")]
    Journal(String),

    #[error("Bus error: {0}")]
    Bus(String),

    #[error("Service unreachable: {0}")]
    ServiceUnreachable(String),

    #[error("Service response timeout: {0}")]
    ServiceTimeout(String),

    #[error("Service rejected command: {0}")]
    ServiceRejected(String),

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Policy evaluation failed: {0}")]
    PolicyError(String),

    #[error("Workflow error: {0}")]
    WorkflowError(String),

    #[error("Saga failed: {0}")]
    SagaFailed(String),

    #[error("Saga compensation failed: {0}")]
    SagaCompensationFailed(String),

    #[error("Invalid operation mode transition: from {from} to {to}")]
    InvalidModeTransition { from: String, to: String },

    #[error("Kill switch activation failed: {0}")]
    KillSwitchFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Already processing command, try again later")]
    CommandQueueFull,

    #[error("Service not registered: {0}")]
    ServiceNotRegistered(String),

    #[error("Idempotency conflict: duplicate request {0}")]
    IdempotencyConflict(String),

    #[error("Circuit breaker open for service: {0}")]
    CircuitBreakerOpen(String),

    #[error("Rate limit exceeded for service: {0}")]
    RateLimitExceeded(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, OrchestratorError>;
