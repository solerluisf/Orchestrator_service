use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::domain::commands::{CommandAck, CommandId};
use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::operation_mode::{ModeTransitionRecord, OperationMode};
use crate::core::ports::event_bus_port::IEventBusPort;
use crate::core::ports::journal_port::{IJournalPort, JournalEntry};

pub struct ModeController {
    current_mode: Arc<RwLock<OperationMode>>,
    history: Arc<RwLock<Vec<ModeTransitionRecord>>>,
    journal: Arc<dyn IJournalPort>,
    event_bus: Arc<dyn IEventBusPort>,
}

impl ModeController {
    pub fn new(
        journal: Arc<dyn IJournalPort>,
        event_bus: Arc<dyn IEventBusPort>,
    ) -> Self {
        Self {
            current_mode: Arc::new(RwLock::new(OperationMode::Paper)),
            history: Arc::new(RwLock::new(Vec::new())),
            journal,
            event_bus,
        }
    }

    pub async fn transition(
        &self,
        target: OperationMode,
        reason: &str,
        actor: &str,
        dry_run: bool,
    ) -> Result<CommandAck, OrchestratorError> {
        let current = *self.current_mode.read().await;

        if !current.can_transition_to(&target) {
            return Err(OrchestratorError::InvalidModeTransition {
                from: current.to_string(),
                to: target.to_string(),
            });
        }

        if dry_run {
            return Ok(CommandAck {
                command_id: CommandId::new(),
                accepted: true,
                message: format!(
                    "Dry run: would transition from {} to {}",
                    current, target
                ),
                saga_id: None,
            });
        }

        // Log transition started
        let started_event = crate::core::domain::events::WorkflowEvent::new(
            crate::core::domain::events::SystemEvent::ModeTransitionStarted {
                from: current,
                to: target,
            },
        );
        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: started_event.timestamp_ns,
            entry_type: "mode_transition_started".to_string(),
            payload: serde_json::to_value(&started_event)
                .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?,
            checksum: None,
        };
        self.journal.append(entry).await?;

        // Perform the transition
        *self.current_mode.write().await = target;

        // Record in history
        let record = ModeTransitionRecord {
            from: current,
            to: target,
            timestamp_ns: now_nanos(),
            actor: actor.to_string(),
            reason: reason.to_string(),
            success: true,
        };
        self.history.write().await.push(record);

        // Log transition completed
        let completed_event = crate::core::domain::events::WorkflowEvent::new(
            crate::core::domain::events::SystemEvent::ModeTransitionCompleted {
                from: current,
                to: target,
            },
        );
        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: completed_event.timestamp_ns,
            entry_type: "mode_transition_completed".to_string(),
            payload: serde_json::to_value(&completed_event)
                .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?,
            checksum: None,
        };
        self.journal.append(entry).await?;

        // Publish to bus
        let _ = self.event_bus
            .publish("system.mode", &completed_event.event)
            .await;

        tracing::info!(from = %current, to = %target, reason = %reason, "Mode transition completed");

        Ok(CommandAck {
            command_id: CommandId::new(),
            accepted: true,
            message: format!("Mode transitioned from {} to {}", current, target),
            saga_id: None,
        })
    }

    pub async fn current_mode(&self) -> OperationMode {
        *self.current_mode.read().await
    }

    pub async fn history(&self) -> Vec<ModeTransitionRecord> {
        self.history.read().await.clone()
    }

    pub async fn restore_from_journal(&self) -> Result<(), OrchestratorError> {
        let entries = self.journal.read_latest(100).await?;
        for entry in entries.iter().rev() {
            if entry.entry_type == "mode_transition_completed" {
                if let Ok(event) = serde_json::from_value::<
                    crate::core::domain::events::WorkflowEvent,
                >(entry.payload.clone()) {
                    if let crate::core::domain::events::SystemEvent::ModeTransitionCompleted { to, .. } = event.event {
                        *self.current_mode.write().await = to;
                        tracing::info!(mode = %to, "Operation mode restored from journal");
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
