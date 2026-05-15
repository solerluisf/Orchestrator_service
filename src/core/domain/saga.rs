use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::workflow::WorkflowAction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaDefinition {
    pub id: String,
    pub saga_type: String,
    pub steps: Vec<SagaStepDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStepDef {
    pub name: String,
    pub action: WorkflowAction,
    pub compensation: WorkflowAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaInstance {
    pub instance_id: String,
    pub saga_type: String,
    pub current_step: usize,
    pub state: SagaInstanceState,
    pub completed_steps: Vec<usize>,
    pub started_at_ns: u64,
    pub updated_at_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SagaInstanceState {
    Running,
    Compensating,
    Completed,
    Failed { reason: String },
}

impl SagaInstance {
    pub fn new(saga_type: &str) -> Self {
        let now = now_nanos();
        Self {
            instance_id: Uuid::new_v4().to_string(),
            saga_type: saga_type.to_string(),
            current_step: 0,
            state: SagaInstanceState::Running,
            completed_steps: Vec::new(),
            started_at_ns: now,
            updated_at_ns: now,
        }
    }

    pub fn advance_step(&mut self) {
        self.completed_steps.push(self.current_step);
        self.current_step += 1;
        self.updated_at_ns = now_nanos();
    }

    pub fn start_compensation(&mut self) {
        self.state = SagaInstanceState::Compensating;
        self.updated_at_ns = now_nanos();
    }

    pub fn compensate_step(&mut self) {
        if let Some(last) = self.completed_steps.pop() {
            self.current_step = last;
        }
        self.updated_at_ns = now_nanos();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaInstanceSummary {
    pub instance_id: String,
    pub saga_type: String,
    pub current_step: usize,
    pub total_steps: usize,
    pub state: SagaInstanceState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationAction {
    pub step_index: usize,
    pub action: WorkflowAction,
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
