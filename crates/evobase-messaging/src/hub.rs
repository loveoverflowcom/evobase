use std::{
    cmp,
    collections::{HashMap, VecDeque},
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use evobase_core::{
    AppError, AppResult, MessagingConnection, MessagingDelivery, MessagingService, ServerEvent,
};
use tokio::{
    runtime::Handle,
    sync::mpsc::{self, UnboundedSender},
    time::{self, MissedTickBehavior},
};
use tracing::{debug, warn};
use uuid::Uuid;

const DEFAULT_OFFLINE_TTL: Duration = Duration::from_secs(3 * 24 * 60 * 60);
const MIN_CLEANUP_INTERVAL: Duration = Duration::from_secs(1);
const MAX_CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

pub struct InMemoryMessagingHub {
    state: Arc<RwLock<HubState>>,
    offline_message_ttl: Duration,
}

#[derive(Default)]
struct HubState {
    connections: HashMap<Uuid, HashMap<Uuid, UnboundedSender<ServerEvent>>>,
    queued_messages: HashMap<Uuid, VecDeque<QueuedMessage>>,
}

struct QueuedMessage {
    event: ServerEvent,
    expires_at: Instant,
}

impl Default for InMemoryMessagingHub {
    fn default() -> Self {
        Self::with_offline_ttl(DEFAULT_OFFLINE_TTL)
    }
}

impl InMemoryMessagingHub {
    pub fn with_offline_ttl(offline_message_ttl: Duration) -> Self {
        let state = Arc::new(RwLock::new(HubState::default()));
        Self::spawn_cleanup_task(state.clone(), offline_message_ttl);

        Self {
            state,
            offline_message_ttl,
        }
    }

    fn spawn_cleanup_task(state: Arc<RwLock<HubState>>, offline_message_ttl: Duration) {
        let Ok(runtime) = Handle::try_current() else {
            debug!("tokio runtime unavailable, queued-message cleanup will run lazily");
            return;
        };

        let cleanup_interval = cmp::max(
            MIN_CLEANUP_INTERVAL,
            cmp::min(offline_message_ttl, MAX_CLEANUP_INTERVAL),
        );

        runtime.spawn(async move {
            let mut ticker = time::interval(cleanup_interval);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                ticker.tick().await;

                let Ok(mut state) = state.write() else {
                    warn!("failed to acquire messaging lock during queued-message cleanup");
                    continue;
                };

                let expired_messages = state.prune_expired_messages(Instant::now());
                if expired_messages > 0 {
                    debug!(expired_messages, "pruned expired queued messages");
                }
            }
        });
    }
}

impl MessagingService for InMemoryMessagingHub {
    fn connect(&self, user_id: Uuid) -> AppResult<MessagingConnection> {
        let connection_id = Uuid::new_v4();
        let (sender, receiver) = mpsc::unbounded_channel();

        let queued_messages = {
            let mut state = self
                .state
                .write()
                .map_err(|_| AppError::Internal("messaging hub lock poisoned".to_string()))?;

            state.prune_expired_messages(Instant::now());
            state
                .connections
                .entry(user_id)
                .or_default()
                .insert(connection_id, sender.clone());

            state.take_queued_messages(user_id)
        };

        let replayed_messages = queued_messages.len();
        for message in queued_messages {
            if sender.send(message).is_err() {
                warn!(%user_id, %connection_id, "failed to replay queued message");
                break;
            }
        }

        debug!(
            %user_id,
            %connection_id,
            replayed_messages,
            "registered messaging connection"
        );

        Ok(MessagingConnection {
            connection_id,
            receiver,
            replayed_messages,
        })
    }

    fn disconnect(&self, user_id: Uuid, connection_id: Uuid) {
        let Ok(mut state) = self.state.write() else {
            warn!(%user_id, %connection_id, "failed to acquire messaging lock during disconnect");
            return;
        };

        state.prune_expired_messages(Instant::now());

        if let Some(user_connections) = state.connections.get_mut(&user_id) {
            user_connections.remove(&connection_id);

            if user_connections.is_empty() {
                state.connections.remove(&user_id);
            }
        }

        debug!(%user_id, %connection_id, "removed messaging connection");
    }

    fn send(&self, user_id: Uuid, event: ServerEvent) -> AppResult<MessagingDelivery> {
        let mut state = self
            .state
            .write()
            .map_err(|_| AppError::Internal("messaging hub lock poisoned".to_string()))?;

        let now = Instant::now();
        state.prune_expired_messages(now);

        let delivered_connections = state.deliver_to_active_connections(user_id, &event);
        let queued_messages = if delivered_connections == 0 {
            state.queue_message(user_id, event, now + self.offline_message_ttl);
            1
        } else {
            0
        };

        debug!(
            %user_id,
            delivered_connections,
            queued_messages,
            "fan-out completed"
        );

        Ok(MessagingDelivery {
            delivered_connections,
            queued_messages,
        })
    }
}

impl HubState {
    fn take_queued_messages(&mut self, user_id: Uuid) -> Vec<ServerEvent> {
        self.queued_messages
            .remove(&user_id)
            .map(|messages| messages.into_iter().map(|message| message.event).collect())
            .unwrap_or_default()
    }

    fn queue_message(&mut self, user_id: Uuid, event: ServerEvent, expires_at: Instant) {
        self.queued_messages
            .entry(user_id)
            .or_default()
            .push_back(QueuedMessage { event, expires_at });
    }

    fn deliver_to_active_connections(&mut self, user_id: Uuid, event: &ServerEvent) -> usize {
        let mut delivered_connections = 0usize;
        let should_remove_user = if let Some(user_connections) = self.connections.get_mut(&user_id)
        {
            let stale_ids: Vec<Uuid> = user_connections
                .iter()
                .filter_map(|(connection_id, sender)| match sender.send(event.clone()) {
                    Ok(_) => {
                        delivered_connections += 1;
                        None
                    }
                    Err(_) => Some(*connection_id),
                })
                .collect();

            for stale_id in stale_ids {
                user_connections.remove(&stale_id);
            }

            user_connections.is_empty()
        } else {
            return 0;
        };

        if should_remove_user {
            self.connections.remove(&user_id);
        }

        delivered_connections
    }

    fn prune_expired_messages(&mut self, now: Instant) -> usize {
        let mut expired_messages = 0usize;

        self.queued_messages.retain(|_, messages| {
            let original_len = messages.len();
            messages.retain(|message| message.expires_at > now);
            expired_messages += original_len.saturating_sub(messages.len());
            !messages.is_empty()
        });

        expired_messages
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use evobase_core::MessagingService;
    use serde_json::json;
    use tokio::sync::mpsc::error::TryRecvError;
    use uuid::Uuid;

    use super::InMemoryMessagingHub;

    #[test]
    fn sends_immediately_to_online_connections() {
        let hub = InMemoryMessagingHub::with_offline_ttl(Duration::from_secs(60));
        let user_id = Uuid::new_v4();
        let message = evobase_core::ServerEvent {
            event: "chat.message".to_string(),
            payload: json!({ "body": "hello" }),
        };

        let mut connection = hub.connect(user_id).expect("connect should succeed");
        assert_eq!(connection.replayed_messages, 0);

        let delivery = hub
            .send(user_id, message.clone())
            .expect("send should succeed");
        assert_eq!(delivery.delivered_connections, 1);
        assert_eq!(delivery.queued_messages, 0);
        assert_eq!(
            connection
                .receiver
                .try_recv()
                .expect("message should be delivered"),
            message
        );
    }

    #[test]
    fn queues_messages_while_user_is_offline_and_replays_on_connect() {
        let hub = InMemoryMessagingHub::with_offline_ttl(Duration::from_secs(60));
        let user_id = Uuid::new_v4();
        let message = evobase_core::ServerEvent {
            event: "chat.message".to_string(),
            payload: json!({ "body": "queued" }),
        };

        let delivery = hub
            .send(user_id, message.clone())
            .expect("send should succeed");
        assert_eq!(delivery.delivered_connections, 0);
        assert_eq!(delivery.queued_messages, 1);

        let mut connection = hub.connect(user_id).expect("connect should succeed");
        assert_eq!(connection.replayed_messages, 1);
        assert_eq!(
            connection
                .receiver
                .try_recv()
                .expect("queued message should be replayed"),
            message
        );
        assert!(matches!(
            connection.receiver.try_recv(),
            Err(TryRecvError::Empty)
        ));
    }

    #[test]
    fn drops_queued_messages_after_ttl_expires() {
        let hub = InMemoryMessagingHub::with_offline_ttl(Duration::from_millis(20));
        let user_id = Uuid::new_v4();

        hub.send(
            user_id,
            evobase_core::ServerEvent {
                event: "chat.message".to_string(),
                payload: json!({ "body": "expired" }),
            },
        )
        .expect("send should succeed");

        thread::sleep(Duration::from_millis(30));

        let mut connection = hub.connect(user_id).expect("connect should succeed");
        assert_eq!(connection.replayed_messages, 0);
        assert!(matches!(
            connection.receiver.try_recv(),
            Err(TryRecvError::Empty)
        ));
    }
}
