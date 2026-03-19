use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedReceiver;
use uuid::Uuid;

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerEvent {
    pub event: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageRequest {
    pub to_user_id: Uuid,
    pub event: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MessagingDelivery {
    pub delivered_connections: usize,
    pub queued_messages: usize,
}

#[derive(Debug)]
pub struct MessagingConnection {
    pub connection_id: Uuid,
    pub receiver: UnboundedReceiver<ServerEvent>,
    pub replayed_messages: usize,
}

pub trait MessagingService: Send + Sync {
    fn connect(&self, user_id: Uuid) -> AppResult<MessagingConnection>;
    fn disconnect(&self, user_id: Uuid, connection_id: Uuid);
    fn send(&self, user_id: Uuid, event: ServerEvent) -> AppResult<MessagingDelivery>;
}
