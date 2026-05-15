use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyConfig {
    #[serde(default)]
    pub drawdown: DrawdownPolicyConfig,

    #[serde(default)]
    pub volatility: VolatilityPolicyConfig,

    #[serde(default)]
    pub rate_limit: RateLimitPolicyConfig,

    #[serde(default)]
    pub mode_transition: ModeTransitionPolicyConfig,

    #[serde(default)]
    pub circuit_breaker: CircuitBreakerPolicyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownPolicyConfig {
    pub max_drawdown_pct: f64,
}

impl Default for DrawdownPolicyConfig {
    fn default() -> Self {
        Self {
            max_drawdown_pct: 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityPolicyConfig {
    pub threshold: f64,
    pub guard_active: bool,
}

impl Default for VolatilityPolicyConfig {
    fn default() -> Self {
        Self {
            threshold: 0.02,
            guard_active: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitPolicyConfig {
    pub global_max_orders_per_min: u32,
    pub per_service: HashMap<String, u32>,
}

impl Default for RateLimitPolicyConfig {
    fn default() -> Self {
        Self {
            global_max_orders_per_min: 500,
            per_service: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeTransitionPolicyConfig {
    pub allowed_transitions: Vec<String>,
}

impl Default for ModeTransitionPolicyConfig {
    fn default() -> Self {
        Self {
            allowed_transitions: vec![
                "offline->paper".to_string(),
                "paper->live".to_string(),
                "paper->readonly".to_string(),
                "live->paper".to_string(),
                "live->readonly".to_string(),
                "readonly->paper".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerPolicyConfig {
    pub failure_threshold: u32,
    pub cooldown_secs: u64,
    pub half_open_probe_interval_secs: u64,
}

impl Default for CircuitBreakerPolicyConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            cooldown_secs: 30,
            half_open_probe_interval_secs: 10,
        }
    }
}
