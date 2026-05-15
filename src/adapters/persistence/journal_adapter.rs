use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::io::Write;

use async_trait::async_trait;

use crate::core::domain::errors::OrchestratorError;
use crate::core::ports::journal_port::{IJournalPort, JournalEntry, SequenceNumber};

pub struct AppendOnlyJournalAdapter {
    dir: PathBuf,
    current_sequence: Arc<Mutex<SequenceNumber>>,
}

impl AppendOnlyJournalAdapter {
    pub fn new(dir: &str) -> Result<Self, OrchestratorError> {
        let path = PathBuf::from(dir);
        std::fs::create_dir_all(&path)
            .map_err(|e| OrchestratorError::Journal(e.to_string()))?;

        Ok(Self {
            dir: path,
            current_sequence: Arc::new(Mutex::new(0)),
        })
    }

    fn today_file(&self) -> PathBuf {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        self.dir.join(format!("journal_{}.jsonl", today))
    }

    async fn read_all_entries(&self) -> Result<Vec<JournalEntry>, OrchestratorError> {
        let mut entries = Vec::new();
        let mut entries_dir: Vec<_> = std::fs::read_dir(&self.dir)
            .map_err(|e| OrchestratorError::Journal(e.to_string()))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "jsonl").unwrap_or(false))
            .collect();

        entries_dir.sort_by_key(|e| e.path());

        for entry in entries_dir {
            let content = std::fs::read_to_string(entry.path())
                .map_err(|e| OrchestratorError::Journal(e.to_string()))?;
            for line in content.lines() {
                if !line.trim().is_empty() {
                    if let Ok(journal_entry) = serde_json::from_str::<JournalEntry>(line) {
                        entries.push(journal_entry);
                    }
                }
            }
        }

        Ok(entries)
    }
}

#[async_trait]
impl IJournalPort for AppendOnlyJournalAdapter {
    async fn append(&self, mut entry: JournalEntry) -> Result<SequenceNumber, OrchestratorError> {
        let mut seq = self.current_sequence.lock().await;
        *seq += 1;
        entry.sequence = Some(*seq);

        let json = serde_json::to_string(&entry)
            .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?;
        let line = format!("{}\n", json);

        let file = self.today_file();
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file)
            .map_err(|e| OrchestratorError::Journal(e.to_string()))?
            .write_all(line.as_bytes())
            .map_err(|e| OrchestratorError::Journal(e.to_string()))?;

        Ok(*seq)
    }

    async fn read_from(
        &self,
        seq: SequenceNumber,
        limit: usize,
    ) -> Result<Vec<JournalEntry>, OrchestratorError> {
        let all = self.read_all_entries().await?;
        Ok(all
            .into_iter()
            .filter(|e| e.sequence.map_or(false, |s| s >= seq))
            .take(limit)
            .collect())
    }

    async fn read_latest(&self, limit: usize) -> Result<Vec<JournalEntry>, OrchestratorError> {
        let all = self.read_all_entries().await?;
        let len = all.len();
        Ok(all.into_iter().skip(len.saturating_sub(limit)).collect())
    }

    async fn get_sequence(&self) -> Result<SequenceNumber, OrchestratorError> {
        let seq = self.current_sequence.lock().await;
        Ok(*seq)
    }
}
