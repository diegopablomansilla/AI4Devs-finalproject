use std::net::SocketAddr;

use api::{AppState, build_router};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn http_get_status(addr: SocketAddr, path: &str) -> u16 {
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
    status_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse()
        .unwrap_or(0)
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
    for _ in 0..50 {
        if let Ok(s) = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            http_get_status(addr, "/no-such-route"),
        )
        .await
        {
            status = s;
            if status == 404 {
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    handle.abort();
    assert_eq!(status, 404, "unknown route must return 404");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn app_state_clone_serves_two_requests() {
    let state = AppState::new();
    let _clone = state.clone();
    let (addr, handle) = serve_router(state).await;

    let status_a = http_get_status(addr, "/first-unknown").await;
    let status_b = http_get_status(addr, "/second-unknown").await;
    handle.abort();
    assert_eq!(status_a, 404);
    assert_eq!(status_b, 404);
}
