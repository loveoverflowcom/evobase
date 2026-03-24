use axum::{Json, extract::State};
use evobase_protocol::{
    ApiResponse, AuthResponseDto, LoginRequest, RefreshRequest, RegisterRequest,
};

use crate::{ApiResult, AppState};

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> ApiResult<Json<ApiResponse<AuthResponseDto>>> {
    let domain_request = request.into();
    let response = state.auth_service.register(domain_request).await?;
    let dto = AuthResponseDto::from(response);
    Ok(Json(ApiResponse::new(dto)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> ApiResult<Json<ApiResponse<AuthResponseDto>>> {
    let domain_request = request.into();
    let response = state.auth_service.login(domain_request).await?;
    let dto = AuthResponseDto::from(response);
    Ok(Json(ApiResponse::new(dto)))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> ApiResult<Json<ApiResponse<AuthResponseDto>>> {
    let response = state.auth_service.refresh(&request.refresh_token).await?;
    let dto = AuthResponseDto::from(response);
    Ok(Json(ApiResponse::new(dto)))
}
