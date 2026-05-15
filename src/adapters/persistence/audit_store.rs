use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::domain::audit::AuditEntry;

pub struct AuditStore {
    entries: Arc<RwLock<Vec<AuditEntry>>>,
}

impl AuditStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn save(&self, entry: AuditEntry) {
        let mut entries = self.entries.write().await;
        entries.push(entry);
    }

    pub async fn query(
        &self,
        from: Option<u64>,
        to: Option<u64>,
        limit: usize,
    ) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|e| {
                let in_from = from.map_or(true, |f| e.timestamp_ns >= f);
                let in_to = to.map_or(true, |t| e.timestamp_ns <= t);
                in_from && in_to
            })
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn get_by_id(&self, id: &str) -> Option<AuditEntry> {
        let entries = self.entries.read().await;
        entries.iter().find(|e| e.id.to_string() == id).cloned()
    }
}
