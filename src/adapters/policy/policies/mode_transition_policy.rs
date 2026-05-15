use crate::core::domain::policy::PolicyDecision;
use crate::core::domain::operation_mode::OperationMode;

pub struct ModeTransitionPolicy {
    allowed_transitions: Vec<(OperationMode, OperationMode)>,
}

impl ModeTransitionPolicy {
    pub fn new() -> Self {
        Self {
            allowed_transitions: vec![
                (OperationMode::Offline, OperationMode::Paper),
                (OperationMode::Offline, OperationMode::Readonly),
                (OperationMode::Paper, OperationMode::Live),
                (OperationMode::Paper, OperationMode::Readonly),
                (OperationMode::Paper, OperationMode::Offline),
                (OperationMode::Live, OperationMode::Paper),
                (OperationMode::Live, OperationMode::Readonly),
                (OperationMode::Live, OperationMode::Offline),
                (OperationMode::Readonly, OperationMode::Paper),
                (OperationMode::Readonly, OperationMode::Offline),
            ],
        }
    }

    pub fn evaluate(&self, from: &OperationMode, to: &OperationMode) -> PolicyDecision {
        if self.allowed_transitions.contains(&(*from, *to)) {
            PolicyDecision::Allow
        } else {
            PolicyDecision::Deny {
                reason: format!(
                    "Transition from {} to {} is not allowed",
                    from, to
                ),
            }
        }
    }
}
