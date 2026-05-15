use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    #[serde(default)]
    pub health_endpoint: Option<String>,
    #[serde(default)]
    pub control_endpoint: Option<String>,
    #[serde(default = "default_retry_max_attempts")]
    pub retry_max_attempts: u32,
    #[serde(default = "default_retry_base_backoff_ms")]
    pub retry_base_backoff_ms: u64,
    #[serde(default = "default_retry_max_backoff_ms")]
    pub retry_max_backoff_ms: u64,
    #[serde(default)]
    pub retry_jitter: bool,
}

fn default_retry_max_attempts() -> u32 {
    3
}
fn default_retry_base_backoff_ms() -> u64 {
    100
}
fn default_retry_max_backoff_ms() -> u64 {
    2000
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyParameters {
    pub values: HashMap<String, serde_json::Value>,
}
