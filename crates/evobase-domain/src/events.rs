use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{Email, Money, OrderId, PaymentId, UserId, VerifiedUserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    UserRegistered {
        user_id: UserId,
        email: Email,
        verified: bool,
    },
    EmailVerified {
        user_id: UserId,
    },
    OrderDrafted {
        order_id: OrderId,
        buyer_id: UserId,
        total: Money,
    },
    OrderPaid {
        order_id: OrderId,
        paid_by: VerifiedUserId,
        payment_id: PaymentId,
        amount: Money,
    },
    OrderShipped {
        order_id: OrderId,
        shipped_by: UserId,
    },
    OrderCancelled {
        order_id: OrderId,
        cancelled_by: UserId,
        reason: String,
    },
}

impl DomainEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::UserRegistered { .. } => "user.registered.v1",
            Self::EmailVerified { .. } => "user.email_verified.v1",
            Self::OrderDrafted { .. } => "order.drafted.v1",
            Self::OrderPaid { .. } => "order.paid.v1",
            Self::OrderShipped { .. } => "order.shipped.v1",
            Self::OrderCancelled { .. } => "order.cancelled.v1",
        }
    }

    pub fn aggregate_id(&self) -> Uuid {
        match self {
            Self::UserRegistered { user_id, .. } | Self::EmailVerified { user_id } => user_id.0,
            Self::OrderDrafted { order_id, .. }
            | Self::OrderPaid { order_id, .. }
            | Self::OrderShipped { order_id, .. }
            | Self::OrderCancelled { order_id, .. } => order_id.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub payload: Value,
}
