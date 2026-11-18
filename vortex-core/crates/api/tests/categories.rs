mod support;

use api::{AppState, build_router};
use domain::Categoria;
use domain::Id;
use std::net::SocketAddr;
use support::fake_catalog::test_category_gateway;
use support::openapi::assert_conforms;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn http_send(
    addr: SocketAddr,
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

async fn serve(state: AppState) -> (SocketAddr, tokio::task::JoinHandle<()>) {
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

fn state_with_categories() -> AppState {
    AppState::new().with_categories(test_category_gateway())
}

async fn post_category(addr: SocketAddr, json: &str) -> (u16, String, serde_json::Value) {
    let (status, ctype, body) = http_send(addr, "POST", "/api/v1/categories", Some(json)).await;
    (status, ctype, json_body(&body))
}

async fn get_categories(addr: SocketAddr, query: &str) -> (u16, String, serde_json::Value) {
    let path = if query.is_empty() {
        "/api/v1/categories".to_string()
    } else {
        format!("/api/v1/categories?{query}")
    };
    let (status, ctype, body) = http_send(addr, "GET", &path, None).await;
    (status, ctype, json_body(&body))
}

async fn patch_category(
    addr: SocketAddr,
    id: &str,
    json: &str,
) -> (u16, String, serde_json::Value) {
    let path = format!("/api/v1/categories/{id}");
    let (status, ctype, body) = http_send(addr, "PATCH", &path, Some(json)).await;
    (status, ctype, json_body(&body))
}

async fn delete_category(addr: SocketAddr, id: &str) -> (u16, String, serde_json::Value) {
    let path = format!("/api/v1/categories/{id}");
    let (status, ctype, body) = http_send(addr, "DELETE", &path, None).await;
    (status, ctype, json_body(&body))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_category_child_returns_201_with_path() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (s1, ct1, j1) = post_category(addr, r#"{"name":"Fijaciones"}"#).await;
    assert_eq!(s1, 201);
    assert_eq!(ct1, "application/json");
    assert_conforms("POST", "/api/v1/categories", 201, "application/json", &j1);
    let parent = j1["id"].as_str().expect("id");

    let body = format!(r#"{{"name":"Pernos","parentId":"{parent}"}}"#);
    let (s2, ct2, j2) = post_category(addr, &body).await;
    handle.abort();
    assert_eq!(s2, 201);
    assert_eq!(ct2, "application/json");
    assert_eq!(j2["name"], "Pernos");
    assert_eq!(j2["parentId"].as_str(), Some(parent));
    assert_eq!(j2["path"], serde_json::json!(["Fijaciones", "Pernos"]));
    assert!(j2.get("nombre").is_none());
    assert!(j2.get("padre").is_none());
    assert_conforms("POST", "/api/v1/categories", 201, "application/json", &j2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_categories_omits_inactive_by_default() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (s1, _, j1) = post_category(addr, r#"{"name":"Activa"}"#).await;
    assert_eq!(s1, 201);
    let active_id = j1["id"].as_str().unwrap();

    let (s2, _, j2) = post_category(addr, r#"{"name":"Inactiva"}"#).await;
    assert_eq!(s2, 201);
    let inactive_id = j2["id"].as_str().unwrap();
    let (sd, _, _) = delete_category(addr, inactive_id).await;
    assert_eq!(sd, 204);

    let (s, ct, json) = get_categories(addr, "").await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(ct, "application/json");
    assert_conforms("GET", "/api/v1/categories", 200, "application/json", &json);
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
async fn get_categories_include_inactive_lists_inactive() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_category(addr, r#"{"name":"Baja"}"#).await;
    let id = j["id"].as_str().unwrap();
    delete_category(addr, id).await;

    let (s, _, json) = get_categories(addr, "includeInactive=true").await;
    handle.abort();
    assert_eq!(s, 200);
    let found = json["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["id"].as_str() == Some(id) && c["active"] == false);
    assert!(found);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_category_renames() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_category(addr, r#"{"name":"Fijaciones"}"#).await;
    let id = j["id"].as_str().unwrap();

    let (s, ct, json) = patch_category(addr, id, r#"{"name":"Tornillería"}"#).await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(ct, "application/json");
    assert_eq!(json["name"], "Tornillería");
    assert_conforms(
        "PATCH",
        "/api/v1/categories/{categoryId}",
        200,
        "application/json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_category_moves() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (_, _, fij) = post_category(addr, r#"{"name":"Fijaciones"}"#).await;
    let (_, _, her) = post_category(addr, r#"{"name":"Herramientas"}"#).await;
    let parent_fij = fij["id"].as_str().unwrap();
    let parent_her = her["id"].as_str().unwrap();
    let body = format!(r#"{{"name":"Pernos","parentId":"{parent_fij}"}}"#);
    let (_, _, per) = post_category(addr, &body).await;
    let per_id = per["id"].as_str().unwrap();

    let patch = format!(r#"{{"parentId":"{parent_her}"}}"#);
    let (s, _, json) = patch_category(addr, per_id, &patch).await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(json["parentId"].as_str(), Some(parent_her));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_category_reactivates() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_category(addr, r#"{"name":"Pernos"}"#).await;
    let id = j["id"].as_str().unwrap();
    delete_category(addr, id).await;

    let (s, _, json) = patch_category(addr, id, r#"{"active":true}"#).await;
    handle.abort();
    assert_eq!(s, 200);
    assert_eq!(json["active"], true);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_category_returns_204() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_category(addr, r#"{"name":"Pernos"}"#).await;
    let id = j["id"].as_str().unwrap();

    let (s, ct, body) = http_send(addr, "DELETE", &format!("/api/v1/categories/{id}"), None).await;
    handle.abort();
    assert_eq!(s, 204);
    assert_eq!(ct, "");
    assert!(body.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_category_duplicate_slug_returns_409() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    post_category(addr, r#"{"name":"Fijaciones"}"#).await;
    let (s, ct, json) = post_category(addr, r#"{"name":"Fijaciones"}"#).await;
    handle.abort();
    assert_eq!(s, 409);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "duplicate_slug");
    assert_conforms(
        "POST",
        "/api/v1/categories",
        409,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_category_cycle_returns_400() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (_, _, fij) = post_category(addr, r#"{"name":"Fijaciones"}"#).await;
    let fij_id = fij["id"].as_str().unwrap();
    let body = format!(r#"{{"name":"Pernos","parentId":"{fij_id}"}}"#);
    let (_, _, per) = post_category(addr, &body).await;
    let per_id = per["id"].as_str().unwrap();

    let patch = format!(r#"{{"parentId":"{per_id}"}}"#);
    let (s, ct, json) = patch_category(addr, fij_id, &patch).await;
    handle.abort();
    assert_eq!(s, 400);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "category_cycle");
    assert_conforms(
        "PATCH",
        "/api/v1/categories/{categoryId}",
        400,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_category_with_active_child_returns_409() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (_, _, fij) = post_category(addr, r#"{"name":"Fijaciones"}"#).await;
    let fij_id = fij["id"].as_str().unwrap();
    let body = format!(r#"{{"name":"Pernos","parentId":"{fij_id}"}}"#);
    post_category(addr, &body).await;

    let (s, ct, json) = delete_category(addr, fij_id).await;
    handle.abort();
    assert_eq!(s, 409);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "conflict");
    assert_conforms(
        "DELETE",
        "/api/v1/categories/{categoryId}",
        409,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_category_unknown_id_returns_404() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let missing = Id::<Categoria>::generate().to_string();
    let (s, ct, json) = patch_category(addr, &missing, r#"{"name":"X"}"#).await;
    handle.abort();
    assert_eq!(s, 404);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "not_found");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_category_unknown_id_returns_404() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let missing = Id::<Categoria>::generate().to_string();
    let (s, ct, json) = delete_category(addr, &missing).await;
    handle.abort();
    assert_eq!(s, 404);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "not_found");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_category_missing_name_returns_400() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = post_category(addr, r#"{"parentId":null}"#).await;
    handle.abort();
    assert_eq!(s, 400);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "invalid_input");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_category_slug_only_returns_400() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (_, _, j) = post_category(addr, r#"{"name":"Fijaciones"}"#).await;
    let id = j["id"].as_str().unwrap();

    let (s, ct, json) = patch_category(addr, id, r#"{"slug":"nuevo-slug"}"#).await;
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
    assert!(fields.contains(&"slug"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn category_id_invalid_ulid_returns_400() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = delete_category(addr, "not-a-ulid").await;
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
    assert!(fields.contains(&"categoryId"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_categories_without_service_returns_503() {
    let state = AppState::new();
    let (addr, handle) = serve(state).await;

    let (s, ct, json) = get_categories(addr, "").await;
    handle.abort();
    assert_eq!(s, 503);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "unavailable");
    assert_conforms(
        "GET",
        "/api/v1/categories",
        503,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn category_problem_field_is_parent_id_not_padre() {
    let state = state_with_categories();
    let (addr, handle) = serve(state).await;

    let (_, _, fij) = post_category(addr, r#"{"name":"Fijaciones"}"#).await;
    let fij_id = fij["id"].as_str().unwrap();
    let body = format!(r#"{{"name":"Pernos","parentId":"{fij_id}"}}"#);
    let (_, _, per) = post_category(addr, &body).await;
    let per_id = per["id"].as_str().unwrap();

    let patch = format!(r#"{{"parentId":"{per_id}"}}"#);
    let (_, _, json) = patch_category(addr, fij_id, &patch).await;
    handle.abort();
    let empty = vec![];
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"parentId"));
    let raw = serde_json::to_string(&json).unwrap();
    assert!(!raw.contains("padre"));
}
