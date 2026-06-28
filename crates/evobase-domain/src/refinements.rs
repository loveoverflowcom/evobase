use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn parse(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::Validation(
                "NonEmptyString cannot be empty".to_string(),
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Email(String);

impl Email {
    pub fn parse(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into().trim().to_ascii_lowercase();
        let has_basic_shape = value.len() >= 3
            && value.contains('@')
            && value
                .split_once('@')
                .map(|(local, domain)| !local.is_empty() && domain.contains('.'))
                .unwrap_or(false);

        if !has_basic_shape {
            return Err(DomainError::Validation(format!(
                "invalid email address: {value}"
            )));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PositiveInt(i64);

impl PositiveInt {
    pub fn parse(value: i64) -> DomainResult<Self> {
        if value <= 0 {
            return Err(DomainError::Validation(format!(
                "expected a positive integer, got {value}"
            )));
        }
        Ok(Self(value))
    }

    pub fn get(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    USD,
    VND,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::USD => f.write_str("USD"),
            Self::VND => f.write_str("VND"),
        }
    }
}

impl FromStr for Currency {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_uppercase().as_str() {
            "USD" => Ok(Self::USD),
            "VND" => Ok(Self::VND),
            other => Err(DomainError::Validation(format!(
                "unsupported currency: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub cents: PositiveInt,
    pub currency: Currency,
}

impl Money {
    pub fn parse(cents: i64, currency: impl AsRef<str>) -> DomainResult<Self> {
        Ok(Self {
            cents: PositiveInt::parse(cents)?,
            currency: Currency::from_str(currency.as_ref())?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VerifiedUserId(pub UserId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrderId(pub Uuid);

impl OrderId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for OrderId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PaymentId(pub Uuid);
