use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::health::HealthSnapshot;

pub struct QueryHandler {
    health_snapshot: Arc<RwLock<HealthSnapshot>>,
    kill_switch_active: Arc<std::sync::atomic::AtomicBool>,
}

impl QueryHandler {
    pub fn new(
        health_snapshot: Arc<RwLock<HealthSnapshot>>,
        kill_switch_active: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self {
            health_snapshot,
            kill_switch_active,
        }
    }

    pub async fn get_system_health(&self) -> serde_json::Value {
        let snapshot = self.health_snapshot.read().await;
        serde_json::json!({
            "overall_status": format!("{:?}", snapshot.overall_status),
            "services": snapshot.services.len(),
            "timestamp_ns": snapshot.timestamp_ns,
        })
    }

    pub async fn get_service_health(&self, service_id: &str) -> Result<serde_json::Value, OrchestratorError> {
        let snapshot = self.health_snapshot.read().await;
        match snapshot.services.get(service_id) {
            Some(health) => Ok(serde_json::to_value(health)
                .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?),
            None => Ok(serde_json::json!({
                "service_id": service_id,
                "status": "Unknown",
                "message": "No health data available"
            })),
        }
    }

    pub async fn get_all_service_health(&self) -> Result<serde_json::Value, OrchestratorError> {
        let snapshot = self.health_snapshot.read().await;
        Ok(serde_json::to_value(&snapshot.services)
            .map_err(|e| OrchestratorError::SerializationError(e.to_string()))?)
    }

    pub async fn get_kill_switch_status(&self) -> serde_json::Value {
        let active = self.kill_switch_active.load(std::sync::atomic::Ordering::SeqCst);
        serde_json::json!({
            "active": active,
            "state": if active { "active" } else { "inactive" }
        })
    }
}
