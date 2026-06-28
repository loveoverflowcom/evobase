use std::{collections::BTreeSet, env, net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use evobase_db::PostgresStore;
use evobase_domain::{
    Actor, Capability, CreateOrderCommand, CreateUserCommand, DomainError, DomainService, Email,
    EventRepository, Money, NonEmptyString, OrderId, PaymentId, UserId, ir::demo_domain_ir,
};
use serde::Deserialize;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    store: Arc<PostgresStore>,
    service: DomainService<PostgresStore>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "evobase_api=debug,evobase_db=debug,tower_http=info".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is required; see .env.example for a local demo value")?;
    let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let server_addr: SocketAddr = server_addr.parse()?;

    let store = Arc::new(PostgresStore::connect(&database_url).await?);
    store.run_migrations().await?;

    let state = AppState {
        service: DomainService::new(store.clone()),
        store,
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/demo/domain-ir", get(domain_ir))
        .route("/demo/events", get(list_events))
        .route("/demo/users", post(create_user))
        .route("/demo/users/{user_id}/verify", post(verify_user))
        .route("/demo/orders", post(create_order))
        .route("/demo/orders/{order_id}", get(get_order))
        .route("/demo/orders/{order_id}/pay", post(pay_order))
        .route("/demo/orders/{order_id}/ship", post(ship_order))
        .route("/demo/orders/{order_id}/cancel", post(cancel_order))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    tracing::info!(%server_addr, "starting EvoBase domain-first demo API");
    let listener = tokio::net::TcpListener::bind(server_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn domain_ir() -> Json<serde_json::Value> {
    Json(serde_json::to_value(demo_domain_ir()).expect("demo IR is serializable"))
}

async fn list_events(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let events = state.store.list_events(100).await?;
    Ok(Json(serde_json::json!({ "data": events })))
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    email: String,
    #[serde(default)]
    verified: bool,
}

async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = state
        .service
        .create_user(CreateUserCommand {
            email: Email::parse(request.email)?,
            verified: request.verified,
        })
        .await?;

    Ok(Json(serde_json::json!({ "data": user })))
}

async fn verify_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = state.service.verify_user(UserId(user_id)).await?;
    Ok(Json(serde_json::json!({ "data": user })))
}

#[derive(Debug, Deserialize)]
struct CreateOrderRequest {
    buyer_id: Uuid,
    total_cents: i64,
    currency: String,
}

async fn create_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateOrderRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let actor = actor_from_headers(&headers)?;
    let order = state
        .service
        .create_order(
            actor,
            CreateOrderCommand {
                buyer_id: UserId(request.buyer_id),
                total: Money::parse(request.total_cents, request.currency)?,
            },
        )
        .await?;

    Ok(Json(serde_json::json!({ "data": order })))
}

async fn get_order(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use evobase_domain::OrderRepository;

    let order = state
        .store
        .find_order(OrderId(order_id))
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("order {order_id}")))?;

    Ok(Json(serde_json::json!({ "data": order })))
}

#[derive(Debug, Deserialize)]
struct PayOrderRequest {
    payment_id: Uuid,
}

async fn pay_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<Uuid>,
    Json(request): Json<PayOrderRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let actor = actor_from_headers(&headers)?;
    let order = state
        .service
        .pay_order(actor, OrderId(order_id), PaymentId(request.payment_id))
        .await?;

    Ok(Json(serde_json::json!({ "data": order })))
}

async fn ship_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let actor = actor_from_headers(&headers)?;
    let order = state.service.ship_order(actor, OrderId(order_id)).await?;
    Ok(Json(serde_json::json!({ "data": order })))
}

#[derive(Debug, Deserialize)]
struct CancelOrderRequest {
    reason: String,
}

async fn cancel_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<Uuid>,
    Json(request): Json<CancelOrderRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let actor = actor_from_headers(&headers)?;
    let order = state
        .service
        .cancel_order(
            actor,
            OrderId(order_id),
            NonEmptyString::parse(request.reason)?,
        )
        .await?;

    Ok(Json(serde_json::json!({ "data": order })))
}

fn actor_from_headers(headers: &HeaderMap) -> Result<Actor, ApiError> {
    let actor_id = required_header(headers, "x-actor-id")?;
    let actor_id = Uuid::parse_str(actor_id)
        .map_err(|error| DomainError::Validation(format!("x-actor-id must be a UUID: {error}")))?;

    let verified = optional_header(headers, "x-actor-verified")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let capabilities = optional_header(headers, "x-capabilities")
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(parse_capability)
        .collect::<Result<BTreeSet<_>, _>>()?;

    Ok(Actor {
        user_id: UserId(actor_id),
        verified,
        capabilities,
    })
}

fn parse_capability(value: &str) -> Result<Capability, ApiError> {
    match value {
        "pay_order" => Ok(Capability::PayOrder),
        "ship_order" => Ok(Capability::ShipOrder),
        "cancel_order" => Ok(Capability::CancelOrder),
        other => Err(DomainError::Validation(format!(
            "unknown capability in x-capabilities: {other}"
        ))
        .into()),
    }
}

fn required_header<'a>(headers: &'a HeaderMap, name: &str) -> Result<&'a str, ApiError> {
    optional_header(headers, name)
        .ok_or_else(|| DomainError::PolicyDenied(format!("missing required header {name}")).into())
}

fn optional_header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

struct ApiError(DomainError);

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            DomainError::Validation(_) => StatusCode::BAD_REQUEST,
            DomainError::NotFound(_) => StatusCode::NOT_FOUND,
            DomainError::PolicyDenied(_) => StatusCode::FORBIDDEN,
            DomainError::IllegalTransition { .. } | DomainError::Conflict => StatusCode::CONFLICT,
            DomainError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(serde_json::json!({
            "error": {
                "message": self.0.to_string()
            }
        }));

        (status, body).into_response()
    }
}
