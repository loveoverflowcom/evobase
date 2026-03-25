use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc};
use tokio::sync::{Mutex, RwLock};

use crate::{AppError, AppResult, QualifiedTable, StorageAdapter, TableDoc, validate_identifier};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseKind {
    Default,
    Bootstrapped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseStatus {
    Ready,
    BootstrapFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExistingDatabasePolicy {
    #[default]
    Fail,
    UseExisting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapFailureStage {
    CreateDatabase,
    ConnectDatabase,
    ExecuteScript,
    IntrospectSchema,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseBootstrapFailure {
    pub stage: BootstrapFailureStage,
    pub message: String,
    pub script_name: Option<String>,
    pub script_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapSqlScript {
    pub name: String,
    pub sql: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapDatabaseRequest {
    pub database_id: String,
    pub postgres_database: String,
    pub scripts: Vec<BootstrapSqlScript>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
    pub existing_database_policy: ExistingDatabasePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedDatabase {
    pub database_id: String,
    pub postgres_database: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
    pub kind: DatabaseKind,
    pub status: DatabaseStatus,
    pub script_count: usize,
    pub failure: Option<DatabaseBootstrapFailure>,
}

struct DatabaseEntry {
    database: ManagedDatabase,
    storage: Option<Arc<dyn StorageAdapter>>,
}

pub struct DatabaseRegistry {
    default_database_id: String,
    entries: RwLock<BTreeMap<String, DatabaseEntry>>,
}

pub trait DatabaseProvisioner: Send + Sync {
    fn provision_database(
        &self,
        request: &BootstrapDatabaseRequest,
    ) -> Pin<Box<dyn Future<Output = AppResult<ProvisionDatabaseOutcome>> + Send>>;
}

pub enum ProvisionDatabaseOutcome {
    Ready { storage: Arc<dyn StorageAdapter> },
    Failed { failure: DatabaseBootstrapFailure },
}

pub struct DatabaseManager {
    registry: Arc<DatabaseRegistry>,
    provisioner: Arc<dyn DatabaseProvisioner>,
    bootstrap_lock: Mutex<()>,
}

impl BootstrapDatabaseRequest {
    pub fn validate(&self) -> AppResult<()> {
        validate_database_id(&self.database_id)?;
        validate_identifier(&self.postgres_database)?;

        if self.scripts.is_empty() {
            return Err(AppError::BadRequest(
                "bootstrap request must contain at least one SQL script".to_string(),
            ));
        }

        validate_optional_text("description", self.description.as_deref(), 512)?;
        validate_optional_text("owner", self.owner.as_deref(), 128)?;

        for tag in &self.tags {
            validate_tag(tag)?;
        }

        for (index, script) in self.scripts.iter().enumerate() {
            let script_number = index + 1;
            let script_name = script.name.trim();
            if script_name.is_empty() {
                return Err(AppError::BadRequest(format!(
                    "script #{script_number} must have a non-empty name"
                )));
            }

            if script_name.chars().count() > 128 {
                return Err(AppError::BadRequest(format!(
                    "script `{script_name}` is too long; max 128 characters"
                )));
            }

            if script.sql.trim().is_empty() {
                return Err(AppError::BadRequest(format!(
                    "script `{script_name}` cannot be empty"
                )));
            }
        }

        Ok(())
    }
}

impl ManagedDatabase {
    pub fn default_database(database_id: String, postgres_database: String) -> Self {
        Self {
            database_id,
            postgres_database,
            description: Some("Default database from DATABASE_URL".to_string()),
            tags: Vec::new(),
            owner: None,
            kind: DatabaseKind::Default,
            status: DatabaseStatus::Ready,
            script_count: 0,
            failure: None,
        }
    }

    pub fn from_bootstrap_request(
        request: &BootstrapDatabaseRequest,
        status: DatabaseStatus,
        failure: Option<DatabaseBootstrapFailure>,
    ) -> Self {
        Self {
            database_id: request.database_id.clone(),
            postgres_database: request.postgres_database.clone(),
            description: request.description.clone(),
            tags: request.tags.clone(),
            owner: request.owner.clone(),
            kind: DatabaseKind::Bootstrapped,
            status,
            script_count: request.scripts.len(),
            failure,
        }
    }

    pub fn docs_available(&self) -> bool {
        self.status == DatabaseStatus::Ready
    }
}

impl DatabaseRegistry {
    pub fn new(
        default_database: ManagedDatabase,
        default_storage: Arc<dyn StorageAdapter>,
    ) -> Self {
        let default_database_id = default_database.database_id.clone();
        let mut entries = BTreeMap::new();
        entries.insert(
            default_database.database_id.clone(),
            DatabaseEntry {
                database: default_database,
                storage: Some(default_storage),
            },
        );

        Self {
            default_database_id,
            entries: RwLock::new(entries),
        }
    }

    pub fn default_database_id(&self) -> &str {
        &self.default_database_id
    }

    pub async fn list(&self) -> Vec<ManagedDatabase> {
        let entries = self.entries.read().await;
        let mut databases = entries
            .values()
            .map(|entry| entry.database.clone())
            .collect::<Vec<_>>();

        databases.sort_by(|left, right| {
            let left_default = left.database_id == self.default_database_id;
            let right_default = right.database_id == self.default_database_id;

            right_default
                .cmp(&left_default)
                .then_with(|| left.database_id.cmp(&right.database_id))
        });

        databases
    }

    pub async fn get(&self, database_id: &str) -> Option<ManagedDatabase> {
        let entries = self.entries.read().await;
        entries.get(database_id).map(|entry| entry.database.clone())
    }

    pub async fn contains(&self, database_id: &str) -> bool {
        let entries = self.entries.read().await;
        entries.contains_key(database_id)
    }

    pub async fn register(
        &self,
        database: ManagedDatabase,
        storage: Option<Arc<dyn StorageAdapter>>,
    ) -> AppResult<()> {
        let mut entries = self.entries.write().await;
        if entries.contains_key(&database.database_id) {
            return Err(AppError::Conflict(format!(
                "database `{}` is already registered",
                database.database_id
            )));
        }

        entries.insert(
            database.database_id.clone(),
            DatabaseEntry { database, storage },
        );
        Ok(())
    }

    async fn resolve(
        &self,
        database_id: Option<&str>,
    ) -> AppResult<(ManagedDatabase, Arc<dyn StorageAdapter>)> {
        let resolved_id = database_id.unwrap_or(&self.default_database_id);
        let entries = self.entries.read().await;
        let entry = entries
            .get(resolved_id)
            .ok_or_else(|| AppError::NotFound(format!("database `{resolved_id}` was not found")))?;

        if entry.database.status != DatabaseStatus::Ready {
            let detail = entry
                .database
                .failure
                .as_ref()
                .map(|failure| failure.message.clone())
                .unwrap_or_else(|| "bootstrap has not completed successfully".to_string());

            return Err(AppError::Conflict(format!(
                "database `{resolved_id}` is not ready: {detail}"
            )));
        }

        let storage = entry.storage.clone().ok_or_else(|| {
            AppError::Internal(format!(
                "database `{resolved_id}` is ready but no storage adapter is attached"
            ))
        })?;

        Ok((entry.database.clone(), storage))
    }
}

impl DatabaseManager {
    pub fn new(registry: Arc<DatabaseRegistry>, provisioner: Arc<dyn DatabaseProvisioner>) -> Self {
        Self {
            registry,
            provisioner,
            bootstrap_lock: Mutex::new(()),
        }
    }

    pub fn default_database_id(&self) -> &str {
        self.registry.default_database_id()
    }

    pub async fn list_databases(&self) -> Vec<ManagedDatabase> {
        self.registry.list().await
    }

    pub async fn get_database(&self, database_id: &str) -> AppResult<ManagedDatabase> {
        self.registry
            .get(database_id)
            .await
            .ok_or_else(|| AppError::NotFound(format!("database `{database_id}` was not found")))
    }

    pub async fn describe_tables(
        &self,
        database_id: Option<&str>,
        table: Option<QualifiedTable>,
    ) -> AppResult<(ManagedDatabase, Vec<TableDoc>)> {
        let (database, storage) = self.registry.resolve(database_id).await?;
        let tables = storage.describe_tables(table).await?;
        Ok((database, tables))
    }

    pub async fn bootstrap_database(
        &self,
        request: BootstrapDatabaseRequest,
    ) -> AppResult<ManagedDatabase> {
        request.validate()?;

        let _guard = self.bootstrap_lock.lock().await;
        if self.registry.contains(&request.database_id).await {
            return Err(AppError::Conflict(format!(
                "database `{}` is already registered",
                request.database_id
            )));
        }

        match self.provisioner.provision_database(&request).await? {
            ProvisionDatabaseOutcome::Ready { storage } => {
                if let Err(error) = storage.describe_tables(None).await {
                    let failure = DatabaseBootstrapFailure {
                        stage: BootstrapFailureStage::IntrospectSchema,
                        message: error.to_string(),
                        script_name: None,
                        script_index: None,
                    };
                    let database = ManagedDatabase::from_bootstrap_request(
                        &request,
                        DatabaseStatus::BootstrapFailed,
                        Some(failure),
                    );
                    self.registry.register(database.clone(), None).await?;
                    return Ok(database);
                }

                let database =
                    ManagedDatabase::from_bootstrap_request(&request, DatabaseStatus::Ready, None);
                self.registry
                    .register(database.clone(), Some(storage))
                    .await?;
                Ok(database)
            }
            ProvisionDatabaseOutcome::Failed { failure } => {
                let database = ManagedDatabase::from_bootstrap_request(
                    &request,
                    DatabaseStatus::BootstrapFailed,
                    Some(failure),
                );
                self.registry.register(database.clone(), None).await?;
                Ok(database)
            }
        }
    }
}

pub fn validate_database_id(database_id: &str) -> AppResult<()> {
    let mut chars = database_id.chars();
    let Some(first) = chars.next() else {
        return Err(AppError::BadRequest(
            "database_id cannot be empty".to_string(),
        ));
    };

    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(AppError::BadRequest(format!(
            "database_id `{database_id}` must start with a letter or underscore"
        )));
    }

    if chars.any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')) {
        return Err(AppError::BadRequest(format!(
            "database_id `{database_id}` contains unsupported characters"
        )));
    }

    Ok(())
}

fn validate_optional_text(field: &str, value: Option<&str>, max_chars: usize) -> AppResult<()> {
    let Some(value) = value else {
        return Ok(());
    };

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(format!("{field} cannot be blank")));
    }

    if trimmed.chars().count() > max_chars {
        return Err(AppError::BadRequest(format!(
            "{field} is too long; max {max_chars} characters"
        )));
    }

    Ok(())
}

fn validate_tag(tag: &str) -> AppResult<()> {
    let trimmed = tag.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(
            "tags cannot contain blank values".to_string(),
        ));
    }

    if trimmed.chars().count() > 32 {
        return Err(AppError::BadRequest(format!(
            "tag `{trimmed}` is too long; max 32 characters"
        )));
    }

    if trimmed
        .chars()
        .any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
    {
        return Err(AppError::BadRequest(format!(
            "tag `{trimmed}` contains unsupported characters"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use super::{
        BootstrapDatabaseRequest, BootstrapFailureStage, BootstrapSqlScript,
        DatabaseBootstrapFailure, DatabaseKind, DatabaseManager, DatabaseProvisioner,
        DatabaseRegistry, DatabaseStatus, ExistingDatabasePolicy, ManagedDatabase,
        ProvisionDatabaseOutcome,
    };
    use crate::{
        AppError, AppResult, AuthContext, ColumnDoc, QualifiedTable, QueryDoc, RlsDoc,
        StorageAdapter, TableDelete, TableDetailFields, TableDoc, TableInsert, TableMethods,
        TableSelect, TableUpdate, UserRecord,
    };

    struct FakeStorage {
        docs: Vec<TableDoc>,
        fail_describe: bool,
    }

    #[async_trait::async_trait]
    impl StorageAdapter for FakeStorage {
        async fn create_user(
            &self,
            _username: &str,
            _password_hash: &str,
        ) -> AppResult<UserRecord> {
            Err(AppError::Internal("not implemented".to_string()))
        }

        async fn find_user_by_username(&self, _username: &str) -> AppResult<Option<UserRecord>> {
            Ok(None)
        }

        async fn find_user_by_id(&self, _user_id: Uuid) -> AppResult<Option<UserRecord>> {
            Ok(None)
        }

        async fn describe_tables(
            &self,
            _table: Option<QualifiedTable>,
        ) -> AppResult<Vec<TableDoc>> {
            if self.fail_describe {
                return Err(AppError::Database("introspection failed".to_string()));
            }

            Ok(self.docs.clone())
        }

        async fn describe_detail_fields(
            &self,
            _table: QualifiedTable,
        ) -> AppResult<Vec<TableDetailFields>> {
            Ok(Vec::new())
        }

        async fn select_rows(
            &self,
            _request: TableSelect,
            _auth: Option<&AuthContext>,
        ) -> AppResult<Vec<serde_json::Value>> {
            Err(AppError::Internal("not implemented".to_string()))
        }

        async fn insert_rows(
            &self,
            _request: TableInsert,
            _auth: Option<&AuthContext>,
        ) -> AppResult<Vec<serde_json::Value>> {
            Err(AppError::Internal("not implemented".to_string()))
        }

        async fn update_rows(
            &self,
            _request: TableUpdate,
            _auth: Option<&AuthContext>,
        ) -> AppResult<Vec<serde_json::Value>> {
            Err(AppError::Internal("not implemented".to_string()))
        }

        async fn delete_rows(
            &self,
            _request: TableDelete,
            _auth: Option<&AuthContext>,
        ) -> AppResult<Vec<serde_json::Value>> {
            Err(AppError::Internal("not implemented".to_string()))
        }
    }

    struct FakeProvisioner {
        outcome: std::sync::Mutex<Option<ProvisionDatabaseOutcome>>,
    }

    impl DatabaseProvisioner for FakeProvisioner {
        fn provision_database(
            &self,
            _request: &BootstrapDatabaseRequest,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = AppResult<ProvisionDatabaseOutcome>> + Send>,
        > {
            let outcome = self.outcome.lock().expect("mutex").take().ok_or_else(|| {
                AppError::Internal("provisioner outcome already consumed".to_string())
            });

            Box::pin(async move { outcome })
        }
    }

    fn sample_doc() -> TableDoc {
        TableDoc {
            name: "public.notes".to_string(),
            schema: "public".to_string(),
            table: "notes".to_string(),
            endpoint: "/rest/public.notes".to_string(),
            columns: vec![ColumnDoc {
                name: "id".to_string(),
                data_type: "uuid".to_string(),
                nullable: false,
                has_default: true,
            }],
            methods: TableMethods {
                get: true,
                post: false,
                patch: false,
                delete: false,
            },
            rls: RlsDoc {
                enabled: false,
                policies: Vec::new(),
            },
            query: QueryDoc::from_columns(&[]),
        }
    }

    fn bootstrap_request() -> BootstrapDatabaseRequest {
        BootstrapDatabaseRequest {
            database_id: "tenant_alpha".to_string(),
            postgres_database: "tenant_alpha".to_string(),
            scripts: vec![BootstrapSqlScript {
                name: "001_init".to_string(),
                sql: "create schema public;".to_string(),
            }],
            description: Some("Tenant alpha".to_string()),
            tags: vec!["demo".to_string()],
            owner: Some("platform-team".to_string()),
            existing_database_policy: ExistingDatabasePolicy::Fail,
        }
    }

    #[test]
    fn bootstrap_request_rejects_empty_scripts() {
        let mut request = bootstrap_request();
        request.scripts.clear();

        let error = request.validate().expect_err("request should be invalid");
        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[tokio::test]
    async fn database_manager_resolves_default_database_docs() {
        let default_storage: Arc<dyn StorageAdapter> = Arc::new(FakeStorage {
            docs: vec![sample_doc()],
            fail_describe: false,
        });
        let registry = Arc::new(DatabaseRegistry::new(
            ManagedDatabase::default_database("default".to_string(), "evobase".to_string()),
            default_storage,
        ));
        let manager = DatabaseManager::new(
            registry,
            Arc::new(FakeProvisioner {
                outcome: std::sync::Mutex::new(None),
            }),
        );

        let (database, docs) = manager
            .describe_tables(None, None)
            .await
            .expect("default database docs should resolve");

        assert_eq!(database.database_id, "default");
        assert_eq!(database.kind, DatabaseKind::Default);
        assert_eq!(docs.len(), 1);
    }

    #[tokio::test]
    async fn database_manager_registers_failed_bootstrap_state() {
        let default_storage: Arc<dyn StorageAdapter> = Arc::new(FakeStorage {
            docs: vec![sample_doc()],
            fail_describe: false,
        });
        let registry = Arc::new(DatabaseRegistry::new(
            ManagedDatabase::default_database("default".to_string(), "evobase".to_string()),
            default_storage,
        ));
        let manager = DatabaseManager::new(
            registry.clone(),
            Arc::new(FakeProvisioner {
                outcome: std::sync::Mutex::new(Some(ProvisionDatabaseOutcome::Failed {
                    failure: DatabaseBootstrapFailure {
                        stage: BootstrapFailureStage::ExecuteScript,
                        message: "syntax error at or near CREATE".to_string(),
                        script_name: Some("001_init".to_string()),
                        script_index: Some(0),
                    },
                })),
            }),
        );

        let database = manager
            .bootstrap_database(bootstrap_request())
            .await
            .expect("bootstrap result should be returned");

        assert_eq!(database.status, DatabaseStatus::BootstrapFailed);
        assert!(database.failure.is_some());

        let stored = registry
            .get("tenant_alpha")
            .await
            .expect("database should be registered");
        assert_eq!(stored.status, DatabaseStatus::BootstrapFailed);
    }

    #[tokio::test]
    async fn database_manager_marks_introspection_failure() {
        let default_storage: Arc<dyn StorageAdapter> = Arc::new(FakeStorage {
            docs: vec![sample_doc()],
            fail_describe: false,
        });
        let registry = Arc::new(DatabaseRegistry::new(
            ManagedDatabase::default_database("default".to_string(), "evobase".to_string()),
            default_storage,
        ));
        let manager = DatabaseManager::new(
            registry.clone(),
            Arc::new(FakeProvisioner {
                outcome: std::sync::Mutex::new(Some(ProvisionDatabaseOutcome::Ready {
                    storage: Arc::new(FakeStorage {
                        docs: Vec::new(),
                        fail_describe: true,
                    }),
                })),
            }),
        );

        let database = manager
            .bootstrap_database(bootstrap_request())
            .await
            .expect("bootstrap result should be returned");

        assert_eq!(database.status, DatabaseStatus::BootstrapFailed);
        assert_eq!(
            database.failure.expect("failure").stage,
            BootstrapFailureStage::IntrospectSchema
        );

        let error = manager
            .describe_tables(Some("tenant_alpha"), None)
            .await
            .expect_err("failed bootstrap database should not expose docs");
        assert!(matches!(error, AppError::Conflict(_)));
    }
}
