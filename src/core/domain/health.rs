use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl HealthStatus {
    pub fn is_operational(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub service_id: String,
    pub status: HealthStatus,
    pub last_heartbeat_ns: Option<u64>,
    pub uptime_secs: Option<u64>,
    pub details: Option<serde_json::Value>,
}

impl ServiceHealth {
    pub fn unknown(service_id: &str) -> Self {
        Self {
            service_id: service_id.to_string(),
            status: HealthStatus::Unknown,
            last_heartbeat_ns: None,
            uptime_secs: None,
            details: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub timestamp_ns: u64,
    pub services: HashMap<String, ServiceHealth>,
    pub overall_status: HealthStatus,
}

impl HealthSnapshot {
    pub fn new() -> Self {
        Self {
            timestamp_ns: now_nanos(),
            services: HashMap::new(),
            overall_status: HealthStatus::Unknown,
        }
    }

    pub fn update_service(&mut self, service_id: String, health: ServiceHealth) {
        self.services.insert(service_id, health);
        self.recalculate_overall();
    }

    fn recalculate_overall(&mut self) {
        if self.services.is_empty() {
            self.overall_status = HealthStatus::Unknown;
            return;
        }

        let unhealthy = self
            .services
            .values()
            .any(|h| h.status == HealthStatus::Unhealthy);
        let degraded = self
            .services
            .values()
            .any(|h| h.status == HealthStatus::Degraded);

        self.overall_status = if unhealthy {
            HealthStatus::Unhealthy
        } else if degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        self.timestamp_ns = now_nanos();
    }
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
