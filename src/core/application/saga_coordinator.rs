use std::collections::HashMap;
use std::sync::Arc;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::saga::{SagaDefinition, SagaInstance, SagaInstanceState, SagaInstanceSummary};
use crate::core::ports::event_bus_port::IEventBusPort;
use crate::core::ports::journal_port::{IJournalPort, JournalEntry};
use crate::core::domain::events::{SystemEvent, WorkflowEvent};

pub struct SagaCoordinator {
    definitions: HashMap<String, SagaDefinition>,
    instances: Arc<tokio::sync::RwLock<HashMap<String, SagaInstance>>>,
    journal: Arc<dyn IJournalPort>,
    event_bus: Arc<dyn IEventBusPort>,
}

impl SagaCoordinator {
    pub fn new(journal: Arc<dyn IJournalPort>, event_bus: Arc<dyn IEventBusPort>) -> Self {
        Self {
            definitions: HashMap::new(),
            instances: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            journal,
            event_bus,
        }
    }

    pub fn register_saga(&mut self, definition: SagaDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    pub async fn start_saga(&self, saga_type: &str) -> Result<SagaInstance, OrchestratorError> {
        let _definition = self.definitions.values()
            .find(|d| d.saga_type == saga_type)
            .ok_or_else(|| {
                OrchestratorError::SagaFailed(format!("Saga definition not found: {}", saga_type))
            })?;

        let instance = SagaInstance::new(saga_type);

        let event = WorkflowEvent::new(SystemEvent::SagaStarted {
            saga_id: instance.instance_id.clone(),
            saga_type: saga_type.to_string(),
        });
        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: event.timestamp_ns,
            entry_type: "saga_started".to_string(),
            payload: serde_json::to_value(&event)
                .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?,
            checksum: None,
        };
        self.journal.append(entry).await?;

        let mut instances = self.instances.write().await;
        instances.insert(instance.instance_id.clone(), instance.clone());

        tracing::info!(saga_id = %instance.instance_id, saga_type = %saga_type, "Saga started");
        Ok(instance)
    }

    pub async fn advance_saga(&self, saga_id: &str) -> Result<(), OrchestratorError> {
        let mut instances = self.instances.write().await;
        let instance = instances.get_mut(saga_id).ok_or_else(|| {
            OrchestratorError::SagaFailed(format!("Saga not found: {}", saga_id))
        })?;

        let definition = self.definitions.values().find(|d| d.saga_type == instance.saga_type).ok_or_else(|| {
            OrchestratorError::SagaFailed(format!(
                "Saga definition not found: {}",
                instance.saga_type
            ))
        })?;

        if instance.current_step >= definition.steps.len() {
            instance.state = SagaInstanceState::Completed;
            let event = WorkflowEvent::new(SystemEvent::SagaCompleted {
                saga_id: saga_id.to_string(),
            });
            let entry = JournalEntry {
                sequence: None,
                timestamp_ns: event.timestamp_ns,
                entry_type: "saga_completed".to_string(),
                payload: serde_json::to_value(&event)
                    .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?,
                checksum: None,
            };
            self.journal.append(entry).await?;
            return Ok(());
        }

        instance.advance_step();

        let event = WorkflowEvent::new(SystemEvent::SagaStepCompleted {
            saga_id: saga_id.to_string(),
            step_index: instance.current_step,
        });
        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: event.timestamp_ns,
            entry_type: "saga_step_completed".to_string(),
            payload: serde_json::to_value(&event)
                .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?,
            checksum: None,
        };
        self.journal.append(entry).await?;

        Ok(())
    }

    pub async fn compensate_saga(&self, saga_id: &str) -> Result<(), OrchestratorError> {
        let mut instances = self.instances.write().await;
        let instance = instances.get_mut(saga_id).ok_or_else(|| {
            OrchestratorError::SagaFailed(format!("Saga not found: {}", saga_id))
        })?;

        instance.start_compensation();

        let mut compensated_steps = Vec::new();
        while !instance.completed_steps.is_empty() {
            instance.compensate_step();
            compensated_steps.push(instance.current_step);
        }

        let event = WorkflowEvent::new(SystemEvent::SagaCompensated {
            saga_id: saga_id.to_string(),
            compensated_steps: compensated_steps.clone(),
        });
        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: event.timestamp_ns,
            entry_type: "saga_compensated".to_string(),
            payload: serde_json::to_value(&event)
                .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?,
            checksum: None,
        };
        self.journal.append(entry).await?;

        tracing::warn!(saga_id = %saga_id, steps = ?compensated_steps, "Saga compensated");
        Ok(())
    }

    pub async fn list_instances(&self) -> Vec<SagaInstanceSummary> {
        let instances = self.instances.read().await;
        instances
            .values()
            .map(|i| {
                let total_steps = self
                    .definitions
                    .values()
                    .find(|d| d.saga_type == i.saga_type)
                    .map(|d| d.steps.len())
                    .unwrap_or(0);
                SagaInstanceSummary {
                    instance_id: i.instance_id.clone(),
                    saga_type: i.saga_type.clone(),
                    current_step: i.current_step,
                    total_steps,
                    state: i.state.clone(),
                }
            })
            .collect()
    }

    pub async fn get_instance(&self, instance_id: &str) -> Option<SagaInstance> {
        let instances = self.instances.read().await;
        instances.get(instance_id).cloned()
    }

    pub async fn restore_from_journal(&self) -> Result<(), OrchestratorError> {
        let entries = self.journal.read_latest(10000).await?;
        let mut instances: HashMap<String, SagaInstance> = HashMap::new();

        for entry in &entries {
            if let Ok(workflow_event) = serde_json::from_value::<WorkflowEvent>(entry.payload.clone()) {
                match &workflow_event.event {
                    SystemEvent::SagaStarted { saga_id, saga_type } => {
                        instances.insert(
                            saga_id.clone(),
                            SagaInstance {
                                instance_id: saga_id.clone(),
                                saga_type: saga_type.clone(),
                                current_step: 0,
                                state: SagaInstanceState::Running,
                                completed_steps: Vec::new(),
                                started_at_ns: workflow_event.timestamp_ns,
                                updated_at_ns: workflow_event.timestamp_ns,
                            },
                        );
                    }
                    SystemEvent::SagaStepCompleted { saga_id, step_index } => {
                        if let Some(instance) = instances.get_mut(saga_id) {
                            instance.completed_steps.push(instance.current_step);
                            instance.current_step = *step_index;
                            instance.updated_at_ns = workflow_event.timestamp_ns;
                        }
                    }
                    SystemEvent::SagaCompensated { saga_id, compensated_steps: _ } => {
                        if let Some(instance) = instances.get_mut(saga_id) {
                            instance.state = SagaInstanceState::Compensating;
                            instance.updated_at_ns = workflow_event.timestamp_ns;
                        }
                    }
                    SystemEvent::SagaCompleted { saga_id } => {
                        if let Some(instance) = instances.get_mut(saga_id) {
                            instance.state = SagaInstanceState::Completed;
                            instance.updated_at_ns = workflow_event.timestamp_ns;
                        }
                    }
                    _ => {}
                }
            }
        }

        let count = instances.len();
        let mut write_instances = self.instances.write().await;
        for (id, instance) in instances {
            write_instances.insert(id, instance);
        }

        if count > 0 {
            tracing::info!(count = count, "Restored saga instances from journal");
        }
        Ok(())
    }
}
