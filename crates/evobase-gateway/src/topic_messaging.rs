use std::collections::{HashMap, VecDeque};

use evobase_core::{AppError, AppResult};
use serde_json::Value;
use tokio::sync::broadcast;

const TOPIC_CHANNEL_CAPACITY: usize = 256;
const TOPIC_QUEUE_CAPACITY: usize = 200;

#[derive(Debug, Clone)]
pub struct TopicMessage {
    pub event: String,
    pub payload: Value,
}

#[derive(Default)]
pub struct TopicMessagingHub {
    topics: std::sync::RwLock<HashMap<String, TopicState>>,
}

struct TopicState {
    sender: broadcast::Sender<TopicMessage>,
    queued_messages: VecDeque<TopicMessage>,
}

impl TopicMessagingHub {
    pub fn publish(&self, topic: &str, message: TopicMessage) -> AppResult<()> {
        let mut topics = self
            .topics
            .write()
            .map_err(|_| AppError::Internal("topic messaging lock poisoned".to_string()))?;
        let state = topics.entry(topic.to_string()).or_insert_with(|| {
            let (sender, _receiver) = broadcast::channel(TOPIC_CHANNEL_CAPACITY);
            TopicState {
                sender,
                queued_messages: VecDeque::new(),
            }
        });

        if state.sender.receiver_count() == 0 {
            if state.queued_messages.len() >= TOPIC_QUEUE_CAPACITY {
                state.queued_messages.pop_front();
            }
            state.queued_messages.push_back(message);
            return Ok(());
        }

        let _ = state.sender.send(message);
        Ok(())
    }

    pub fn subscribe(
        &self,
        topic: &str,
    ) -> AppResult<(broadcast::Receiver<TopicMessage>, Vec<TopicMessage>)> {
        let mut topics = self
            .topics
            .write()
            .map_err(|_| AppError::Internal("topic messaging lock poisoned".to_string()))?;
        let state = topics.entry(topic.to_string()).or_insert_with(|| {
            let (sender, _receiver) = broadcast::channel(TOPIC_CHANNEL_CAPACITY);
            TopicState {
                sender,
                queued_messages: VecDeque::new(),
            }
        });

        let receiver = state.sender.subscribe();
        let queued = state.queued_messages.drain(..).collect();
        Ok((receiver, queued))
    }
}
