use std::{collections::HashMap, sync::RwLock};

use evobase_core::{AppError, AppResult, MessagingConnection, MessagingService, ServerEvent};
use tokio::sync::mpsc::{self, UnboundedSender};
use tracing::{debug, warn};
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryMessagingHub {
    connections: RwLock<HashMap<Uuid, HashMap<Uuid, UnboundedSender<ServerEvent>>>>,
}

impl MessagingService for InMemoryMessagingHub {
    fn connect(&self, user_id: Uuid) -> AppResult<MessagingConnection> {
        let connection_id = Uuid::new_v4();
        let (sender, receiver) = mpsc::unbounded_channel();

        let mut connections = self
            .connections
            .write()
            .map_err(|_| AppError::Internal("messaging hub lock poisoned".to_string()))?;

        connections
            .entry(user_id)
            .or_default()
            .insert(connection_id, sender);

        debug!(%user_id, %connection_id, "registered messaging connection");

        Ok(MessagingConnection {
            connection_id,
            receiver,
        })
    }

    fn disconnect(&self, user_id: Uuid, connection_id: Uuid) {
        let Ok(mut connections) = self.connections.write() else {
            warn!(%user_id, %connection_id, "failed to acquire messaging lock during disconnect");
            return;
        };

        if let Some(user_connections) = connections.get_mut(&user_id) {
            user_connections.remove(&connection_id);

            if user_connections.is_empty() {
                connections.remove(&user_id);
            }
        }

        debug!(%user_id, %connection_id, "removed messaging connection");
    }

    fn send(&self, user_id: Uuid, event: ServerEvent) -> AppResult<usize> {
        let mut delivered = 0usize;

        let mut connections = self
            .connections
            .write()
            .map_err(|_| AppError::Internal("messaging hub lock poisoned".to_string()))?;

        if let Some(user_connections) = connections.get_mut(&user_id) {
            let stale_ids: Vec<Uuid> = user_connections
                .iter()
                .filter_map(|(connection_id, sender)| match sender.send(event.clone()) {
                    Ok(_) => {
                        delivered += 1;
                        None
                    }
                    Err(_) => Some(*connection_id),
                })
                .collect();

            for stale_id in stale_ids {
                user_connections.remove(&stale_id);
            }

            if user_connections.is_empty() {
                connections.remove(&user_id);
            }
        }

        debug!(%user_id, delivered, "fan-out completed");

        Ok(delivered)
    }
}
