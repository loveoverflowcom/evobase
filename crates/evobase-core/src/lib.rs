pub mod auth;
pub mod config;
pub mod docs;
pub mod error;
pub mod messaging;
pub mod rest;
pub mod storage;

pub use auth::{
    AuthContext, AuthResponse, AuthService, Credentials, LoginRequest, RefreshRequest,
    RegisterRequest, TokenBundle, TokenClaims,
};
pub use config::{AdminConfig, AppConfig, DatabaseConfig, ServerConfig, TokenConfig};
pub use docs::{ApiDocs, ColumnDoc, QueryDoc, RlsDoc, RlsPolicyDoc, TableDoc, TableMethods};
pub use error::{AppError, AppResult};
pub use messaging::{
    MessagingConnection, MessagingDelivery, MessagingService, SendMessageRequest, ServerEvent,
};
pub use rest::{
    Filter, FilterOperator, OrderBy, QualifiedTable, SelectList, TableDelete, TableInsert,
    TableSelect, TableUpdate, quoted_identifier, validate_identifier,
};
pub use storage::{StorageAdapter, UserRecord};
