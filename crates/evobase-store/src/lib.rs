//! One configured tenant, fixed-layout libSQL storage of checked AppSpec snapshots.
//! Database destinations come from host configuration, never from request selectors.
use evobase_appspec::{AppId, CheckedAppSpec, CheckedRecords, Scope};
use libsql::{Builder, Connection, Database, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Duration};
mod commands;
pub use commands::CommitReceipt;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("storage operation failed")]
    Database(#[from] libsql::Error),
    #[error(transparent)]
    AppSpec(#[from] evobase_appspec::Error),
    #[error("selected scope is outside the configured tenant")]
    ScopeMismatch,
    #[error("application is not bootstrapped")]
    AppNotFound,
    #[error("application is already bootstrapped with different facts or definition")]
    AppAlreadyExists,
    #[error("stored snapshot is inconsistent")]
    CorruptSnapshot,
    #[error("stored schema version is unsupported")]
    UnsupportedSchema,
    #[error(transparent)]
    Authority(#[from] evobase_appspec::policy::AuthorityError),
    #[error(transparent)]
    Command(#[from] evobase_appspec::commands::CommandError),
    #[error("expected revision {expected} conflicts with current revision {actual}")]
    Conflict { expected: u64, actual: u64 },
    #[error("request key is already bound to a different immutable intent")]
    IntentConflict,
    #[error("current registry binding does not select this store")]
    BindingMismatch,
    #[error("authority changed during the operation")]
    AuthorityChanged,
    #[error("data revision exceeds the storage profile")]
    RevisionOverflow,
    #[error("payload exceeds the storage profile")]
    PayloadLimit,
    #[error("invalid receipt request key")]
    InvalidReceiptKey,
    #[error("invalid authenticated remote storage configuration")]
    InvalidRemoteConfiguration,
    #[error("injected storage failure")]
    InjectedFailure,
}

/// Bounded facts revalidated against the immutable, content-addressed definition on every load.
#[derive(Debug, Clone)]
pub struct Snapshot {
    spec: CheckedAppSpec,
    facts: CheckedRecords,
    revision: u64,
    spec_identity: String,
}
impl Snapshot {
    pub fn spec(&self) -> &CheckedAppSpec {
        &self.spec
    }
    pub fn facts(&self) -> &CheckedRecords {
        &self.facts
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn spec_identity(&self) -> &str {
        &self.spec_identity
    }
}

pub struct Store {
    database: Database,
    tenant_id: String,
    binding_id: String,
    local: bool,
}
impl Store {
    /// Opens an embedded file. It does not require an account, token or hosted registry.
    pub async fn open_local(path: impl AsRef<Path>, tenant_id: &str) -> Result<Self, StoreError> {
        Self::check_tenant(tenant_id)?;
        let database = Builder::new_local(path).build().await?;
        Self::open(database, tenant_id, true).await
    }

    /// Opens the configured remote primary using libSQL's authenticated HTTPS transport.
    /// This is a remote capability; local conformance does not establish a Turso cloud pass.
    pub async fn open_remote(url: &str, token: &str, tenant_id: &str) -> Result<Self, StoreError> {
        Self::check_tenant(tenant_id)?;
        let parsed = url::Url::parse(url).map_err(|_| StoreError::InvalidRemoteConfiguration)?;
        if !matches!(parsed.scheme(), "https" | "libsql")
            || parsed.host_str().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
            || token.is_empty()
        {
            return Err(StoreError::InvalidRemoteConfiguration);
        }
        let database = Builder::new_remote(url.to_owned(), token.to_owned())
            .build()
            .await?;
        Self::open(database, tenant_id, false).await
    }

    fn check_tenant(tenant_id: &str) -> Result<(), StoreError> {
        // Reuse the canonical tenant grammar without exposing an unchecked tenant type.
        Scope::new(tenant_id, AppId::new("app_store")?)?;
        Ok(())
    }

    async fn open(database: Database, tenant_id: &str, local: bool) -> Result<Self, StoreError> {
        let store = Self {
            database,
            tenant_id: tenant_id.to_owned(),
            binding_id: format!("store:{tenant_id}"),
            local,
        };
        let connection = store.connection()?;
        // Existing bindings are checked before any schema initialization, including partial stores.
        let mut metadata = connection
            .query(
                "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = 'engine_metadata'",
                (),
            )
            .await?;
        if metadata.next().await?.is_some() {
            let mut rows = connection
                .query(
                    "SELECT schema_version, tenant_id FROM engine_metadata WHERE singleton = 1",
                    (),
                )
                .await?;
            if let Some(row) = rows.next().await? {
                if row.get::<i64>(0)? != 1 {
                    return Err(StoreError::UnsupportedSchema);
                }
                if row.get::<String>(1)? != tenant_id {
                    return Err(StoreError::ScopeMismatch);
                }
            }
        }
        connection.execute_batch(include_str!("schema.sql")).await?;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;
        let initialized = async {
            tx.execute(
                "INSERT OR IGNORE INTO engine_metadata VALUES (1, 1, ?1)",
                [tenant_id],
            )
            .await?;
            let mut rows = tx
                .query(
                    "SELECT schema_version, tenant_id FROM engine_metadata WHERE singleton = 1",
                    (),
                )
                .await?;
            let row = rows.next().await?.ok_or(StoreError::CorruptSnapshot)?;
            if row.get::<i64>(0)? != 1 {
                return Err(StoreError::UnsupportedSchema);
            }
            if row.get::<String>(1)? != tenant_id {
                return Err(StoreError::ScopeMismatch);
            }
            Ok::<_, StoreError>(())
        }
        .await;
        match initialized {
            Ok(()) => tx.commit().await?,
            Err(error) => {
                tx.rollback().await?;
                return Err(error);
            }
        }
        Ok(store)
    }

    fn connection(&self) -> Result<Connection, StoreError> {
        let connection = self.database.connect()?;
        if self.local {
            connection.busy_timeout(Duration::from_secs(2))?;
        }
        Ok(connection)
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }
    /// The configured host registry must return this binding for commands targeting this store.
    pub fn binding_id(&self) -> &str {
        &self.binding_id
    }

    fn check_scope(&self, scope: &Scope) -> Result<(), StoreError> {
        if scope.tenant_id() != self.tenant_id {
            return Err(StoreError::ScopeMismatch);
        }
        Ok(())
    }

    /// Trusted provisioning operation: only checked, same-scope facts can enter the store.
    /// Repeating the identical bootstrap is safe; it cannot overwrite an existing app.
    pub async fn bootstrap(
        &self,
        spec: &CheckedAppSpec,
        facts: &CheckedRecords,
    ) -> Result<Snapshot, StoreError> {
        self.check_scope(facts.scope())?;
        let checked = spec.validate_records(facts.scope(), &facts.to_raw())?;
        let spec_json = spec.encode()?;
        let facts_json = checked.encode()?;
        let identity = spec_identity(&spec_json);
        let connection = self.connection()?;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;
        let result = async {
            let mut existing = tx
                .query(
                    "SELECT spec_identity, facts_json FROM applications WHERE app_id = ?1",
                    [spec.app_id().as_str()],
                )
                .await?;
            if let Some(row) = existing.next().await? {
                if row.get::<String>(0)? != identity || row.get::<Vec<u8>>(1)? != facts_json {
                    return Err(StoreError::AppAlreadyExists);
                }
            } else {
                tx.execute(
                    "INSERT INTO app_releases VALUES (?1, ?2, ?3)",
                    params![spec.app_id().as_str(), identity.clone(), spec_json],
                )
                .await?;
                tx.execute(
                    "INSERT INTO applications VALUES (?1, ?2, 1, ?3)",
                    params![spec.app_id().as_str(), identity, facts_json],
                )
                .await?;
            }
            self.load(&tx, facts.scope()).await
        }
        .await;
        match result {
            Ok(snapshot) => {
                tx.commit().await?;
                Ok(snapshot)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }

    /// Internal host facts, not an authorized client projection. Gateway policy owns output.
    pub async fn snapshot(&self, scope: &Scope) -> Result<Snapshot, StoreError> {
        self.check_scope(scope)?;
        self.load(&self.connection()?, scope).await
    }

    async fn load(&self, connection: &Connection, scope: &Scope) -> Result<Snapshot, StoreError> {
        let mut rows = connection.query(
            "SELECT a.spec_identity, a.revision, a.facts_json, r.spec_json FROM applications a JOIN app_releases r ON r.app_id = a.app_id AND r.spec_identity = a.spec_identity WHERE a.app_id = ?1", [scope.app_id().as_str()]
        ).await?;
        let row = rows.next().await?.ok_or(StoreError::AppNotFound)?;
        let identity = row.get::<String>(0)?;
        let revision =
            u64::try_from(row.get::<i64>(1)?).map_err(|_| StoreError::CorruptSnapshot)?;
        if revision == 0 {
            return Err(StoreError::CorruptSnapshot);
        }
        let facts_json = row.get::<Vec<u8>>(2)?;
        let spec_json = row.get::<Vec<u8>>(3)?;
        let spec = CheckedAppSpec::decode(&spec_json)?;
        if spec_identity(&spec_json) != identity || spec.encode()? != spec_json {
            return Err(StoreError::CorruptSnapshot);
        }
        let facts = spec.decode_records(scope, &facts_json)?;
        if facts.encode()? != facts_json {
            return Err(StoreError::CorruptSnapshot);
        }
        Ok(Snapshot {
            spec,
            facts,
            revision,
            spec_identity: identity,
        })
    }

    /// Fixed catalog evidence for conformance and operator diagnostics, never arbitrary SQL.
    pub async fn catalog(&self) -> Result<Vec<(String, String)>, StoreError> {
        let mut rows = self
            .connection()?
            .query(
                "SELECT name, sql FROM sqlite_schema WHERE type = 'table' ORDER BY name",
                (),
            )
            .await?;
        let mut catalog = Vec::new();
        while let Some(row) = rows.next().await? {
            catalog.push((row.get(0)?, row.get(1)?));
        }
        Ok(catalog)
    }
}

fn spec_identity(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
