use std::error::Error;

use api::ApiError;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use domain::{
    ConflictCode, DomainCode, DomainError, FieldError, InvalidInputCode, UnavailableCode,
};

fn expected_status(code: &str) -> StatusCode {
    match code {
        "invalid_input" | "self_relation" | "category_cycle" | "negative_stock" => {
            StatusCode::BAD_REQUEST
        }
        "not_found" => StatusCode::NOT_FOUND,
        "conflict" | "version_conflict" | "duplicate_sku" | "duplicate_slug"
        | "duplicate_relation" | "attribute_in_use" => StatusCode::CONFLICT,
        "unavailable" | "ai_unavailable" | "speech_unavailable" => StatusCode::SERVICE_UNAVAILABLE,
        "timeout" => StatusCode::GATEWAY_TIMEOUT,
        "internal" => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn expected_title(code: &str) -> &str {
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
        "internal" => "Error interno",
        _ => "Error",
    }
}

async fn response_json(err: ApiError) -> (StatusCode, String, serde_json::Value) {
    let resp = err.into_response();
    let status = resp.status();
    let ctype = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    assert_eq!(ctype, "application/problem+json");
    let body = resp.into_body();
    let bytes = axum::body::to_bytes(body, 1_000_000).await.expect("body");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    (status, ctype, json)
}

#[tokio::test]
async fn domain_error_variants_map_to_problem_json() {
    for code in DomainCode::all() {
        let err = DomainError::from_domain_code(*code, Vec::new());
        let (status, _, json) = response_json(ApiError::from(err)).await;
        let code_str = code.as_str();
        assert_eq!(status, expected_status(code_str), "code={code_str}");
        assert_eq!(json["code"], code_str);
        assert_eq!(json["title"], expected_title(code_str));
        assert_eq!(
            json["type"],
            format!(
                "https://vectra.local/problems/{}",
                code_str.replace('_', "-")
            )
        );
        assert_eq!(json["status"], status.as_u16());
    }

    let field = FieldError::try_new("quantity", "debe ser positivo").expect("field");
    let invalid = DomainError::invalid_input(InvalidInputCode::NegativeStock, vec![field]);
    let (_, _, json) = response_json(ApiError::from(invalid)).await;
    assert_eq!(json["code"], "negative_stock");
    assert!(json.get("errors").is_some());

    let unavailable = DomainError::unavailable(UnavailableCode::AiUnavailable);
    let resp = ApiError::from(unavailable).into_response();
    assert_eq!(resp.headers().get(header::RETRY_AFTER).unwrap(), "5");
}

#[tokio::test]
async fn conflict_does_not_expose_errors_key() {
    let err = DomainError::conflict(ConflictCode::VersionConflict);
    let (_, _, json) = response_json(ApiError::from(err)).await;
    assert!(json.get("errors").is_none());
}

#[derive(Debug)]
struct SecretError;

impl std::fmt::Display for SecretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("secret=hunter2")
    }
}

impl Error for SecretError {}

#[tokio::test]
async fn internal_error_does_not_leak_cause() {
    let err = SecretError;
    let (status, _, json) = response_json(ApiError::internal(&err)).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(json["code"], "internal");
    let blob = json.to_string();
    assert!(!blob.contains("hunter2"));
    assert!(!blob.contains("secret="));
}
