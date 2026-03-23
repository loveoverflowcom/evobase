use axum::{
    Json,
    response::{IntoResponse, Response},
};
use evobase_core::AppError;
use evobase_protocol::ErrorEnvelope;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.0.status_code();
        let body = Json(ErrorEnvelope {
            error: evobase_protocol::ErrorDetail {
                code: error_code(&self.0),
                message: self.0.public_message(),
                field: None,
            },
        });

        (status, body).into_response()
    }
}

fn error_code(error: &AppError) -> &'static str {
    match error {
        AppError::Config(_) => "config_error",
        AppError::BadRequest(_) => "bad_request",
        AppError::Unauthorized => "unauthorized",
        AppError::NotFound(_) => "not_found",
        AppError::Conflict(_) => "conflict",
        AppError::Database(_) => "database_error",
        AppError::Internal(_) => "internal_error",
    }
}
