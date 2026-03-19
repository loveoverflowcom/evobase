use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppResult;

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub type RegisterRequest = Credentials;
pub type LoginRequest = Credentials;

#[derive(Debug, Clone, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: Uuid,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenBundle {
    pub access_token: String,
    pub refresh_token: String,
    pub notification_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub user_id: Uuid,
    pub username: String,
    pub tokens: TokenBundle,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
}

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn register(&self, request: RegisterRequest) -> AppResult<AuthResponse>;
    async fn login(&self, request: LoginRequest) -> AppResult<AuthResponse>;
    async fn refresh(&self, refresh_token: &str) -> AppResult<AuthResponse>;
    fn verify_access_token(&self, token: &str) -> AppResult<AuthContext>;
    fn verify_notification_token(&self, token: &str) -> AppResult<AuthContext>;
}
