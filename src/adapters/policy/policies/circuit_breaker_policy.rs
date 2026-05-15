use crate::core::domain::policy::{PolicyContext, PolicyDecision};

pub struct CircuitBreakerPolicy {
    failure_threshold: u32,
    cooldown_secs: u64,
    half_open_probe_interval_secs: u64,
}

impl CircuitBreakerPolicy {
    pub fn new(failure_threshold: u32, cooldown_secs: u64, half_open_probe_interval_secs: u64) -> Self {
        Self {
            failure_threshold,
            cooldown_secs,
            half_open_probe_interval_secs,
        }
    }
}

impl Default for CircuitBreakerPolicy {
    fn default() -> Self {
        Self::new(5, 30, 10)
    }
}

impl CircuitBreakerPolicy {
    pub fn evaluate(&self, ctx: &PolicyContext) -> PolicyDecision {
        if let Some(failure_count) = ctx.metrics.get("consecutive_failures") {
            if *failure_count as u32 >= self.failure_threshold {
                return PolicyDecision::CircuitBreak {
                    cooldown_secs: self.cooldown_secs,
                };
            }
        }
        PolicyDecision::Allow
    }

    pub fn config(&self) -> (u32, u64, u64) {
        (
            self.failure_threshold,
            self.cooldown_secs,
            self.half_open_probe_interval_secs,
        )
    }
}
