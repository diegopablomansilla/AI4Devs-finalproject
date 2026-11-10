//! Crate `domain` — entities, ports and domain errors.

pub mod decimal;
pub mod error;
pub mod id;
pub mod timestamp;

pub use decimal::{Decimal, InvalidDecimal};
pub use error::{
    ConflictCode, DomainCode, DomainError, FieldError, FieldErrorValidationError, InvalidInputCode,
    UnavailableCode, UnknownDomainCode,
};
pub use id::{Id, InvalidId};
pub use timestamp::{InvalidTimestamp, Timestamp};
