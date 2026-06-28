use async_trait::async_trait;

use crate::{
    DomainEvent, DomainResult, Email, Money, OrderId, OrderRecord, OrderStatus, StoredEvent, UserId,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserRecord {
    pub id: UserId,
    pub email: Email,
    pub verified: bool,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert_user(&self, email: Email, verified: bool) -> DomainResult<UserRecord>;
    async fn find_user(&self, id: UserId) -> DomainResult<Option<UserRecord>>;
    async fn mark_user_verified(&self, id: UserId) -> DomainResult<UserRecord>;
}

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn insert_draft_order(&self, buyer_id: UserId, total: Money)
    -> DomainResult<OrderRecord>;

    async fn find_order(&self, id: OrderId) -> DomainResult<Option<OrderRecord>>;

    async fn transition_order(
        &self,
        id: OrderId,
        expected_status: OrderStatus,
        next_status: OrderStatus,
        expected_version: i64,
    ) -> DomainResult<OrderRecord>;
}

#[async_trait]
pub trait DomainEventPublisher: Send + Sync {
    async fn publish(&self, event: DomainEvent) -> DomainResult<()>;
}

#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn list_events(&self, limit: i64) -> DomainResult<Vec<StoredEvent>>;
}
