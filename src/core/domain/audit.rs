use serde::{Deserialize, Serialize};
use uuid::Uuid;

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp_ns: u64,
    pub actor: AuditActor,
    pub action: AuditAction,
    pub target: Option<String>,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub reason: Option<String>,
    pub trace_id: Option<String>,
    pub outcome: AuditOutcome,
    #[serde(default)]
    pub signature: Option<String>,
}

impl AuditEntry {
    pub fn new(actor: AuditActor, action: AuditAction) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp_ns: now_nanos(),
            actor,
            action,
            target: None,
            before_state: None,
            after_state: None,
            reason: None,
            trace_id: None,
            outcome: AuditOutcome::Success,
            signature: None,
        }
    }

    pub fn with_target(mut self, target: &str) -> Self {
        self.target = Some(target.to_string());
        self
    }

    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }

    pub fn with_before_state(mut self, state: serde_json::Value) -> Self {
        self.before_state = Some(state);
        self
    }

    pub fn with_after_state(mut self, state: serde_json::Value) -> Self {
        self.after_state = Some(state);
        self
    }

    pub fn with_trace_id(mut self, trace_id: &str) -> Self {
        self.trace_id = Some(trace_id.to_string());
        self
    }

    pub fn with_outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    pub fn sign(mut self, secret: &[u8]) -> Self {
        let payload = self.signing_payload();
        let mut mac = HmacSha256::new_from_slice(secret)
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        self.signature = Some(hex::encode(result.into_bytes()));
        self
    }

    pub fn verify(&self, secret: &[u8]) -> bool {
        let Some(ref sig_hex) = self.signature else {
            return false;
        };
        let Ok(sig_bytes) = hex::decode(sig_hex) else {
            return false;
        };
        let payload = self.signing_payload();
        let mut mac = HmacSha256::new_from_slice(secret)
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        mac.verify_slice(&sig_bytes).is_ok()
    }

    fn signing_payload(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.id,
            self.timestamp_ns,
            serde_json::to_string(&self.actor).unwrap_or_default(),
            serde_json::to_string(&self.action).unwrap_or_default(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActor {
    Human { user_id: String },
    Automated { policy_id: String },
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    KillSwitchActivated,
    KillSwitchCleared,
    ModeTransitionStarted,
    ModeTransitionCompleted,
    ModeTransitionFailed,
    PolicyUpdated,
    PoliciesReloaded,
    ServicePaused,
    ServiceResumed,
    CircuitBreakerReset,
    CircuitBreakerConfigured,
    RateLimiterConfigured,
    WorkflowTriggered,
    WorkflowCancelled,
    ConfigReloaded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    PartialSuccess,
    Failure,
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
