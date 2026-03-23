use axum::{Router, middleware, routing::{get, post}};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{AppState, handlers, middleware::{require_access_token, require_admin_token}};

pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/messages/send", post(handlers::messaging::send_message))
        .route(
            "/rest/{table}",
            get(handlers::rest::select_rows)
                .post(handlers::rest::insert_rows)
                .patch(handlers::rest::update_rows)
                .delete(handlers::rest::delete_rows),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_access_token,
        ));

    let admin_protected = Router::new()
        .route("/docs", get(handlers::docs::list_docs))
        .route("/docs/{table}", get(handlers::docs::get_table_docs))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_admin_token,
        ));

    Router::new()
        .route("/healthz", get(handlers::health::healthcheck))
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh))
        .route("/events", get(handlers::messaging::events))
        .merge(protected)
        .merge(admin_protected)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
