use std::{convert::Infallible, time::Duration};

use async_stream::stream;
use axum::{
    Extension, Json,
    extract::{Query, State},
    http::{HeaderMap, header::AUTHORIZATION},
    response::sse::{Event, KeepAlive, Sse},
};
use evobase_core::{AppError, AuthContext, MessagingDelivery, SendMessageRequest, ServerEvent};
use evobase_protocol::ApiResponse;
use serde::Deserialize;
use serde_json::json;
use tracing::info;
use uuid::Uuid;

use crate::{ApiResult, AppState, middleware::extract_bearer_token};

pub async fn events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<EventsQuery>,
) -> ApiResult<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>> {
    let notification_token = extract_notification_token(&headers, &query)?;
    let auth = state
        .auth_service
        .verify_notification_token(&notification_token)?;
    let connection = state.messaging_service.connect(auth.user_id)?;
    let guard = ConnectionGuard {
        messaging_service: state.messaging_service.clone(),
        user_id: auth.user_id,
        connection_id: connection.connection_id,
    };
    let user_id = auth.user_id;
    let replayed_messages = connection.replayed_messages;
    let mut receiver = connection.receiver;

    let stream = stream! {
        let _guard = guard;
        yield Ok(sse_event(ServerEvent {
            event: "system.ready".to_string(),
            payload: json!({
                "status": "connected",
                "user_id": user_id,
                "replayed_messages": replayed_messages,
            }),
        }));

        while let Some(message) = receiver.recv().await {
            yield Ok(sse_event(message));
        }
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

pub async fn send_message(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Json(request): Json<SendMessageRequest>,
) -> ApiResult<Json<ApiResponse<MessagingDelivery>>> {
    let delivery = state.messaging_service.send(
        request.to_user_id,
        ServerEvent {
            event: request.event,
            payload: request.payload,
        },
    )?;

    info!(
        from_user_id = %current_user.user_id,
        to_user_id = %request.to_user_id,
        delivered_connections = delivery.delivered_connections,
        queued_messages = delivery.queued_messages,
        "message delivered"
    );

    Ok(Json(ApiResponse::new(delivery)))
}

fn sse_event(message: ServerEvent) -> Event {
    let payload = serde_json::to_string(&message.payload)
        .unwrap_or_else(|_| "{\"error\":\"failed to serialize event payload\"}".to_string());

    Event::default().event(message.event).data(payload)
}

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub token: Option<String>,
}

struct ConnectionGuard {
    messaging_service: std::sync::Arc<dyn evobase_core::MessagingService>,
    user_id: Uuid,
    connection_id: Uuid,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.messaging_service
            .disconnect(self.user_id, self.connection_id);
    }
}

fn extract_notification_token(
    headers: &HeaderMap,
    query: &EventsQuery,
) -> Result<String, AppError> {
    if let Some(header) = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        return extract_bearer_token(header).map(str::to_owned);
    }

    let token = query
        .token
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or(AppError::Unauthorized)?;

    Ok(token.to_string())
}
