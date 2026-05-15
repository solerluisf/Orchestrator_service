use crate::core::domain::policy::{PolicyContext, PolicyDecision};

pub struct VolatilityPolicy {
    threshold: f64,
    active: bool,
}

impl VolatilityPolicy {
    pub fn new(threshold: f64, active: bool) -> Self {
        Self { threshold, active }
    }
}

impl Default for VolatilityPolicy {
    fn default() -> Self {
        Self::new(0.02, false)
    }
}

impl VolatilityPolicy {
    pub fn evaluate(&self, ctx: &PolicyContext) -> PolicyDecision {
        if !self.active {
            return PolicyDecision::Allow;
        }

        if let Some(current_vol) = ctx.metrics.get("current_volatility") {
            if *current_vol > self.threshold {
                return PolicyDecision::Deny {
                    reason: format!(
                        "Volatility {} exceeds threshold {}",
                        current_vol, self.threshold
                    ),
                };
            }
        }
        PolicyDecision::Allow
    }
}
