use std::net::SocketAddr;

use api::{AppJson, AppState, build_router};
use axum::Router;
use axum::routing::post;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Deserialize)]
struct ProbeBody {
    #[allow(dead_code)]
    name: String,
}

async fn probe_handler(_body: AppJson<ProbeBody>) -> axum::http::StatusCode {
    axum::http::StatusCode::NO_CONTENT
}

fn router_with_probe(state: AppState) -> Router {
    build_router(state).route("/_probe/json", post(probe_handler))
}

async fn http_post(
    addr: SocketAddr,
    path: &str,
    body: &str,
    content_type: Option<&str>,
) -> (u16, String, serde_json::Value) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let host = format!("{}:{}", addr.ip(), addr.port());
    let mut request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\nContent-Length: {}\r\n",
        body.len()
    );
    if let Some(ct) = content_type {
        request.push_str(&format!("Content-Type: {ct}\r\n"));
    }
    request.push_str("\r\n");
    request.push_str(body);
    stream.write_all(request.as_bytes()).await.expect("write");
    let mut buf = vec![0u8; 16_384];
    let n = stream.read(&mut buf).await.expect("read");
    let response = String::from_utf8_lossy(&buf[..n]);
    let mut lines = response.lines();
    let status_line = lines.next().unwrap_or("");
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
        if let Some(v) = line
            .strip_prefix("Content-Type:")
            .or_else(|| line.strip_prefix("content-type:"))
        {
            ctype = v.trim().split(';').next().unwrap_or(v).trim().to_string();
        }
    }
    let body_bytes = &buf[body_start..n];
    let json: serde_json::Value =
        serde_json::from_slice(body_bytes).unwrap_or(serde_json::Value::Null);
    (status, ctype, json)
}

async fn serve_probe() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let app = router_with_probe(AppState::new());
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
async fn json_rejection_returns_400_invalid_input() {
    let (addr, handle) = serve_probe().await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let (status, ctype, json) =
        http_post(addr, "/_probe/json", "{", Some("application/json")).await;
    assert_eq!(status, 400);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(json["code"], "invalid_input");
    assert!(json["errors"][0]["field"].as_str().map(str::is_empty) == Some(false));

    let (status, _, json) = http_post(addr, "/_probe/json", "{}", Some("application/json")).await;
    assert_eq!(status, 400);
    assert_eq!(json["code"], "invalid_input");

    let (status, _, json) = http_post(
        addr,
        "/_probe/json",
        r#"{"name":123}"#,
        Some("application/json"),
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(json["code"], "invalid_input");

    let (status, _, json) = http_post(addr, "/_probe/json", "{}", None).await;
    assert_eq!(status, 400);
    assert_eq!(json["code"], "invalid_input");

    handle.abort();
}
