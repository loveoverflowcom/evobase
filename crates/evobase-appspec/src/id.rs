use crate::Error;
use serde::{Deserialize, Deserializer, Serialize};

macro_rules! identity {
    ($name:ident, $prefix:literal, $kind:literal) => {
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);
        impl $name {
            pub fn new(value: impl AsRef<str>) -> Result<Self, Error> {
                let value = value.as_ref();
                if value.len() > 80
                    || !value.starts_with($prefix)
                    || value.len() == $prefix.len()
                    || !value
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
                {
                    return Err(Error::InvalidId {
                        kind: $kind,
                        value: value.to_owned(),
                    });
                }
                Ok(Self(value.to_owned()))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl TryFrom<&str> for $name {
            type Error = Error;
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
                let value = String::deserialize(de)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}
identity!(AppId, "app_", "app");
identity!(TableId, "tbl_", "table");
identity!(FieldId, "fld_", "field");
identity!(RecordId, "rec_", "record");
identity!(ConstraintId, "constraint_", "constraint");

/// A syntactically valid scope supplied by the caller; it does not establish authorization.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Scope {
    tenant_id: String,
    app_id: AppId,
}
impl Scope {
    pub fn new(tenant_id: impl AsRef<str>, app_id: AppId) -> Result<Self, Error> {
        let tenant = tenant_id.as_ref();
        if tenant.is_empty()
            || tenant.len() > 80
            || !tenant
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        {
            return Err(Error::InvalidTenant);
        }
        Ok(Self {
            tenant_id: tenant.to_owned(),
            app_id,
        })
    }
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }
    pub fn app_id(&self) -> &AppId {
        &self.app_id
    }
}
impl<'de> Deserialize<'de> for Scope {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawScope {
            tenant_id: String,
            app_id: AppId,
        }
        let raw = RawScope::deserialize(de)?;
        Self::new(raw.tenant_id, raw.app_id).map_err(serde::de::Error::custom)
    }
}
