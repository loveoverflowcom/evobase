use std::{marker::PhantomData, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    DomainError, DomainEvent, DomainResult, Money, OrderId, PaymentId, UserId, VerifiedUserId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Draft,
    Paid,
    Shipped,
    Cancelled,
}

impl OrderStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Paid => "paid",
            Self::Shipped => "shipped",
            Self::Cancelled => "cancelled",
        }
    }
}

impl FromStr for OrderStatus {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "draft" => Ok(Self::Draft),
            "paid" => Ok(Self::Paid),
            "shipped" => Ok(Self::Shipped),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(DomainError::Validation(format!(
                "unknown order status: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRecord {
    pub id: OrderId,
    pub buyer_id: UserId,
    pub total: Money,
    pub status: OrderStatus,
    pub version: i64,
}

#[derive(Debug, Clone)]
pub struct Draft;

#[derive(Debug, Clone)]
pub struct Paid;

#[derive(Debug, Clone)]
pub struct Shipped;

#[derive(Debug, Clone)]
pub struct Cancelled;

#[derive(Debug, Clone)]
pub struct Order<S> {
    record: OrderRecord,
    _state: PhantomData<S>,
}

impl<S> Order<S> {
    pub fn record(&self) -> &OrderRecord {
        &self.record
    }

    fn new(record: OrderRecord) -> Self {
        Self {
            record,
            _state: PhantomData,
        }
    }
}

impl TryFrom<OrderRecord> for Order<Draft> {
    type Error = DomainError;

    fn try_from(record: OrderRecord) -> Result<Self, Self::Error> {
        require_status(record, OrderStatus::Draft).map(Self::new)
    }
}

impl TryFrom<OrderRecord> for Order<Paid> {
    type Error = DomainError;

    fn try_from(record: OrderRecord) -> Result<Self, Self::Error> {
        require_status(record, OrderStatus::Paid).map(Self::new)
    }
}

impl TryFrom<OrderRecord> for Order<Shipped> {
    type Error = DomainError;

    fn try_from(record: OrderRecord) -> Result<Self, Self::Error> {
        require_status(record, OrderStatus::Shipped).map(Self::new)
    }
}

impl TryFrom<OrderRecord> for Order<Cancelled> {
    type Error = DomainError;

    fn try_from(record: OrderRecord) -> Result<Self, Self::Error> {
        require_status(record, OrderStatus::Cancelled).map(Self::new)
    }
}

impl Order<Draft> {
    pub fn pay(&self, payment_id: PaymentId, actor: VerifiedUserId) -> DomainEvent {
        DomainEvent::OrderPaid {
            order_id: self.record.id,
            paid_by: actor,
            payment_id,
            amount: self.record.total,
        }
    }

    pub fn cancel(&self, actor: UserId, reason: String) -> DomainEvent {
        DomainEvent::OrderCancelled {
            order_id: self.record.id,
            cancelled_by: actor,
            reason,
        }
    }
}

impl Order<Paid> {
    pub fn ship(&self, actor: UserId) -> DomainEvent {
        DomainEvent::OrderShipped {
            order_id: self.record.id,
            shipped_by: actor,
        }
    }

    pub fn cancel(&self, actor: UserId, reason: String) -> DomainEvent {
        DomainEvent::OrderCancelled {
            order_id: self.record.id,
            cancelled_by: actor,
            reason,
        }
    }
}

fn require_status(record: OrderRecord, expected: OrderStatus) -> DomainResult<OrderRecord> {
    if record.status == expected {
        Ok(record)
    } else {
        Err(DomainError::IllegalTransition {
            expected: expected.as_str().to_string(),
            found: record.status.as_str().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::{Currency, PositiveInt};

    #[test]
    fn draft_can_be_refined_to_draft_order() {
        let order = OrderRecord {
            id: OrderId(Uuid::new_v4()),
            buyer_id: UserId(Uuid::new_v4()),
            total: Money {
                cents: PositiveInt::parse(100).unwrap(),
                currency: Currency::USD,
            },
            status: OrderStatus::Draft,
            version: 0,
        };

        assert!(Order::<Draft>::try_from(order).is_ok());
    }

    #[test]
    fn draft_cannot_be_refined_to_paid_order() {
        let order = OrderRecord {
            id: OrderId(Uuid::new_v4()),
            buyer_id: UserId(Uuid::new_v4()),
            total: Money {
                cents: PositiveInt::parse(100).unwrap(),
                currency: Currency::USD,
            },
            status: OrderStatus::Draft,
            version: 0,
        };

        assert!(Order::<Paid>::try_from(order).is_err());
    }
}
