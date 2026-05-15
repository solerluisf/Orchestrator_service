use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::core::domain::commands::OrchestratorCommand;
use crate::core::domain::operation_mode::OperationMode;
use crate::core::domain::policy::PolicyParameters;

pub struct AppState {
    pub orchestrator: Arc<crate::core::application::orchestrator_service::OrchestratorService>,
}

pub async fn run(addr: SocketAddr) -> anyhow::Result<()> {
    let journal = Arc::new(
        crate::adapters::persistence::journal_adapter::AppendOnlyJournalAdapter::new("./data/journal")
            .map_err(|e| anyhow::anyhow!(e))?,
    );

    let event_bus = Arc::new(crate::adapters::bus::zmq_bus_adapter::ZmqBusAdapter::new());
    let service_command = Arc::new(
        crate::adapters::bus::bus_service_command::BusServiceCommandAdapter::new(event_bus.clone()),
    );

    let health_port = Arc::new(
        crate::adapters::health::http_health_adapter::HttpHealthAdapter::new(HashMap::new()),
    );

    let metrics = Arc::new(crate::adapters::metrics::prometheus_adapter::PrometheusAdapter::new());

    let orchestrator = Arc::new(
        crate::core::application::orchestrator_service::OrchestratorService::new(
            journal.clone(),
            event_bus.clone(),
            health_port.clone(),
            metrics.clone(),
            service_command.clone(),
        ),
    );

    orchestrator.start().await?;

    let state = Arc::new(AppState {
        orchestrator: orchestrator.clone(),
    });

    let app = Router::new()
        // Health endpoints
        .route("/health", get(health_check))
        .route("/health/services", get(get_all_service_health))
        .route("/health/services/:service_id", get(get_service_health))
        // Kill switch
        .route("/control/kill-switch/activate", post(activate_kill_switch))
        .route("/control/kill-switch/clear", post(clear_kill_switch))
        .route("/control/kill-switch/status", get(get_kill_switch_status))
        // Operation mode
        .route("/control/mode", get(get_operation_mode))
        .route("/control/mode/transition", post(transition_mode))
        .route("/control/mode/history", get(get_mode_history))
        // Policies
        .route("/control/policies", get(list_policies))
        .route("/control/policies/:policy_id/update", post(update_policy))
        .route("/control/policies/reload", post(reload_policies))
        // Audit
        .route("/audit", get(get_audit_log))
        // Service control
        .route("/control/services", get(get_all_services))
        .route("/control/services/:service_id/pause", post(pause_service))
        .route("/control/services/:service_id/resume", post(resume_service))
        .layer(CorsLayer::permissive())
        .with_state(state);

    tracing::info!("HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "orchestrator"
    }))
}

async fn get_all_service_health(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.orchestrator.query_handler().get_all_service_health().await {
        Ok(health) => (StatusCode::OK, Json(health)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

async fn get_service_health(
    State(state): State<Arc<AppState>>,
    Path(service_id): Path<String>,
) -> impl IntoResponse {
    match state.orchestrator.query_handler().get_service_health(&service_id).await {
        Ok(health) => (StatusCode::OK, Json(health)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

async fn activate_kill_switch(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let reason = body.get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("Manual activation");
    let actor = body.get("actor")
        .and_then(|v| v.as_str())
        .unwrap_or("operator");

    match state.orchestrator.kill_switch().activate(reason, actor).await {
        Ok(ack) => (StatusCode::OK, Json(serde_json::to_value(ack).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

async fn clear_kill_switch(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let reason = body.get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("Manual clear");
    let actor = body.get("actor")
        .and_then(|v| v.as_str())
        .unwrap_or("operator");

    match state.orchestrator.kill_switch().clear(reason, actor).await {
        Ok(ack) => (StatusCode::OK, Json(serde_json::to_value(ack).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

async fn get_kill_switch_status(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let active = state.orchestrator.kill_switch().is_active();
    Json(serde_json::json!({
        "active": active,
        "state": if active { "active" } else { "inactive" }
    }))
}

async fn get_operation_mode(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mode = state.orchestrator.mode_controller().current_mode().await;
    Json(serde_json::json!({
        "mode": mode.to_string()
    }))
}

async fn transition_mode(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let target_str = body.get("target_mode")
        .and_then(|v| v.as_str())
        .unwrap_or("paper");
    let reason = body.get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("Mode transition");
    let dry_run = body.get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let target = match target_str.parse::<OperationMode>() {
        Ok(m) => m,
        Err(e) => return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e})),
        ),
    };

    let cmd = OrchestratorCommand::TransitionMode {
        target_mode: target,
        reason: reason.to_string(),
        dry_run,
    };

    match state.orchestrator.handle_command(cmd).await {
        Ok(ack) => (StatusCode::OK, Json(serde_json::to_value(ack).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

async fn get_mode_history(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let history = state.orchestrator.mode_controller().history().await;
    Json(serde_json::to_value(history).unwrap())
}

async fn list_policies(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let policies = state.orchestrator.policy_engine().list_policies().await;
    Json(serde_json::to_value(policies).unwrap())
}

async fn update_policy(
    State(state): State<Arc<AppState>>,
    Path(policy_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let parameters = PolicyParameters {
        values: body.as_object()
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
    };

    match state.orchestrator.policy_engine().update_policy(&policy_id, parameters).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "updated"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

async fn reload_policies(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let policies = state.orchestrator.policy_engine().list_policies().await;
    let params: HashMap<String, PolicyParameters> = policies
        .iter()
        .map(|p| (p.policy_id.clone(), p.parameters.clone()))
        .collect();

    match state.orchestrator.policy_engine().reload_policies(&params).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "reloaded"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

async fn get_audit_log(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit = params.get("limit")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(100);

    match state.orchestrator.audit_trail().query(None, None, limit).await {
        Ok(entries) => (StatusCode::OK, Json(serde_json::to_value(entries).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

async fn get_all_services(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let services = state.orchestrator.service_registry().get_all().await;
    Json(serde_json::to_value(services).unwrap())
}

async fn pause_service(
    State(state): State<Arc<AppState>>,
    Path(service_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let reason = body.get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("Manual pause");

    let cmd = OrchestratorCommand::PauseService {
        service_id,
        reason: reason.to_string(),
    };

    match state.orchestrator.handle_command(cmd).await {
        Ok(ack) => (StatusCode::OK, Json(serde_json::to_value(ack).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

async fn resume_service(
    State(state): State<Arc<AppState>>,
    Path(service_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let reason = body.get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("Manual resume");

    let cmd = OrchestratorCommand::ResumeService {
        service_id,
        reason: reason.to_string(),
    };

    match state.orchestrator.handle_command(cmd).await {
        Ok(ack) => (StatusCode::OK, Json(serde_json::to_value(ack).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}
