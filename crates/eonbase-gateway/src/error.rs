use axum::{
    Json,
    response::{IntoResponse, Response},
};
use eonbase_core::AppError;
use serde::Serialize;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub struct ApiError(pub AppError);

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.0.status_code();
        let body = Json(ErrorBody {
            error: self.0.public_message(),
        });

        (status, body).into_response()
    }
}
