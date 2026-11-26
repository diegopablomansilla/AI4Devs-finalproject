mod support;

use api::{AppState, build_router, prometheus_handle};
use recommend::RecommendationGateway;
use std::sync::Arc;
use support::fake_recommendations::{
    CANONICAL_ULID, FakeRecommendationGateway, canonical_product_id, highlight_substitutes_panel,
    panel_excludes_inactive, three_blocks_panel,
};
use support::openapi::assert_conforms;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const OPENAPI_PATH: &str = "/api/v1/products/{productId}/recommendations";
const UNKNOWN_ULID: &str = "01J9ZK4Q6Y3M2V8W0XAB1CD2EF";

fn rec_path(product_id: &str) -> String {
    format!("/api/v1/products/{product_id}/recommendations")
}

async fn http_get(addr: std::net::SocketAddr, path: &str) -> (u16, String, String) {
    http_send(addr, "GET", path, None).await
}

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
                    .unwrap_or(value)
                    .trim()
                    .to_string();
            }
        }
    }
    let body = String::from_utf8_lossy(&buf[body_start..n]).into_owned();
    (status, ctype, body)
}

struct RawResponse {
    status: u16,
    headers: std::collections::HashMap<String, String>,
    body: String,
}

async fn http_get_raw(addr: std::net::SocketAddr, path: &str) -> RawResponse {
    let mut stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let host = format!("{}:{}", addr.ip(), addr.port());
    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
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
    let mut headers = std::collections::HashMap::new();
    let mut body_start = 0usize;
    for (i, _) in response.match_indices("\r\n\r\n") {
        body_start = i + 4;
        break;
    }
    for line in response.lines().skip(1) {
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    let body = String::from_utf8_lossy(&buf[body_start..n]).into_owned();
    RawResponse {
        status,
        headers,
        body,
    }
}

fn json_body(body: &str) -> serde_json::Value {
    serde_json::from_str(body).unwrap_or(serde_json::Value::Null)
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

async fn serve_recommendations(
    gateway: Arc<dyn RecommendationGateway>,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let state = AppState::new().with_recommendations(gateway);
    serve(state).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_three_blocks_returns_200() {
    let product = canonical_product_id();
    let gateway: Arc<dyn RecommendationGateway> = Arc::new(FakeRecommendationGateway::with_panel(
        product.clone(),
        three_blocks_panel(product.clone()),
    ));
    let (addr, handle) = serve_recommendations(gateway).await;
    let path = rec_path(CANONICAL_ULID);
    let (status, ctype, body) = http_get(addr, &path).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 200);
    assert_eq!(ctype, "application/json");
    assert_eq!(json["similar"][0]["priority"], 1);
    assert_eq!(json["similar"][1]["priority"], 2);
    assert!(json.get("similares").is_none());
    assert_conforms("GET", OPENAPI_PATH, 200, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_highlights_substitutes_when_stock_zero() {
    let product = canonical_product_id();
    let gateway: Arc<dyn RecommendationGateway> = Arc::new(FakeRecommendationGateway::with_panel(
        product.clone(),
        highlight_substitutes_panel(product.clone()),
    ));
    let (addr, handle) = serve_recommendations(gateway).await;
    let (status, _, body) = http_get(addr, &rec_path(CANONICAL_ULID)).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 200);
    assert_eq!(json["highlightSubstitutes"], true);
    assert_eq!(json["substitutes"][0]["stock"], "12");
    assert_conforms("GET", OPENAPI_PATH, 200, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_excludes_inactive_target() {
    let product = canonical_product_id();
    let gateway: Arc<dyn RecommendationGateway> = Arc::new(FakeRecommendationGateway::with_panel(
        product.clone(),
        panel_excludes_inactive(product.clone()),
    ));
    let (addr, handle) = serve_recommendations(gateway).await;
    let (status, _, body) = http_get(addr, &rec_path(CANONICAL_ULID)).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 200);
    assert_eq!(json["similar"].as_array().map(|a| a.len()), Some(1));
    assert_conforms("GET", OPENAPI_PATH, 200, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_empty_when_no_edges_returns_200() {
    let product = canonical_product_id();
    let gateway: Arc<dyn RecommendationGateway> =
        Arc::new(FakeRecommendationGateway::empty_edges(product));
    let (addr, handle) = serve_recommendations(gateway).await;
    let (status, _, body) = http_get(addr, &rec_path(CANONICAL_ULID)).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 200);
    assert!(json["similar"].as_array().unwrap().is_empty());
    assert!(json["substitutes"].as_array().unwrap().is_empty());
    assert!(json["complements"].as_array().unwrap().is_empty());
    assert_conforms("GET", OPENAPI_PATH, 200, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_image_url_is_null() {
    let product = canonical_product_id();
    let gateway: Arc<dyn RecommendationGateway> = Arc::new(FakeRecommendationGateway::with_panel(
        product.clone(),
        three_blocks_panel(product.clone()),
    ));
    let (addr, handle) = serve_recommendations(gateway).await;
    let (_, _, body) = http_get(addr, &rec_path(CANONICAL_ULID)).await;
    handle.abort();
    let json = json_body(&body);
    assert!(json["similar"][0]["imageUrl"].is_null());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_json_is_english() {
    let product = canonical_product_id();
    let gateway: Arc<dyn RecommendationGateway> = Arc::new(FakeRecommendationGateway::with_panel(
        product.clone(),
        three_blocks_panel(product.clone()),
    ));
    let (addr, handle) = serve_recommendations(gateway).await;
    let (_, _, body) = http_get(addr, &rec_path(CANONICAL_ULID)).await;
    handle.abort();
    let json = json_body(&body);
    assert!(json.get("productId").is_some());
    assert!(json.get("highlightSubstitutes").is_some());
    assert!(json["similar"][0].get("name").is_some());
    assert!(json.get("nombre").is_none());
    assert!(json.get("destacar_sustitutos").is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_invalid_ulid_returns_400() {
    let product = canonical_product_id();
    let gateway: Arc<dyn RecommendationGateway> =
        Arc::new(FakeRecommendationGateway::empty_edges(product));
    let (addr, handle) = serve_recommendations(gateway).await;
    let (status, ctype, body) = http_get(addr, &rec_path("not-a-ulid")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 400);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(json["code"], "invalid_input");
    assert_eq!(json["errors"][0]["field"], "productId");
    assert_conforms("GET", OPENAPI_PATH, 400, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_unknown_product_returns_404() {
    let product = canonical_product_id();
    let gateway: Arc<dyn RecommendationGateway> =
        Arc::new(FakeRecommendationGateway::empty_edges(product));
    let (addr, handle) = serve_recommendations(gateway).await;
    let (status, ctype, body) = http_get(addr, &rec_path(UNKNOWN_ULID)).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 404);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(json["code"], "not_found");
    assert_conforms("GET", OPENAPI_PATH, 404, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_without_service_returns_503() {
    let (addr, handle) = serve(AppState::new()).await;
    let (status, _, body) = http_get(addr, &rec_path(CANONICAL_ULID)).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 503);
    assert_eq!(json["code"], "unavailable");
    assert_conforms("GET", OPENAPI_PATH, 503, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_unavailable_returns_503() {
    let gateway: Arc<dyn RecommendationGateway> =
        Arc::new(FakeRecommendationGateway::unavailable());
    let (addr, handle) = serve_recommendations(gateway).await;
    let (status, _, body) = http_get(addr, &rec_path(CANONICAL_ULID)).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 503);
    assert_eq!(json["code"], "unavailable");
    assert_conforms("GET", OPENAPI_PATH, 503, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_has_no_max_age_cache() {
    let product = canonical_product_id();
    let gateway: Arc<dyn RecommendationGateway> = Arc::new(FakeRecommendationGateway::with_panel(
        product.clone(),
        three_blocks_panel(product.clone()),
    ));
    let (addr, handle) = serve_recommendations(gateway).await;
    let resp = http_get_raw(addr, &rec_path(CANONICAL_ULID)).await;
    handle.abort();
    assert_eq!(resp.status, 200);
    let cc = resp
        .headers
        .get("cache-control")
        .map(String::as_str)
        .unwrap_or("");
    assert!(
        !cc.contains("max-age"),
        "cache-control must not use max-age: {cc}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_records_duration_histogram() {
    let _recorder = prometheus_handle();
    let product = canonical_product_id();
    let gateway: Arc<dyn RecommendationGateway> = Arc::new(FakeRecommendationGateway::with_panel(
        product.clone(),
        three_blocks_panel(product.clone()),
    ));
    let (addr, handle) = serve_recommendations(gateway).await;
    let _ = http_get(addr, &rec_path(CANONICAL_ULID)).await;
    let (status, _, metrics_body) = http_get(addr, "/metrics").await;
    handle.abort();
    assert_eq!(status, 200);
    assert!(metrics_body.contains("recommendations_duration_seconds"));
    assert!(metrics_body.contains("_bucket"));
    assert!(metrics_body.contains("_sum"));
    assert!(metrics_body.contains("_count"));
}
