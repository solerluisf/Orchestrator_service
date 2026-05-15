use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationMode {
    Live,
    Paper,
    Readonly,
    Offline,
}

impl OperationMode {
    pub fn can_transition_to(&self, target: &OperationMode) -> bool {
        matches!(
            (self, target),
            (OperationMode::Offline, OperationMode::Paper)
                | (OperationMode::Offline, OperationMode::Readonly)
                | (OperationMode::Paper, OperationMode::Live)
                | (OperationMode::Paper, OperationMode::Readonly)
                | (OperationMode::Paper, OperationMode::Offline)
                | (OperationMode::Live, OperationMode::Paper)
                | (OperationMode::Live, OperationMode::Readonly)
                | (OperationMode::Live, OperationMode::Offline)
                | (OperationMode::Readonly, OperationMode::Paper)
                | (OperationMode::Readonly, OperationMode::Offline)
        )
    }
}

impl fmt::Display for OperationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationMode::Live => write!(f, "live"),
            OperationMode::Paper => write!(f, "paper"),
            OperationMode::Readonly => write!(f, "readonly"),
            OperationMode::Offline => write!(f, "offline"),
        }
    }
}

impl std::str::FromStr for OperationMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "live" => Ok(OperationMode::Live),
            "paper" => Ok(OperationMode::Paper),
            "readonly" => Ok(OperationMode::Readonly),
            "offline" => Ok(OperationMode::Offline),
            _ => Err(format!("Unknown operation mode: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeTransitionRecord {
    pub from: OperationMode,
    pub to: OperationMode,
    pub timestamp_ns: u64,
    pub actor: String,
    pub reason: String,
    pub success: bool,
}
