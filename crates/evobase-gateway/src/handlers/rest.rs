use axum::{Extension, Json, extract::{Path, RawQuery, State}};
use evobase_core::AuthContext;
use evobase_protocol::ApiResponse;
use serde_json::Value;

use crate::{
    ApiResult, AppState,
    rest::{parse_delete_request, parse_insert_request, parse_select_request, parse_update_request},
};

pub async fn select_rows(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path(table): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<ApiResponse<Vec<Value>>>> {
    let request = parse_select_request(&table, raw_query.as_deref())?;
    let rows = state
        .storage
        .select_rows(request, Some(&current_user))
        .await?;

    Ok(Json(ApiResponse::new(rows)))
}

pub async fn insert_rows(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path(table): Path<String>,
    Json(body): Json<Value>,
) -> ApiResult<Json<ApiResponse<Vec<Value>>>> {
    let request = parse_insert_request(&table, body)?;
    let rows = state
        .storage
        .insert_rows(request, Some(&current_user))
        .await?;

    Ok(Json(ApiResponse::new(rows)))
}

pub async fn update_rows(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path(table): Path<String>,
    RawQuery(raw_query): RawQuery,
    Json(body): Json<Value>,
) -> ApiResult<Json<ApiResponse<Vec<Value>>>> {
    let request = parse_update_request(&table, raw_query.as_deref(), body)?;
    let rows = state
        .storage
        .update_rows(request, Some(&current_user))
        .await?;

    Ok(Json(ApiResponse::new(rows)))
}

pub async fn delete_rows(
    State(state): State<AppState>,
    Extension(current_user): Extension<AuthContext>,
    Path(table): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<Json<ApiResponse<Vec<Value>>>> {
    let request = parse_delete_request(&table, raw_query.as_deref())?;
    let rows = state
        .storage
        .delete_rows(request, Some(&current_user))
        .await?;

    Ok(Json(ApiResponse::new(rows)))
}
