use thiserror::Error;

pub type DomainResult<T> = Result<T, DomainError>;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation failed: {0}")]
    Validation(String),

    #[error("resource not found: {0}")]
    NotFound(String),

    #[error("policy denied: {0}")]
    PolicyDenied(String),

    #[error("illegal workflow transition: expected {expected}, found {found}")]
    IllegalTransition { expected: String, found: String },

    #[error("concurrent modification detected")]
    Conflict,

    #[error("storage error: {0}")]
    Storage(String),
}
