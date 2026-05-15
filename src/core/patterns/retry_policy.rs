use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_backoff: Duration,
    pub max_backoff: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl RetryPolicy {
    pub fn new(
        max_attempts: u32,
        base_backoff_ms: u64,
        max_backoff_ms: u64,
        backoff_multiplier: f64,
        jitter: bool,
    ) -> Self {
        Self {
            max_attempts,
            base_backoff: Duration::from_millis(base_backoff_ms),
            max_backoff: Duration::from_millis(max_backoff_ms),
            backoff_multiplier,
            jitter,
        }
    }

    pub fn default_policy() -> Self {
        Self::new(3, 100, 2000, 2.0, true)
    }

    pub fn backoff_for_attempt(&self, attempt: u32) -> Duration {
        let base = self.base_backoff.as_millis() as f64;
        let backoff = base * self.backoff_multiplier.powi(attempt as i32 - 1);
        let capped = backoff.min(self.max_backoff.as_millis() as f64);

        if self.jitter {
            let jitter_range = capped * 0.1;
            let jitter = (rand_factor() * jitter_range) as u64;
            Duration::from_millis(capped as u64 + jitter)
        } else {
            Duration::from_millis(capped as u64)
        }
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}

fn rand_factor() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}
