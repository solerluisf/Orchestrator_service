use axum::{
    extract::ws::{Message, WebSocket},
    extract::WebSocketUpgrade,
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct TelemetryBroadcaster {
    tx: broadcast::Sender<String>,
}

impl TelemetryBroadcaster {
    pub fn new(capacity: usize) -> Arc<Self> {
        let (tx, _) = broadcast::channel(capacity);
        Arc::new(Self { tx })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    pub fn broadcast(&self, message: String) {
        let _ = self.tx.send(message);
    }
}

pub async fn run_ws_server(addr: SocketAddr, broadcaster: Arc<TelemetryBroadcaster>) -> anyhow::Result<()> {
    let app = axum::Router::new()
        .route("/ws/telemetry", axum::routing::get(ws_handler))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(broadcaster);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("WebSocket server listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(broadcaster): axum::extract::State<Arc<TelemetryBroadcaster>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, broadcaster))
}

async fn handle_socket(socket: WebSocket, broadcaster: Arc<TelemetryBroadcaster>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = broadcaster.subscribe();

    // Send welcome message
    let welcome = serde_json::json!({
        "topic": "connected",
        "message": "Connected to Orchestrator telemetry",
        "ts": now_nanos(),
    });
    if sender.send(Message::Text(welcome.to_string().into())).await.is_err() {
        return;
    }

    // Spawn task to forward broadcast messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages from client (subscriptions, etc.)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(sub_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                        tracing::debug!(topics = ?sub_msg.get("subscribe"), "Client subscription");
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    tracing::debug!("WebSocket client disconnected");
}

fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
