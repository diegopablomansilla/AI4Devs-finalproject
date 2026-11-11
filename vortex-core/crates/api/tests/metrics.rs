use std::net::SocketAddr;
use std::sync::Arc;

use api::{AppState, build_router};
use domain::HealthSnapshot;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn http_request_text(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: Option<&[u8]>,
) -> (u16, String, String, Vec<(String, String)>) {
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
    let mut buf = vec![0u8; 65536];
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
    let body = String::from_utf8_lossy(&buf[body_start..n]).into_owned();
    (status, ctype, body, headers)
}

async fn serve() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let state = AppState::with_health(Arc::new(HealthSnapshot::all_disabled(env!(
        "CARGO_PKG_VERSION"
    ))));
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
async fn http_histogram_records_health_route() {
    let (addr, handle) = serve().await;
    let (_, _, _, _) = http_request_text(addr, "GET", "/health", None).await;
    let (_, _, body, _) = http_request_text(addr, "GET", "/metrics", None).await;
    handle.abort();
    assert!(body.contains("http_request_duration_seconds"));
    assert!(body.contains(r#"route="/health""#) || body.contains(r#"route=\"/health\""#));
    assert!(body.contains(r#"method="GET""#) || body.contains("method=\"GET\""));
    assert!(body.contains("_bucket"));
    assert!(body.contains("_sum"));
    assert!(body.contains("_count"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unmatched_route_uses_unmatched_label() {
    let (addr, handle) = serve().await;
    let (_, _, _, _) = http_request_text(addr, "GET", "/no-such-route-xyz", None).await;
    let (_, _, body, _) = http_request_text(addr, "GET", "/metrics", None).await;
    handle.abort();
    assert!(body.contains(r#"route="unmatched""#) || body.contains(r#"route=\"unmatched\""#));
    assert!(body.contains(r#"status="404""#) || body.contains(r#"status=\"404\""#));
    assert!(
        !body.contains("/no-such-route-xyz"),
        "must not use raw URI as route label"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scrape_does_not_emit_domain_series() {
    let (addr, handle) = serve().await;
    let (_, _, _, _) = http_request_text(addr, "GET", "/health", None).await;
    let (_, _, body, _) = http_request_text(addr, "GET", "/metrics", None).await;
    handle.abort();
    assert!(!body.contains("outbox_lag"));
    assert!(!body.contains("outbox_failed_total"));
    assert!(!body.contains("backup_failures_total"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_metrics_returns_prometheus_text() {
    let (addr, handle) = serve().await;
    let (_, _, _, _) = http_request_text(addr, "GET", "/health", None).await;
    let (status, ctype, body, _) = http_request_text(addr, "GET", "/metrics", None).await;
    handle.abort();
    assert_eq!(status, 200);
    assert!(ctype.starts_with("text/plain"));
    assert!(!body.is_empty());
    assert!(!body.contains(r#""code":"not_found""#));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_metrics_includes_prior_request_histogram() {
    let (addr, handle) = serve().await;
    let (_, _, _, _) = http_request_text(addr, "GET", "/health", None).await;
    let (status, _, body, _) = http_request_text(addr, "GET", "/metrics", None).await;
    handle.abort();
    assert_eq!(status, 200);
    assert!(body.contains("http_request_duration_seconds"));
    assert!(body.contains("/health"));
    assert!(body.contains("GET"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_metrics_is_not_problem_json() {
    let (addr, handle) = serve().await;
    let (_, ctype, body, _) = http_request_text(addr, "GET", "/metrics", None).await;
    handle.abort();
    assert_ne!(ctype, "application/problem+json");
    assert!(!body.contains(r#""code":"not_found""#));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_metrics_omits_cache_control_no_store() {
    let (addr, handle) = serve().await;
    let (_, _, _, headers) = http_request_text(addr, "GET", "/metrics", None).await;
    handle.abort();
    assert!(
        !headers
            .iter()
            .any(|(k, v)| k == "cache-control" && v == "no-store")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_metrics_is_not_prometheus_200() {
    let (addr, handle) = serve().await;
    let (status, ctype, body, _) = http_request_text(addr, "POST", "/metrics", Some(b"{}")).await;
    handle.abort();
    let ok_scrape = status == 200
        && ctype.starts_with("text/plain")
        && body.contains("http_request_duration_seconds");
    assert!(!ok_scrape);
}
