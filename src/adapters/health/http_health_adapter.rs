use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::health::{HealthStatus, ServiceHealth};
use crate::core::ports::health_port::IHealthPort;

pub struct HttpHealthAdapter {
    client: reqwest::Client,
    endpoints: HashMap<String, String>,
}

impl HttpHealthAdapter {
    pub fn new(endpoints: HashMap<String, String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            endpoints,
        }
    }
}

#[async_trait]
impl IHealthPort for HttpHealthAdapter {
    async fn get_health(&self, service_id: &str) -> Result<ServiceHealth, OrchestratorError> {
        let endpoint = self.endpoints.get(service_id).ok_or_else(|| {
            OrchestratorError::HealthCheckFailed(format!(
                "No health endpoint for service: {}",
                service_id
            ))
        })?;

        match self.client.get(endpoint).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(ServiceHealth {
                        service_id: service_id.to_string(),
                        status: HealthStatus::Healthy,
                        last_heartbeat_ns: Some(now_nanos()),
                        uptime_secs: None,
                        details: None,
                    })
                } else {
                    Ok(ServiceHealth {
                        service_id: service_id.to_string(),
                        status: HealthStatus::Degraded,
                        last_heartbeat_ns: Some(now_nanos()),
                        uptime_secs: None,
                        details: None,
                    })
                }
            }
            Err(_) => Ok(ServiceHealth {
                service_id: service_id.to_string(),
                status: HealthStatus::Unhealthy,
                last_heartbeat_ns: None,
                uptime_secs: None,
                details: None,
            }),
        }
    }

    async fn get_all_health(&self) -> Result<HashMap<String, ServiceHealth>, OrchestratorError> {
        let mut results = HashMap::new();
        for service_id in self.endpoints.keys() {
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
