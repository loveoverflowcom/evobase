pub mod auth;
pub mod error;
pub mod events;
pub mod ir;
pub mod ports;
pub mod refinements;
pub mod service;
pub mod workflow;

pub use auth::{Actor, Capability};
pub use error::{DomainError, DomainResult};
pub use events::{DomainEvent, StoredEvent};
pub use ports::{DomainEventPublisher, EventRepository, OrderRepository, UserRepository};
pub use refinements::{
    Currency, Email, Money, NonEmptyString, OrderId, PaymentId, PositiveInt, UserId, VerifiedUserId,
};
pub use service::{CreateOrderCommand, CreateUserCommand, DomainService};
pub use workflow::{OrderRecord, OrderStatus};
