use std::collections::HashMap;
use std::sync::Arc;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::workflow::{WorkflowDefinition, WorkflowInstance, WorkflowInstanceState, WorkflowInstanceSummary};
use crate::core::ports::event_bus_port::IEventBusPort;
use crate::core::ports::journal_port::{IJournalPort, JournalEntry};
use crate::core::domain::events::{SystemEvent, WorkflowEvent};

pub struct WorkflowEngine {
    definitions: HashMap<String, WorkflowDefinition>,
    instances: Arc<tokio::sync::RwLock<HashMap<String, WorkflowInstance>>>,
    journal: Arc<dyn IJournalPort>,
    event_bus: Arc<dyn IEventBusPort>,
}

impl WorkflowEngine {
    pub fn new(journal: Arc<dyn IJournalPort>, event_bus: Arc<dyn IEventBusPort>) -> Self {
        Self {
            definitions: HashMap::new(),
            instances: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            journal,
            event_bus,
        }
    }

    pub fn register_workflow(&mut self, definition: WorkflowDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    pub async fn trigger_workflow(
        &self,
        workflow_id: &str,
    ) -> Result<WorkflowInstance, OrchestratorError> {
        let _definition = self.definitions.get(workflow_id).ok_or_else(|| {
            OrchestratorError::WorkflowError(format!("Workflow not found: {}", workflow_id))
        })?;

        let instance = WorkflowInstance::new(workflow_id);

        // Journal the start
        let event = WorkflowEvent::new(SystemEvent::WorkflowStarted {
            workflow_id: workflow_id.to_string(),
            instance_id: instance.instance_id.clone(),
        });
        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: event.timestamp_ns,
            entry_type: "workflow_started".to_string(),
            payload: serde_json::to_value(&event)
                .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?,
            checksum: None,
        };
        self.journal.append(entry).await?;

        let mut instances = self.instances.write().await;
        instances.insert(instance.instance_id.clone(), instance.clone());

        tracing::info!(workflow_id = %workflow_id, instance_id = %instance.instance_id, "Workflow triggered");
        Ok(instance)
    }

    pub async fn advance_workflow(
        &self,
        instance_id: &str,
    ) -> Result<(), OrchestratorError> {
        let mut instances = self.instances.write().await;
        let instance = instances.get_mut(instance_id).ok_or_else(|| {
            OrchestratorError::WorkflowError(format!("Instance not found: {}", instance_id))
        })?;

        let definition = self.definitions.get(&instance.workflow_id).ok_or_else(|| {
            OrchestratorError::WorkflowError(format!(
                "Workflow definition not found: {}",
                instance.workflow_id
            ))
        })?;

        if instance.current_step >= definition.steps.len() {
            instance.state = WorkflowInstanceState::Completed;
            return Ok(());
        }

        instance.current_step += 1;
        instance.updated_at_ns = now_nanos();

        // Journal step completion
        let event = WorkflowEvent::new(SystemEvent::WorkflowStepCompleted {
            instance_id: instance_id.to_string(),
            step_index: instance.current_step,
        });
        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: event.timestamp_ns,
            entry_type: "workflow_step_completed".to_string(),
            payload: serde_json::to_value(&event)
                .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?,
            checksum: None,
        };
        self.journal.append(entry).await?;

        Ok(())
    }

    pub async fn list_instances(&self) -> Vec<WorkflowInstanceSummary> {
        let instances = self.instances.read().await;
        instances
            .values()
            .map(|i| {
                let total_steps = self
                    .definitions
                    .get(&i.workflow_id)
                    .map(|d| d.steps.len())
                    .unwrap_or(0);
                WorkflowInstanceSummary {
                    instance_id: i.instance_id.clone(),
                    workflow_id: i.workflow_id.clone(),
                    current_step: i.current_step,
                    total_steps,
                    state: i.state.clone(),
                }
            })
            .collect()
    }

    pub async fn get_instance(&self, instance_id: &str) -> Option<WorkflowInstance> {
        let instances = self.instances.read().await;
        instances.get(instance_id).cloned()
    }
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
