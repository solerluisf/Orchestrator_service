use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::policy::{PolicyContext, PolicyDecision, PolicyParameters};

pub trait IPolicyPort: Send + Sync {
    fn evaluate(&self, policy_id: &str, ctx: &PolicyContext)
        -> Result<PolicyDecision, OrchestratorError>;

    fn reload_policies(
        &mut self,
        params: &std::collections::HashMap<String, PolicyParameters>,
    ) -> Result<(), OrchestratorError>;

    fn get_all_policies(&self) -> Vec<(String, bool)>;
}
