use evobase_core::{
    BootstrapDatabaseRequest, BootstrapFailureStage, BootstrapSqlScript, DatabaseBootstrapFailure,
    DatabaseKind, DatabaseStatus, ExistingDatabasePolicy, ManagedDatabase,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BootstrapDatabaseRequestDto {
    pub database_id: String,
    pub postgres_database: String,
    pub scripts: Vec<BootstrapSqlScriptDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default)]
    pub existing_database_policy: ExistingDatabasePolicyDto,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BootstrapSqlScriptDto {
    pub name: String,
    pub sql: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseCatalogDto {
    pub default_database_id: String,
    pub databases: Vec<DatabaseDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseDto {
    pub database_id: String,
    pub postgres_database: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub kind: DatabaseKindDto,
    pub status: DatabaseStatusDto,
    pub docs_available: bool,
    pub is_default: bool,
    pub script_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<DatabaseBootstrapFailureDto>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExistingDatabasePolicyDto {
    #[default]
    Fail,
    UseExisting,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseKindDto {
    Default,
    Bootstrapped,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseStatusDto {
    Ready,
    BootstrapFailed,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseBootstrapFailureDto {
    pub stage: BootstrapFailureStageDto,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapFailureStageDto {
    CreateDatabase,
    ConnectDatabase,
    ExecuteScript,
    IntrospectSchema,
}

impl DatabaseCatalogDto {
    pub fn from_core(default_database_id: &str, databases: Vec<ManagedDatabase>) -> Self {
        Self {
            default_database_id: default_database_id.to_string(),
            databases: databases
                .into_iter()
                .map(|database| DatabaseDto::from_core(default_database_id, database))
                .collect(),
        }
    }
}

impl DatabaseDto {
    pub fn from_core(default_database_id: &str, database: ManagedDatabase) -> Self {
        let is_default = database.database_id == default_database_id;
        let docs_available = database.docs_available();

        Self {
            database_id: database.database_id.clone(),
            postgres_database: database.postgres_database,
            description: database.description,
            tags: database.tags,
            owner: database.owner,
            kind: database.kind.into(),
            status: database.status.into(),
            docs_available,
            is_default,
            script_count: database.script_count,
            failure: database.failure.map(Into::into),
        }
    }
}

impl From<BootstrapDatabaseRequestDto> for BootstrapDatabaseRequest {
    fn from(value: BootstrapDatabaseRequestDto) -> Self {
        Self {
            database_id: value.database_id,
            postgres_database: value.postgres_database,
            scripts: value.scripts.into_iter().map(Into::into).collect(),
            description: value.description,
            tags: value.tags,
            owner: value.owner,
            existing_database_policy: value.existing_database_policy.into(),
        }
    }
}

impl From<BootstrapSqlScriptDto> for BootstrapSqlScript {
    fn from(value: BootstrapSqlScriptDto) -> Self {
        Self {
            name: value.name,
            sql: value.sql,
        }
    }
}

impl From<ExistingDatabasePolicyDto> for ExistingDatabasePolicy {
    fn from(value: ExistingDatabasePolicyDto) -> Self {
        match value {
            ExistingDatabasePolicyDto::Fail => Self::Fail,
            ExistingDatabasePolicyDto::UseExisting => Self::UseExisting,
        }
    }
}

impl From<DatabaseKind> for DatabaseKindDto {
    fn from(value: DatabaseKind) -> Self {
        match value {
            DatabaseKind::Default => Self::Default,
            DatabaseKind::Bootstrapped => Self::Bootstrapped,
        }
    }
}

impl From<DatabaseStatus> for DatabaseStatusDto {
    fn from(value: DatabaseStatus) -> Self {
        match value {
            DatabaseStatus::Ready => Self::Ready,
            DatabaseStatus::BootstrapFailed => Self::BootstrapFailed,
        }
    }
}

impl From<DatabaseBootstrapFailure> for DatabaseBootstrapFailureDto {
    fn from(value: DatabaseBootstrapFailure) -> Self {
        Self {
            stage: value.stage.into(),
            message: value.message,
            script_name: value.script_name,
            script_index: value.script_index,
        }
    }
}

impl From<BootstrapFailureStage> for BootstrapFailureStageDto {
    fn from(value: BootstrapFailureStage) -> Self {
        match value {
            BootstrapFailureStage::CreateDatabase => Self::CreateDatabase,
            BootstrapFailureStage::ConnectDatabase => Self::ConnectDatabase,
            BootstrapFailureStage::ExecuteScript => Self::ExecuteScript,
            BootstrapFailureStage::IntrospectSchema => Self::IntrospectSchema,
        }
    }
}
