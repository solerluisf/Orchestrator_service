mod mocks;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::mocks::{MockEventBus, MockServiceCommand};
    use orchestrator_service::core::domain::operation_mode::OperationMode;
    use orchestrator_service::core::application::mode_controller::ModeController;
    use orchestrator_service::adapters::persistence::journal_adapter::AppendOnlyJournalAdapter;

    fn setup() -> (Arc<AppendOnlyJournalAdapter>, Arc<MockEventBus>) {
        let _ = std::fs::remove_dir_all("./test_data/journal_mode");
        let journal = Arc::new(
            AppendOnlyJournalAdapter::new("./test_data/journal_mode")
                .expect("Failed to create journal"),
        );
        let event_bus = Arc::new(MockEventBus::new());
        (journal, event_bus)
    }

    #[tokio::test]
    async fn test_mode_transition_paper_to_live() {
        let (journal, event_bus) = setup();
        let controller = ModeController::new(journal, event_bus);

        let result = controller
            .transition(OperationMode::Live, "Test transition", "test", false)
            .await;

        assert!(result.is_ok());
        assert_eq!(controller.current_mode().await, OperationMode::Live);
    }

    #[tokio::test]
    async fn test_invalid_mode_transition() {
        let (journal, event_bus) = setup();
        let controller = ModeController::new(journal, event_bus);

        // Paper -> Live is valid, but Paper -> Paper (self) is not allowed
        let result = controller
            .transition(OperationMode::Paper, "Test invalid transition", "test", false)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dry_run_mode_transition() {
        let (journal, event_bus) = setup();
        let controller = ModeController::new(journal, event_bus);

        let result = controller
            .transition(OperationMode::Live, "Test dry run", "test", true)
            .await;

        assert!(result.is_ok());
        assert_eq!(controller.current_mode().await, OperationMode::Paper);
    }
}
