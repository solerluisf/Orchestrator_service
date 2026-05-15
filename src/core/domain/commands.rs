use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::operation_mode::OperationMode;
use super::policy::PolicyParameters;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandId(pub Uuid);

impl CommandId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum OrchestratorCommand {
    ActivateKillSwitch {
        reason: String,
        actor: String,
    },
    ClearKillSwitch {
        reason: String,
        actor: String,
    },
    TransitionMode {
        target_mode: OperationMode,
        reason: String,
        dry_run: bool,
    },
    UpdatePolicy {
        policy_id: String,
        parameters: PolicyParameters,
    },
    ReloadPolicies,
    PauseService {
        service_id: String,
        reason: String,
    },
    ResumeService {
        service_id: String,
        reason: String,
    },
    ResetCircuitBreaker {
        service_id: String,
    },
    ConfigureCircuitBreaker {
        service_id: String,
        failure_threshold: u32,
        cooldown_secs: u64,
    },
    ConfigureRateLimiter {
        service_id: String,
        max_requests_per_min: u32,
        burst_capacity: u32,
    },
    TriggerWorkflow {
        workflow_id: String,
        input: serde_json::Value,
    },
    CancelWorkflow {
        instance_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAck {
    pub command_id: CommandId,
    pub accepted: bool,
    pub message: String,
    pub saga_id: Option<String>,
}
