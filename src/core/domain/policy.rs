use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait Policy: Send + Sync {
    fn id(&self) -> &str;
    fn evaluate(&self, ctx: &PolicyContext) -> PolicyDecision;
    fn update_parameters(&mut self, params: PolicyParameters);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContext {
    pub service_id: Option<String>,
    pub operation_mode: Option<String>,
    pub metrics: HashMap<String, f64>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "decision")]
pub enum PolicyDecision {
    Allow,
    Deny { reason: String },
    Throttle { max_rate: u32 },
    CircuitBreak { cooldown_secs: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyParameters {
    pub values: HashMap<String, serde_json::Value>,
}

impl PolicyParameters {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.values.get(key).and_then(|v| v.as_f64())
    }

    pub fn get_u32(&self, key: &str) -> Option<u32> {
        self.values.get(key).and_then(|v| v.as_u64()).map(|v| v as u32)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.get(key).and_then(|v| v.as_bool())
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.values.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySummary {
    pub policy_id: String,
    pub enabled: bool,
    pub parameters: PolicyParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDetail {
    pub policy_id: String,
    pub enabled: bool,
    pub parameters: PolicyParameters,
    pub description: String,
}
