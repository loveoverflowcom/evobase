use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    Actor, Capability, DomainError, DomainEvent, DomainEventPublisher, DomainResult, Email, Money,
    NonEmptyString, OrderId, OrderRecord, OrderRepository, OrderStatus, PaymentId, UserId,
    UserRepository, VerifiedUserId,
    ports::UserRecord,
    workflow::{Draft, Order, Paid},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserCommand {
    pub email: Email,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderCommand {
    pub buyer_id: UserId,
    pub total: Money,
}

#[derive(Clone)]
pub struct DomainService<P> {
    ports: Arc<P>,
}

impl<P> DomainService<P>
where
    P: UserRepository + OrderRepository + DomainEventPublisher,
{
    pub fn new(ports: Arc<P>) -> Self {
        Self { ports }
    }

    pub async fn create_user(&self, command: CreateUserCommand) -> DomainResult<UserRecord> {
        let user = self
            .ports
            .insert_user(command.email.clone(), command.verified)
            .await?;

        self.ports
            .publish(DomainEvent::UserRegistered {
                user_id: user.id,
                email: user.email.clone(),
                verified: user.verified,
            })
            .await?;

        Ok(user)
    }

    pub async fn verify_user(&self, user_id: UserId) -> DomainResult<UserRecord> {
        let user = self.ports.mark_user_verified(user_id).await?;
        self.ports
            .publish(DomainEvent::EmailVerified { user_id: user.id })
            .await?;
        Ok(user)
    }

    pub async fn create_order(
        &self,
        actor: Actor,
        command: CreateOrderCommand,
    ) -> DomainResult<OrderRecord> {
        self.require_verified_buyer(&actor, command.buyer_id)
            .await?;

        let order = self
            .ports
            .insert_draft_order(command.buyer_id, command.total)
            .await?;

        self.ports
            .publish(DomainEvent::OrderDrafted {
                order_id: order.id,
                buyer_id: order.buyer_id,
                total: order.total,
            })
            .await?;

        Ok(order)
    }

    pub async fn pay_order(
        &self,
        actor: Actor,
        order_id: OrderId,
        payment_id: PaymentId,
    ) -> DomainResult<OrderRecord> {
        let verified_actor = self.require_verified_actor(&actor).await?;
        let record = self.require_order(order_id).await?;
        let draft = Order::<Draft>::try_from(record)?;

        if !actor.has(&Capability::PayOrder) {
            return Err(DomainError::PolicyDenied(
                "actor lacks pay_order capability".to_string(),
            ));
        }

        if !actor.is_buyer_of(draft.record()) {
            return Err(DomainError::PolicyDenied(
                "only the buyer can pay this order".to_string(),
            ));
        }

        let event = draft.pay(payment_id, verified_actor);
        let updated = self
            .ports
            .transition_order(
                order_id,
                OrderStatus::Draft,
                OrderStatus::Paid,
                draft.record().version,
            )
            .await?;

        self.ports.publish(event).await?;
        Ok(updated)
    }

    pub async fn ship_order(&self, actor: Actor, order_id: OrderId) -> DomainResult<OrderRecord> {
        if !actor.has(&Capability::ShipOrder) {
            return Err(DomainError::PolicyDenied(
                "actor lacks ship_order capability".to_string(),
            ));
        }

        let record = self.require_order(order_id).await?;
        let paid = Order::<Paid>::try_from(record)?;
        let event = paid.ship(actor.user_id);
        let updated = self
            .ports
            .transition_order(
                order_id,
                OrderStatus::Paid,
                OrderStatus::Shipped,
                paid.record().version,
            )
            .await?;

        self.ports.publish(event).await?;
        Ok(updated)
    }

    pub async fn cancel_order(
        &self,
        actor: Actor,
        order_id: OrderId,
        reason: NonEmptyString,
    ) -> DomainResult<OrderRecord> {
        let record = self.require_order(order_id).await?;
        let can_cancel = actor.has(&Capability::CancelOrder) || actor.is_buyer_of(&record);
        if !can_cancel {
            return Err(DomainError::PolicyDenied(
                "actor cannot cancel this order".to_string(),
            ));
        }

        let event = match record.status {
            OrderStatus::Draft => Order::<Draft>::try_from(record.clone())?
                .cancel(actor.user_id, reason.as_str().to_string()),
            OrderStatus::Paid => Order::<Paid>::try_from(record.clone())?
                .cancel(actor.user_id, reason.as_str().to_string()),
            status => {
                return Err(DomainError::IllegalTransition {
                    expected: "draft or paid".to_string(),
                    found: status.as_str().to_string(),
                });
            }
        };

        let updated = self
            .ports
            .transition_order(
                order_id,
                record.status,
                OrderStatus::Cancelled,
                record.version,
            )
            .await?;

        self.ports.publish(event).await?;
        Ok(updated)
    }

    async fn require_order(&self, id: OrderId) -> DomainResult<OrderRecord> {
        self.ports
            .find_order(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("order {}", id.0)))
    }

    async fn require_verified_buyer(&self, actor: &Actor, buyer_id: UserId) -> DomainResult<()> {
        if actor.user_id != buyer_id {
            return Err(DomainError::PolicyDenied(
                "actor must create orders for itself in this demo".to_string(),
            ));
        }
        self.require_verified_actor(actor).await.map(|_| ())
    }

    async fn require_verified_actor(&self, actor: &Actor) -> DomainResult<VerifiedUserId> {
        let user = self
            .ports
            .find_user(actor.user_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("user {}", actor.user_id.0)))?;

        if actor.verified && user.verified {
            Ok(VerifiedUserId(actor.user_id))
        } else {
            Err(DomainError::PolicyDenied(
                "actor must be a verified user in both claims and database".to_string(),
            ))
        }
    }
}
