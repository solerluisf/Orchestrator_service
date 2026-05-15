use std::collections::HashMap;
use std::sync::Arc;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::policy::{PolicyParameters, PolicySummary};
use crate::core::ports::event_bus_port::IEventBusPort;
use crate::core::ports::journal_port::{IJournalPort, JournalEntry};

pub struct PolicyEngine {
    policies: Arc<tokio::sync::RwLock<HashMap<String, PolicyState>>>,
    journal: Arc<dyn IJournalPort>,
    event_bus: Arc<dyn IEventBusPort>,
}

struct PolicyState {
    enabled: bool,
    parameters: PolicyParameters,
}

impl PolicyEngine {
    pub fn new(journal: Arc<dyn IJournalPort>, event_bus: Arc<dyn IEventBusPort>) -> Self {
        Self {
            policies: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            journal,
            event_bus,
        }
    }

    pub async fn update_policy(
        &self,
        policy_id: &str,
        parameters: PolicyParameters,
    ) -> Result<(), OrchestratorError> {
        let mut policies = self.policies.write().await;
        let state = policies
            .entry(policy_id.to_string())
            .or_insert_with(|| PolicyState {
                enabled: true,
                parameters: PolicyParameters::new(),
            });
        state.parameters = parameters.clone();

        // Journal the update
        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: now_nanos(),
            entry_type: "policy_updated".to_string(),
            payload: serde_json::json!({
                "policy_id": policy_id,
                "parameters": parameters,
            }),
            checksum: None,
        };
        self.journal.append(entry).await?;

        // Publish event
        let event = crate::core::domain::events::SystemEvent::PolicyUpdated {
            policy_id: policy_id.to_string(),
        };
        let _ = self.event_bus.publish("system.events", &event).await;

        tracing::info!(policy_id = %policy_id, "Policy updated");
        Ok(())
    }

    pub async fn reload_policies(
        &self,
        params: &HashMap<String, PolicyParameters>,
    ) -> Result<(), OrchestratorError> {
        let mut policies = self.policies.write().await;
        for (policy_id, parameters) in params {
            let state = policies
                .entry(policy_id.clone())
                .or_insert_with(|| PolicyState {
                    enabled: true,
                    parameters: PolicyParameters::new(),
                });
            state.parameters = parameters.clone();
        }

        let entry = JournalEntry {
            sequence: None,
            timestamp_ns: now_nanos(),
            entry_type: "policies_reloaded".to_string(),
            payload: serde_json::json!({
                "count": params.len(),
            }),
            checksum: None,
        };
        self.journal.append(entry).await?;

        let event = crate::core::domain::events::SystemEvent::PoliciesReloaded;
        let _ = self.event_bus.publish("system.events", &event).await;

        tracing::info!(count = params.len(), "Policies reloaded");
        Ok(())
    }

    pub async fn list_policies(&self) -> Vec<PolicySummary> {
        let policies = self.policies.read().await;
        policies
            .iter()
            .map(|(id, state)| PolicySummary {
                policy_id: id.clone(),
                enabled: state.enabled,
                parameters: state.parameters.clone(),
            })
            .collect()
    }

    pub async fn get_policy(&self, policy_id: &str) -> Option<PolicySummary> {
        let policies = self.policies.read().await;
        policies.get(policy_id).map(|state| PolicySummary {
            policy_id: policy_id.to_string(),
            enabled: state.enabled,
            parameters: state.parameters.clone(),
        })
    }
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
