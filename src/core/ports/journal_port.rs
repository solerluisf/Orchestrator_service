use async_trait::async_trait;

use crate::core::domain::errors::OrchestratorError;

pub type SequenceNumber = u64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JournalEntry {
    pub sequence: Option<SequenceNumber>,
    pub timestamp_ns: u64,
    pub entry_type: String,
    pub payload: serde_json::Value,
    pub checksum: Option<String>,
}

#[async_trait]
pub trait IJournalPort: Send + Sync {
    async fn append(&self, entry: JournalEntry) -> Result<SequenceNumber, OrchestratorError>;

    async fn read_from(
        &self,
        seq: SequenceNumber,
        limit: usize,
    ) -> Result<Vec<JournalEntry>, OrchestratorError>;

    async fn read_latest(&self, limit: usize) -> Result<Vec<JournalEntry>, OrchestratorError>;

    async fn get_sequence(&self) -> Result<SequenceNumber, OrchestratorError>;
}
