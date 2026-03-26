use axum::{
    Router, middleware,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    AppState, handlers,
    middleware::{require_access_token, require_admin_token},
};

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
        .route(
            "/rest/{table}/{value}",
            get(handlers::rest::select_row_by_primary_key),
        )
        .route(
            "/rest/{table}/{lookup}/{value}",
            get(handlers::rest::select_row_by_unique_field),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_access_token,
        ));

    let admin_protected = Router::new()
        .route("/docs", get(handlers::docs::list_docs))
        .route("/docs/{table}", get(handlers::docs::get_table_docs))
        .route(
            "/admin/databases",
            get(handlers::databases::list_databases).post(handlers::databases::bootstrap_database),
        )
        .route(
            "/admin/databases/{database_id}",
            get(handlers::databases::get_database),
        )
        .route(
            "/admin/databases/{database_id}/docs",
            get(handlers::databases::list_database_docs),
        )
        .route(
            "/admin/databases/{database_id}/docs/{table}",
            get(handlers::databases::get_database_table_docs),
        )
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
        .route(
            "/evobase_messaging/publish",
            post(handlers::topic_messaging::publish),
        )
        .route(
            "/evobase_messaging/subscribe",
            get(handlers::topic_messaging::subscribe),
        )
        .merge(protected)
        .merge(admin_protected)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
