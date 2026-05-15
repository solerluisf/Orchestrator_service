use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub version: u32,
    pub steps: Vec<WorkflowStepDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStepDef {
    pub name: String,
    pub action: WorkflowAction,
    pub compensation: Option<WorkflowAction>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowAction {
    SendCommand {
        target_service: String,
        command: serde_json::Value,
    },
    WaitForEvent {
        event_type: String,
        timeout_secs: Option<u64>,
    },
    EvaluatePolicy {
        policy_id: String,
    },
    Delay {
        duration_secs: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub instance_id: String,
    pub workflow_id: String,
    pub current_step: usize,
    pub state: WorkflowInstanceState,
    pub started_at_ns: u64,
    pub updated_at_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowInstanceState {
    Running,
    Paused,
    Completed,
    Failed { reason: String },
    Compensating,
}

impl WorkflowInstance {
    pub fn new(workflow_id: &str) -> Self {
        let now = now_nanos();
        Self {
            instance_id: Uuid::new_v4().to_string(),
            workflow_id: workflow_id.to_string(),
            current_step: 0,
            state: WorkflowInstanceState::Running,
            started_at_ns: now,
            updated_at_ns: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstanceSummary {
    pub instance_id: String,
    pub workflow_id: String,
    pub current_step: usize,
    pub total_steps: usize,
    pub state: WorkflowInstanceState,
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
