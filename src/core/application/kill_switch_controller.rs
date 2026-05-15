use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::core::domain::commands::{CommandAck, CommandId};
use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::events::SystemEvent;
use crate::core::ports::event_bus_port::IEventBusPort;
use crate::core::ports::journal_port::{IJournalPort, JournalEntry};
use crate::core::ports::service_command_port::{IServiceCommandPort, ServiceCommand};

pub struct KillSwitchController {
    active: Arc<AtomicBool>,
    journal: Arc<dyn IJournalPort>,
    event_bus: Arc<dyn IEventBusPort>,
    service_command: Arc<dyn IServiceCommandPort>,
}

impl KillSwitchController {
    pub fn new(
        journal: Arc<dyn IJournalPort>,
        event_bus: Arc<dyn IEventBusPort>,
        service_command: Arc<dyn IServiceCommandPort>,
    ) -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            journal,
            event_bus,
            service_command,
        }
    }

    pub async fn activate(&self, reason: &str, actor: &str) -> Result<CommandAck, OrchestratorError> {
        // 1. Persist to journal BEFORE anything else
        let now_ns = now_nanos();
        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: now_ns,
            entry_type: "kill_switch_activate".to_string(),
            payload: serde_json::json!({
                "action": "activate",
                "reason": reason,
                "actor": actor,
                "timestamp_ns": now_ns,
            }),
            checksum: None,
        };
        self.journal.append(entry).await?;

        // 2. Set kill switch state
        self.active.store(true, Ordering::SeqCst);

        // 3. Broadcast kill switch to all services via bus PUSH (fire-and-forget, max speed)
        let event = SystemEvent::KillSwitchActivated {
            actor: actor.to_string(),
            reason: reason.to_string(),
        };
        let _ = self.event_bus.publish("system.kill_switch", &event).await;

        // 4. Send cancel-all to Execution Service (await confirmation)
        let exec_cmd = ServiceCommand {
            command_type: "cancel_all_orders".to_string(),
            payload: serde_json::json!({"reason": reason}),
            timeout_secs: Some(5),
        };
        let _ = self.service_command.send_command("execution_service", exec_cmd).await;

        // 5. Send cancel-all to Broker Gateway (await confirmation)
        let broker_cmd = ServiceCommand {
            command_type: "cancel_all_orders".to_string(),
            payload: serde_json::json!({"reason": reason}),
            timeout_secs: Some(5),
        };
        let _ = self.service_command.send_command("broker_gateway", broker_cmd).await;

        tracing::warn!(reason = %reason, actor = %actor, "Kill switch activated");

        Ok(CommandAck {
            command_id: CommandId::new(),
            accepted: true,
            message: format!("Kill switch activated: {}", reason),
            saga_id: None,
        })
    }

    pub async fn clear(&self, reason: &str, actor: &str) -> Result<CommandAck, OrchestratorError> {
        let now_ns = now_nanos();
        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: now_ns,
            entry_type: "kill_switch_clear".to_string(),
            payload: serde_json::json!({
                "action": "clear",
                "reason": reason,
                "actor": actor,
                "timestamp_ns": now_ns,
            }),
            checksum: None,
        };
        self.journal.append(entry).await?;

        self.active.store(false, Ordering::SeqCst);

        let event = SystemEvent::KillSwitchCleared {
            actor: actor.to_string(),
            reason: reason.to_string(),
        };
        let _ = self.event_bus.publish("system.kill_switch", &event).await;

        tracing::info!(reason = %reason, actor = %actor, "Kill switch cleared");

        Ok(CommandAck {
            command_id: CommandId::new(),
            accepted: true,
            message: format!("Kill switch cleared: {}", reason),
            saga_id: None,
        })
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub async fn restore_from_journal(&self) -> Result<(), OrchestratorError> {
        let entries = self.journal.read_latest(100).await?;
        for entry in entries.iter().rev() {
            if entry.entry_type == "kill_switch_activate" {
                self.active.store(true, Ordering::SeqCst);
                tracing::warn!("Kill switch restored from journal: active");
                return Ok(());
            }
            if entry.entry_type == "kill_switch_clear" {
                self.active.store(false, Ordering::SeqCst);
                tracing::info!("Kill switch restored from journal: inactive");
                return Ok(());
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
