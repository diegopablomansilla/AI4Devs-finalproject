//! Gateway middleware integration tests (T-00-12).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use api::{AppState, DEFAULT_BODY_LIMIT, apply_gateway_layers, build_router, router_shell};
use axum::Router;
use axum::body::Body;
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn probe_slow() -> impl IntoResponse {
    tokio::time::sleep(Duration::from_secs(60)).await;
    axum::http::StatusCode::NO_CONTENT
}

fn large_json_payload() -> String {
    format!(r#"{{"pad":"{}"}}"#, "x".repeat(4096))
}

async fn probe_large() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "data": large_json_payload()
    }))
}

async fn probe_sse() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/event-stream")],
        Body::from("data: hello\n\n"),
    )
}

fn router_with_probes(state: AppState) -> Router {
    apply_gateway_layers(
        router_shell(state)
            .route("/_probe/slow", get(probe_slow))
            .route("/_probe/large", get(probe_large))
            .route("/_probe/sse", get(probe_sse))
            .route(
                "/_probe/echo",
                post(|body: String| async move { (axum::http::StatusCode::OK, body) }),
            ),
    )
}

struct HttpResponse {
    status: u16,
    headers: HashMap<String, String>,
    json: serde_json::Value,
}

fn parse_headers(response: &str) -> (u16, HashMap<String, String>, usize) {
    let status_line = response.lines().next().unwrap_or("");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);
    let mut headers = HashMap::new();
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
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    (status, headers, body_start)
}

async fn http_raw(
    addr: SocketAddr,
    method: &str,
    path: &str,
    extra_headers: &[(&str, &str)],
    body: Option<&[u8]>,
) -> HttpResponse {
    let mut stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let host = format!("{}:{}", addr.ip(), addr.port());
    let mut request = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n");
    for (k, v) in extra_headers {
        request.push_str(&format!("{k}: {v}\r\n"));
    }
    if let Some(bytes) = body {
        request.push_str(&format!("Content-Length: {}\r\n", bytes.len()));
        request.push_str("\r\n");
        stream
            .write_all(request.as_bytes())
            .await
            .expect("write hdr");
        stream.write_all(bytes).await.expect("write body");
    } else {
        request.push_str("\r\n");
        stream.write_all(request.as_bytes()).await.expect("write");
    }
    let mut buf = vec![0u8; 512 * 1024];
    let n = stream.read(&mut buf).await.expect("read");
    let response = String::from_utf8_lossy(&buf[..n]);
    let (status, headers, body_start) = parse_headers(&response);
    let json: serde_json::Value =
        serde_json::from_slice(&buf[body_start..n]).unwrap_or(serde_json::Value::Null);
    HttpResponse {
        status,
        headers,
        json,
    }
}

async fn serve_tcp(router: Router) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let handle = tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });
    (addr, handle)
}

async fn wait_server(addr: SocketAddr) {
    for _ in 0..50 {
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("server did not start");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_body_returns_413_payload_too_large() {
    let router = router_with_probes(AppState::new());
    let (addr, handle) = serve_tcp(router).await;
    wait_server(addr).await;

    let cl = (DEFAULT_BODY_LIMIT + 1).to_string();
    let resp = http_raw(
        addr,
        "POST",
        "/_probe/echo",
        &[
            ("Content-Type", "application/json"),
            ("Content-Length", &cl),
        ],
        None,
    )
    .await;
    handle.abort();

    assert_eq!(resp.status, 413);
    assert_eq!(
        resp.headers.get("content-type").map(String::as_str),
        Some("application/problem+json")
    );
    assert_eq!(resp.json["code"], "payload_too_large");
    assert_eq!(resp.json["title"], "El cuerpo es demasiado grande");
    assert!(resp.json.get("errors").is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn body_at_limit_is_not_413() {
    let router = build_router(AppState::new());
    let (addr, handle) = serve_tcp(router).await;
    wait_server(addr).await;

    let cl = DEFAULT_BODY_LIMIT.to_string();
    let resp = http_raw(
        addr,
        "POST",
        "/no-such-route",
        &[("Content-Length", &cl)],
        None,
    )
    .await;
    handle.abort();

    assert_ne!(resp.status, 413);
    assert_ne!(resp.json["code"], "payload_too_large");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn slow_handler_returns_504_timeout() {
    let router = router_with_probes(AppState::new());
    let (addr, handle) = serve_tcp(router).await;
    wait_server(addr).await;

    let client =
        tokio::spawn(async move { http_raw(addr, "GET", "/_probe/slow", &[], None).await });
    tokio::time::sleep(Duration::from_secs(6)).await;
    let resp = client.await.expect("join");
    handle.abort();

    assert_eq!(resp.status, 504);
    assert_eq!(
        resp.headers.get("content-type").map(String::as_str),
        Some("application/problem+json")
    );
    assert_eq!(resp.json["code"], "timeout");
    assert_eq!(
        resp.json["title"],
        "La operación superó el tiempo de espera"
    );
    assert!(resp.json.get("errors").is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fast_request_is_not_504() {
    let router = build_router(AppState::new());
    let (addr, handle) = serve_tcp(router).await;
    wait_server(addr).await;

    let resp = http_raw(addr, "GET", "/no-such-route", &[], None).await;
    handle.abort();

    assert_eq!(resp.status, 404);
    assert_eq!(resp.json["code"], "not_found");
    assert_ne!(resp.status, 504);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn api_path_sends_cache_control_no_store() {
    let router = build_router(AppState::new());
    let (addr, handle) = serve_tcp(router).await;
    wait_server(addr).await;

    let resp = http_raw(addr, "GET", "/api/v1/no-such-route", &[], None).await;
    handle.abort();

    assert_eq!(resp.status, 404);
    assert_eq!(resp.json["code"], "not_found");
    assert_eq!(
        resp.headers.get("cache-control").map(String::as_str),
        Some("no-store")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn non_api_path_omits_cache_control_no_store() {
    let router = build_router(AppState::new());
    let (addr, handle) = serve_tcp(router).await;
    wait_server(addr).await;

    let resp = http_raw(addr, "GET", "/no-such-route", &[], None).await;
    handle.abort();

    assert_eq!(resp.status, 404);
    assert_eq!(resp.json["code"], "not_found");
    assert!(resp.headers.get("cache-control").is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn json_response_is_gzip_compressed() {
    let router = router_with_probes(AppState::new());
    let (addr, handle) = serve_tcp(router).await;
    wait_server(addr).await;

    let resp = http_raw(
        addr,
        "GET",
        "/_probe/large",
        &[("Accept-Encoding", "gzip")],
        None,
    )
    .await;
    handle.abort();

    assert_eq!(
        resp.headers.get("content-encoding").map(String::as_str),
        Some("gzip")
    );
    let ctype = resp
        .headers
        .get("content-type")
        .map(String::as_str)
        .unwrap_or("");
    assert!(ctype.contains("json"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn json_response_is_brotli_compressed() {
    let router = router_with_probes(AppState::new());
    let (addr, handle) = serve_tcp(router).await;
    wait_server(addr).await;

    let resp = http_raw(
        addr,
        "GET",
        "/_probe/large",
        &[("Accept-Encoding", "br")],
        None,
    )
    .await;
    handle.abort();

    assert_eq!(
        resp.headers.get("content-encoding").map(String::as_str),
        Some("br")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sse_response_is_not_compressed() {
    let router = router_with_probes(AppState::new());
    let (addr, handle) = serve_tcp(router).await;
    wait_server(addr).await;

    let resp = http_raw(
        addr,
        "GET",
        "/_probe/sse",
        &[("Accept-Encoding", "gzip")],
        None,
    )
    .await;
    handle.abort();

    let ctype = resp
        .headers
        .get("content-type")
        .map(String::as_str)
        .unwrap_or("");
    assert!(ctype.contains("text/event-stream"));
    assert!(resp.headers.get("content-encoding").is_none());
}

#[derive(Default)]
struct SpanLog {
    events: Vec<String>,
}

impl SpanLog {
    fn joined(&self) -> String {
        self.events.join("\n")
    }
}

struct TestWriter(Arc<Mutex<SpanLog>>);

impl std::io::Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(s) = std::str::from_utf8(buf) {
            self.0.lock().expect("lock").events.push(s.to_string());
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn http_request_span_includes_method_route_status_duration() {
    let log = Arc::new(Mutex::new(SpanLog::default()));
    let log_writer = Arc::clone(&log);
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(move || TestWriter(Arc::clone(&log_writer)))
        .finish();
    let _guard =
        tracing::subscriber::set_global_default(subscriber).expect("set global subscriber");

    let router = build_router(AppState::new());
    let (addr, handle) = serve_tcp(router).await;
    wait_server(addr).await;
    let _resp = http_raw(addr, "GET", "/no-such-route", &[], None).await;
    handle.abort();

    let text = log.lock().expect("lock").joined().to_ascii_lowercase();
    assert!(
        text.contains("get"),
        "expected HTTP method in span log: {text}"
    );
    assert!(
        text.contains("/no-such-route") || text.contains("no-such-route"),
        "expected route in span log: {text}"
    );
    assert!(
        text.contains("404") || text.contains("status"),
        "expected status in span log: {text}"
    );
    assert!(
        text.contains("duration")
            || text.contains("latency")
            || text.contains("ms")
            || text.contains("µs")
            || text.contains("finished"),
        "expected duration in span log: {text}"
    );
}
