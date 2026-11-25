mod support;

use std::sync::Arc;
use std::time::{Duration, Instant};

use api::{AppState, build_router, prometheus_handle};
use catalog::ProductGateway;
use domain::{Codigo, NewProducto, SearchOrden, TipoCodigo, UnidadMedida};
use search::SearchGateway;
use support::fake_catalog::test_product_gateway;
use support::fake_search::{FakeSearchGateway, test_search_gateway};
use support::openapi::assert_conforms;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const SEARCH_PATH: &str = "/api/v1/search";
const BY_CODE_OPENAPI_PATH: &str = "/api/v1/search/by-code/{code}";
const CANONICAL_ULID: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";
const BRAND_ULID: &str = "01J9ZK4Q6Y3M2V8W0XAB1CD2EF";

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

async fn http_get_abort(addr: std::net::SocketAddr, path: &str) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let host = format!("{}:{}", addr.ip(), addr.port());
    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\n\r\n");
    stream.write_all(request.as_bytes()).await.expect("write");
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

async fn serve_search(
    gateway: Arc<dyn SearchGateway>,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let state = AppState::new().with_search(gateway);
    serve(state).await
}

async fn serve_products(
    gateway: Arc<dyn ProductGateway>,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let state = AppState::new().with_products(gateway);
    serve(state).await
}

fn perno_with_fabricante(
    seeds: &support::fake_catalog::ProductTestSeeds,
    sku: &str,
    fabricante: &str,
) -> NewProducto {
    NewProducto {
        sku: sku.into(),
        nombre: format!("Producto {sku}"),
        tipo: "Perno".into(),
        descripcion: None,
        marca: Some(seeds.brand_id.clone()),
        categoria: seeds.category_id.clone(),
        atributos: vec![],
        codigos: vec![Codigo::try_new(TipoCodigo::Fabricante, fabricante).expect("codigo")],
        unidad_medida: UnidadMedida::Unidad,
        precio: "10".parse().expect("precio"),
        ubicacion: None,
        stock_inicial: "0".parse().expect("stock"),
        terminal: None,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_q_perno_returns_200() {
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let (status, ctype, body) = http_get(addr, &format!("{SEARCH_PATH}?q=perno")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 200);
    assert_eq!(ctype, "application/json");
    assert_eq!(json.get("query").and_then(|v| v.as_str()), Some("perno"));
    assert!(
        json.get("hits")
            .and_then(|v| v.as_array())
            .is_some_and(|a| !a.is_empty())
    );
    assert_eq!(json.get("degraded").and_then(|v| v.as_bool()), Some(false));
    assert_conforms("GET", SEARCH_PATH, 200, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_is_registered() {
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let (status, _, _) = http_get(addr, &format!("{SEARCH_PATH}?q=x")).await;
    handle.abort();
    assert_ne!(status, 404);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_empty_q_with_in_stock_returns_200() {
    let fake = FakeSearchGateway::perno_hit();
    let probe = fake.clone();
    let gateway: Arc<dyn SearchGateway> = Arc::new(fake);
    let (addr, handle) = serve_search(gateway).await;
    let (status, ctype, body) = http_get(addr, &format!("{SEARCH_PATH}?inStock=true")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 200);
    assert_conforms("GET", SEARCH_PATH, 200, "application/json", &json);
    let recorded = probe.take_last().expect("recorded");
    assert!(recorded.q.is_empty());
    assert_eq!(recorded.filtros.con_stock, Some(true));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_maps_attr_deep_object() {
    let fake = FakeSearchGateway::perno_hit();
    let probe = fake.clone();
    let gateway: Arc<dyn SearchGateway> = Arc::new(fake);
    let (addr, handle) = serve_search(gateway).await;
    let path = format!("{SEARCH_PATH}?q=perno&attr[medida]=M8&attr[grado]=5");
    let (status, ctype, body) = http_get(addr, &path).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 200);
    assert_conforms("GET", SEARCH_PATH, 200, "application/json", &json);
    let recorded = probe.take_last().expect("recorded");
    assert_eq!(
        recorded.filtros.atributos.get("medida").map(String::as_str),
        Some("M8")
    );
    assert_eq!(
        recorded.filtros.atributos.get("grado").map(String::as_str),
        Some("5")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_maps_category_brand_in_stock() {
    let fake = FakeSearchGateway::perno_hit();
    let probe = fake.clone();
    let gateway: Arc<dyn SearchGateway> = Arc::new(fake);
    let (addr, handle) = serve_search(gateway).await;
    let path = format!(
        "{SEARCH_PATH}?q=perno&categoryId={CANONICAL_ULID}&brandId={BRAND_ULID}&inStock=false"
    );
    let (status, ctype, body) = http_get(addr, &path).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 200);
    assert_conforms("GET", SEARCH_PATH, 200, "application/json", &json);
    let recorded = probe.take_last().expect("recorded");
    assert_eq!(recorded.filtros.con_stock, Some(false));
    assert_eq!(recorded.filtros.categoria_ids.len(), 1);
    assert!(recorded.filtros.marca_id.is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_maps_sort_price_asc() {
    let fake = FakeSearchGateway::perno_hit();
    let probe = fake.clone();
    let gateway: Arc<dyn SearchGateway> = Arc::new(fake);
    let (addr, handle) = serve_search(gateway).await;
    let (status, ctype, body) =
        http_get(addr, &format!("{SEARCH_PATH}?q=perno&sort=price_asc")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 200);
    assert_conforms("GET", SEARCH_PATH, 200, "application/json", &json);
    let recorded = probe.take_last().expect("recorded");
    assert_eq!(recorded.orden, SearchOrden::PrecioAsc);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_highlight_from_formatted() {
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let (status, ctype, body) = http_get(addr, &format!("{SEARCH_PATH}?q=perno")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 200);
    assert_conforms("GET", SEARCH_PATH, 200, "application/json", &json);
    let highlight = json
        .get("hits")
        .and_then(|h| h.as_array())
        .and_then(|a| a.first())
        .and_then(|h| h.get("highlight"));
    assert!(highlight.is_some_and(|h| !h.is_null()));
    let name = highlight
        .and_then(|h| h.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");
    assert!(name.contains("<mark>"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_facets_true_returns_null_facets() {
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let (status, ctype, body) = http_get(addr, &format!("{SEARCH_PATH}?q=perno&facets=true")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 200);
    assert_conforms("GET", SEARCH_PATH, 200, "application/json", &json);
    assert!(
        json.get("facets").is_none_or(|v| v.is_null()),
        "facets must be null or absent"
    );
    assert_eq!(json.get("degraded").and_then(|v| v.as_bool()), Some(false));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_q_too_long_returns_400() {
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let q = "x".repeat(201);
    let (status, ctype, body) = http_get(addr, &format!("{SEARCH_PATH}?q={q}")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 400);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(
        json.get("code").and_then(|c| c.as_str()),
        Some("invalid_input")
    );
    assert!(
        json.get("errors")
            .and_then(|e| e.as_array())
            .is_some_and(|arr| arr
                .iter()
                .any(|e| e.get("field").and_then(|f| f.as_str()) == Some("q")))
    );
    assert_conforms("GET", SEARCH_PATH, 400, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_offset_1001_returns_400() {
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let (status, ctype, body) = http_get(addr, &format!("{SEARCH_PATH}?offset=1001")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 400);
    assert_conforms("GET", SEARCH_PATH, 400, "application/problem+json", &json);
    assert!(
        json.get("errors")
            .and_then(|e| e.as_array())
            .is_some_and(|arr| arr
                .iter()
                .any(|e| e.get("field").and_then(|f| f.as_str()) == Some("offset")))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_limit_zero_returns_400() {
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let (status, _, body) = http_get(addr, &format!("{SEARCH_PATH}?limit=0")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 400);
    assert_conforms("GET", SEARCH_PATH, 400, "application/problem+json", &json);
    assert!(
        json.get("errors")
            .and_then(|e| e.as_array())
            .is_some_and(|arr| arr
                .iter()
                .any(|e| e.get("field").and_then(|f| f.as_str()) == Some("limit")))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_limit_51_returns_400() {
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let (status, _, body) = http_get(addr, &format!("{SEARCH_PATH}?limit=51")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 400);
    assert_conforms("GET", SEARCH_PATH, 400, "application/problem+json", &json);
    assert!(
        json.get("errors")
            .and_then(|e| e.as_array())
            .is_some_and(|arr| arr
                .iter()
                .any(|e| e.get("field").and_then(|f| f.as_str()) == Some("limit")))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_invalid_category_id_returns_400() {
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let (status, _, body) = http_get(addr, &format!("{SEARCH_PATH}?categoryId=no-ulid")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 400);
    assert_conforms("GET", SEARCH_PATH, 400, "application/problem+json", &json);
    assert!(
        json.get("errors")
            .and_then(|e| e.as_array())
            .is_some_and(|arr| arr
                .iter()
                .any(|e| e.get("field").and_then(|f| f.as_str()) == Some("categoryId")))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_invalid_brand_id_returns_400() {
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let (status, _, body) = http_get(addr, &format!("{SEARCH_PATH}?brandId=no-ulid")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 400);
    assert_conforms("GET", SEARCH_PATH, 400, "application/problem+json", &json);
    assert!(
        json.get("errors")
            .and_then(|e| e.as_array())
            .is_some_and(|arr| arr
                .iter()
                .any(|e| e.get("field").and_then(|f| f.as_str()) == Some("brandId")))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_invalid_sort_returns_400() {
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let (status, _, body) = http_get(addr, &format!("{SEARCH_PATH}?sort=nombre")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 400);
    assert_conforms("GET", SEARCH_PATH, 400, "application/problem+json", &json);
    assert!(
        json.get("errors")
            .and_then(|e| e.as_array())
            .is_some_and(|arr| arr
                .iter()
                .any(|e| e.get("field").and_then(|f| f.as_str()) == Some("sort")))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_attr_prefix_clave_returns_400() {
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let (status, _, body) = http_get(addr, &format!("{SEARCH_PATH}?attr[attr_medida]=M8")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 400);
    assert_conforms("GET", SEARCH_PATH, 400, "application/problem+json", &json);
    assert_eq!(
        json.get("code").and_then(|c| c.as_str()),
        Some("invalid_input")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_unavailable_returns_503() {
    let gateway: Arc<dyn SearchGateway> = Arc::new(FakeSearchGateway::unavailable());
    let (addr, handle) = serve_search(gateway).await;
    let (status, ctype, body) = http_get(addr, &format!("{SEARCH_PATH}?q=perno")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 503);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(
        json.get("code").and_then(|c| c.as_str()),
        Some("unavailable")
    );
    assert!(json.get("degraded").is_none());
    assert_conforms("GET", SEARCH_PATH, 503, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_without_service_returns_503() {
    let (addr, handle) = serve(AppState::new()).await;
    let (status, ctype, body) = http_get(addr, &format!("{SEARCH_PATH}?q=perno")).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 503);
    assert_eq!(ctype, "application/problem+json");
    assert_conforms("GET", SEARCH_PATH, 503, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_timeout_returns_504() {
    let gateway: Arc<dyn SearchGateway> = Arc::new(FakeSearchGateway::slow());
    let (addr, handle) = serve_search(gateway).await;
    let start = Instant::now();
    let (status, ctype, body) = http_get(addr, &format!("{SEARCH_PATH}?q=perno")).await;
    handle.abort();
    let elapsed = start.elapsed();
    let json = json_body(&body);
    assert_eq!(status, 504);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(json.get("code").and_then(|c| c.as_str()), Some("timeout"));
    assert!(
        elapsed < Duration::from_secs(5),
        "must beat global 5s timeout"
    );
    assert!(elapsed >= Duration::from_millis(1500));
    assert_conforms("GET", SEARCH_PATH, 504, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_client_abort_drops_work() {
    let gw = FakeSearchGateway::hang_until_drop();
    let probe = gw.clone();
    let gateway: Arc<dyn SearchGateway> = Arc::new(gw);
    let (addr, handle) = serve_search(gateway).await;
    http_get_abort(addr, &format!("{SEARCH_PATH}?q=perno")).await;
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if probe.was_dropped() {
            handle.abort();
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    handle.abort();
    panic!("search future was not dropped after client abort");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_records_duration_histogram() {
    let _recorder = prometheus_handle();
    let gateway = test_search_gateway();
    let (addr, handle) = serve_search(gateway).await;
    let _ = http_get(addr, &format!("{SEARCH_PATH}?q=perno")).await;
    let (status, _, metrics_body) = http_get(addr, "/metrics").await;
    handle.abort();
    assert_eq!(status, 200);
    assert!(metrics_body.contains("search_duration_seconds"));
    assert!(metrics_body.contains("_bucket"));
    assert!(metrics_body.contains("_sum"));
    assert!(metrics_body.contains("_count"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_by_code_returns_product_summaries() {
    let (gateway, _, seeds) = test_product_gateway().await;
    gateway
        .create(perno_with_fabricante(&seeds, "SKU-FAB", "HX-0840Z"))
        .await
        .expect("create");
    let products: Arc<dyn ProductGateway> = gateway;
    let (addr, handle) = serve_products(products).await;
    let path = "/api/v1/search/by-code/HX-0840Z";
    let (status, ctype, body) = http_get(addr, path).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 200);
    assert_eq!(ctype, "application/json");
    assert_eq!(
        json.get("items").and_then(|v| v.as_array()).map(Vec::len),
        Some(1)
    );
    assert_eq!(
        json.get("items")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|o| o.get("sku"))
            .and_then(|v| v.as_str()),
        Some("SKU-FAB")
    );
    assert_conforms("GET", BY_CODE_OPENAPI_PATH, 200, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_by_code_works_without_search_gateway() {
    let (gateway, _, seeds) = test_product_gateway().await;
    gateway
        .create(perno_with_fabricante(&seeds, "SKU-2", "CODE-2"))
        .await
        .expect("create");
    let (addr, handle) = serve_products(gateway).await;
    let (status, _, body) = http_get(addr, "/api/v1/search/by-code/CODE-2").await;
    handle.abort();
    assert_eq!(status, 200);
    assert_eq!(
        json_body(&body)
            .get("items")
            .and_then(|v| v.as_array())
            .map(Vec::len),
        Some(1)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_by_code_does_not_prefix_match() {
    let (gateway, _, seeds) = test_product_gateway().await;
    gateway
        .create(perno_with_fabricante(&seeds, "SKU-FAB", "HX-0840Z"))
        .await
        .expect("create");
    let (addr, handle) = serve_products(gateway).await;
    let (status, _, body) = http_get(addr, "/api/v1/search/by-code/HX-0841Z").await;
    handle.abort();
    assert_eq!(status, 200);
    assert!(
        json_body(&body)
            .get("items")
            .and_then(|v| v.as_array())
            .is_some_and(|a| a.is_empty())
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_by_code_rejects_code_longer_than_64() {
    let (gateway, _, _) = test_product_gateway().await;
    let (addr, handle) = serve_products(gateway).await;
    let long_code = "x".repeat(65);
    let path = format!("/api/v1/search/by-code/{long_code}");
    let (status, ctype, body) = http_get(addr, &path).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 400);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(
        json.get("code").and_then(|c| c.as_str()),
        Some("invalid_input")
    );
    assert!(
        json.get("errors")
            .and_then(|e| e.as_array())
            .and_then(|a| a.first())
            .and_then(|o| o.get("field"))
            .and_then(|f| f.as_str())
            == Some("code")
    );
    assert_conforms(
        "GET",
        BY_CODE_OPENAPI_PATH,
        400,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_search_by_code_without_catalog_is_503() {
    let (addr, handle) = serve(AppState::new()).await;
    let (status, ctype, body) = http_get(addr, "/api/v1/search/by-code/X").await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(status, 503);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(
        json.get("code").and_then(|c| c.as_str()),
        Some("unavailable")
    );
    assert_conforms(
        "GET",
        BY_CODE_OPENAPI_PATH,
        503,
        "application/problem+json",
        &json,
    );
}
