use std::sync::Arc;
use tokio::sync::mpsc;

use crate::core::domain::commands::{CommandAck, CommandId, OrchestratorCommand};
use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::events::{SystemEvent, WorkflowEvent};
use crate::core::ports::event_bus_port::IEventBusPort;
use crate::core::ports::journal_port::{IJournalPort, JournalEntry};

pub struct CommandHandler {
    journal: Arc<dyn IJournalPort>,
    event_bus: Arc<dyn IEventBusPort>,
    command_rx: mpsc::Receiver<(OrchestratorCommand, tokio::sync::oneshot::Sender<Result<CommandAck, OrchestratorError>>)>,
}

impl CommandHandler {
    pub fn new(
        journal: Arc<dyn IJournalPort>,
        event_bus: Arc<dyn IEventBusPort>,
        command_rx: mpsc::Receiver<(OrchestratorCommand, tokio::sync::oneshot::Sender<Result<CommandAck, OrchestratorError>>)>,
    ) -> Self {
        Self {
            journal,
            event_bus,
            command_rx,
        }
    }

    pub async fn run(&mut self) {
        while let Some((cmd, reply_tx)) = self.command_rx.recv().await {
            let result = self.process_command(cmd).await;
            let _ = reply_tx.send(result);
        }
    }

    async fn process_command(&self, cmd: OrchestratorCommand) -> Result<CommandAck, OrchestratorError> {
        let workflow_event = match &cmd {
            OrchestratorCommand::ActivateKillSwitch { reason, actor } => {
                tracing::warn!(reason = %reason, actor = %actor, "Kill switch activated");
                WorkflowEvent::new(SystemEvent::KillSwitchActivated {
                    actor: actor.clone(),
                    reason: reason.clone(),
                })
            }
            OrchestratorCommand::ClearKillSwitch { reason, actor } => {
                tracing::info!(reason = %reason, actor = %actor, "Kill switch cleared");
                WorkflowEvent::new(SystemEvent::KillSwitchCleared {
                    actor: actor.clone(),
                    reason: reason.clone(),
                })
            }
            OrchestratorCommand::TransitionMode { target_mode, reason, .. } => {
                tracing::info!(target = %target_mode, reason = %reason, "Mode transition requested");
                WorkflowEvent::new(SystemEvent::ModeTransitionStarted {
                    from: crate::core::domain::operation_mode::OperationMode::Paper,
                    to: *target_mode,
                })
            }
            OrchestratorCommand::UpdatePolicy { policy_id, .. } => {
                tracing::info!(policy_id = %policy_id, "Policy update requested");
                WorkflowEvent::new(SystemEvent::PolicyUpdated {
                    policy_id: policy_id.clone(),
                })
            }
            OrchestratorCommand::ReloadPolicies => {
                tracing::info!("Policy reload requested");
                WorkflowEvent::new(SystemEvent::PoliciesReloaded)
            }
            OrchestratorCommand::PauseService { service_id, reason } => {
                tracing::info!(service_id = %service_id, reason = %reason, "Service pause requested");
                WorkflowEvent::new(SystemEvent::ServiceDeregistered {
                    service_id: service_id.clone(),
                })
            }
            OrchestratorCommand::ResumeService { service_id, .. } => {
                tracing::info!(service_id = %service_id, "Service resume requested");
                WorkflowEvent::new(SystemEvent::ServiceRegistered {
                    service_id: service_id.clone(),
                    metadata: serde_json::json!({}),
                })
            }
            _ => {
                WorkflowEvent::new(SystemEvent::OrchestratorStarted)
            }
        };

        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: workflow_event.timestamp_ns,
            entry_type: "command".to_string(),
            payload: serde_json::to_value(&workflow_event)
                .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?,
            checksum: None,
        };

        self.journal.append(entry).await?;

        Ok(CommandAck {
            command_id: CommandId::new(),
            accepted: true,
            message: "Command accepted".to_string(),
            saga_id: None,
        })
    }
}
