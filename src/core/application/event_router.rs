use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::events::{SystemEvent, WorkflowEvent};
use crate::core::ports::event_bus_port::IEventBusPort;
use crate::core::ports::journal_port::{IJournalPort, JournalEntry};

pub struct EventRouter {
    event_bus: Arc<dyn IEventBusPort>,
    journal: Arc<dyn IJournalPort>,
    handlers: RwLock<Vec<Box<dyn EventHandler>>>,
}

pub trait EventHandler: Send + Sync {
    fn can_handle(&self, event: &SystemEvent) -> bool;
    fn handle(&self, event: &SystemEvent) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), OrchestratorError>> + Send>,
    >;
}

pub struct EventRouterService {
    journal: Arc<dyn IJournalPort>,
    event_bus: Arc<dyn IEventBusPort>,
}

impl EventRouterService {
    pub fn new(journal: Arc<dyn IJournalPort>, event_bus: Arc<dyn IEventBusPort>) -> Self {
        Self { journal, event_bus }
    }

    pub async fn route_event(&self, event: SystemEvent) -> Result<(), OrchestratorError> {
        let workflow_event = WorkflowEvent::new(event.clone());

        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: workflow_event.timestamp_ns,
            entry_type: "event".to_string(),
            payload: serde_json::to_value(&workflow_event)
                .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?,
            checksum: None,
        };

        self.journal.append(entry).await?;

        let topic = match &event {
            SystemEvent::KillSwitchActivated { .. } | SystemEvent::KillSwitchCleared { .. } => {
                "system.kill_switch"
            }
            SystemEvent::ModeTransitionStarted { .. }
            | SystemEvent::ModeTransitionCompleted { .. }
            | SystemEvent::ModeTransitionFailed { .. } => "system.mode",
            SystemEvent::CircuitBreakerOpened { .. } | SystemEvent::CircuitBreakerClosed { .. } => {
                "system.circuit_breaker"
            }
            _ => "system.events",
        };

        self.event_bus.publish(topic, &event).await.map_err(|e| {
            OrchestratorError::Bus(format!("Failed to publish event: {}", e))
        })?;

        Ok(())
    }
}
