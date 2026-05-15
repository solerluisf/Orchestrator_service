use tokio::sync::mpsc;

use crate::core::domain::events::SystemEvent;

pub struct BusEventIngress {
    event_tx: mpsc::Sender<SystemEvent>,
}

impl BusEventIngress {
    pub fn new(event_tx: mpsc::Sender<SystemEvent>) -> Self {
        Self { event_tx }
    }

    pub async fn start_listening(&self, endpoint: &str) -> Result<(), crate::core::domain::errors::OrchestratorError> {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::SocketType::SUB)
            .map_err(|e| crate::core::domain::errors::OrchestratorError::Bus(e.to_string()))?;

        socket.connect(endpoint)
            .map_err(|e| crate::core::domain::errors::OrchestratorError::Bus(e.to_string()))?;
        socket.set_subscribe(b"")
            .map_err(|e| crate::core::domain::errors::OrchestratorError::Bus(e.to_string()))?;

        let event_tx = self.event_tx.clone();

        tokio::task::spawn_blocking(move || {
            loop {
                let mut msg = zmq::Message::new();
                if socket.recv(&mut msg, 0).is_ok() {
                    if let Ok(text) = std::str::from_utf8(&msg) {
                        if let Some(json_str) = text.split('\0').nth(1) {
                            if let Ok(event) = serde_json::from_str::<SystemEvent>(json_str) {
                                if event_tx.blocking_send(event).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
