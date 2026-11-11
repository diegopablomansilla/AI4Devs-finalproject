mod support;

use std::net::SocketAddr;
use std::sync::Arc;

use api::{AppState, build_router};
use domain::{ComponentHealth, ComponentStatus, HealthComponents, HealthSnapshot};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn snapshot_ok() -> HealthSnapshot {
    HealthSnapshot::from_components("1.0.0", 42, HealthComponents::all_disabled())
}

fn snapshot_degraded() -> HealthSnapshot {
    let mut c = HealthComponents::all_disabled();
    c.storage = ComponentHealth::new(ComponentStatus::Up);
    c.meilisearch = ComponentHealth::new(ComponentStatus::Down);
    HealthSnapshot::from_components("1.0.0", 0, c)
}

fn snapshot_starting() -> HealthSnapshot {
    let mut c = HealthComponents::all_up();
    c.meilisearch = ComponentHealth::new(ComponentStatus::Starting);
    HealthSnapshot::from_components("1.0.0", 0, c)
}

fn snapshot_down() -> HealthSnapshot {
    let mut c = HealthComponents::all_up();
    c.storage = ComponentHealth::new(ComponentStatus::Down);
    HealthSnapshot::from_components("1.0.0", 0, c)
}

async fn http_request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: Option<&[u8]>,
) -> (u16, String, serde_json::Value, Vec<(String, String)>) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let host = format!("{}:{}", addr.ip(), addr.port());
    let mut request = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n");
    if let Some(b) = body {
        request.push_str(&format!("Content-Length: {}\r\n", b.len()));
    }
    request.push_str("\r\n");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("write headers");
    if let Some(b) = body {
        stream.write_all(b).await.expect("write body");
    }
    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await.expect("read");
    let response = String::from_utf8_lossy(&buf[..n]);
    let status_line = response.lines().next().unwrap_or("");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);
    let mut headers = Vec::new();
    let mut body_start = 0usize;
    for (i, _) in response.match_indices("\r\n\r\n") {
        body_start = i + 4;
        break;
    }
    let mut ctype = String::new();
    for line in response.lines() {
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim().to_string();
            if name == "content-type" {
                ctype = value.split(';').next().unwrap_or(&value).trim().to_string();
            }
            headers.push((name, value));
        }
    }
    let json: serde_json::Value =
        serde_json::from_slice(&buf[body_start..n]).unwrap_or(serde_json::Value::Null);
    (status, ctype, json, headers)
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_health_ok_returns_200() {
    let state = AppState::with_health(Arc::new(snapshot_ok()));
    let (addr, handle) = serve(state).await;
    let (status, ctype, json, _) = http_request(addr, "GET", "/health", None).await;
    handle.abort();
    assert_eq!(status, 200);
    assert_eq!(ctype, "application/json");
    assert_eq!(json["status"], "ok");
    assert!(!json["version"].as_str().unwrap_or("").is_empty());
    assert!(json["uptimeSeconds"].is_number());
    for key in [
        "storage",
        "meilisearch",
        "llm",
        "speech",
        "textEmbeddings",
        "sync",
    ] {
        assert!(json["components"][key].is_object());
    }
    support::openapi::assert_conforms("GET", "/health", status, &ctype, &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_health_degraded_returns_200() {
    let state = AppState::with_health(Arc::new(snapshot_degraded()));
    let (addr, handle) = serve(state).await;
    let (status, _, json, _) = http_request(addr, "GET", "/health", None).await;
    handle.abort();
    assert_eq!(status, 200);
    assert_eq!(json["status"], "degraded");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_health_starting_returns_503() {
    let state = AppState::with_health(Arc::new(snapshot_starting()));
    let (addr, handle) = serve(state).await;
    let (status, ctype, json, _) = http_request(addr, "GET", "/health", None).await;
    handle.abort();
    assert_eq!(status, 503);
    assert_eq!(ctype, "application/json");
    assert_eq!(json["status"], "starting");
    assert!(json.get("code").is_none());
    support::openapi::assert_conforms("GET", "/health", status, &ctype, &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_health_down_returns_503() {
    let state = AppState::with_health(Arc::new(snapshot_down()));
    let (addr, handle) = serve(state).await;
    let (status, ctype, json, _) = http_request(addr, "GET", "/health", None).await;
    handle.abort();
    assert_eq!(status, 503);
    assert_eq!(ctype, "application/json");
    assert_eq!(json["status"], "down");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_health_is_not_problem_json() {
    let state = AppState::with_health(Arc::new(snapshot_ok()));
    let (addr, handle) = serve(state).await;
    let (_, ctype, json, headers) = http_request(addr, "GET", "/health", None).await;
    handle.abort();
    assert_ne!(ctype, "application/problem+json");
    assert!(json.get("code").is_none());
    assert!(
        !headers
            .iter()
            .any(|(k, v)| k == "cache-control" && v == "no-store")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_health_is_not_200_health() {
    let state = AppState::new();
    let (addr, handle) = serve(state).await;
    let (status, ctype, json, _) = http_request(addr, "POST", "/health", Some(b"{}")).await;
    handle.abort();
    assert_ne!(status, 200);
    assert_ne!(ctype, "application/json");
    assert_ne!(json.get("status").and_then(|v| v.as_str()), Some("ok"));
}
