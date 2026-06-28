use std::str::FromStr;

use async_trait::async_trait;
use evobase_domain::{
    Currency, DomainError, DomainEvent, DomainEventPublisher, DomainResult, Email, EventRepository,
    Money, OrderId, OrderRecord, OrderRepository, OrderStatus, PositiveInt, StoredEvent, UserId,
    UserRepository, ports::UserRecord,
};
use serde_json::Value;
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use uuid::Uuid;

const DEMO_MIGRATION: &str = include_str!("../../../migrations/0001_demo.sql");

#[derive(Clone)]
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub async fn connect(database_url: &str) -> DomainResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect(database_url)
            .await
            .map_err(to_storage_error)?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> DomainResult<()> {
        for statement in DEMO_MIGRATION.split("-- statement-breakpoint") {
            let statement = statement.trim();
            if statement.is_empty() {
                continue;
            }

            tracing::debug!(statement = %first_line(statement), "applying demo migration statement");
            sqlx::query(statement)
                .execute(&self.pool)
                .await
                .map_err(to_storage_error)?;
        }

        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl UserRepository for PostgresStore {
    async fn insert_user(&self, email: Email, verified: bool) -> DomainResult<UserRecord> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO demo_users (id, email, verified)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(id)
        .bind(email.as_str())
        .bind(verified)
        .execute(&self.pool)
        .await
        .map_err(to_storage_error)?;

        Ok(UserRecord {
            id: UserId(id),
            email,
            verified,
        })
    }

    async fn find_user(&self, id: UserId) -> DomainResult<Option<UserRecord>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, verified
            FROM demo_users
            WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(to_storage_error)?;

        row.map(row_to_user).transpose()
    }

    async fn mark_user_verified(&self, id: UserId) -> DomainResult<UserRecord> {
        let row = sqlx::query(
            r#"
            UPDATE demo_users
            SET verified = TRUE
            WHERE id = $1
            RETURNING id, email, verified
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(to_storage_error)?
        .ok_or_else(|| DomainError::NotFound(format!("user {}", id.0)))?;

        row_to_user(row)
    }
}

#[async_trait]
impl OrderRepository for PostgresStore {
    async fn insert_draft_order(
        &self,
        buyer_id: UserId,
        total: Money,
    ) -> DomainResult<OrderRecord> {
        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO demo_orders (id, buyer_id, total_cents, currency, status)
            VALUES ($1, $2, $3, $4, 'draft')
            RETURNING id, buyer_id, total_cents, currency, status, version
            "#,
        )
        .bind(id)
        .bind(buyer_id.0)
        .bind(total.cents.get())
        .bind(total.currency.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(to_storage_error)?;

        row_to_order(row)
    }

    async fn find_order(&self, id: OrderId) -> DomainResult<Option<OrderRecord>> {
        let row = sqlx::query(
            r#"
            SELECT id, buyer_id, total_cents, currency, status, version
            FROM demo_orders
            WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(to_storage_error)?;

        row.map(row_to_order).transpose()
    }

    async fn transition_order(
        &self,
        id: OrderId,
        expected_status: OrderStatus,
        next_status: OrderStatus,
        expected_version: i64,
    ) -> DomainResult<OrderRecord> {
        let row = sqlx::query(
            r#"
            UPDATE demo_orders
            SET status = $2,
                version = version + 1,
                updated_at = now()
            WHERE id = $1
              AND status = $3
              AND version = $4
            RETURNING id, buyer_id, total_cents, currency, status, version
            "#,
        )
        .bind(id.0)
        .bind(next_status.as_str())
        .bind(expected_status.as_str())
        .bind(expected_version)
        .fetch_optional(&self.pool)
        .await
        .map_err(to_storage_error)?;

        match row {
            Some(row) => row_to_order(row),
            None => Err(DomainError::Conflict),
        }
    }
}

#[async_trait]
impl DomainEventPublisher for PostgresStore {
    async fn publish(&self, event: DomainEvent) -> DomainResult<()> {
        sqlx::query(
            r#"
            INSERT INTO demo_domain_events (id, aggregate_id, event_type, payload)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(event.aggregate_id())
        .bind(event.event_type())
        .bind(
            serde_json::to_value(&event).map_err(|error| {
                DomainError::Storage(format!("failed to serialize event: {error}"))
            })?,
        )
        .execute(&self.pool)
        .await
        .map_err(to_storage_error)?;

        Ok(())
    }
}

#[async_trait]
impl EventRepository for PostgresStore {
    async fn list_events(&self, limit: i64) -> DomainResult<Vec<StoredEvent>> {
        let rows = sqlx::query(
            r#"
            SELECT id, aggregate_id, event_type, payload
            FROM demo_domain_events
            ORDER BY occurred_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(to_storage_error)?;

        rows.into_iter().map(row_to_event).collect()
    }
}

fn row_to_user(row: sqlx::postgres::PgRow) -> DomainResult<UserRecord> {
    let id: Uuid = row.try_get("id").map_err(to_storage_error)?;
    let email: String = row.try_get("email").map_err(to_storage_error)?;
    let verified: bool = row.try_get("verified").map_err(to_storage_error)?;

    Ok(UserRecord {
        id: UserId(id),
        email: Email::parse(email)?,
        verified,
    })
}

fn row_to_order(row: sqlx::postgres::PgRow) -> DomainResult<OrderRecord> {
    let id: Uuid = row.try_get("id").map_err(to_storage_error)?;
    let buyer_id: Uuid = row.try_get("buyer_id").map_err(to_storage_error)?;
    let total_cents: i64 = row.try_get("total_cents").map_err(to_storage_error)?;
    let currency: String = row.try_get("currency").map_err(to_storage_error)?;
    let status: String = row.try_get("status").map_err(to_storage_error)?;
    let version: i64 = row.try_get("version").map_err(to_storage_error)?;

    Ok(OrderRecord {
        id: OrderId(id),
        buyer_id: UserId(buyer_id),
        total: Money {
            cents: PositiveInt::parse(total_cents)?,
            currency: Currency::from_str(&currency)?,
        },
        status: OrderStatus::from_str(&status)?,
        version,
    })
}

fn row_to_event(row: sqlx::postgres::PgRow) -> DomainResult<StoredEvent> {
    Ok(StoredEvent {
        id: row.try_get("id").map_err(to_storage_error)?,
        aggregate_id: row.try_get("aggregate_id").map_err(to_storage_error)?,
        event_type: row.try_get("event_type").map_err(to_storage_error)?,
        payload: row
            .try_get::<Value, _>("payload")
            .map_err(to_storage_error)?,
    })
}

fn to_storage_error(error: impl std::fmt::Display) -> DomainError {
    DomainError::Storage(error.to_string())
}

fn first_line(statement: &str) -> &str {
    statement.lines().next().unwrap_or(statement)
}
