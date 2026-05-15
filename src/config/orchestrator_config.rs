use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use super::policy_config::PolicyConfig;
use super::service_config::ServiceConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    #[serde(default = "default_http_addr")]
    pub http_addr: SocketAddr,

    #[serde(default = "default_ws_addr")]
    pub ws_addr: SocketAddr,

    #[serde(default = "default_journal_path")]
    pub journal_path: String,

    #[serde(default = "default_health_poll_interval_secs")]
    pub health_poll_interval_secs: u64,

    #[serde(default = "default_command_channel_size")]
    pub command_channel_size: usize,

    #[serde(default)]
    pub services: Vec<ServiceConfig>,

    #[serde(default)]
    pub policies: PolicyConfig,

    #[serde(default)]
    pub tls_cert_path: Option<String>,

    #[serde(default)]
    pub tls_key_path: Option<String>,

    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
}

fn default_http_addr() -> SocketAddr {
    "0.0.0.0:9090".parse().unwrap()
}

fn default_ws_addr() -> SocketAddr {
    "0.0.0.0:9091".parse().unwrap()
}

fn default_journal_path() -> String {
    "./data/journal".to_string()
}

fn default_health_poll_interval_secs() -> u64 {
    5
}

fn default_command_channel_size() -> usize {
    1000
}

fn default_jwt_secret() -> String {
    "orchestrator-secret-change-me".to_string()
}

impl OrchestratorConfig {
    pub fn load() -> Result<Self, crate::core::domain::errors::OrchestratorError> {
        let config_str = std::fs::read_to_string("orchestrator.toml").unwrap_or_else(|_| {
            tracing::warn!("No orchestrator.toml found, using defaults");
            String::new()
        });

        if config_str.is_empty() {
            return Ok(Self::default());
        }

        toml::from_str(&config_str)
            .map_err(|e| crate::core::domain::errors::OrchestratorError::ConfigError(e.to_string()))
    }
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            http_addr: default_http_addr(),
            ws_addr: default_ws_addr(),
            journal_path: default_journal_path(),
            health_poll_interval_secs: default_health_poll_interval_secs(),
            command_channel_size: default_command_channel_size(),
            services: Vec::new(),
            policies: PolicyConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            jwt_secret: default_jwt_secret(),
        }
    }
}
