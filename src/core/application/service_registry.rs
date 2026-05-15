use std::sync::Arc;

use crate::core::domain::errors::OrchestratorError;
use crate::core::domain::service_registry::{ServiceDescriptor, ServiceRegistry};

pub struct ServiceRegistryService {
    registry: Arc<tokio::sync::RwLock<ServiceRegistry>>,
}

impl ServiceRegistryService {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(tokio::sync::RwLock::new(ServiceRegistry::new())),
        }
    }

    pub async fn register(&self, descriptor: ServiceDescriptor) -> Result<(), OrchestratorError> {
        let mut registry = self.registry.write().await;
        tracing::info!(service_id = %descriptor.id, "Service registered");
        registry.register(descriptor);
        Ok(())
    }

    pub async fn deregister(&self, service_id: &str) -> Result<(), OrchestratorError> {
        let mut registry = self.registry.write().await;
        registry.deregister(service_id);
        tracing::info!(service_id = %service_id, "Service deregistered");
        Ok(())
    }

    pub async fn get_all(&self) -> Vec<ServiceDescriptor> {
        let registry = self.registry.read().await;
        registry.services.values().cloned().collect()
    }

    pub async fn get(&self, service_id: &str) -> Option<ServiceDescriptor> {
        let registry = self.registry.read().await;
        registry.get(service_id).cloned()
    }

    pub async fn all_ids(&self) -> Vec<String> {
        let registry = self.registry.read().await;
        registry.all_ids().into_iter().map(|s| s.to_string()).collect()
    }
}
