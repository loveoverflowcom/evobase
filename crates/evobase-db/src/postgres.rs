use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use evobase_core::{
    AppError, AppResult, AuthContext, BootstrapDatabaseRequest, BootstrapFailureStage,
    BootstrapSqlScript, ColumnDoc, DatabaseBootstrapFailure, DatabaseProvisioner,
    ExistingDatabasePolicy, Filter, FilterOperator, OrderBy, ProvisionDatabaseOutcome,
    QualifiedTable, QueryDoc, RlsDoc, RlsPolicyDoc, SelectList, StorageAdapter, TableDelete,
    TableDoc, TableInsert, TableMethods, TableSelect, TableUpdate, UserRecord, quoted_identifier,
    validate_identifier,
};
use serde_json::{Map, Value};
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgPoolOptions};
use tracing::{debug, warn};
use url::Url;
use uuid::Uuid;

pub struct PostgresStorage {
    pool: PgPool,
}

#[derive(Debug, Clone)]
pub struct PostgresDatabaseProvisioner {
    admin_database_url: String,
}

#[derive(Debug, Clone)]
struct ColumnRow {
    schema: String,
    table: String,
    column: String,
    data_type: String,
    nullable: bool,
    has_default: bool,
}

#[derive(Debug, Clone)]
struct GrantRow {
    schema: String,
    table: String,
    privilege: String,
}

#[derive(Debug, Clone)]
struct RlsRow {
    schema: String,
    table: String,
    enabled: bool,
}

#[derive(Debug, Clone)]
struct PolicyRow {
    schema: String,
    table: String,
    name: String,
    command: String,
    using_expr: Option<String>,
    with_check: Option<String>,
}

impl PostgresStorage {
    pub async fn connect(database_url: &str) -> AppResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(map_sqlx_error)?;

        Ok(Self { pool })
    }

    async fn apply_auth_context<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        auth: Option<&AuthContext>,
    ) -> AppResult<()> {
        if let Some(auth) = auth {
            sqlx::query(
                "select \
                    set_config('request.jwt.claim.sub', $1, true), \
                    set_config('app.current_user_id', $1, true)",
            )
            .bind(auth.user_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        Ok(())
    }

    fn validate_insert_rows(rows: &[Map<String, Value>]) -> AppResult<Vec<String>> {
        let Some(first_row) = rows.first() else {
            return Err(AppError::BadRequest(
                "insert payload must contain at least one object".to_string(),
            ));
        };

        if first_row.is_empty() {
            return Err(AppError::BadRequest(
                "insert payload objects cannot be empty".to_string(),
            ));
        }

        let mut columns: Vec<String> = first_row.keys().cloned().collect();
        columns.sort();

        for column in &columns {
            validate_identifier(column)?;
        }

        for row in rows.iter().skip(1) {
            let mut keys: Vec<String> = row.keys().cloned().collect();
            keys.sort();

            if keys != columns {
                return Err(AppError::BadRequest(
                    "all insert objects must share the same shape".to_string(),
                ));
            }
        }

        Ok(columns)
    }

    fn validate_patch(patch: &Map<String, Value>) -> AppResult<Vec<String>> {
        if patch.is_empty() {
            return Err(AppError::BadRequest(
                "update payload cannot be empty".to_string(),
            ));
        }

        let mut columns: Vec<String> = patch.keys().cloned().collect();
        columns.sort();

        for column in &columns {
            validate_identifier(column)?;
        }

        Ok(columns)
    }

    fn push_select_list(query: &mut QueryBuilder<'_, Postgres>, select: &SelectList) {
        match select {
            SelectList::All => {
                query.push("t.*");
            }
            SelectList::Columns(columns) => {
                for (index, column) in columns.iter().enumerate() {
                    if index > 0 {
                        query.push(", ");
                    }

                    query.push("t.");
                    query.push(quoted_identifier(column));
                }
            }
        }
    }

    fn push_columns(
        query: &mut QueryBuilder<'_, Postgres>,
        columns: &[String],
        alias: Option<&str>,
    ) {
        for (index, column) in columns.iter().enumerate() {
            if index > 0 {
                query.push(", ");
            }

            if let Some(alias) = alias {
                query.push(alias);
                query.push(".");
            }

            query.push(quoted_identifier(column));
        }
    }

    fn push_table(query: &mut QueryBuilder<'_, Postgres>, table: &QualifiedTable) {
        query.push(table.quoted());
    }

    fn push_order_by(query: &mut QueryBuilder<'_, Postgres>, order_by: &[OrderBy]) {
        if order_by.is_empty() {
            return;
        }

        query.push(" ORDER BY ");

        for (index, order) in order_by.iter().enumerate() {
            if index > 0 {
                query.push(", ");
            }

            query.push("t.");
            query.push(quoted_identifier(&order.column));
            query.push(if order.descending { " DESC" } else { " ASC" });
        }
    }

    fn push_filters(
        query: &mut QueryBuilder<'_, Postgres>,
        table: &QualifiedTable,
        filters: &[Filter],
        table_alias: &str,
    ) {
        if filters.is_empty() {
            return;
        }

        query.push(" WHERE ");

        for (index, filter) in filters.iter().enumerate() {
            if index > 0 {
                query.push(" AND ");
            }

            match filter.operator {
                FilterOperator::Like | FilterOperator::ILike => {
                    query.push(table_alias);
                    query.push(".");
                    query.push(quoted_identifier(&filter.column));
                    query.push("::text ");
                    query.push(filter.operator.sql_operator());
                    query.push(" ");
                    query.push_bind(filter.value.clone());
                }
                _ => {
                    query.push(table_alias);
                    query.push(".");
                    query.push(quoted_identifier(&filter.column));
                    query.push(" ");
                    query.push(filter.operator.sql_operator());
                    query.push(" (SELECT x.");
                    query.push(quoted_identifier(&filter.column));
                    query.push(" FROM jsonb_populate_record(NULL::");
                    query.push(table.quoted());
                    query.push(", jsonb_build_object(");
                    query.push_bind(filter.column.clone());
                    query.push(", to_jsonb(");
                    query.push_bind(filter.value.clone());
                    query.push("))) AS x)");
                }
            }
        }
    }

    fn build_table_docs(
        columns: Vec<ColumnRow>,
        grants: Vec<GrantRow>,
        rls_rows: Vec<RlsRow>,
        policies: Vec<PolicyRow>,
    ) -> Vec<TableDoc> {
        let mut tables = BTreeMap::<(String, String), TableDoc>::new();

        for column in columns {
            let key = (column.schema.clone(), column.table.clone());
            let table_doc = tables.entry(key).or_insert_with(|| {
                let name = format!("{}.{}", column.schema, column.table);
                TableDoc {
                    name: name.clone(),
                    schema: column.schema.clone(),
                    table: column.table.clone(),
                    endpoint: format!("/rest/{name}"),
                    columns: Vec::new(),
                    methods: TableMethods::default(),
                    rls: RlsDoc {
                        enabled: false,
                        policies: Vec::new(),
                    },
                    query: QueryDoc::from_columns(&[]),
                }
            });

            table_doc.columns.push(ColumnDoc {
                name: column.column,
                data_type: column.data_type,
                nullable: column.nullable,
                has_default: column.has_default,
            });
        }

        for grant in grants {
            let Some(table_doc) = tables.get_mut(&(grant.schema, grant.table)) else {
                continue;
            };

            match grant.privilege.as_str() {
                "SELECT" => table_doc.methods.get = true,
                "INSERT" => table_doc.methods.post = true,
                "UPDATE" => table_doc.methods.patch = true,
                "DELETE" => table_doc.methods.delete = true,
                _ => {}
            }
        }

        for rls in rls_rows {
            let Some(table_doc) = tables.get_mut(&(rls.schema, rls.table)) else {
                continue;
            };

            table_doc.rls.enabled = rls.enabled;
        }

        for policy in policies {
            let Some(table_doc) = tables.get_mut(&(policy.schema, policy.table)) else {
                continue;
            };

            table_doc.rls.policies.push(RlsPolicyDoc {
                name: policy.name,
                command: policy.command,
                r#using: policy.using_expr,
                with_check: policy.with_check,
            });
        }

        let mut docs = tables.into_values().collect::<Vec<_>>();

        for table_doc in &mut docs {
            table_doc.query = QueryDoc::from_columns(&table_doc.columns);
        }

        docs.retain(|table_doc| table_doc.methods.any());
        docs
    }
}

impl PostgresDatabaseProvisioner {
    pub fn new(admin_database_url: impl Into<String>) -> Self {
        Self {
            admin_database_url: admin_database_url.into(),
        }
    }

    async fn connect_pool(database_url: &str) -> AppResult<PgPool> {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(map_sqlx_error)
    }

    fn target_database_url(admin_database_url: &str, database_name: &str) -> AppResult<String> {
        let mut url = Url::parse(admin_database_url)
            .map_err(|error| AppError::Config(format!("invalid DATABASE_ADMIN_URL: {error}")))?;
        url.set_path(&format!("/{database_name}"));
        Ok(url.to_string())
    }

    async fn database_exists(pool: &PgPool, database_name: &str) -> AppResult<bool> {
        sqlx::query_scalar::<_, bool>("select exists(select 1 from pg_database where datname = $1)")
            .bind(database_name)
            .fetch_one(pool)
            .await
            .map_err(map_sqlx_error)
    }

    async fn create_database(pool: &PgPool, database_name: &str) -> AppResult<()> {
        let statement = format!("create database {}", quoted_identifier(database_name));
        sqlx::raw_sql(&statement)
            .execute(pool)
            .await
            .map(|_| ())
            .map_err(map_sqlx_error)
    }

    async fn execute_scripts(
        pool: PgPool,
        scripts: Vec<BootstrapSqlScript>,
    ) -> Result<(), DatabaseBootstrapFailure> {
        warn!(
            script_count = scripts.len(),
            "Executing bootstrap SQL scripts sequentially without an explicit transaction block"
        );

        for (index, script) in scripts.iter().enumerate() {
            let sql = script.sql.clone();
            if let Err(error) = sqlx::raw_sql(&sql).execute(&pool).await {
                return Err(DatabaseBootstrapFailure {
                    stage: BootstrapFailureStage::ExecuteScript,
                    message: error.to_string(),
                    script_name: Some(script.name.clone()),
                    script_index: Some(index),
                });
            }
        }

        Ok(())
    }

    async fn provision_database_inner(
        admin_database_url: String,
        request: BootstrapDatabaseRequest,
    ) -> AppResult<ProvisionDatabaseOutcome> {
        let postgres_database = request.postgres_database.clone();
        let scripts = request.scripts.clone();
        let existing_database_policy = request.existing_database_policy;

        let admin_pool = match Self::connect_pool(&admin_database_url).await {
            Ok(pool) => pool,
            Err(error) => {
                return Ok(ProvisionDatabaseOutcome::Failed {
                    failure: DatabaseBootstrapFailure {
                        stage: BootstrapFailureStage::CreateDatabase,
                        message: error.to_string(),
                        script_name: None,
                        script_index: None,
                    },
                });
            }
        };

        let database_exists = match Self::database_exists(&admin_pool, &postgres_database).await {
            Ok(exists) => exists,
            Err(error) => {
                return Ok(ProvisionDatabaseOutcome::Failed {
                    failure: DatabaseBootstrapFailure {
                        stage: BootstrapFailureStage::CreateDatabase,
                        message: error.to_string(),
                        script_name: None,
                        script_index: None,
                    },
                });
            }
        };

        if database_exists && existing_database_policy == ExistingDatabasePolicy::Fail {
            return Err(AppError::Conflict(format!(
                "postgres database `{}` already exists",
                postgres_database
            )));
        }

        if !database_exists {
            if let Err(error) = Self::create_database(&admin_pool, &postgres_database).await {
                return Ok(ProvisionDatabaseOutcome::Failed {
                    failure: DatabaseBootstrapFailure {
                        stage: BootstrapFailureStage::CreateDatabase,
                        message: error.to_string(),
                        script_name: None,
                        script_index: None,
                    },
                });
            }
        }

        let target_database_url =
            Self::target_database_url(&admin_database_url, &postgres_database)?;
        let concrete_storage = match PostgresStorage::connect(&target_database_url).await {
            Ok(storage) => Arc::new(storage),
            Err(error) => {
                return Ok(ProvisionDatabaseOutcome::Failed {
                    failure: DatabaseBootstrapFailure {
                        stage: BootstrapFailureStage::ConnectDatabase,
                        message: error.to_string(),
                        script_name: None,
                        script_index: None,
                    },
                });
            }
        };

        if let Err(failure) = Self::execute_scripts(concrete_storage.pool.clone(), scripts).await {
            return Ok(ProvisionDatabaseOutcome::Failed { failure });
        }

        let storage: Arc<dyn StorageAdapter> = concrete_storage;
        Ok(ProvisionDatabaseOutcome::Ready { storage })
    }
}

impl DatabaseProvisioner for PostgresDatabaseProvisioner {
    fn provision_database(
        &self,
        request: &BootstrapDatabaseRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = AppResult<ProvisionDatabaseOutcome>> + Send>,
    > {
        let admin_database_url = self.admin_database_url.clone();
        let request = request.clone();
        Box::pin(async move {
            PostgresDatabaseProvisioner::provision_database_inner(admin_database_url, request).await
        })
    }
}

#[async_trait]
impl StorageAdapter for PostgresStorage {
    async fn create_user(&self, username: &str, password_hash: &str) -> AppResult<UserRecord> {
        let result = sqlx::query_as::<_, (Uuid, String, String)>(
            "insert into auth.users (username, password_hash)
             values ($1, $2)
             returning id, username, password_hash",
        )
        .bind(username)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok((id, username, password_hash)) => Ok(UserRecord {
                id,
                username,
                password_hash,
            }),
            Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("23505") => Err(
                AppError::Conflict("username is already registered".to_string()),
            ),
            Err(error) => Err(map_sqlx_error(error)),
        }
    }

    async fn find_user_by_username(&self, username: &str) -> AppResult<Option<UserRecord>> {
        sqlx::query_as::<_, (Uuid, String, String)>(
            "select id, username, password_hash
             from auth.users
             where username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map(|record| {
            record.map(|(id, username, password_hash)| UserRecord {
                id,
                username,
                password_hash,
            })
        })
        .map_err(map_sqlx_error)
    }

    async fn find_user_by_id(&self, user_id: Uuid) -> AppResult<Option<UserRecord>> {
        sqlx::query_as::<_, (Uuid, String, String)>(
            "select id, username, password_hash
             from auth.users
             where id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map(|record| {
            record.map(|(id, username, password_hash)| UserRecord {
                id,
                username,
                password_hash,
            })
        })
        .map_err(map_sqlx_error)
    }

    async fn describe_tables(&self, table: Option<QualifiedTable>) -> AppResult<Vec<TableDoc>> {
        let (schema_filter, table_filter) = match table {
            Some(table) => (table.schema, Some(table.table)),
            None => (None, None),
        };

        let columns = sqlx::query_as::<_, (String, String, String, String, i32, bool, bool)>(
            "select
                c.table_schema,
                c.table_name,
                c.column_name,
                case
                    when c.data_type = 'USER-DEFINED' then c.udt_name
                    when c.data_type = 'ARRAY' then c.udt_name
                    else c.data_type
                end as data_type,
                c.ordinal_position,
                c.is_nullable = 'YES' as nullable,
                c.column_default is not null as has_default
             from information_schema.columns c
             where c.table_schema not in ('pg_catalog', 'information_schema')
               and ($1::text is null or c.table_schema = $1)
               and ($2::text is null or c.table_name = $2)
             order by c.table_schema, c.table_name, c.ordinal_position",
        )
        .bind(schema_filter.as_deref())
        .bind(table_filter.as_deref())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(
            |(schema, table, column, data_type, _ordinal_position, nullable, has_default)| {
                ColumnRow {
                    schema,
                    table,
                    column,
                    data_type,
                    nullable,
                    has_default,
                }
            },
        )
        .collect();

        let grants = sqlx::query_as::<_, (String, String, String)>(
            "select
                g.table_schema,
                g.table_name,
                g.privilege_type
             from information_schema.role_table_grants g
             where g.grantee = current_user
               and g.table_schema not in ('pg_catalog', 'information_schema')
               and g.privilege_type in ('SELECT', 'INSERT', 'UPDATE', 'DELETE')
               and ($1::text is null or g.table_schema = $1)
               and ($2::text is null or g.table_name = $2)
             order by g.table_schema, g.table_name, g.privilege_type",
        )
        .bind(schema_filter.as_deref())
        .bind(table_filter.as_deref())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(|(schema, table, privilege)| GrantRow {
            schema,
            table,
            privilege,
        })
        .collect();

        let rls_rows = sqlx::query_as::<_, (String, String, bool)>(
            "select
                n.nspname as schema_name,
                c.relname as table_name,
                c.relrowsecurity
             from pg_class c
             join pg_namespace n on n.oid = c.relnamespace
             where c.relkind in ('r', 'p')
               and n.nspname not in ('pg_catalog', 'information_schema')
               and ($1::text is null or n.nspname = $1)
               and ($2::text is null or c.relname = $2)
             order by n.nspname, c.relname",
        )
        .bind(schema_filter.as_deref())
        .bind(table_filter.as_deref())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(|(schema, table, enabled)| RlsRow {
            schema,
            table,
            enabled,
        })
        .collect();

        let policies = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                Option<String>,
                Option<String>,
            ),
        >(
            "select
                p.schemaname,
                p.tablename,
                p.policyname,
                p.cmd,
                p.qual,
                p.with_check
             from pg_policies p
             where p.schemaname not in ('pg_catalog', 'information_schema')
               and ($1::text is null or p.schemaname = $1)
               and ($2::text is null or p.tablename = $2)
             order by p.schemaname, p.tablename, p.policyname",
        )
        .bind(schema_filter.as_deref())
        .bind(table_filter.as_deref())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(
            |(schema, table, name, command, using_expr, with_check)| PolicyRow {
                schema,
                table,
                name,
                command,
                using_expr,
                with_check,
            },
        )
        .collect();

        Ok(Self::build_table_docs(columns, grants, rls_rows, policies))
    }

    async fn select_rows(
        &self,
        request: TableSelect,
        auth: Option<&AuthContext>,
    ) -> AppResult<Vec<Value>> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        self.apply_auth_context(&mut tx, auth).await?;

        let TableSelect {
            table,
            select,
            filters,
            order_by,
            limit,
            offset,
        } = request;

        let mut query = QueryBuilder::<Postgres>::new("select to_jsonb(t) as row from (select ");
        Self::push_select_list(&mut query, &select);
        query.push(" from ");
        Self::push_table(&mut query, &table);
        query.push(" as t");
        Self::push_filters(&mut query, &table, &filters, "t");
        Self::push_order_by(&mut query, &order_by);

        if let Some(limit) = limit {
            query.push(" limit ");
            query.push_bind(limit);
        }

        if let Some(offset) = offset {
            query.push(" offset ");
            query.push_bind(offset);
        }

        query.push(") as t");

        let rows = query
            .build_query_scalar::<Value>()
            .fetch_all(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(rows)
    }

    async fn insert_rows(
        &self,
        request: TableInsert,
        auth: Option<&AuthContext>,
    ) -> AppResult<Vec<Value>> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        self.apply_auth_context(&mut tx, auth).await?;

        let columns = Self::validate_insert_rows(&request.rows)?;
        let payload = Value::Array(request.rows.into_iter().map(Value::Object).collect());

        let mut query = QueryBuilder::<Postgres>::new("with input_rows as (select ");
        Self::push_columns(&mut query, &columns, None);
        query.push(" from jsonb_populate_recordset(NULL::");
        Self::push_table(&mut query, &request.table);
        query.push(", ");
        query.push_bind(payload);
        query.push(")) insert into ");
        Self::push_table(&mut query, &request.table);
        query.push(" as inserted (");
        Self::push_columns(&mut query, &columns, None);
        query.push(") select ");
        Self::push_columns(&mut query, &columns, None);
        query.push(" from input_rows returning to_jsonb(inserted.*)");

        let rows = query
            .build_query_scalar::<Value>()
            .fetch_all(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(rows)
    }

    async fn update_rows(
        &self,
        request: TableUpdate,
        auth: Option<&AuthContext>,
    ) -> AppResult<Vec<Value>> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        self.apply_auth_context(&mut tx, auth).await?;

        let columns = Self::validate_patch(&request.patch)?;
        let payload = Value::Object(request.patch);

        let mut query = QueryBuilder::<Postgres>::new("with patch as (select ");
        Self::push_columns(&mut query, &columns, None);
        query.push(" from jsonb_populate_record(NULL::");
        Self::push_table(&mut query, &request.table);
        query.push(", ");
        query.push_bind(payload);
        query.push(")) update ");
        Self::push_table(&mut query, &request.table);
        query.push(" as t set ");

        for (index, column) in columns.iter().enumerate() {
            if index > 0 {
                query.push(", ");
            }

            query.push(quoted_identifier(column));
            query.push(" = patch.");
            query.push(quoted_identifier(column));
        }

        query.push(" from patch");
        Self::push_filters(&mut query, &request.table, &request.filters, "t");
        query.push(" returning to_jsonb(t.*)");

        let rows = query
            .build_query_scalar::<Value>()
            .fetch_all(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(rows)
    }

    async fn delete_rows(
        &self,
        request: TableDelete,
        auth: Option<&AuthContext>,
    ) -> AppResult<Vec<Value>> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        self.apply_auth_context(&mut tx, auth).await?;

        let mut query = QueryBuilder::<Postgres>::new("delete from ");
        Self::push_table(&mut query, &request.table);
        query.push(" as t");
        Self::push_filters(&mut query, &request.table, &request.filters, "t");
        query.push(" returning to_jsonb(t.*)");

        let rows = query
            .build_query_scalar::<Value>()
            .fetch_all(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;

        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(rows)
    }
}

fn map_sqlx_error(error: sqlx::Error) -> AppError {
    debug!(error = %error, "postgres query failed");
    AppError::Database(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        ColumnRow, GrantRow, PolicyRow, PostgresDatabaseProvisioner, PostgresStorage, RlsRow,
    };

    #[test]
    fn build_table_docs_combines_introspection_metadata() {
        let docs = PostgresStorage::build_table_docs(
            vec![
                ColumnRow {
                    schema: "public".to_string(),
                    table: "notes".to_string(),
                    column: "id".to_string(),
                    data_type: "uuid".to_string(),
                    nullable: false,
                    has_default: true,
                },
                ColumnRow {
                    schema: "public".to_string(),
                    table: "notes".to_string(),
                    column: "body".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    has_default: false,
                },
            ],
            vec![
                GrantRow {
                    schema: "public".to_string(),
                    table: "notes".to_string(),
                    privilege: "SELECT".to_string(),
                },
                GrantRow {
                    schema: "public".to_string(),
                    table: "notes".to_string(),
                    privilege: "INSERT".to_string(),
                },
            ],
            vec![RlsRow {
                schema: "public".to_string(),
                table: "notes".to_string(),
                enabled: true,
            }],
            vec![PolicyRow {
                schema: "public".to_string(),
                table: "notes".to_string(),
                name: "notes_owner_policy".to_string(),
                command: "ALL".to_string(),
                using_expr: Some("owner_id = auth.uid()".to_string()),
                with_check: Some("owner_id = auth.uid()".to_string()),
            }],
        );

        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].name, "public.notes");
        assert_eq!(docs[0].endpoint, "/rest/public.notes");
        assert_eq!(docs[0].columns.len(), 2);
        assert!(docs[0].methods.get);
        assert!(docs[0].methods.post);
        assert!(!docs[0].methods.patch);
        assert!(docs[0].rls.enabled);
        assert_eq!(docs[0].rls.policies.len(), 1);
        assert_eq!(
            docs[0].query.selectable_columns,
            vec!["id".to_string(), "body".to_string()]
        );
    }

    #[test]
    fn build_table_docs_filters_tables_without_rest_privileges() {
        let docs = PostgresStorage::build_table_docs(
            vec![ColumnRow {
                schema: "public".to_string(),
                table: "audit_log".to_string(),
                column: "id".to_string(),
                data_type: "uuid".to_string(),
                nullable: false,
                has_default: true,
            }],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        assert!(docs.is_empty());
    }

    #[test]
    fn target_database_url_rewrites_database_name() {
        let url = PostgresDatabaseProvisioner::target_database_url(
            "postgres://postgres:postgres@localhost:5432/postgres",
            "tenant_alpha",
        )
        .expect("url should be rewritten");

        assert_eq!(
            url,
            "postgres://postgres:postgres@localhost:5432/tenant_alpha"
        );
    }
}
