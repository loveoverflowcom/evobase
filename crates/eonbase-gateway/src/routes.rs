use std::{convert::Infallible, time::Duration};

use async_stream::stream;
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, RawQuery, State},
    middleware,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use eonbase_core::{AuthContext, AuthResponse, RefreshRequest, SendMessageRequest, ServerEvent};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use uuid::Uuid;

use crate::{
    ApiResult, AppState,
    middleware::require_access_token,
    rest::{
        parse_delete_request, parse_insert_request, parse_select_request, parse_update_request,
    },
};

pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/messages/send", post(send_message))
        .route(
            "/rest/{table}",
            get(select_rows)
                .post(insert_rows)
                .patch(update_rows)
                .delete(delete_rows),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_access_token,
        ));

    Router::new()
        .route("/healthz", get(healthcheck))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/events", get(events))
        .merge(protected)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn healthcheck() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn register(
    State(state): State<AppState>,
    Json(request): Json<eonbase_core::RegisterRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let response = state.auth_service.register(request).await?;
    Ok(Json(response))
}

async fn login(
    State(state): State<AppState>,
    Json(request): Json<eonbase_core::LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let response = state.auth_service.login(request).await?;
    Ok(Json(response))
}

async fn refresh(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let response = state.auth_service.refresh(&request.refresh_token).await?;
    Ok(Json(response))
}

async fn events(
    State(state): State<AppState>,
    Query(query): Query<EventsQuery>,
) -> ApiResult<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>> {
    let auth = state.auth_service.verify_notification_token(&query.token)?;
    let connection = state.messaging_service.connect(auth.user_id)?;
    let guard = ConnectionGuard {
        messaging_service: state.messaging_service.clone(),
        user_id: auth.user_id,
        connection_id: connection.connection_id,
    };
    let user_id = auth.user_id;
    let mut receiver = connection.receiver;

    let stream = stream! {
        let _guard = guard;
        yield Ok(sse_event(ServerEvent {
            event: "system.ready".to_string(),
            payload: json!({
                "status": "connected",
                "user_id": user_id,
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

async fn send_message(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Json(request): Json<SendMessageRequest>,
) -> ApiResult<Json<SendMessageResponse>> {
    let delivered_connections = state.messaging_service.send(
        request.to_user_id,
        ServerEvent {
            event: request.event,
            payload: request.payload,
        },
    )?;

    info!(
        from_user_id = %current_user.user_id,
        to_user_id = %request.to_user_id,
        delivered_connections,
        "message delivered"
    );

    Ok(Json(SendMessageResponse {
        delivered_connections,
    }))
}

async fn select_rows(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path(table): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<Vec<Value>>> {
    let request = parse_select_request(&table, raw_query.as_deref())?;
    let rows = state
        .storage
        .select_rows(request, Some(&current_user))
        .await?;

    Ok(Json(rows))
}

async fn insert_rows(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path(table): Path<String>,
    Json(body): Json<Value>,
) -> ApiResult<Json<Vec<Value>>> {
    let request = parse_insert_request(&table, body)?;
    let rows = state
        .storage
        .insert_rows(request, Some(&current_user))
        .await?;

    Ok(Json(rows))
}

async fn update_rows(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path(table): Path<String>,
    RawQuery(raw_query): RawQuery,
    Json(body): Json<Value>,
) -> ApiResult<Json<Vec<Value>>> {
    let request = parse_update_request(&table, raw_query.as_deref(), body)?;
    let rows = state
        .storage
        .update_rows(request, Some(&current_user))
        .await?;

    Ok(Json(rows))
}

async fn delete_rows(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path(table): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<Vec<Value>>> {
    let request = parse_delete_request(&table, raw_query.as_deref())?;
    let rows = state
        .storage
        .delete_rows(request, Some(&current_user))
        .await?;

    Ok(Json(rows))
}

fn sse_event(message: ServerEvent) -> Event {
    let payload = serde_json::to_string(&message.payload)
        .unwrap_or_else(|_| "{\"error\":\"failed to serialize event payload\"}".to_string());

    Event::default().event(message.event).data(payload)
}

#[derive(Debug, Deserialize)]
struct EventsQuery {
    token: String,
}

#[derive(Debug, Serialize)]
struct SendMessageResponse {
    delivered_connections: usize,
}

struct ConnectionGuard {
    messaging_service: std::sync::Arc<dyn eonbase_core::MessagingService>,
    user_id: Uuid,
    connection_id: Uuid,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.messaging_service
            .disconnect(self.user_id, self.connection_id);
    }
}
