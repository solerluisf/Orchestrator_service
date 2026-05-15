use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct IdempotencyStore {
    processed_keys: Arc<RwLock<HashMap<String, u64>>>,
    ttl_secs: u64,
}

impl IdempotencyStore {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            processed_keys: Arc::new(RwLock::new(HashMap::new())),
            ttl_secs,
        }
    }

    pub async fn is_duplicate(&self, key: &str) -> bool {
        let store = self.processed_keys.read().await;
        if let Some(timestamp) = store.get(key) {
            let now = now_secs();
            now - timestamp < self.ttl_secs
        } else {
            false
        }
    }

    pub async fn mark_processed(&self, key: &str) {
        let mut store = self.processed_keys.write().await;
        store.insert(key.to_string(), now_secs());
    }

    pub async fn cleanup(&self) {
        let now = now_secs();
        let mut store = self.processed_keys.write().await;
        store.retain(|_, ts| now - *ts < self.ttl_secs);
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
