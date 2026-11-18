mod support;

use api::{AppState, build_router};
use support::fake_catalog::test_attribute_def_gateway;
use support::openapi::assert_conforms;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn http_send(
    addr: std::net::SocketAddr,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> (u16, String, String) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let host = format!("{}:{}", addr.ip(), addr.port());
    let mut request = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n");
    if let Some(b) = body {
        request.push_str("Content-Type: application/json\r\n");
        request.push_str(&format!("Content-Length: {}\r\n", b.len()));
    }
    request.push_str("\r\n");
    if let Some(b) = body {
        request.push_str(b);
    }
    stream.write_all(request.as_bytes()).await.expect("write");
    let mut buf = vec![0u8; 64 * 1024];
    let n = stream.read(&mut buf).await.expect("read");
    let response = String::from_utf8_lossy(&buf[..n]);
    let status_line = response.lines().next().unwrap_or("");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);
    let mut ctype = String::new();
    let mut body_start = 0usize;
    for (i, _) in response.match_indices("\r\n\r\n") {
        body_start = i + 4;
        break;
    }
    for line in response.lines() {
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-type") {
                ctype = value
                    .trim()
                    .split(';')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
            }
        }
    }
    let body = String::from_utf8_lossy(&buf[body_start..n]).into_owned();
    (status, ctype, body)
}

fn json_body(body: &str) -> serde_json::Value {
    if body.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_str(body).unwrap_or(serde_json::Value::Null)
    }
}

async fn serve(state: AppState) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    (addr, handle)
}

fn state_with_attribute_defs() -> AppState {
    AppState::new().with_attribute_defs(test_attribute_def_gateway())
}

const ATTR_PATH: &str = "/api/v1/attribute-definitions";

async fn post_attr(addr: std::net::SocketAddr, json: &str) -> (u16, String, serde_json::Value) {
    let (status, ctype, body) = http_send(addr, "POST", ATTR_PATH, Some(json)).await;
    (status, ctype, json_body(&body))
}

async fn get_attrs(addr: std::net::SocketAddr, query: &str) -> (u16, String, serde_json::Value) {
    let path = if query.is_empty() {
        ATTR_PATH.to_string()
    } else {
        format!("{ATTR_PATH}?{query}")
    };
    let (status, ctype, body) = http_send(addr, "GET", &path, None).await;
    (status, ctype, json_body(&body))
}

async fn patch_attr(
    addr: std::net::SocketAddr,
    id: &str,
    json: &str,
) -> (u16, String, serde_json::Value) {
    let path = format!("{ATTR_PATH}/{id}");
    let (status, ctype, body) = http_send(addr, "PATCH", &path, Some(json)).await;
    (status, ctype, json_body(&body))
}

async fn delete_attr(addr: std::net::SocketAddr, id: &str) -> (u16, String, serde_json::Value) {
    let path = format!("{ATTR_PATH}/{id}");
    let (status, ctype, body) = http_send(addr, "DELETE", &path, None).await;
    (status, ctype, json_body(&body))
}

const GRADO_JSON: &str =
    r#"{"key":"grado","label":"Grado","type":"enum","options":["2","5","8"],"facetable":true}"#;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_attribute_def_grado_returns_201() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = post_attr(addr, GRADO_JSON).await;
    handle.abort();
    assert_eq!(s, 201);
    assert_eq!(ct, "application/json");
    assert_eq!(json["key"], "grado");
    assert_eq!(json["type"], "enum");
    assert_eq!(json["facetable"], true);
    assert_eq!(json["active"], true);
    assert!(!json.as_object().unwrap().contains_key("clave"));
    assert_conforms("POST", ATTR_PATH, 201, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_attribute_defs_omits_inactive_by_default() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_attr(addr, GRADO_JSON).await;
    let id = j["id"].as_str().unwrap();
    delete_attr(addr, id).await;

    let (s, ct, json) = get_attrs(addr, "").await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(ct, "application/json");
    assert!(json["items"].as_array().unwrap().is_empty());
    assert_conforms("GET", ATTR_PATH, 200, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_attribute_defs_include_inactive_lists_inactive() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_attr(addr, GRADO_JSON).await;
    let id = j["id"].as_str().unwrap();
    delete_attr(addr, id).await;

    let (s, _, json) = get_attrs(addr, "includeInactive=true").await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(json["items"][0]["key"], "grado");
    assert_eq!(json["items"][0]["active"], false);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_attribute_defs_ordered_by_order() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    post_attr(
        addr,
        r#"{"key":"medida","label":"Medida","type":"text","order":10}"#,
    )
    .await;
    post_attr(addr, GRADO_JSON).await;

    let (s, _, json) = get_attrs(addr, "").await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(json["items"][0]["key"], "grado");
    assert_eq!(json["items"][1]["key"], "medida");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_attribute_def_updates_label() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_attr(addr, GRADO_JSON).await;
    let id = j["id"].as_str().unwrap();
    let (s, _ct, json) = patch_attr(addr, id, r#"{"label":"Grado SAE"}"#).await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(json["label"], "Grado SAE");
    assert_conforms(
        "PATCH",
        "/api/v1/attribute-definitions/{attributeId}",
        200,
        "application/json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_attribute_def_changes_key_without_in_use() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_attr(addr, r#"{"key":"medida","label":"Medida","type":"text"}"#).await;
    let id = j["id"].as_str().unwrap();
    let (s, _ct, json) = patch_attr(addr, id, r#"{"key":"medida_mm"}"#).await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(json["key"], "medida_mm");
    assert_ne!(
        json.get("code").and_then(|c| c.as_str()),
        Some("attribute_in_use")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_attribute_def_reactivates() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_attr(addr, GRADO_JSON).await;
    let id = j["id"].as_str().unwrap();
    delete_attr(addr, id).await;
    let (s, _, json) = patch_attr(addr, id, r#"{"active":true}"#).await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(json["active"], true);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_attribute_def_returns_204() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_attr(addr, GRADO_JSON).await;
    let id = j["id"].as_str().unwrap();
    let (s, _ct, json) = delete_attr(addr, id).await;
    handle.abort();
    assert_eq!(s, 204);
    assert!(json.is_null());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_attribute_def_duplicate_key_returns_409() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    post_attr(addr, GRADO_JSON).await;
    let (s, ct, json) = post_attr(addr, GRADO_JSON).await;
    handle.abort();
    assert_eq!(s, 409);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "conflict");
    assert_conforms("POST", ATTR_PATH, 409, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_attribute_def_enum_without_options_returns_400() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = post_attr(addr, r#"{"key":"grado","label":"Grado","type":"enum"}"#).await;
    handle.abort();
    assert_eq!(s, 400);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "invalid_input");
    assert_conforms("POST", ATTR_PATH, 400, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_attribute_def_unknown_id_returns_404() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let unknown = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
    let (s, _, json) = patch_attr(addr, unknown, r#"{"label":"X"}"#).await;
    handle.abort();
    assert_eq!(s, 404);
    assert_eq!(json["code"], "not_found");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_attribute_def_unknown_id_returns_404() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let unknown = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
    let (s, _, json) = delete_attr(addr, unknown).await;
    handle.abort();
    assert_eq!(s, 404);
    assert_eq!(json["code"], "not_found");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_attribute_def_missing_key_returns_400() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = post_attr(addr, r#"{"label":"Grado","type":"enum","options":["2"]}"#).await;
    handle.abort();
    assert_eq!(s, 400);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "invalid_input");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_attribute_def_empty_body_returns_400() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_attr(addr, GRADO_JSON).await;
    let id = j["id"].as_str().unwrap();
    let (s, ct, json) = patch_attr(addr, id, r#"{}"#).await;
    handle.abort();
    assert_eq!(s, 400);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "invalid_input");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn attribute_id_invalid_ulid_returns_400() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = delete_attr(addr, "not-a-ulid").await;
    handle.abort();
    assert_eq!(s, 400);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "invalid_input");
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"attributeId"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_attribute_defs_without_service_returns_503() {
    let state = AppState::new();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = get_attrs(addr, "").await;
    handle.abort();
    assert_eq!(s, 503);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "unavailable");
    assert_conforms("GET", ATTR_PATH, 503, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn attribute_def_problem_field_is_key_not_clave() {
    let state = state_with_attribute_defs();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = post_attr(addr, r#"{"key":"Grado","label":"X","type":"text"}"#).await;
    handle.abort();
    assert_eq!(s, 400);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "invalid_input");
    let empty = vec![];
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"key"));
    assert!(!fields.iter().any(|f| *f == "clave"));
}
