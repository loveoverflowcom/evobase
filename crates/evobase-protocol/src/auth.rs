use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponseDto {
    pub user_id: Uuid,
    pub username: String,
    pub tokens: TokenDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenDto {
    pub access_token: String,
    pub refresh_token: String,
    pub notification_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

impl From<evobase_core::AuthResponse> for AuthResponseDto {
    fn from(r: evobase_core::AuthResponse) -> Self {
        Self {
            user_id: r.user_id,
            username: r.username,
            tokens: TokenDto {
                access_token: r.tokens.access_token,
                refresh_token: r.tokens.refresh_token,
                notification_token: r.tokens.notification_token,
                token_type: r.tokens.token_type.to_string(),
                expires_in: r.tokens.expires_in,
            },
        }
    }
}

impl From<RegisterRequest> for evobase_core::RegisterRequest {
    fn from(r: RegisterRequest) -> Self {
        Self {
            username: r.username,
            password: r.password,
        }
    }
}

impl From<LoginRequest> for evobase_core::LoginRequest {
    fn from(r: LoginRequest) -> Self {
        Self {
            username: r.username,
            password: r.password,
        }
    }
}
