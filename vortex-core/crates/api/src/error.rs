//! RFC 9457 `application/problem+json` for the API Gateway (T-00-11).

use std::error::Error as StdError;

use axum::{
    Json,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use domain::{DomainCode, DomainError, FieldError};
use serde::Serialize;

const PROBLEM_BASE: &str = "https://vectra.local/problems";

#[derive(Debug, Clone, Serialize)]
pub struct FieldProblem {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
struct ProblemBody {
    #[serde(rename = "type")]
    type_uri: String,
    title: String,
    status: u16,
    code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<FieldProblem>>,
}

/// HTTP error envelope aligned with `docs/openapi.yaml` `Problem`.
#[derive(Debug, Clone)]
pub struct ApiError {
    status: StatusCode,
    body: ProblemBody,
    retry_after_secs: Option<u32>,
}

impl ApiError {
    #[must_use]
    pub fn payload_too_large(instance: Option<String>) -> Self {
        Self {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            body: ProblemBody {
                type_uri: type_uri_for_code("payload_too_large"),
                title: title_for_code("payload_too_large").into(),
                status: 413,
                code: "payload_too_large".into(),
                detail: Some("El cuerpo de la solicitud supera el límite permitido.".into()),
                instance,
                errors: None,
            },
            retry_after_secs: None,
        }
    }

    #[must_use]
    pub fn gateway_timeout(instance: Option<String>) -> Self {
        Self {
            status: StatusCode::GATEWAY_TIMEOUT,
            body: ProblemBody {
                type_uri: type_uri_for_code("timeout"),
                title: title_for_code("timeout").into(),
                status: 504,
                code: "timeout".into(),
                detail: Some("La operación superó el tiempo de espera.".into()),
                instance,
                errors: None,
            },
            retry_after_secs: None,
        }
    }

    pub fn route_not_found(instance: String) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: ProblemBody {
                type_uri: type_uri_for_code("not_found"),
                title: title_for_code("not_found").into(),
                status: 404,
                code: "not_found".into(),
                detail: Some("La ruta solicitada no existe.".into()),
                instance: Some(instance),
                errors: None,
            },
            retry_after_secs: None,
        }
    }

    pub fn internal(_cause: &(dyn StdError + 'static)) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ProblemBody {
                type_uri: type_uri_for_code("internal"),
                title: title_for_code("internal").into(),
                status: 500,
                code: "internal".into(),
                detail: Some("Ocurrió un error interno. Intente nuevamente más tarde.".into()),
                instance: None,
                errors: None,
            },
            retry_after_secs: None,
        }
    }

    fn invalid_input_with_fields(
        errors: Vec<FieldProblem>,
        detail: Option<String>,
        instance: Option<String>,
        code: &str,
    ) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ProblemBody {
                type_uri: type_uri_for_code(code),
                title: title_for_code(code).into(),
                status: 400,
                code: code.into(),
                detail,
                instance,
                errors: Some(errors),
            },
            retry_after_secs: None,
        }
    }

    #[must_use]
    pub fn from_json_rejection(
        rejection: axum::extract::rejection::JsonRejection,
        instance: Option<String>,
    ) -> Self {
        use axum::extract::rejection::JsonRejection;

        let (field, message) = match rejection {
            JsonRejection::JsonDataError(err) => (
                serde_field_path(&err).unwrap_or_else(|| "body".into()),
                err.body_text(),
            ),
            JsonRejection::JsonSyntaxError(err) => ("body".into(), err.body_text()),
            JsonRejection::MissingJsonContentType(err) => ("body".into(), err.body_text()),
            JsonRejection::BytesRejection(err) => ("body".into(), err.body_text()),
            _ => (
                "body".into(),
                "El cuerpo de la solicitud no es JSON válido.".into(),
            ),
        };

        let field_problem = FieldProblem { field, message };
        Self::invalid_input_with_fields(
            vec![field_problem],
            Some("Revise el cuerpo de la solicitud.".into()),
            instance,
            "invalid_input",
        )
    }
}

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        let code = err.code();
        let code_str = code.as_str();
        let status = status_for_domain_code(code);

        let (errors, detail) = match &err {
            DomainError::InvalidInput { errors, .. } => {
                let mapped = errors
                    .iter()
                    .map(|FieldError { field, message }| FieldProblem {
                        field: field.clone(),
                        message: message.clone(),
                    })
                    .collect::<Vec<_>>();
                (Some(mapped), Some("Revise los campos indicados.".into()))
            }
            DomainError::NotFound => (None, Some("El recurso solicitado no existe.".into())),
            DomainError::Conflict { .. } => (
                None,
                Some("Conflicto con el estado actual del recurso.".into()),
            ),
            DomainError::Unavailable { .. } => (
                None,
                Some("El servicio no está disponible en este momento.".into()),
            ),
            DomainError::Timeout => (
                None,
                Some("La operación superó el tiempo de espera.".into()),
            ),
        };

        let retry_after_secs = matches!(err, DomainError::Unavailable { .. }).then_some(5_u32);

        Self {
            status,
            body: ProblemBody {
                type_uri: type_uri_for_code(code_str),
                title: title_for_code(code_str).into(),
                status: status.as_u16(),
                code: code_str.into(),
                detail,
                instance: None,
                errors,
            },
            retry_after_secs,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        if let Some(secs) = self.retry_after_secs
            && let Ok(value) = HeaderValue::from_str(&secs.to_string())
        {
            headers.insert(axum::http::header::RETRY_AFTER, value);
        }
        (self.status, headers, Json(self.body)).into_response()
    }
}

fn type_uri_for_code(code: &str) -> String {
    let kebab = code.replace('_', "-");
    format!("{PROBLEM_BASE}/{kebab}")
}

fn status_for_domain_code(code: DomainCode) -> StatusCode {
    match code {
        DomainCode::InvalidInput
        | DomainCode::SelfRelation
        | DomainCode::CategoryCycle
        | DomainCode::NegativeStock => StatusCode::BAD_REQUEST,
        DomainCode::NotFound => StatusCode::NOT_FOUND,
        DomainCode::Conflict
        | DomainCode::VersionConflict
        | DomainCode::DuplicateSku
        | DomainCode::DuplicateSlug
        | DomainCode::DuplicateRelation
        | DomainCode::AttributeInUse => StatusCode::CONFLICT,
        DomainCode::Unavailable | DomainCode::AiUnavailable | DomainCode::SpeechUnavailable => {
            StatusCode::SERVICE_UNAVAILABLE
        }
        DomainCode::Timeout => StatusCode::GATEWAY_TIMEOUT,
    }
}

fn title_for_code(code: &str) -> &'static str {
    match code {
        "invalid_input" => "La solicitud no es válida",
        "self_relation" => "Un producto no puede relacionarse consigo mismo",
        "category_cycle" => "La categoría formaría un ciclo",
        "negative_stock" => "El stock no puede ser negativo",
        "not_found" => "Recurso no encontrado",
        "conflict" => "Conflicto con el estado actual",
        "version_conflict" => "El producto fue modificado por otra terminal",
        "duplicate_sku" => "El SKU ya existe",
        "duplicate_slug" => "El slug ya existe",
        "duplicate_relation" => "La relación ya existe",
        "attribute_in_use" => "El atributo está en uso",
        "unavailable" => "El servicio no está disponible",
        "ai_unavailable" => "El modo IA no está disponible",
        "speech_unavailable" => "El reconocimiento de voz no está disponible",
        "timeout" => "La operación superó el tiempo de espera",
        "payload_too_large" => "El cuerpo es demasiado grande",
        "internal" => "Error interno",
        _ => "Error",
    }
}

fn serde_field_path(err: &axum::extract::rejection::JsonDataError) -> Option<String> {
    let text = err.body_text();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("missing field `")
            && let Some(field) = rest.strip_suffix('`')
        {
            return Some(field.to_string());
        }
        if let Some(rest) = trimmed.strip_prefix("invalid type:")
            && rest.contains("at line")
        {
            return Some("body".into());
        }
    }
    None
}
