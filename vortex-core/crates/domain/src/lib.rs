//! Crate `domain` — entities, ports and domain errors.

pub mod decimal;
pub mod error;
pub mod health;
pub mod id;
pub mod outbox;
pub mod timestamp;

pub use decimal::{Decimal, InvalidDecimal};
pub use error::{
    ConflictCode, DomainCode, DomainError, FieldError, FieldErrorValidationError, InvalidInputCode,
    UnavailableCode, UnknownDomainCode,
};
pub use health::{
    AggregateStatus, ComponentHealth, ComponentId, ComponentStatus, HealthComponents, HealthReport,
    HealthSnapshot, aggregate,
};
pub use id::{Id, InvalidId};
pub use outbox::{OutboxEntity, OutboxEvent, OutboxEventError, OutboxOp};
pub use timestamp::{InvalidTimestamp, Timestamp};
