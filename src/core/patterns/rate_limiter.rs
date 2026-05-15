use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct RateLimiter {
    entries: Arc<RwLock<HashMap<String, RateBucket>>>,
}

struct RateBucket {
    max_per_minute: u32,
    burst_capacity: u32,
    current_count: u32,
    window_start: Instant,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn configure(&self, service_id: &str, max_per_min: u32, burst: u32) {
        let mut entries = self.entries.write().await;
        entries.insert(
            service_id.to_string(),
            RateBucket {
                max_per_minute: max_per_min,
                burst_capacity: burst,
                current_count: 0,
                window_start: Instant::now(),
            },
        );
    }

    pub async fn check_rate(&self, service_id: &str) -> bool {
        let mut entries = self.entries.write().await;
        if let Some(bucket) = entries.get_mut(service_id) {
            if bucket.window_start.elapsed() >= Duration::from_secs(60) {
                bucket.current_count = 0;
                bucket.window_start = Instant::now();
            }

            if bucket.current_count < bucket.max_per_minute + bucket.burst_capacity {
                bucket.current_count += 1;
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    pub async fn current_usage(&self, service_id: &str) -> Option<(u32, u32)> {
        let entries = self.entries.read().await;
        entries.get(service_id).map(|b| (b.current_count, b.max_per_minute))
    }
}
