use std::{env, net::SocketAddr, str::FromStr};

use crate::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub tokens: TokenConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub addr: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct TokenConfig {
    pub access_secret: String,
    pub refresh_secret: String,
    pub notification_secret: String,
    pub access_ttl_secs: u64,
    pub refresh_ttl_secs: u64,
    pub notification_ttl_secs: u64,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        let _ = dotenvy::dotenv();

        Ok(Self {
            server: ServerConfig {
                addr: env_optional_parse("SERVER_ADDR")?
                    .unwrap_or_else(|| "0.0.0.0:3000".parse().expect("valid default address")),
            },
            database: DatabaseConfig {
                url: env_required("DATABASE_URL")?,
            },
            tokens: TokenConfig {
                access_secret: env_required("ACCESS_TOKEN_SECRET")?,
                refresh_secret: env_required("REFRESH_TOKEN_SECRET")?,
                notification_secret: env_required("NOTIFICATION_TOKEN_SECRET")?,
                access_ttl_secs: env_optional_parse("ACCESS_TOKEN_TTL_SECS")?.unwrap_or(900),
                refresh_ttl_secs: env_optional_parse("REFRESH_TOKEN_TTL_SECS")?
                    .unwrap_or(60 * 60 * 24 * 30),
                notification_ttl_secs: env_optional_parse("NOTIFICATION_TOKEN_TTL_SECS")?
                    .unwrap_or(60 * 60 * 24 * 30),
            },
        })
    }
}

fn env_required(name: &str) -> AppResult<String> {
    env::var(name).map_err(|_| AppError::Config(format!("missing required env var {name}")))
}

fn env_optional_parse<T>(name: &str) -> AppResult<Option<T>>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(name) {
        Ok(value) => value
            .parse::<T>()
            .map(Some)
            .map_err(|error| AppError::Config(format!("invalid value for {name}: {error}"))),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(AppError::Config(format!(
            "env var {name} is not valid unicode"
        ))),
    }
}
