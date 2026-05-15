mod mocks;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::mocks::{MockEventBus, MockServiceCommand};
    use orchestrator_service::core::application::policy_engine::PolicyEngine;
    use orchestrator_service::core::domain::policy::PolicyParameters;
    use orchestrator_service::adapters::persistence::journal_adapter::AppendOnlyJournalAdapter;

    #[tokio::test]
    async fn test_policy_update_and_list() {
        let _ = std::fs::remove_dir_all("./test_data/journal_policy");
        let journal = Arc::new(
            AppendOnlyJournalAdapter::new("./test_data/journal_policy")
                .expect("Failed to create journal"),
        );
        let event_bus = Arc::new(MockEventBus::new());
        let engine = PolicyEngine::new(journal, event_bus);

        let mut params = PolicyParameters::new();
        params.values.insert("max_drawdown_pct".to_string(), serde_json::json!(5.0));

        let result = engine.update_policy("drawdown", params).await;
        assert!(result.is_ok());

        let policies = engine.list_policies().await;
        assert!(!policies.is_empty());

        let policy = engine.get_policy("drawdown").await;
        assert!(policy.is_some());
    }
}
