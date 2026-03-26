use std::{convert::Infallible, time::Duration};

use async_stream::stream;
use axum::{
    Json,
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
};
use evobase_core::AppError;
use evobase_protocol::ApiResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ApiResult, AppState, topic_messaging::TopicMessage};

#[derive(Debug, Deserialize)]
pub struct TopicPublishRequest {
    pub event: String,
    pub payload: Value,
    pub topic: Option<String>,
    pub to_user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TopicSubscribeQuery {
    pub topic: String,
}

#[derive(Debug, Serialize)]
pub struct TopicPublishResponse {
    pub ok: bool,
}

pub async fn publish(
    State(state): State<AppState>,
    Json(request): Json<TopicPublishRequest>,
) -> ApiResult<Json<ApiResponse<TopicPublishResponse>>> {
    let event = request.event.trim().to_string();
    if event.is_empty() {
        return Err(AppError::BadRequest("event is required".to_string()).into());
    }

    let topic = resolve_topic(request.topic.as_deref(), request.to_user_id.as_deref())?;

    state.topic_messaging.publish(
        &topic,
        TopicMessage {
            event,
            payload: request.payload,
        },
    )?;

    Ok(Json(ApiResponse::new(TopicPublishResponse { ok: true })))
}

pub async fn subscribe(
    State(state): State<AppState>,
    Query(query): Query<TopicSubscribeQuery>,
) -> ApiResult<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>> {
    let topic = query.topic.trim().to_string();
    if topic.is_empty() {
        return Err(AppError::BadRequest("topic is required".to_string()).into());
    }

    let (mut receiver, queued_messages) = state.topic_messaging.subscribe(&topic)?;
    let stream = stream! {
        for message in queued_messages {
            yield Ok(sse_event(message));
        }

        loop {
            match receiver.recv().await {
                Ok(message) => yield Ok(sse_event(message)),
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

fn resolve_topic(topic: Option<&str>, to_user_id: Option<&str>) -> Result<String, AppError> {
    if let Some(to_user_id) = to_user_id.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(format!("user:{to_user_id}"));
    }

    topic
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| AppError::BadRequest("topic is required when to_user_id is empty".to_string()))
}

fn sse_event(message: TopicMessage) -> Event {
    let payload = serde_json::to_string(&message.payload)
        .unwrap_or_else(|_| "{\"error\":\"failed to serialize event payload\"}".to_string());

    Event::default().event(message.event).data(payload)
}
