use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    cooldown_duration: Duration,
    half_open_interval: Duration,
    state: RwLock<CircuitState>,
    failure_count: AtomicU64,
    last_failure_time: RwLock<Option<Instant>>,
    opened_at: RwLock<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(
        failure_threshold: u32,
        cooldown_secs: u64,
        half_open_probe_interval_secs: u64,
    ) -> Arc<Self> {
        Arc::new(Self {
            failure_threshold,
            cooldown_duration: Duration::from_secs(cooldown_secs),
            half_open_interval: Duration::from_secs(half_open_probe_interval_secs),
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU64::new(0),
            last_failure_time: RwLock::new(None),
            opened_at: RwLock::new(None),
        })
    }

    pub async fn can_execute(&self) -> bool {
        let state = *self.state.read().await;
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(opened) = *self.opened_at.read().await {
                    if opened.elapsed() >= self.cooldown_duration {
                        *self.state.write().await = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub async fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        *self.state.write().await = CircuitState::Closed;
    }

    pub async fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure_time.write().await = Some(Instant::now());

        let mut state = self.state.write().await;
        if failures >= self.failure_threshold as u64 {
            *state = CircuitState::Open;
            *self.opened_at.write().await = Some(Instant::now());
        }
    }

    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }

    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::SeqCst)
    }

    pub async fn reset(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        *self.state.write().await = CircuitState::Closed;
        *self.last_failure_time.write().await = None;
        *self.opened_at.write().await = None;
    }
}
