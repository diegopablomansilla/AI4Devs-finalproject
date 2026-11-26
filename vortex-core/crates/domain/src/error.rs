//! Domain error taxonomy (`DomainError`, stable `code`s aligned with API 5.4).

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldErrorValidationError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl FieldError {
    pub fn try_new(
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<Self, FieldErrorValidationError> {
        let field = field.into();
        let message = message.into();
        if field.trim().is_empty() || message.trim().is_empty() {
            return Err(FieldErrorValidationError);
        }
        Ok(Self { field, message })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidInputCode {
    InvalidInput,
    SelfRelation,
    CategoryCycle,
    NegativeStock,
}

impl InvalidInputCode {
    pub fn as_domain_code(self) -> DomainCode {
        match self {
            Self::InvalidInput => DomainCode::InvalidInput,
            Self::SelfRelation => DomainCode::SelfRelation,
            Self::CategoryCycle => DomainCode::CategoryCycle,
            Self::NegativeStock => DomainCode::NegativeStock,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictCode {
    Conflict,
    VersionConflict,
    DuplicateSku,
    DuplicateSlug,
    DuplicateRelation,
    AttributeInUse,
}

impl ConflictCode {
    pub fn as_domain_code(self) -> DomainCode {
        match self {
            Self::Conflict => DomainCode::Conflict,
            Self::VersionConflict => DomainCode::VersionConflict,
            Self::DuplicateSku => DomainCode::DuplicateSku,
            Self::DuplicateSlug => DomainCode::DuplicateSlug,
            Self::DuplicateRelation => DomainCode::DuplicateRelation,
            Self::AttributeInUse => DomainCode::AttributeInUse,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnavailableCode {
    Unavailable,
    AiUnavailable,
    SpeechUnavailable,
}

impl UnavailableCode {
    pub fn as_domain_code(self) -> DomainCode {
        match self {
            Self::Unavailable => DomainCode::Unavailable,
            Self::AiUnavailable => DomainCode::AiUnavailable,
            Self::SpeechUnavailable => DomainCode::SpeechUnavailable,
        }
    }
}

/// Stable snake_case codes shared with `docs/openapi.yaml` (`ErrorCode`), excluding Gateway-only values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainCode {
    InvalidInput,
    SelfRelation,
    CategoryCycle,
    NegativeStock,
    NotFound,
    Conflict,
    VersionConflict,
    DuplicateSku,
    DuplicateSlug,
    DuplicateRelation,
    AttributeInUse,
    Unavailable,
    AiUnavailable,
    SpeechUnavailable,
    Timeout,
}

impl DomainCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::SelfRelation => "self_relation",
            Self::CategoryCycle => "category_cycle",
            Self::NegativeStock => "negative_stock",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::VersionConflict => "version_conflict",
            Self::DuplicateSku => "duplicate_sku",
            Self::DuplicateSlug => "duplicate_slug",
            Self::DuplicateRelation => "duplicate_relation",
            Self::AttributeInUse => "attribute_in_use",
            Self::Unavailable => "unavailable",
            Self::AiUnavailable => "ai_unavailable",
            Self::SpeechUnavailable => "speech_unavailable",
            Self::Timeout => "timeout",
        }
    }

    /// Every domain `code` from contract 5.4 (excludes Gateway-only codes).
    pub fn all() -> &'static [DomainCode] {
        &[
            Self::InvalidInput,
            Self::SelfRelation,
            Self::CategoryCycle,
            Self::NegativeStock,
            Self::NotFound,
            Self::Conflict,
            Self::VersionConflict,
            Self::DuplicateSku,
            Self::DuplicateSlug,
            Self::DuplicateRelation,
            Self::AttributeInUse,
            Self::Unavailable,
            Self::AiUnavailable,
            Self::SpeechUnavailable,
            Self::Timeout,
        ]
    }
}

impl fmt::Display for DomainCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownDomainCode;

impl FromStr for DomainCode {
    type Err = UnknownDomainCode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "invalid_input" => Ok(Self::InvalidInput),
            "self_relation" => Ok(Self::SelfRelation),
            "category_cycle" => Ok(Self::CategoryCycle),
            "negative_stock" => Ok(Self::NegativeStock),
            "not_found" => Ok(Self::NotFound),
            "conflict" => Ok(Self::Conflict),
            "version_conflict" => Ok(Self::VersionConflict),
            "duplicate_sku" => Ok(Self::DuplicateSku),
            "duplicate_slug" => Ok(Self::DuplicateSlug),
            "duplicate_relation" => Ok(Self::DuplicateRelation),
            "attribute_in_use" => Ok(Self::AttributeInUse),
            "unavailable" => Ok(Self::Unavailable),
            "ai_unavailable" => Ok(Self::AiUnavailable),
            "speech_unavailable" => Ok(Self::SpeechUnavailable),
            "timeout" => Ok(Self::Timeout),
            _ => Err(UnknownDomainCode),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("resource not found")]
    NotFound,
    #[error("conflict ({code:?})")]
    Conflict { code: ConflictCode },
    #[error("invalid input ({code:?})")]
    InvalidInput {
        code: InvalidInputCode,
        errors: Vec<FieldError>,
    },
    #[error("service unavailable ({code:?})")]
    Unavailable { code: UnavailableCode },
    #[error("operation timed out")]
    Timeout,
}

impl DomainError {
    pub fn not_found() -> Self {
        Self::NotFound
    }

    pub fn timeout() -> Self {
        Self::Timeout
    }

    pub fn conflict(code: ConflictCode) -> Self {
        Self::Conflict { code }
    }

    pub fn invalid_input(code: InvalidInputCode, errors: Vec<FieldError>) -> Self {
        Self::InvalidInput { code, errors }
    }

    pub fn unavailable(code: UnavailableCode) -> Self {
        Self::Unavailable { code }
    }

    pub fn from_domain_code(code: DomainCode, errors: Vec<FieldError>) -> Self {
        match code {
            DomainCode::NotFound => Self::NotFound,
            DomainCode::Timeout => Self::Timeout,
            DomainCode::InvalidInput => Self::InvalidInput {
                code: InvalidInputCode::InvalidInput,
                errors,
            },
            DomainCode::SelfRelation => Self::InvalidInput {
                code: InvalidInputCode::SelfRelation,
                errors,
            },
            DomainCode::CategoryCycle => Self::InvalidInput {
                code: InvalidInputCode::CategoryCycle,
                errors,
            },
            DomainCode::NegativeStock => Self::InvalidInput {
                code: InvalidInputCode::NegativeStock,
                errors,
            },
            DomainCode::Conflict => Self::Conflict {
                code: ConflictCode::Conflict,
            },
            DomainCode::VersionConflict => Self::Conflict {
                code: ConflictCode::VersionConflict,
            },
            DomainCode::DuplicateSku => Self::Conflict {
                code: ConflictCode::DuplicateSku,
            },
            DomainCode::DuplicateSlug => Self::Conflict {
                code: ConflictCode::DuplicateSlug,
            },
            DomainCode::DuplicateRelation => Self::Conflict {
                code: ConflictCode::DuplicateRelation,
            },
            DomainCode::AttributeInUse => Self::Conflict {
                code: ConflictCode::AttributeInUse,
            },
            DomainCode::Unavailable => Self::Unavailable {
                code: UnavailableCode::Unavailable,
            },
            DomainCode::AiUnavailable => Self::Unavailable {
                code: UnavailableCode::AiUnavailable,
            },
            DomainCode::SpeechUnavailable => Self::Unavailable {
                code: UnavailableCode::SpeechUnavailable,
            },
        }
    }

    pub fn code(&self) -> DomainCode {
        match self {
            Self::NotFound => DomainCode::NotFound,
            Self::Timeout => DomainCode::Timeout,
            Self::Conflict { code } => code.as_domain_code(),
            Self::InvalidInput { code, .. } => code.as_domain_code(),
            Self::Unavailable { code } => code.as_domain_code(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn each_variant_constructs() {
        let _ = DomainError::not_found();
        let _ = DomainError::timeout();
        let _ = DomainError::conflict(ConflictCode::Conflict);
        let _ = DomainError::unavailable(UnavailableCode::Unavailable);
        let field =
            FieldError::try_new("attributes[1].value", "invalid").expect("valid field error");
        let _ = DomainError::invalid_input(InvalidInputCode::InvalidInput, vec![field]);
    }

    #[test]
    fn invalid_input_preserves_field_error() {
        let field = FieldError::try_new("attributes[1].value", "must be positive").unwrap();
        let err = DomainError::invalid_input(InvalidInputCode::InvalidInput, vec![field.clone()]);
        match err {
            DomainError::InvalidInput { errors, .. } => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0], field);
            }
            _ => panic!("expected InvalidInput"),
        }
    }

    #[test]
    fn invalid_input_empty_field_errors() {
        let err = DomainError::invalid_input(InvalidInputCode::InvalidInput, Vec::new());
        assert_eq!(err.code(), DomainCode::InvalidInput);
        match err {
            DomainError::InvalidInput { errors, code } => {
                assert!(errors.is_empty());
                assert_eq!(code, InvalidInputCode::InvalidInput);
            }
            _ => panic!("expected InvalidInput"),
        }
    }

    #[test]
    fn domain_error_implements_std_error() {
        let err = DomainError::not_found();
        let _: &dyn Error = &err;
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn representative_codes() {
        assert_eq!(DomainError::not_found().code(), DomainCode::NotFound);
        assert_eq!(DomainError::timeout().code(), DomainCode::Timeout);
        assert_eq!(
            DomainError::conflict(ConflictCode::VersionConflict).code(),
            DomainCode::VersionConflict
        );
        assert_eq!(
            DomainError::invalid_input(InvalidInputCode::NegativeStock, vec![]).code(),
            DomainCode::NegativeStock
        );
        assert_eq!(
            DomainError::unavailable(UnavailableCode::AiUnavailable).code(),
            DomainCode::AiUnavailable
        );
        assert_eq!(DomainError::not_found().code().as_str(), "not_found");
    }

    #[test]
    fn all_domain_codes_constructible() {
        for &code in DomainCode::all() {
            let err = DomainError::from_domain_code(code, Vec::new());
            assert_eq!(err.code(), code);
            assert_eq!(err.code().as_str(), code.as_str());
        }
    }

    #[test]
    fn parse_unknown_code_fails() {
        assert!(DomainCode::from_str("no_existe").is_err());
    }

    #[test]
    fn parse_gateway_only_codes_fails() {
        for s in ["payload_too_large", "unsupported_media_type", "internal"] {
            assert!(
                DomainCode::from_str(s).is_err(),
                "expected parse failure for {s}"
            );
        }
    }

    #[test]
    fn version_conflict_maps_to_conflict_not_invalid_input() {
        let err = DomainError::from_domain_code(DomainCode::VersionConflict, Vec::new());
        assert!(matches!(
            err,
            DomainError::Conflict {
                code: ConflictCode::VersionConflict
            }
        ));
        assert!(!matches!(err, DomainError::InvalidInput { .. }));
    }

    #[test]
    fn field_error_rejects_empty_field() {
        assert!(FieldError::try_new("", "msg").is_err());
        assert!(FieldError::try_new("   ", "msg").is_err());
    }

    #[test]
    fn field_error_rejects_empty_message() {
        assert!(FieldError::try_new("field", "").is_err());
        assert!(FieldError::try_new("field", "  ").is_err());
    }
}
