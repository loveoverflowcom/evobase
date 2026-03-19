use async_trait::async_trait;
use evobase_core::{
    AppError, AppResult, AuthContext, Filter, FilterOperator, OrderBy, QualifiedTable, SelectList,
    StorageAdapter, TableDelete, TableInsert, TableSelect, TableUpdate, UserRecord,
    quoted_identifier, validate_identifier,
};
use serde_json::{Map, Value};
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgPoolOptions};
use tracing::debug;
use uuid::Uuid;

pub struct PostgresStorage {
    pool: PgPool,
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
