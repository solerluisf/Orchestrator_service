use std::collections::HashMap;

use crate::core::domain::policy::PolicyParameters;

pub struct PolicyRegistry {
    policies: HashMap<String, PolicyParameters>,
}

impl PolicyRegistry {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    pub fn register(&mut self, policy_id: &str, params: PolicyParameters) {
        self.policies.insert(policy_id.to_string(), params);
    }

    pub fn get(&self, policy_id: &str) -> Option<&PolicyParameters> {
        self.policies.get(policy_id)
    }

    pub fn list(&self) -> Vec<(String, PolicyParameters)> {
        self.policies.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}
