use std::collections::HashMap;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::policy::{PolicyContext, PolicyDecision, PolicyParameters};
use crate::core::ports::policy_port::IPolicyPort;

pub struct ConfigPolicyAdapter {
    parameters: HashMap<String, PolicyParameters>,
}

impl ConfigPolicyAdapter {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
        }
    }

    fn evaluate_drawdown(&self, ctx: &PolicyContext) -> PolicyDecision {
        if let Some(params) = self.parameters.get("drawdown") {
            if let Some(max_drawdown) = params.get_f64("max_drawdown_pct") {
                if let Some(current_drawdown) = ctx.metrics.get("current_drawdown_pct") {
                    if *current_drawdown > max_drawdown {
                        return PolicyDecision::CircuitBreak { cooldown_secs: 60 };
                    }
                }
            }
        }
        PolicyDecision::Allow
    }

    fn evaluate_rate_limit(&self, ctx: &PolicyContext) -> PolicyDecision {
        if let Some(params) = self.parameters.get("rate_limit") {
            if let Some(max_rate) = params.get_u32("global_max_orders_per_min") {
                if let Some(current_rate) = ctx.metrics.get("current_orders_per_min") {
                    if *current_rate as u32 > max_rate {
                        return PolicyDecision::Throttle {
                            max_rate: max_rate,
                        };
                    }
                }
            }
        }
        PolicyDecision::Allow
    }

    fn evaluate_volatility(&self, ctx: &PolicyContext) -> PolicyDecision {
        if let Some(params) = self.parameters.get("volatility") {
            if let Some(guard_active) = params.get_bool("guard_active") {
                if guard_active {
                    if let Some(threshold) = params.get_f64("threshold") {
                        if let Some(current_vol) = ctx.metrics.get("current_volatility") {
                            if *current_vol > threshold {
                                return PolicyDecision::Deny {
                                    reason: format!(
                                        "Volatility {} exceeds threshold {}",
                                        current_vol, threshold
                                    ),
                                };
                            }
                        }
                    }
                }
            }
        }
        PolicyDecision::Allow
    }
}

impl IPolicyPort for ConfigPolicyAdapter {
    fn evaluate(
        &self,
        policy_id: &str,
        ctx: &PolicyContext,
    ) -> Result<PolicyDecision, OrchestratorError> {
        let decision = match policy_id {
            "drawdown" => self.evaluate_drawdown(ctx),
            "rate_limit" => self.evaluate_rate_limit(ctx),
            "volatility" => self.evaluate_volatility(ctx),
            _ => PolicyDecision::Allow,
        };
        Ok(decision)
    }

    fn reload_policies(
        &mut self,
        params: &HashMap<String, PolicyParameters>,
    ) -> Result<(), OrchestratorError> {
        for (id, params) in params {
            self.parameters.insert(id.clone(), params.clone());
        }
        Ok(())
    }

    fn get_all_policies(&self) -> Vec<(String, bool)> {
        self.parameters.keys().map(|k| (k.clone(), true)).collect()
    }
}
