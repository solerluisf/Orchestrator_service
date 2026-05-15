use crate::core::domain::policy::{PolicyContext, PolicyDecision};

pub struct DrawdownPolicy {
    max_drawdown_pct: f64,
}

impl DrawdownPolicy {
    pub fn new(max_drawdown_pct: f64) -> Self {
        Self { max_drawdown_pct }
    }
}

impl Default for DrawdownPolicy {
    fn default() -> Self {
        Self::new(5.0)
    }
}

impl DrawdownPolicy {
    pub fn evaluate(&self, ctx: &PolicyContext) -> PolicyDecision {
        if let Some(current_drawdown) = ctx.metrics.get("current_drawdown_pct") {
            if *current_drawdown > self.max_drawdown_pct {
                return PolicyDecision::CircuitBreak { cooldown_secs: 60 };
            }
        }
        PolicyDecision::Allow
    }
}
