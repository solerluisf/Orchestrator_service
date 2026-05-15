mod mocks;

#[cfg(test)]
mod load_tests {
    use std::sync::Arc;
    use super::mocks::{MockEventBus, MockServiceCommand};
    use orchestrator_service::core::application::kill_switch_controller::KillSwitchController;
    use orchestrator_service::core::application::mode_controller::ModeController;
    use orchestrator_service::core::application::policy_engine::PolicyEngine;
    use orchestrator_service::core::domain::operation_mode::OperationMode;
    use orchestrator_service::core::domain::policy::PolicyParameters;
    use orchestrator_service::adapters::persistence::journal_adapter::AppendOnlyJournalAdapter;
    use std::time::Instant;

    fn make_journal(dir: &str) -> Arc<AppendOnlyJournalAdapter> {
        let _ = std::fs::remove_dir_all(dir);
        Arc::new(AppendOnlyJournalAdapter::new(dir).expect("Failed to create journal"))
    }

    #[tokio::test]
    async fn load_kill_switch_throughput() {
        let journal = make_journal("./test_data/load_kill_switch");
        let event_bus = Arc::new(MockEventBus::new());
        let service_command = Arc::new(MockServiceCommand);
        let controller = KillSwitchController::new(journal, event_bus, service_command);

        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            controller.activate("load test", "load").await.unwrap();
            controller.clear("load test", "load").await.unwrap();
        }
        let elapsed = start.elapsed();
        let ops_per_sec = (iterations as f64 * 2.0) / elapsed.as_secs_f64();
        println!("Kill switch: {} ops in {:?} ({:.0} ops/sec)", iterations * 2, elapsed, ops_per_sec);
        assert!(ops_per_sec > 100.0, "Throughput too low: {:.0} ops/sec", ops_per_sec);
    }

    #[tokio::test]
    async fn load_mode_transition_throughput() {
        let journal = make_journal("./test_data/load_mode");
        let event_bus = Arc::new(MockEventBus::new());
        let controller = ModeController::new(journal, event_bus);

        let iterations = 500;
        let start = Instant::now();
        for _ in 0..iterations {
            controller.transition(OperationMode::Live, "load", "load", false).await.unwrap();
            controller.transition(OperationMode::Paper, "load", "load", false).await.unwrap();
        }
        let elapsed = start.elapsed();
        let ops_per_sec = (iterations as f64 * 2.0) / elapsed.as_secs_f64();
        println!("Mode transitions: {} ops in {:?} ({:.0} ops/sec)", iterations * 2, elapsed, ops_per_sec);
        assert!(ops_per_sec > 50.0, "Throughput too low: {:.0} ops/sec", ops_per_sec);
    }

    #[tokio::test]
    async fn load_policy_update_throughput() {
        let journal = make_journal("./test_data/load_policy");
        let event_bus = Arc::new(MockEventBus::new());
        let engine = PolicyEngine::new(journal, event_bus);

        let iterations = 1000;
        let start = Instant::now();
        for i in 0..iterations {
            let mut params = PolicyParameters::new();
            params.values.insert("value".to_string(), serde_json::json!(i));
            engine.update_policy(&format!("policy_{}", i % 10), params).await.unwrap();
        }
        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
        println!("Policy updates: {} ops in {:?} ({:.0} ops/sec)", iterations, elapsed, ops_per_sec);
        assert!(ops_per_sec > 100.0, "Throughput too low: {:.0} ops/sec", ops_per_sec);
    }

    #[tokio::test]
    async fn load_journal_write_throughput() {
        let journal = make_journal("./test_data/load_journal");

        let iterations = 5000;
        let start = Instant::now();
        for i in 0..iterations {
            let entry = orchestrator_service::core::ports::journal_port::JournalEntry {
                sequence: None,
                timestamp_ns: i as u64,
                entry_type: "load_test".to_string(),
                payload: serde_json::json!({"index": i}),
                checksum: None,
            };
            journal.append(entry).await.unwrap();
        }
        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
        println!("Journal writes: {} ops in {:?} ({:.0} ops/sec)", iterations, elapsed, ops_per_sec);
        assert!(ops_per_sec > 500.0, "Throughput too low: {:.0} ops/sec", ops_per_sec);

        // Verify read throughput
        let start = Instant::now();
        let entries = journal.read_latest(iterations as usize).await.unwrap();
        let read_elapsed = start.elapsed();
        assert_eq!(entries.len(), iterations);
        println!("Journal read ({} entries): {:?}", iterations, read_elapsed);
    }
}
