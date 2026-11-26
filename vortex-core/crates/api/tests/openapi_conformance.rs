mod support;

use serde_json::json;

#[test]
fn health_json_missing_version_fails_conformance() {
    let body = json!({
        "status": "ok",
        "uptimeSeconds": 0,
        "components": {
            "storage": { "status": "disabled" },
            "meilisearch": { "status": "disabled" },
            "llm": { "status": "disabled" },
            "speech": { "status": "disabled" },
            "textEmbeddings": { "status": "disabled" },
            "sync": { "status": "disabled" }
        }
    });
    let err = support::openapi::conforms("GET", "/health", 200, "application/json", &body)
        .expect_err("missing version must fail");
    let lower = err.to_ascii_lowercase();
    assert!(
        lower.contains("version") || lower.contains("required"),
        "expected version or required in error, got: {err}"
    );
}

#[test]
fn unknown_operation_fails_conformance() {
    let err = support::openapi::conforms("POST", "/health", 200, "application/json", &json!({}))
        .expect_err("POST /health 200 must not exist in openapi");
    let lower = err.to_ascii_lowercase();
    assert!(
        lower.contains("post") || lower.contains("/health"),
        "expected POST or /health in error, got: {err}"
    );
}
