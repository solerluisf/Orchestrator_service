use async_trait::async_trait;

use crate::core::domain::errors::OrchestratorError;

#[derive(Debug, Clone)]
pub enum OrchestratorQuery {
    GetSystemHealth,
    GetServiceHealth { service_id: String },
    GetAllServiceHealth,
    GetKillSwitchStatus,
    GetOperationMode,
    GetModeHistory,
    GetActivePolicies,
    GetPolicy { policy_id: String },
    GetServiceControlSummary,
    GetServiceControlDetail { service_id: String },
    GetAuditLog {
        from: Option<u64>,
        to: Option<u64>,
        actor: Option<String>,
        action: Option<String>,
        limit: Option<usize>,
    },
    GetAuditEntry { entry_id: String },
    ListWorkflows,
    GetWorkflowInstance { instance_id: String },
    ListActiveSagas,
    GetConfig,
    GetConfigDiff,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum QueryResponse {
    SystemHealth(serde_json::Value),
    ServiceHealth(serde_json::Value),
    KillSwitchStatus(serde_json::Value),
    OperationMode(serde_json::Value),
    ModeHistory(serde_json::Value),
    Policies(serde_json::Value),
    PolicyDetail(serde_json::Value),
    ServiceControl(serde_json::Value),
    AuditLog(serde_json::Value),
    AuditEntry(serde_json::Value),
    Workflows(serde_json::Value),
    WorkflowInstance(serde_json::Value),
    ActiveSagas(serde_json::Value),
    Config(serde_json::Value),
    ConfigDiff(serde_json::Value),
}

#[async_trait]
pub trait IQueryPort: Send + Sync {
    async fn handle_query(
        &self,
        query: OrchestratorQuery,
    ) -> Result<QueryResponse, OrchestratorError>;
}
