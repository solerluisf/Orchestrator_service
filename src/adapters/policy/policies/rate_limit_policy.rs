use crate::core::domain::policy::{PolicyContext, PolicyDecision};

pub struct RateLimitPolicy {
    max_per_min: u32,
}

impl RateLimitPolicy {
    pub fn new(max_per_min: u32) -> Self {
        Self { max_per_min }
    }
}

impl Default for RateLimitPolicy {
    fn default() -> Self {
        Self::new(500)
    }
}

impl RateLimitPolicy {
    pub fn evaluate(&self, ctx: &PolicyContext) -> PolicyDecision {
        if let Some(current_rate) = ctx.metrics.get("current_orders_per_min") {
            if *current_rate as u32 > self.max_per_min {
                return PolicyDecision::Throttle {
                    max_rate: self.max_per_min,
                };
            }
        }
        PolicyDecision::Allow
    }
}
