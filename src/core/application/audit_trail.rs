use std::sync::Arc;

use crate::core::domain::audit::AuditEntry;
use crate::core::domain::errors::OrchestratorError;
use crate::core::ports::journal_port::{IJournalPort, JournalEntry};

pub struct AuditTrail {
    journal: Arc<dyn IJournalPort>,
}

impl AuditTrail {
    pub fn new(journal: Arc<dyn IJournalPort>) -> Self {
        Self { journal }
    }

    pub async fn record(&self, entry: AuditEntry) -> Result<(), OrchestratorError> {
        let journal_entry = JournalEntry {
            sequence: None,
            timestamp_ns: entry.timestamp_ns,
            entry_type: "audit".to_string(),
            payload: serde_json::to_value(&entry)
                .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?,
            checksum: None,
        };

        self.journal.append(journal_entry).await?;
        tracing::debug!(audit_id = %entry.id, "Audit entry recorded");
        Ok(())
    }

    pub async fn query(
        &self,
        from: Option<u64>,
        to: Option<u64>,
        limit: usize,
    ) -> Result<Vec<AuditEntry>, OrchestratorError> {
        let entries = self.journal.read_latest(limit).await?;
        let mut audit_entries = Vec::new();

        for entry in entries {
            if entry.entry_type == "audit" {
                if let Ok(audit_entry) =
                    serde_json::from_value::<AuditEntry>(entry.payload.clone())
                {
                    let ts = audit_entry.timestamp_ns;
                    let in_range = from.map_or(true, |f| ts >= f) && to.map_or(true, |t| ts <= t);
                    if in_range {
                        audit_entries.push(audit_entry);
                    }
                }
            }
        }

        Ok(audit_entries)
    }
}
