mod mocks;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::mocks::{MockEventBus, MockServiceCommand};
    use orchestrator_service::core::application::workflow_engine::WorkflowEngine;
    use orchestrator_service::core::domain::workflow::{WorkflowDefinition, WorkflowStepDef, WorkflowAction};
    use orchestrator_service::adapters::persistence::journal_adapter::AppendOnlyJournalAdapter;

    fn test_workflow_definition() -> WorkflowDefinition {
        WorkflowDefinition {
            id: "test_workflow".to_string(),
            name: "Test Workflow".to_string(),
            version: 1,
            steps: vec![
                WorkflowStepDef {
                    name: "step1".to_string(),
                    action: WorkflowAction::SendCommand {
                        target_service: "strategy_service".to_string(),
                        command: serde_json::json!({"type": "pause"}),
                    },
                    compensation: None,
                    timeout_secs: None,
                },
                WorkflowStepDef {
                    name: "step2".to_string(),
                    action: WorkflowAction::WaitForEvent {
                        event_type: "pause_acknowledged".to_string(),
                        timeout_secs: Some(10),
                    },
                    compensation: None,
                    timeout_secs: Some(10),
                },
            ],
        }
    }

    #[tokio::test]
    async fn test_workflow_trigger_and_advance() {
        let _ = std::fs::remove_dir_all("./test_data/journal_workflow");
        let journal = Arc::new(
            AppendOnlyJournalAdapter::new("./test_data/journal_workflow")
                .expect("Failed to create journal"),
        );
        let event_bus = Arc::new(MockEventBus::new());
        let mut engine = WorkflowEngine::new(journal, event_bus);

        engine.register_workflow(test_workflow_definition());

        let instance = engine.trigger_workflow("test_workflow").await;
        assert!(instance.is_ok());

        let instance = instance.unwrap();
        let result = engine.advance_workflow(&instance.instance_id).await;
        assert!(result.is_ok());
    }
}
