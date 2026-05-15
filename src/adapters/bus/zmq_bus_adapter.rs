use std::sync::Mutex;

use async_trait::async_trait;

use crate::core::domain::events::SystemEvent;
use crate::core::ports::event_bus_port::{BusPublishError, IEventBusPort};

pub struct ZmqBusAdapter {
    ctx: zmq::Context,
    publishers: Mutex<Vec<(String, zmq::Socket)>>,
}

impl ZmqBusAdapter {
    pub fn new() -> Self {
        Self {
            ctx: zmq::Context::new(),
            publishers: Mutex::new(Vec::new()),
        }
    }
}

impl Default for ZmqBusAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IEventBusPort for ZmqBusAdapter {
    async fn publish(&self, topic: &str, event: &SystemEvent) -> Result<(), BusPublishError> {
        let payload = serde_json::to_vec(event)
            .map_err(|e| BusPublishError::SerializationFailed(e.to_string()))?;

        let endpoint = std::env::var("ZMQ_PUB_ENDPOINT")
            .unwrap_or_else(|_| "tcp://127.0.0.1:5555".to_string());

        let mut publishers = self.publishers.lock().unwrap();
        if let Some((_ep, pub_socket)) = publishers.iter().find(|(ep, _)| ep == &endpoint) {
            let topic_bytes = format!("{}\0", topic).into_bytes();
            let mut message = topic_bytes;
            message.extend_from_slice(&payload);
            pub_socket.send(&message, zmq::DONTWAIT)
                .map_err(|e| BusPublishError::ConnectionLost(e.to_string()))?;
            return Ok(());
        }

        let pub_socket = self.ctx.socket(zmq::SocketType::PUB)
            .map_err(|e| BusPublishError::ConnectionLost(e.to_string()))?;
        pub_socket.bind(&endpoint)
            .map_err(|e| BusPublishError::ConnectionLost(e.to_string()))?;

        let topic_bytes = format!("{}\0", topic).into_bytes();
        let mut message = topic_bytes;
        message.extend_from_slice(&payload);
        pub_socket.send(&message, zmq::DONTWAIT)
            .map_err(|e| BusPublishError::ConnectionLost(e.to_string()))?;

        publishers.push((endpoint, pub_socket));
        Ok(())
    }

    async fn request_reply(
        &self,
        service: &str,
        payload: &[u8],
    ) -> Result<Vec<u8>, BusPublishError> {
        let endpoint = std::env::var(&format!("{}_ENDPOINT", service.to_uppercase()))
            .unwrap_or_else(|_| "tcp://127.0.0.1:5560".to_string());

        let socket = self.ctx.socket(zmq::SocketType::REQ)
            .map_err(|e| BusPublishError::ConnectionLost(e.to_string()))?;
        socket.connect(&endpoint)
            .map_err(|e| BusPublishError::ConnectionLost(e.to_string()))?;
        socket.send(payload, zmq::DONTWAIT)
            .map_err(|e| BusPublishError::ConnectionLost(e.to_string()))?;

        let mut msg = zmq::Message::new();
        socket.recv(&mut msg, 0)
            .map_err(|e| BusPublishError::ConnectionLost(e.to_string()))?;

        Ok(msg.to_vec())
    }

    async fn subscribe(&self, _topic: &str) -> Result<(), BusPublishError> {
        // Subscription handled at adapter level
        Ok(())
    }

    async fn unsubscribe(&self, _topic: &str) -> Result<(), BusPublishError> {
        Ok(())
    }
}
