pub mod appspec;
#[cfg(feature = "legacy")]
mod auth;
#[cfg(feature = "legacy")]
mod databases;
#[cfg(feature = "legacy")]
mod docs;
#[cfg(feature = "legacy")]
mod envelope;
#[cfg(feature = "legacy")]
mod rest;

#[cfg(feature = "legacy")]
pub use auth::{AuthResponseDto, LoginRequest, RefreshRequest, RegisterRequest, TokenDto};
#[cfg(feature = "legacy")]
pub use databases::{
    BootstrapDatabaseRequestDto, BootstrapFailureStageDto, BootstrapSqlScriptDto,
    DatabaseBootstrapFailureDto, DatabaseCatalogDto, DatabaseDto, DatabaseKindDto,
    DatabaseStatusDto, ExistingDatabasePolicyDto,
};
#[cfg(feature = "legacy")]
pub use docs::{ApiDocsDto, TableDocDto};
#[cfg(feature = "legacy")]
pub use envelope::{ApiResponse, ErrorDetail, ErrorEnvelope, ResponseMeta};
#[cfg(feature = "legacy")]
pub use rest::{InsertBody, PatchBody, TableQueryParams};
