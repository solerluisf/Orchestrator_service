use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::domain::health::HealthSnapshot;
use crate::core::ports::health_port::IHealthPort;
use crate::core::ports::metrics_port::IMetricsPort;

pub struct HealthAggregator {
    health_port: Arc<dyn IHealthPort>,
    health_snapshot: Arc<RwLock<HealthSnapshot>>,
    metrics: Arc<dyn IMetricsPort>,
    poll_interval_secs: u64,
}

impl HealthAggregator {
    pub fn new(
        health_port: Arc<dyn IHealthPort>,
        health_snapshot: Arc<RwLock<HealthSnapshot>>,
        metrics: Arc<dyn IMetricsPort>,
    ) -> Self {
        Self {
            health_port,
            health_snapshot,
            metrics,
            poll_interval_secs: 5,
        }
    }

    pub async fn start_polling(&self) {
        let health_port = self.health_port.clone();
        let snapshot = self.health_snapshot.clone();
        let metrics = self.metrics.clone();
        let interval = self.poll_interval_secs;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(
                std::time::Duration::from_secs(interval),
            );

            loop {
                interval_timer.tick().await;

                match health_port.get_all_health().await {
                    Ok(health_map) => {
                        let mut snap = snapshot.write().await;
                        for (service_id, health) in health_map {
                            metrics.record_gauge(
                                "orchestrator_service_health",
                                &[("service", &service_id)],
                                match health.status {
                                    crate::core::domain::health::HealthStatus::Healthy => 1.0,
                                    crate::core::domain::health::HealthStatus::Degraded => 0.5,
                                    crate::core::domain::health::HealthStatus::Unhealthy => 0.0,
                                    crate::core::domain::health::HealthStatus::Unknown => -1.0,
                                },
                            );
                            snap.update_service(service_id, health);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Health poll failed: {}", e);
                    }
                }
            }
        });
    }

    pub async fn get_snapshot(&self) -> HealthSnapshot {
        self.health_snapshot.read().await.clone()
    }
}
