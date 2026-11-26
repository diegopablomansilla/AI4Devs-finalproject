use std::net::SocketAddr;

use api::{AppState, build_router};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn http_get_full(addr: SocketAddr, path: &str) -> (u16, String, serde_json::Value) {
    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("connect to test server");
    let host = format!("{}:{}", addr.ip(), addr.port());
    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("write request");
    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.expect("read response");
    let response = String::from_utf8_lossy(&buf[..n]);
    let status_line = response.lines().next().unwrap_or("");
    let status = status_line
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
    let json: serde_json::Value = serde_json::from_slice(&buf[body_start..n]).unwrap_or_default();
    (status, ctype, json)
}

async fn serve_router(state: AppState) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local_addr");
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    (addr, handle)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn build_router_unknown_path_returns_404() {
    let state = AppState::new();
    let (addr, handle) = serve_router(state).await;

    let mut status = 0u16;
    let mut ctype = String::new();
    let mut json = serde_json::Value::Null;
    for _ in 0..50 {
        if let Ok((s, c, j)) = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            http_get_full(addr, "/no-such-route"),
        )
        .await
        {
            status = s;
            ctype = c;
            json = j;
            if status == 404 {
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    handle.abort();
    assert_eq!(status, 404, "unknown route must return 404");
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(json["code"], "not_found");
    assert_eq!(json["instance"], "/no-such-route");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn app_state_clone_serves_two_requests() {
    let state = AppState::new();
    let _clone = state.clone();
    let (addr, handle) = serve_router(state).await;

    let (status_a, _, json_a) = http_get_full(addr, "/first-unknown").await;
    let (status_b, _, json_b) = http_get_full(addr, "/second-unknown").await;
    handle.abort();
    assert_eq!(status_a, 404);
    assert_eq!(status_b, 404);
    assert_eq!(json_a["code"], "not_found");
    assert_eq!(json_b["code"], "not_found");
}
