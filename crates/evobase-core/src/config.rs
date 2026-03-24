use std::{env, net::SocketAddr, str::FromStr};

use crate::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub tokens: TokenConfig,
    pub admin: AdminConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub addr: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub admin_url: String,
    pub default_id: String,
    pub default_postgres_database: String,
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

#[derive(Debug, Clone)]
pub struct AdminConfig {
    pub token: String,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        let _ = dotenvy::dotenv();
        Self::from_env_reader(|name| env::var(name))
    }

    fn from_env_reader<F>(mut read_env: F) -> AppResult<Self>
    where
        F: FnMut(&str) -> Result<String, env::VarError>,
    {
        let database_url = env_required(&mut read_env, "DATABASE_URL")?;
        Ok(Self {
            server: ServerConfig {
                addr: env_optional_parse(&mut read_env, "SERVER_ADDR")?
                    .unwrap_or_else(|| "0.0.0.0:3000".parse().expect("valid default address")),
            },
            database: DatabaseConfig {
                admin_url: env_optional(&mut read_env, "DATABASE_ADMIN_URL")?
                    .unwrap_or_else(|| database_url.clone()),
                default_id: env_optional(&mut read_env, "DATABASE_DEFAULT_ID")?
                    .unwrap_or_else(|| "default".to_string()),
                default_postgres_database: extract_database_name(&database_url)?,
                url: database_url,
            },
            tokens: TokenConfig {
                access_secret: env_required(&mut read_env, "ACCESS_TOKEN_SECRET")?,
                refresh_secret: env_required(&mut read_env, "REFRESH_TOKEN_SECRET")?,
                notification_secret: env_required(&mut read_env, "NOTIFICATION_TOKEN_SECRET")?,
                access_ttl_secs: env_optional_parse(&mut read_env, "ACCESS_TOKEN_TTL_SECS")?
                    .unwrap_or(900),
                refresh_ttl_secs: env_optional_parse(&mut read_env, "REFRESH_TOKEN_TTL_SECS")?
                    .unwrap_or(60 * 60 * 24 * 30),
                notification_ttl_secs: env_optional_parse(
                    &mut read_env,
                    "NOTIFICATION_TOKEN_TTL_SECS",
                )?
                .unwrap_or(60 * 60 * 24 * 30),
            },
            admin: AdminConfig {
                token: env_required(&mut read_env, "ADMIN_TOKEN")?,
            },
        })
    }
}

fn env_required<F>(read_env: &mut F, name: &str) -> AppResult<String>
where
    F: FnMut(&str) -> Result<String, env::VarError>,
{
    read_env(name).map_err(|_| AppError::Config(format!("missing required env var {name}")))
}

fn env_optional<F>(read_env: &mut F, name: &str) -> AppResult<Option<String>>
where
    F: FnMut(&str) -> Result<String, env::VarError>,
{
    match read_env(name) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(AppError::Config(format!(
            "env var {name} is not valid unicode"
        ))),
    }
}

fn env_optional_parse<F, T>(read_env: &mut F, name: &str) -> AppResult<Option<T>>
where
    F: FnMut(&str) -> Result<String, env::VarError>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match read_env(name) {
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

fn extract_database_name(database_url: &str) -> AppResult<String> {
    let url = url::Url::parse(database_url)
        .map_err(|error| AppError::Config(format!("invalid DATABASE_URL: {error}")))?;
    let database_name = url.path().trim_start_matches('/').trim();

    if database_name.is_empty() {
        return Err(AppError::Config(
            "DATABASE_URL must include a PostgreSQL database name".to_string(),
        ));
    }

    Ok(database_name.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::AppConfig;

    #[test]
    fn from_env_reader_supports_database_admin_url_and_default_id() {
        let values = [
            ("SERVER_ADDR", "127.0.0.1:4000"),
            (
                "DATABASE_URL",
                "postgres://postgres:postgres@localhost:5432/evobase",
            ),
            (
                "DATABASE_ADMIN_URL",
                "postgres://postgres:postgres@localhost:5432/postgres",
            ),
            ("DATABASE_DEFAULT_ID", "workspace"),
            ("ACCESS_TOKEN_SECRET", "access-secret"),
            ("REFRESH_TOKEN_SECRET", "refresh-secret"),
            ("NOTIFICATION_TOKEN_SECRET", "notification-secret"),
            ("ADMIN_TOKEN", "admin-secret"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<HashMap<_, _>>();

        let config = AppConfig::from_env_reader(|name| {
            values
                .get(name)
                .cloned()
                .ok_or(std::env::VarError::NotPresent)
        })
        .expect("config should parse");

        assert_eq!(
            config.database.admin_url,
            "postgres://postgres:postgres@localhost:5432/postgres"
        );
        assert_eq!(config.database.default_id, "workspace");
        assert_eq!(config.database.default_postgres_database, "evobase");
    }
}
