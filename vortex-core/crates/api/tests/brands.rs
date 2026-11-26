mod support;

use api::{AppState, build_router};
use support::fake_catalog::test_brand_gateway;
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

fn state_with_brands() -> AppState {
    AppState::new().with_brands(test_brand_gateway())
}

async fn post_brand(addr: std::net::SocketAddr, json: &str) -> (u16, String, serde_json::Value) {
    let (status, ctype, body) = http_send(addr, "POST", "/api/v1/brands", Some(json)).await;
    (status, ctype, json_body(&body))
}

async fn get_brands(addr: std::net::SocketAddr, query: &str) -> (u16, String, serde_json::Value) {
    let path = if query.is_empty() {
        "/api/v1/brands".to_string()
    } else {
        format!("/api/v1/brands?{query}")
    };
    let (status, ctype, body) = http_send(addr, "GET", &path, None).await;
    (status, ctype, json_body(&body))
}

async fn patch_brand(
    addr: std::net::SocketAddr,
    id: &str,
    json: &str,
) -> (u16, String, serde_json::Value) {
    let path = format!("/api/v1/brands/{id}");
    let (status, ctype, body) = http_send(addr, "PATCH", &path, Some(json)).await;
    (status, ctype, json_body(&body))
}

async fn delete_brand(addr: std::net::SocketAddr, id: &str) -> (u16, String, serde_json::Value) {
    let path = format!("/api/v1/brands/{id}");
    let (status, ctype, body) = http_send(addr, "DELETE", &path, None).await;
    (status, ctype, json_body(&body))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_brand_fixser_returns_201() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = post_brand(addr, r#"{"name":"Fixser"}"#).await;
    handle.abort();
    assert_eq!(s, 201);
    assert_eq!(ct, "application/json");
    assert_eq!(json["name"], "Fixser");
    assert_eq!(json["active"], true);
    assert!(json.get("nombre").is_none());
    assert_conforms("POST", "/api/v1/brands", 201, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_brands_omits_inactive_by_default() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let (_, _, j1) = post_brand(addr, r#"{"name":"Activa"}"#).await;
    let active_id = j1["id"].as_str().unwrap();
    let (_, _, j2) = post_brand(addr, r#"{"name":"Inactiva"}"#).await;
    let inactive_id = j2["id"].as_str().unwrap();
    delete_brand(addr, inactive_id).await;

    let (s, ct, json) = get_brands(addr, "").await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(ct, "application/json");
    assert_conforms("GET", "/api/v1/brands", 200, "application/json", &json);
    let ids: Vec<_> = json["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["id"].as_str())
        .collect();
    assert!(ids.contains(&active_id));
    assert!(!ids.contains(&inactive_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_brands_include_inactive_lists_inactive() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_brand(addr, r#"{"name":"Baja"}"#).await;
    let id = j["id"].as_str().unwrap();
    delete_brand(addr, id).await;

    let (s, _, json) = get_brands(addr, "includeInactive=true").await;
    handle.abort();
    assert_eq!(s, 200);
    let found = json["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|b| b["id"].as_str() == Some(id) && b["active"] == false);
    assert!(found);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_brands_q_filters_prefix() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    post_brand(addr, r#"{"name":"Fixser"}"#).await;
    post_brand(addr, r#"{"name":"Tramontina"}"#).await;

    let (s, _, json) = get_brands(addr, "q=fix").await;
    handle.abort();
    assert_eq!(s, 200);
    let names: Vec<_> = json["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["name"].as_str())
        .collect();
    assert!(names.iter().any(|n| *n == "Fixser"));
    assert!(!names.iter().any(|n| *n == "Tramontina"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_brands_ordered_by_name() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    post_brand(addr, r#"{"name":"Tramontina"}"#).await;
    post_brand(addr, r#"{"name":"Fixser"}"#).await;

    let (s, _, json) = get_brands(addr, "").await;
    handle.abort();
    assert_eq!(s, 200);
    let items = json["items"].as_array().unwrap();
    assert_eq!(items[0]["name"], "Fixser");
    assert_eq!(items[1]["name"], "Tramontina");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_brand_renames() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_brand(addr, r#"{"name":"Fixser"}"#).await;
    let id = j["id"].as_str().unwrap();

    let (s, ct, json) = patch_brand(addr, id, r#"{"name":"Fixser Pro"}"#).await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(ct, "application/json");
    assert_eq!(json["name"], "Fixser Pro");
    assert_conforms(
        "PATCH",
        "/api/v1/brands/{brandId}",
        200,
        "application/json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_brand_updates_slug() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_brand(addr, r#"{"name":"Fixser"}"#).await;
    let id = j["id"].as_str().unwrap();

    let (s, _, json) = patch_brand(addr, id, r#"{"slug":"fixser-pro"}"#).await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(json["slug"], "fixser-pro");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_brand_reactivates() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_brand(addr, r#"{"name":"Fixser"}"#).await;
    let id = j["id"].as_str().unwrap();
    delete_brand(addr, id).await;

    let (s, _, json) = patch_brand(addr, id, r#"{"active":true}"#).await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(json["active"], true);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_brand_returns_204() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_brand(addr, r#"{"name":"Fixser"}"#).await;
    let id = j["id"].as_str().unwrap();

    let (s, ct, body) = http_send(addr, "DELETE", &format!("/api/v1/brands/{id}"), None).await;
    handle.abort();
    assert_eq!(s, 204);
    assert_eq!(ct, "");
    assert!(body.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_brand_with_associated_products_returns_204() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_brand(addr, r#"{"name":"Fixser"}"#).await;
    let id = j["id"].as_str().unwrap();

    let (s, ct, body) = http_send(addr, "DELETE", &format!("/api/v1/brands/{id}"), None).await;
    handle.abort();
    assert_eq!(s, 204);
    assert_ne!(s, 409);
    assert_eq!(ct, "");
    assert!(body.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_brand_duplicate_slug_returns_409() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    post_brand(addr, r#"{"name":"Fixser","slug":"fixser"}"#).await;
    let (s, ct, json) = post_brand(addr, r#"{"name":"Otra","slug":"fixser"}"#).await;
    handle.abort();
    assert_eq!(s, 409);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "duplicate_slug");
    assert_conforms(
        "POST",
        "/api/v1/brands",
        409,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_brand_unknown_id_returns_404() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let unknown = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
    let (s, _, json) = patch_brand(addr, unknown, r#"{"name":"X"}"#).await;
    handle.abort();
    assert_eq!(s, 404);
    assert_eq!(json["code"], "not_found");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_brand_unknown_id_returns_404() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let unknown = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
    let (s, _, json) = delete_brand(addr, unknown).await;
    handle.abort();
    assert_eq!(s, 404);
    assert_eq!(json["code"], "not_found");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_brand_missing_name_returns_400() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = post_brand(addr, r#"{}"#).await;
    handle.abort();
    assert_eq!(s, 400);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "invalid_input");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_brand_empty_body_returns_400() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_brand(addr, r#"{"name":"Fixser"}"#).await;
    let id = j["id"].as_str().unwrap();

    let (s, ct, json) = patch_brand(addr, id, r#"{}"#).await;
    handle.abort();
    assert_eq!(s, 400);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "invalid_input");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn brand_id_invalid_ulid_returns_400() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = delete_brand(addr, "not-a-ulid").await;
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
    assert!(fields.contains(&"brandId"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_brands_q_too_long_returns_400() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let q = "a".repeat(101);
    let query = format!("q={q}");
    let (s, ct, json) = get_brands(addr, &query).await;
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
    assert!(fields.contains(&"q"));
    assert_conforms(
        "GET",
        "/api/v1/brands",
        400,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_brands_without_service_returns_503() {
    let state = AppState::new();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = get_brands(addr, "").await;
    handle.abort();
    assert_eq!(s, 503);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "unavailable");
    assert_conforms(
        "GET",
        "/api/v1/brands",
        503,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn brand_problem_field_is_name_not_nombre() {
    let state = state_with_brands();
    let (addr, handle) = serve(state).await;

    let long_name = "a".repeat(101);
    let body = format!(r#"{{"name":"{long_name}"}}"#);
    let (s, ct, json) = post_brand(addr, &body).await;
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
    assert!(fields.contains(&"name"));
    assert!(!fields.iter().any(|f| *f == "nombre"));
}
