mod mocks;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::mocks::{MockEventBus, MockServiceCommand};
    use orchestrator_service::core::application::audit_trail::AuditTrail;
    use orchestrator_service::core::domain::audit::{AuditAction, AuditActor, AuditEntry};
    use orchestrator_service::adapters::persistence::journal_adapter::AppendOnlyJournalAdapter;

    #[tokio::test]
    async fn test_audit_trail_record_and_query() {
        let _ = std::fs::remove_dir_all("./test_data/journal_audit");
        let journal = Arc::new(
            AppendOnlyJournalAdapter::new("./test_data/journal_audit")
                .expect("Failed to create journal"),
        );
        let audit_trail = AuditTrail::new(journal);

        let entry = AuditEntry::new(
            AuditActor::Human { user_id: "test_user".to_string() },
            AuditAction::KillSwitchActivated,
        )
        .with_reason("Test activation");

        let result = audit_trail.record(entry).await;
        assert!(result.is_ok());

        let entries = audit_trail.query(None, None, 10).await;
        assert!(entries.is_ok());
        assert!(!entries.unwrap().is_empty());
    }
}
