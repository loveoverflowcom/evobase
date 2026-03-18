use http::StatusCode;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("database error")]
    Database(String),
    #[error("internal server error")]
    Internal(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Config(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Database(_) => StatusCode::BAD_GATEWAY,
        }
    }

    pub fn public_message(&self) -> String {
        match self {
            Self::Config(message)
            | Self::BadRequest(message)
            | Self::NotFound(message)
            | Self::Conflict(message) => message.clone(),
            Self::Unauthorized => "unauthorized".to_string(),
            Self::Database(_) => "database error".to_string(),
            Self::Internal(_) => "internal server error".to_string(),
        }
    }
}
