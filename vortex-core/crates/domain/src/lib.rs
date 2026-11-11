//! Crate `domain` — entities, ports and domain errors.

pub mod categoria;
pub mod decimal;
pub mod error;
pub mod health;
pub mod id;
pub mod outbox;
pub mod timestamp;

pub use categoria::{
    Categoria, CategoriaDraft, CategoryRepo, MAX_DEPTH, NewCategoria, Slug, category_depth,
    reject_cycle, would_form_cycle,
};
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
