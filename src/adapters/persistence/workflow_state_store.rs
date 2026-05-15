use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::domain::workflow::WorkflowInstance;

pub struct WorkflowStateStore {
    states: Arc<RwLock<HashMap<String, WorkflowInstance>>>,
}

impl WorkflowStateStore {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn save(&self, instance: WorkflowInstance) {
        let mut states = self.states.write().await;
        states.insert(instance.instance_id.clone(), instance);
    }

    pub async fn get(&self, instance_id: &str) -> Option<WorkflowInstance> {
        let states = self.states.read().await;
        states.get(instance_id).cloned()
    }

    pub async fn list_all(&self) -> Vec<WorkflowInstance> {
        let states = self.states.read().await;
        states.values().cloned().collect()
    }

    pub async fn remove(&self, instance_id: &str) {
        let mut states = self.states.write().await;
        states.remove(instance_id);
    }
}
