use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use async_trait::async_trait;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::health::{HealthStatus, ServiceHealth};
use crate::core::ports::health_port::IHealthPort;

pub struct BusHealthAdapter {
    heartbeats: Arc<RwLock<HashMap<String, u64>>>,
    ttl_ns: u64,
}

impl BusHealthAdapter {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            heartbeats: Arc::new(RwLock::new(HashMap::new())),
            ttl_ns: ttl_secs * 1_000_000_000,
        }
    }

    pub async fn record_heartbeat(&self, service_id: &str) {
        let mut heartbeats = self.heartbeats.write().await;
        heartbeats.insert(service_id.to_string(), now_nanos());
    }
}

#[async_trait]
impl IHealthPort for BusHealthAdapter {
    async fn get_health(&self, service_id: &str) -> Result<ServiceHealth, OrchestratorError> {
        let heartbeats = self.heartbeats.read().await;
        let now = now_nanos();

        match heartbeats.get(service_id) {
            Some(last_heartbeat) => {
                let age = now.saturating_sub(*last_heartbeat);
                let status = if age < self.ttl_ns {
                    HealthStatus::Healthy
                } else if age < self.ttl_ns * 2 {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Unhealthy
                };

                Ok(ServiceHealth {
                    service_id: service_id.to_string(),
                    status,
                    last_heartbeat_ns: Some(*last_heartbeat),
                    uptime_secs: None,
                    details: None,
                })
            }
            None => Ok(ServiceHealth::unknown(service_id)),
        }
    }

    async fn get_all_health(&self) -> Result<HashMap<String, ServiceHealth>, OrchestratorError> {
        let heartbeats = self.heartbeats.read().await;
        let mut results = HashMap::new();
        for service_id in heartbeats.keys() {
            let health = self.get_health(service_id).await?;
            results.insert(service_id.clone(), health);
        }
        Ok(results)
    }
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
