use axum::Json;
use evobase_protocol::ApiResponse;
use serde_json::{Value, json};

use crate::ApiResult;

pub async fn healthcheck() -> ApiResult<Json<ApiResponse<Value>>> {
    Ok(Json(ApiResponse::new(json!({ "status": "ok" }))))
}
