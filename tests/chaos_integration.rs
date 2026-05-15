mod mocks;

#[cfg(test)]
mod chaos_tests {
    use std::sync::Arc;
    use super::mocks::{MockEventBus, MockServiceCommand};
    use orchestrator_service::core::application::kill_switch_controller::KillSwitchController;
    use orchestrator_service::core::application::mode_controller::ModeController;
    use orchestrator_service::core::application::policy_engine::PolicyEngine;
    use orchestrator_service::core::application::saga_coordinator::SagaCoordinator;
    use orchestrator_service::core::application::workflow_engine::WorkflowEngine;
    use orchestrator_service::core::domain::operation_mode::OperationMode;
    use orchestrator_service::core::domain::policy::PolicyParameters;
    use orchestrator_service::core::domain::workflow::{WorkflowDefinition, WorkflowStepDef, WorkflowAction};
    use orchestrator_service::core::domain::saga::{SagaDefinition, SagaStepDef};
    use orchestrator_service::adapters::persistence::journal_adapter::AppendOnlyJournalAdapter;
    use orchestrator_service::core::ports::journal_port::IJournalPort;
    use std::time::Instant;

    fn make_journal(dir: &str) -> Arc<AppendOnlyJournalAdapter> {
        let _ = std::fs::remove_dir_all(dir);
        Arc::new(AppendOnlyJournalAdapter::new(dir).expect("Failed to create journal"))
    }

    #[tokio::test]
    async fn chaos_rapid_kill_switch_toggles() {
        let journal = make_journal("./test_data/chaos_kill_toggles");
        let event_bus = Arc::new(MockEventBus::new());
        let service_command = Arc::new(MockServiceCommand);
        let controller = KillSwitchController::new(journal, event_bus, service_command);

        let start = Instant::now();
        for i in 0..100 {
            if i % 2 == 0 {
                controller.activate("chaos test", "chaos").await.unwrap();
            } else {
                controller.clear("chaos test", "chaos").await.unwrap();
            }
        }
        let elapsed = start.elapsed();
        assert!(!controller.is_active());
        assert!(elapsed.as_millis() < 5000, "100 toggles took too long: {:?}", elapsed);
    }

    #[tokio::test]
    async fn chaos_rapid_mode_transitions() {
        let journal = make_journal("./test_data/chaos_mode_transitions");
        let event_bus = Arc::new(MockEventBus::new());
        let controller = ModeController::new(journal, event_bus);

        let start = Instant::now();
        // Paper -> Live -> Paper -> Live -> ...
        for i in 0..50 {
            if i % 2 == 0 {
                controller.transition(OperationMode::Live, "chaos", "chaos", false).await.unwrap();
            } else {
                controller.transition(OperationMode::Paper, "chaos", "chaos", false).await.unwrap();
            }
        }
        let elapsed = start.elapsed();
        assert_eq!(controller.current_mode().await, OperationMode::Paper);
        assert!(elapsed.as_millis() < 5000, "50 mode transitions took too long: {:?}", elapsed);
    }

    #[tokio::test]
    async fn chaos_concurrent_policy_updates() {
        let journal = make_journal("./test_data/chaos_policy_updates");
        let event_bus = Arc::new(MockEventBus::new());
        let engine = PolicyEngine::new(journal, event_bus);

        let start = Instant::now();
        for i in 0..100 {
            let mut params = PolicyParameters::new();
            params.values.insert("value".to_string(), serde_json::json!(i));
            engine.update_policy(&format!("policy_{}", i % 5), params).await.unwrap();
        }
        let elapsed = start.elapsed();
        assert!(engine.list_policies().await.len() > 0);
        assert!(elapsed.as_millis() < 5000, "100 policy updates took too long: {:?}", elapsed);
    }

    #[tokio::test]
    async fn chaos_workflow_lifecycle() {
        let journal = make_journal("./test_data/chaos_workflow");
        let event_bus = Arc::new(MockEventBus::new());
        let mut engine = WorkflowEngine::new(journal, event_bus);

        let def = WorkflowDefinition {
            id: "chaos_wf".to_string(),
            name: "Chaos Workflow".to_string(),
            version: 1,
            steps: vec![
                WorkflowStepDef {
                    name: "step1".to_string(),
                    action: WorkflowAction::SendCommand {
                        target_service: "svc".to_string(),
                        command: serde_json::json!({}),
                    },
                    compensation: None,
                    timeout_secs: None,
                },
            ],
        };
        engine.register_workflow(def);

        let start = Instant::now();
        for _ in 0..50 {
            let instance = engine.trigger_workflow("chaos_wf").await.unwrap();
            engine.advance_workflow(&instance.instance_id).await.unwrap();
        }
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 5000, "50 workflow cycles took too long: {:?}", elapsed);
    }

    #[tokio::test]
    async fn chaos_saga_start_advance_compensate() {
        let journal = make_journal("./test_data/chaos_saga");
        let event_bus = Arc::new(MockEventBus::new());
        let mut coordinator = SagaCoordinator::new(journal, event_bus);

        let def = SagaDefinition {
            id: "chaos_saga".to_string(),
            saga_type: "chaos".to_string(),
            steps: vec![
                SagaStepDef {
                    name: "step1".to_string(),
                    action: WorkflowAction::SendCommand {
                        target_service: "svc".to_string(),
                        command: serde_json::json!({}),
                    },
                    compensation: WorkflowAction::SendCommand {
                        target_service: "svc".to_string(),
                        command: serde_json::json!({}),
                    },
                    timeout_secs: None,
                },
                SagaStepDef {
                    name: "step2".to_string(),
                    action: WorkflowAction::SendCommand {
                        target_service: "svc".to_string(),
                        command: serde_json::json!({}),
                    },
                    compensation: WorkflowAction::SendCommand {
                        target_service: "svc".to_string(),
                        command: serde_json::json!({}),
                    },
                    timeout_secs: None,
                },
            ],
        };
        coordinator.register_saga(def);

        let start = Instant::now();
        for _ in 0..50 {
            let instance = coordinator.start_saga("chaos").await.unwrap();
            coordinator.advance_saga(&instance.instance_id).await.unwrap();
            coordinator.compensate_saga(&instance.instance_id).await.unwrap();
        }
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 5000, "50 saga cycles took too long: {:?}", elapsed);
    }

    #[tokio::test]
    async fn chaos_journal_integrity_under_load() {
        let journal = make_journal("./test_data/chaos_journal_integrity");
        let event_bus = Arc::new(MockEventBus::new());
        let service_command = Arc::new(MockServiceCommand);
        let controller = KillSwitchController::new(journal.clone(), event_bus, service_command);

        // Write a bunch of entries
        for i in 0..200 {
            if i % 2 == 0 {
                controller.activate("integrity test", "test").await.unwrap();
            } else {
                controller.clear("integrity test", "test").await.unwrap();
            }
        }

        // Read all entries back
        let entries = journal.read_latest(10000).await.unwrap();
        assert_eq!(entries.len(), 200, "Expected 200 journal entries (100 activates + 100 clears)");

        // Verify all entries have valid JSON
        for entry in &entries {
            assert!(entry.entry_type == "kill_switch_activate" || entry.entry_type == "kill_switch_clear");
            assert!(entry.timestamp_ns > 0);
        }
    }
}
