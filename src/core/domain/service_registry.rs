use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceId(pub String);

impl ServiceId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ServiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDescriptor {
    pub id: ServiceId,
    pub name: String,
    pub version: String,
    pub endpoint: Option<String>,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistry {
    pub services: HashMap<String, ServiceDescriptor>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register(&mut self, descriptor: ServiceDescriptor) {
        self.services
            .insert(descriptor.id.0.clone(), descriptor);
    }

    pub fn deregister(&mut self, service_id: &str) -> Option<ServiceDescriptor> {
        self.services.remove(service_id)
    }

    pub fn get(&self, service_id: &str) -> Option<&ServiceDescriptor> {
        self.services.get(service_id)
    }

    pub fn all_ids(&self) -> Vec<&str> {
        self.services.keys().map(|s| s.as_str()).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConnectorMeta {
    pub service_id: ServiceId,
    pub connector_type: String,
    pub health_endpoint: Option<String>,
    pub control_endpoint: Option<String>,
    pub retry_config: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_backoff_ms: 100,
            max_backoff_ms: 2000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}
