use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use async_trait::async_trait;
use eonbase_core::{
    AppError, AppResult, AuthContext, AuthResponse, AuthService, LoginRequest, RegisterRequest,
    StorageAdapter, TokenBundle, TokenClaims, TokenConfig,
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand_core::OsRng;
use tracing::warn;
use uuid::Uuid;

pub struct JwtAuthService {
    storage: Arc<dyn StorageAdapter>,
    access_encoding: EncodingKey,
    access_decoding: DecodingKey,
    refresh_encoding: EncodingKey,
    refresh_decoding: DecodingKey,
    notification_encoding: EncodingKey,
    notification_decoding: DecodingKey,
    token_config: TokenConfig,
}

impl JwtAuthService {
    pub fn new(storage: Arc<dyn StorageAdapter>, token_config: TokenConfig) -> Self {
        Self {
            storage,
            access_encoding: EncodingKey::from_secret(token_config.access_secret.as_bytes()),
            access_decoding: DecodingKey::from_secret(token_config.access_secret.as_bytes()),
            refresh_encoding: EncodingKey::from_secret(token_config.refresh_secret.as_bytes()),
            refresh_decoding: DecodingKey::from_secret(token_config.refresh_secret.as_bytes()),
            notification_encoding: EncodingKey::from_secret(
                token_config.notification_secret.as_bytes(),
            ),
            notification_decoding: DecodingKey::from_secret(
                token_config.notification_secret.as_bytes(),
            ),
            token_config,
        }
    }

    fn validate_credentials(username: &str, password: &str) -> AppResult<()> {
        let username = username.trim();

        if username.len() < 3 || username.len() > 64 {
            return Err(AppError::BadRequest(
                "username must be between 3 and 64 characters".to_string(),
            ));
        }

        if !username
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        {
            return Err(AppError::BadRequest(
                "username may only contain letters, numbers, `_`, and `-`".to_string(),
            ));
        }

        if password.len() < 8 {
            return Err(AppError::BadRequest(
                "password must be at least 8 characters".to_string(),
            ));
        }

        Ok(())
    }

    fn hash_password(password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|error| AppError::Internal(format!("failed to hash password: {error}")))
    }

    fn verify_password(password: &str, password_hash: &str) -> AppResult<()> {
        let parsed_hash = PasswordHash::new(password_hash).map_err(|error| {
            AppError::Internal(format!("stored password hash is invalid: {error}"))
        })?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized)
    }

    fn issue_bundle(&self, user_id: Uuid) -> AppResult<TokenBundle> {
        Ok(TokenBundle {
            access_token: self.issue_token(
                user_id,
                self.token_config.access_ttl_secs,
                &self.access_encoding,
            )?,
            refresh_token: self.issue_token(
                user_id,
                self.token_config.refresh_ttl_secs,
                &self.refresh_encoding,
            )?,
            notification_token: self.issue_token(
                user_id,
                self.token_config.notification_ttl_secs,
                &self.notification_encoding,
            )?,
            token_type: "Bearer",
            expires_in: self.token_config.access_ttl_secs,
        })
    }

    fn issue_token(
        &self,
        user_id: Uuid,
        ttl_secs: u64,
        encoding_key: &EncodingKey,
    ) -> AppResult<String> {
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| AppError::Internal(format!("system clock error: {error}")))?
            .as_secs()
            .saturating_add(ttl_secs);

        let claims = TokenClaims {
            sub: user_id,
            exp: exp as usize,
        };

        encode(&Header::default(), &claims, encoding_key)
            .map_err(|error| AppError::Internal(format!("failed to encode token: {error}")))
    }

    fn decode_token(token: &str, decoding_key: &DecodingKey) -> AppResult<AuthContext> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        decode::<TokenClaims>(token, decoding_key, &validation)
            .map(|data| AuthContext {
                user_id: data.claims.sub,
            })
            .map_err(|_| AppError::Unauthorized)
    }

    fn build_auth_response(&self, user_id: Uuid, username: String) -> AppResult<AuthResponse> {
        Ok(AuthResponse {
            user_id,
            username,
            tokens: self.issue_bundle(user_id)?,
        })
    }
}

#[async_trait]
impl AuthService for JwtAuthService {
    async fn register(&self, request: RegisterRequest) -> AppResult<AuthResponse> {
        let username = request.username.trim().to_lowercase();
        Self::validate_credentials(&username, &request.password)?;

        let password_hash = Self::hash_password(&request.password)?;
        let user = self.storage.create_user(&username, &password_hash).await?;

        self.build_auth_response(user.id, user.username)
    }

    async fn login(&self, request: LoginRequest) -> AppResult<AuthResponse> {
        let username = request.username.trim().to_lowercase();

        let Some(user) = self.storage.find_user_by_username(&username).await? else {
            return Err(AppError::Unauthorized);
        };

        Self::verify_password(&request.password, &user.password_hash)?;

        self.build_auth_response(user.id, user.username)
    }

    async fn refresh(&self, refresh_token: &str) -> AppResult<AuthResponse> {
        let auth = Self::decode_token(refresh_token, &self.refresh_decoding)?;
        let Some(user) = self.storage.find_user_by_id(auth.user_id).await? else {
            warn!(user_id = %auth.user_id, "refresh attempted for missing user");
            return Err(AppError::Unauthorized);
        };

        self.build_auth_response(user.id, user.username)
    }

    fn verify_access_token(&self, token: &str) -> AppResult<AuthContext> {
        Self::decode_token(token, &self.access_decoding)
    }

    fn verify_notification_token(&self, token: &str) -> AppResult<AuthContext> {
        Self::decode_token(token, &self.notification_decoding)
    }
}
