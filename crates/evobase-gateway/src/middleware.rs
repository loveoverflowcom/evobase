use axum::{
    body::Body,
    extract::State,
    http::{Request, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use evobase_core::{AppError, AuthContext};

use crate::{ApiError, AppState};

pub async fn require_access_token(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = extract_bearer_token(header)?;
    let auth_context = state.auth_service.verify_access_token(token)?;

    request.extensions_mut().insert(AuthContext {
        user_id: auth_context.user_id,
    });

    Ok(next.run(request).await)
}

pub(crate) fn extract_bearer_token(header: &str) -> Result<&str, AppError> {
    let Some(token) = header.strip_prefix("Bearer ") else {
        return Err(AppError::Unauthorized);
    };

    if token.is_empty() {
        return Err(AppError::Unauthorized);
    }

    Ok(token)
}
