use axum::{
    extract::ws::{Message, WebSocket},
    extract::WebSocketUpgrade,
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;

pub async fn run_ws_server(addr: SocketAddr) -> anyhow::Result<()> {
    let app = axum::Router::new()
        .route("/ws/telemetry", axum::routing::get(ws_handler))
        .layer(tower_http::cors::CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("WebSocket server listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    // Send initial state
    let init_msg = serde_json::json!({
        "topic": "connected",
        "message": "Connected to Orchestrator telemetry",
        "ts": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    });

    if sender
        .send(Message::Text(init_msg.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    // Handle client messages (subscriptions)
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(subscribe_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(topics) = subscribe_msg.get("subscribe") {
                        tracing::debug!(topics = ?topics, "Client subscribed to topics");
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }

    tracing::debug!("WebSocket client disconnected");
}
