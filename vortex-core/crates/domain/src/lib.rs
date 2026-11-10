//! Crate `domain` — entities, ports and domain errors.

pub mod error;

pub use error::{
    ConflictCode, DomainCode, DomainError, FieldError, FieldErrorValidationError, InvalidInputCode,
    UnavailableCode, UnknownDomainCode,
};
