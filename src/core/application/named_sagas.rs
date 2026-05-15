use crate::core::domain::saga::{SagaDefinition, SagaStepDef};
use crate::core::domain::workflow::WorkflowAction;

pub fn operation_mode_transition_saga() -> SagaDefinition {
    SagaDefinition {
        id: "operation_mode_transition".to_string(),
        saga_type: "operation_mode_transition".to_string(),
        steps: vec![
            SagaStepDef {
                name: "pause_strategy".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "strategy_service".to_string(),
                    command: serde_json::json!({"type": "pause", "reason": "mode_transition"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "strategy_service".to_string(),
                    command: serde_json::json!({"type": "resume", "reason": "mode_transition_compensated"}),
                },
                timeout_secs: Some(10),
            },
            SagaStepDef {
                name: "flush_pending_intents".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "execution_service".to_string(),
                    command: serde_json::json!({"type": "flush_pending"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "execution_service".to_string(),
                    command: serde_json::json!({"type": "noop"}),
                },
                timeout_secs: Some(15),
            },
            SagaStepDef {
                name: "pause_risk".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "risk_service".to_string(),
                    command: serde_json::json!({"type": "pause", "reason": "mode_transition"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "risk_service".to_string(),
                    command: serde_json::json!({"type": "resume", "reason": "mode_transition_compensated"}),
                },
                timeout_secs: Some(10),
            },
            SagaStepDef {
                name: "wait_for_risk_ack".to_string(),
                action: WorkflowAction::WaitForEvent {
                    event_type: "risk_service.paused".to_string(),
                    timeout_secs: Some(10),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "risk_service".to_string(),
                    command: serde_json::json!({"type": "resume"}),
                },
                timeout_secs: Some(10),
            },
            SagaStepDef {
                name: "broadcast_mode_change".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "_broadcast".to_string(),
                    command: serde_json::json!({"type": "mode_change"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_broadcast".to_string(),
                    command: serde_json::json!({"type": "mode_change_rollback"}),
                },
                timeout_secs: Some(5),
            },
            SagaStepDef {
                name: "resume_strategy".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "strategy_service".to_string(),
                    command: serde_json::json!({"type": "resume", "reason": "mode_transition_complete"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "strategy_service".to_string(),
                    command: serde_json::json!({"type": "pause"}),
                },
                timeout_secs: Some(10),
            },
            SagaStepDef {
                name: "resume_risk".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "risk_service".to_string(),
                    command: serde_json::json!({"type": "resume", "reason": "mode_transition_complete"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "risk_service".to_string(),
                    command: serde_json::json!({"type": "pause"}),
                },
                timeout_secs: Some(10),
            },
        ],
    }
}

pub fn kill_switch_activation_saga() -> SagaDefinition {
    SagaDefinition {
        id: "kill_switch_activation".to_string(),
        saga_type: "kill_switch_activation".to_string(),
        steps: vec![
            SagaStepDef {
                name: "pause_strategy".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "strategy_service".to_string(),
                    command: serde_json::json!({"type": "pause", "reason": "kill_switch"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "strategy_service".to_string(),
                    command: serde_json::json!({"type": "resume"}),
                },
                timeout_secs: Some(5),
            },
            SagaStepDef {
                name: "cancel_all_execution".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "execution_service".to_string(),
                    command: serde_json::json!({"type": "cancel_all_orders", "reason": "kill_switch"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "execution_service".to_string(),
                    command: serde_json::json!({"type": "noop"}),
                },
                timeout_secs: Some(10),
            },
            SagaStepDef {
                name: "cancel_broker_orders".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "broker_gateway".to_string(),
                    command: serde_json::json!({"type": "cancel_all_orders", "reason": "kill_switch"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "broker_gateway".to_string(),
                    command: serde_json::json!({"type": "noop"}),
                },
                timeout_secs: Some(10),
            },
            SagaStepDef {
                name: "pause_risk".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "risk_service".to_string(),
                    command: serde_json::json!({"type": "pause", "reason": "kill_switch"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "risk_service".to_string(),
                    command: serde_json::json!({"type": "resume"}),
                },
                timeout_secs: Some(5),
            },
            SagaStepDef {
                name: "pause_market_data".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "market_data_service".to_string(),
                    command: serde_json::json!({"type": "pause", "reason": "kill_switch"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "market_data_service".to_string(),
                    command: serde_json::json!({"type": "resume"}),
                },
                timeout_secs: Some(5),
            },
        ],
    }
}

pub fn service_restart_saga() -> SagaDefinition {
    SagaDefinition {
        id: "service_restart".to_string(),
        saga_type: "service_restart".to_string(),
        steps: vec![
            SagaStepDef {
                name: "drain_service".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "drain"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "resume"}),
                },
                timeout_secs: Some(30),
            },
            SagaStepDef {
                name: "stop_service".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "stop"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "start"}),
                },
                timeout_secs: Some(10),
            },
            SagaStepDef {
                name: "start_service".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "start"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "stop"}),
                },
                timeout_secs: Some(15),
            },
            SagaStepDef {
                name: "wait_for_health".to_string(),
                action: WorkflowAction::WaitForEvent {
                    event_type: "service_health_changed".to_string(),
                    timeout_secs: Some(30),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "stop"}),
                },
                timeout_secs: Some(30),
            },
            SagaStepDef {
                name: "reconcile_state".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "reconcile"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "stop"}),
                },
                timeout_secs: Some(15),
            },
        ],
    }
}

pub fn policy_update_saga() -> SagaDefinition {
    SagaDefinition {
        id: "policy_update".to_string(),
        saga_type: "policy_update".to_string(),
        steps: vec![
            SagaStepDef {
                name: "validate_new_policy".to_string(),
                action: WorkflowAction::EvaluatePolicy {
                    policy_id: "_target_policy".to_string(),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_noop".to_string(),
                    command: serde_json::json!({"type": "noop"}),
                },
                timeout_secs: Some(5),
            },
            SagaStepDef {
                name: "broadcast_policy_update".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "_broadcast".to_string(),
                    command: serde_json::json!({"type": "policy_update"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_broadcast".to_string(),
                    command: serde_json::json!({"type": "policy_rollback"}),
                },
                timeout_secs: Some(10),
            },
            SagaStepDef {
                name: "wait_for_ack_strategy".to_string(),
                action: WorkflowAction::WaitForEvent {
                    event_type: "policy_applied.strategy_service".to_string(),
                    timeout_secs: Some(15),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_broadcast".to_string(),
                    command: serde_json::json!({"type": "policy_rollback"}),
                },
                timeout_secs: Some(10),
            },
            SagaStepDef {
                name: "wait_for_ack_risk".to_string(),
                action: WorkflowAction::WaitForEvent {
                    event_type: "policy_applied.risk_service".to_string(),
                    timeout_secs: Some(15),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_broadcast".to_string(),
                    command: serde_json::json!({"type": "policy_rollback"}),
                },
                timeout_secs: Some(10),
            },
            SagaStepDef {
                name: "wait_for_ack_execution".to_string(),
                action: WorkflowAction::WaitForEvent {
                    event_type: "policy_applied.execution_service".to_string(),
                    timeout_secs: Some(15),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_broadcast".to_string(),
                    command: serde_json::json!({"type": "policy_rollback"}),
                },
                timeout_secs: Some(10),
            },
        ],
    }
}

pub fn circuit_breaker_open_saga() -> SagaDefinition {
    SagaDefinition {
        id: "circuit_breaker_open".to_string(),
        saga_type: "circuit_breaker_open".to_string(),
        steps: vec![
            SagaStepDef {
                name: "isolate_service".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "isolate"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "reconnect"}),
                },
                timeout_secs: Some(5),
            },
            SagaStepDef {
                name: "notify_dependents".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "_broadcast".to_string(),
                    command: serde_json::json!({"type": "service_degraded"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_broadcast".to_string(),
                    command: serde_json::json!({"type": "service_restored"}),
                },
                timeout_secs: Some(5),
            },
            SagaStepDef {
                name: "start_health_probe".to_string(),
                action: WorkflowAction::WaitForEvent {
                    event_type: "circuit_breaker.half_open".to_string(),
                    timeout_secs: Some(60),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "isolate"}),
                },
                timeout_secs: Some(5),
            },
            SagaStepDef {
                name: "close_circuit_breaker".to_string(),
                action: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "reconnect"}),
                },
                compensation: WorkflowAction::SendCommand {
                    target_service: "_target".to_string(),
                    command: serde_json::json!({"type": "isolate"}),
                },
                timeout_secs: Some(5),
            },
        ],
    }
}

pub fn register_all_named_sagas(
    coordinator: &mut crate::core::application::saga_coordinator::SagaCoordinator,
) {
    coordinator.register_saga(operation_mode_transition_saga());
    coordinator.register_saga(kill_switch_activation_saga());
    coordinator.register_saga(service_restart_saga());
    coordinator.register_saga(policy_update_saga());
    coordinator.register_saga(circuit_breaker_open_saga());
}
