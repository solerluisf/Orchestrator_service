mod mocks;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::mocks::{MockEventBus, MockServiceCommand};
    use orchestrator_service::core::application::saga_coordinator::SagaCoordinator;
    use orchestrator_service::core::domain::saga::{SagaDefinition, SagaStepDef};
    use orchestrator_service::core::domain::workflow::WorkflowAction;
    use orchestrator_service::adapters::persistence::journal_adapter::AppendOnlyJournalAdapter;

    fn test_saga_definition() -> SagaDefinition {
        SagaDefinition {
            id: "test_saga".to_string(),
            saga_type: "mode_transition".to_string(),
            steps: vec![
                SagaStepDef {
                    name: "step1".to_string(),
                    action: WorkflowAction::SendCommand {
                        target_service: "strategy_service".to_string(),
                        command: serde_json::json!({"type": "pause"}),
                    },
                    compensation: WorkflowAction::SendCommand {
                        target_service: "strategy_service".to_string(),
                        command: serde_json::json!({"type": "resume"}),
                    },
                },
                SagaStepDef {
                    name: "step2".to_string(),
                    action: WorkflowAction::SendCommand {
                        target_service: "risk_service".to_string(),
                        command: serde_json::json!({"type": "pause"}),
                    },
                    compensation: WorkflowAction::SendCommand {
                        target_service: "risk_service".to_string(),
                        command: serde_json::json!({"type": "resume"}),
                    },
                },
            ],
        }
    }

    #[tokio::test]
    async fn test_saga_start_and_advance() {
        let _ = std::fs::remove_dir_all("./test_data/journal_saga");
        let journal = Arc::new(
            AppendOnlyJournalAdapter::new("./test_data/journal_saga")
                .expect("Failed to create journal"),
        );
        let event_bus = Arc::new(MockEventBus::new());
        let mut coordinator = SagaCoordinator::new(journal, event_bus);

        coordinator.register_saga(test_saga_definition());

        let instance = coordinator.start_saga("mode_transition").await;
        assert!(instance.is_ok());

        let instance = instance.unwrap();
        let result = coordinator.advance_saga(&instance.instance_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_saga_compensation() {
        let _ = std::fs::remove_dir_all("./test_data/journal_saga2");
        let journal = Arc::new(
            AppendOnlyJournalAdapter::new("./test_data/journal_saga2")
                .expect("Failed to create journal"),
        );
        let event_bus = Arc::new(MockEventBus::new());
        let mut coordinator = SagaCoordinator::new(journal, event_bus);

        coordinator.register_saga(test_saga_definition());

        let instance = coordinator.start_saga("mode_transition").await.unwrap();
        coordinator.advance_saga(&instance.instance_id).await.unwrap();

        let result = coordinator.compensate_saga(&instance.instance_id).await;
        assert!(result.is_ok());
    }
}
