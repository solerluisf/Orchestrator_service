mod mocks;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::mocks::{MockEventBus, MockServiceCommand};
    use orchestrator_service::core::application::kill_switch_controller::KillSwitchController;
    use orchestrator_service::adapters::persistence::journal_adapter::AppendOnlyJournalAdapter;

    fn make_controller(dir: &str) -> (KillSwitchController, Arc<MockEventBus>) {
        let _ = std::fs::remove_dir_all(dir);
        let journal = Arc::new(
            AppendOnlyJournalAdapter::new(dir).expect("Failed to create journal"),
        );
        let event_bus = Arc::new(MockEventBus::new());
        let service_command = Arc::new(MockServiceCommand);
        let controller = KillSwitchController::new(journal, event_bus.clone(), service_command);
        (controller, event_bus)
    }

    #[tokio::test]
    async fn test_kill_switch_activate() {
        let (controller, event_bus) = make_controller("./test_data/journal_killswitch_activate");

        let result = controller.activate("Test activation", "test_user").await;

        assert!(result.is_ok());
        assert!(controller.is_active());
        assert!(!event_bus.published_events().is_empty());
    }

    #[tokio::test]
    async fn test_kill_switch_clear() {
        let (controller, event_bus) = make_controller("./test_data/journal_killswitch_clear");

        controller.activate("Test", "test").await.unwrap();
        assert!(controller.is_active());

        let result = controller.clear("Test clear", "test_user").await;

        assert!(result.is_ok());
        assert!(!controller.is_active());
        assert!(event_bus.published_events().len() >= 2);
    }

    #[tokio::test]
    async fn test_kill_switch_restore_from_journal() {
        let dir = "./test_data/journal_killswitch_restore";
        let _ = std::fs::remove_dir_all(dir);

        let journal = Arc::new(
            AppendOnlyJournalAdapter::new(dir).expect("Failed to create journal"),
        );
        let event_bus = Arc::new(MockEventBus::new());
        let service_command = Arc::new(MockServiceCommand);
        let controller = KillSwitchController::new(journal.clone(), event_bus, service_command);

        controller.activate("Test", "test").await.unwrap();

        let event_bus2 = Arc::new(MockEventBus::new());
        let service_command2 = Arc::new(MockServiceCommand);
        let controller2 = KillSwitchController::new(journal, event_bus2, service_command2);
        controller2.restore_from_journal().await.unwrap();

        assert!(controller2.is_active());
    }
}
