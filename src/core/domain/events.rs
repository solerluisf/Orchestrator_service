use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::health::ServiceHealth;
use super::operation_mode::OperationMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceId(pub String);

impl TraceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SystemEvent {
    // Health events
    ServiceHealthChanged {
        service_id: String,
        health: ServiceHealth,
    },
    ServiceRegistered {
        service_id: String,
        metadata: serde_json::Value,
    },
    ServiceDeregistered {
        service_id: String,
    },

    // Kill-switch events
    KillSwitchActivated {
        actor: String,
        reason: String,
    },
    KillSwitchCleared {
        actor: String,
        reason: String,
    },

    // Mode events
    ModeTransitionStarted {
        from: OperationMode,
        to: OperationMode,
    },
    ModeTransitionCompleted {
        from: OperationMode,
        to: OperationMode,
    },
    ModeTransitionFailed {
        from: OperationMode,
        to: OperationMode,
        reason: String,
    },

    // Policy events
    PolicyUpdated {
        policy_id: String,
    },
    PoliciesReloaded,

    // Workflow events
    WorkflowStarted {
        workflow_id: String,
        instance_id: String,
    },
    WorkflowStepCompleted {
        instance_id: String,
        step_index: usize,
    },
    WorkflowCompleted {
        instance_id: String,
    },
    WorkflowFailed {
        instance_id: String,
        reason: String,
    },

    // Saga events
    SagaStarted {
        saga_id: String,
        saga_type: String,
    },
    SagaStepCompleted {
        saga_id: String,
        step_index: usize,
    },
    SagaStepFailed {
        saga_id: String,
        step_index: usize,
        reason: String,
    },
    SagaCompensated {
        saga_id: String,
        compensated_steps: Vec<usize>,
    },
    SagaCompleted {
        saga_id: String,
    },

    // Circuit breaker events
    CircuitBreakerOpened {
        service_id: String,
    },
    CircuitBreakerClosed {
        service_id: String,
    },

    // Orchestrator lifecycle
    OrchestratorStarted,
    OrchestratorRestarted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub event_id: EventId,
    pub timestamp_ns: u64,
    pub event: SystemEvent,
    pub correlation_id: Option<String>,
}

impl WorkflowEvent {
    pub fn new(event: SystemEvent) -> Self {
        Self {
            event_id: EventId::new(),
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            event,
            correlation_id: None,
        }
    }

    pub fn with_correlation(mut self, id: String) -> Self {
        self.correlation_id = Some(id);
        self
    }
}
