mod support;

use api::{AppState, build_router};
use domain::Id;
use support::fake_catalog::{product_create_json_stock, test_stock_gateways};
use support::openapi::assert_conforms;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn http_send(
    addr: std::net::SocketAddr,
    method: &str,
    path: &str,
    body: Option<&str>,
    extra_headers: &[(&str, &str)],
) -> (u16, String, String) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let host = format!("{}:{}", addr.ip(), addr.port());
    let mut request = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n");
    for (k, v) in extra_headers {
        request.push_str(&format!("{k}: {v}\r\n"));
    }
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
                ctype = value.trim().split(';').next().unwrap_or("").to_string();
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

async fn setup_stock() -> (
    std::net::SocketAddr,
    tokio::task::JoinHandle<()>,
    support::fake_catalog::ProductTestSeeds,
) {
    let (products, stock, seeds) = test_stock_gateways().await;
    let state = AppState::new().with_products(products).with_stock(stock);
    let (addr, handle) = serve(state).await;
    (addr, handle, seeds)
}

async fn create_product_with_stock(
    addr: std::net::SocketAddr,
    seeds: &support::fake_catalog::ProductTestSeeds,
    sku: &str,
    stock: &str,
) -> String {
    let body = product_create_json_stock(seeds, sku, stock);
    let (s, _, resp) = http_send(addr, "POST", "/api/v1/products", Some(&body), &[]).await;
    assert_eq!(s, 201, "create product: {resp}");
    json_body(&resp)["id"]
        .as_str()
        .expect("product id")
        .to_owned()
}

fn stock_path(product_id: &str) -> String {
    format!("/api/v1/products/{product_id}/stock-movements")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_stock_inbound_20_on_10_returns_201_with_terminal() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-10", "10").await;
    let path = stock_path(&product_id);
    let body = r#"{"type":"inbound","quantity":"20"}"#;
    let (s, ct, resp) = http_send(
        addr,
        "POST",
        &path,
        Some(body),
        &[("X-Vectra-Terminal", "caja-1")],
    )
    .await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(s, 201);
    assert_eq!(ct, "application/json");
    assert_eq!(json["type"], "inbound");
    assert_eq!(json["quantity"], "20");
    assert_eq!(json["resultingStock"], "30");
    assert_eq!(json["terminal"], "caja-1");
    assert!(json.get("tipo").is_none());
    assert_conforms("POST", &path, 201, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_stock_adjustment_to_zero_returns_201() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-05", "5").await;
    let path = stock_path(&product_id);
    let body = r#"{"type":"adjustment","quantity":"-5","reason":"Conteo físico"}"#;
    let (s, ct, resp) = http_send(addr, "POST", &path, Some(body), &[]).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(s, 201);
    assert_eq!(ct, "application/json");
    assert_eq!(json["type"], "adjustment");
    assert_eq!(json["quantity"], "-5");
    assert_eq!(json["resultingStock"], "0");
    assert_eq!(json["reason"], "Conteo físico");
    assert_conforms("POST", &path, 201, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_stock_adjustment_without_reason_returns_400() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-NR", "5").await;
    let path = stock_path(&product_id);
    let body = r#"{"type":"adjustment","quantity":"-1"}"#;
    let (s, ct, resp) = http_send(addr, "POST", &path, Some(body), &[]).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(s, 400);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "invalid_input");
    assert_conforms("POST", &path, 400, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_stock_adjustment_minus_8_on_5_returns_400_negative_stock() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-NEG", "5").await;
    let path = stock_path(&product_id);
    let body = r#"{"type":"adjustment","quantity":"-8","reason":"Conteo físico"}"#;
    let (s, ct, resp) = http_send(addr, "POST", &path, Some(body), &[]).await;
    assert_eq!(s, 400);
    assert_eq!(ct, "application/problem+json");
    let json = json_body(&resp);
    assert_eq!(json["code"], "negative_stock");
    assert_eq!(json["title"], "El stock no puede ser negativo");
    assert_conforms("POST", &path, 400, "application/problem+json", &json);

    let get_path = format!("/api/v1/products/{product_id}");
    let (gs, _, gbody) = http_send(addr, "GET", &get_path, None, &[]).await;
    handle.abort();
    assert_eq!(gs, 200);
    assert_eq!(json_body(&gbody)["stock"], "5");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_stock_unknown_product_returns_404() {
    let (addr, handle, _) = setup_stock().await;
    let unknown = Id::<domain::Producto>::generate().to_string();
    let path = stock_path(&unknown);
    let body = r#"{"type":"inbound","quantity":"1"}"#;
    let (s, ct, resp) = http_send(addr, "POST", &path, Some(body), &[]).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(s, 404);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "not_found");
    assert_conforms("POST", &path, 404, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_stock_invalid_product_id_returns_400() {
    let (addr, handle, _) = setup_stock().await;
    let path = stock_path("not-a-ulid");
    let body = r#"{"type":"inbound","quantity":"1"}"#;
    let (s, _, resp) = http_send(addr, "POST", &path, Some(body), &[]).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|e| e.iter().any(|f| f["field"] == "productId"))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_stock_inbound_negative_quantity_returns_400() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-IN", "1").await;
    let path = stock_path(&product_id);
    let body = r#"{"type":"inbound","quantity":"-1"}"#;
    let (s, _, resp) = http_send(addr, "POST", &path, Some(body), &[]).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|e| e.iter().any(|f| f["field"] == "quantity"))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_stock_type_initial_returns_400() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-INIT", "1").await;
    let path = stock_path(&product_id);
    let body = r#"{"type":"initial","quantity":"1"}"#;
    let (s, _, resp) = http_send(addr, "POST", &path, Some(body), &[]).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_stock_type_sale_returns_400() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-SALE", "1").await;
    let path = stock_path(&product_id);
    let body = r#"{"type":"sale","quantity":"-1"}"#;
    let (s, _, resp) = http_send(addr, "POST", &path, Some(body), &[]).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_stock_quantity_as_json_number_returns_400() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-NUM", "1").await;
    let path = stock_path(&product_id);
    let body = r#"{"type":"inbound","quantity":20}"#;
    let (s, _, resp) = http_send(addr, "POST", &path, Some(body), &[]).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_stock_terminal_too_long_returns_400() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-TERM", "1").await;
    let path = stock_path(&product_id);
    let body = r#"{"type":"inbound","quantity":"1"}"#;
    let terminal = "x".repeat(65);
    let (s, _, resp) = http_send(
        addr,
        "POST",
        &path,
        Some(body),
        &[("X-Vectra-Terminal", &terminal)],
    )
    .await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
    assert!(
        json["errors"]
            .as_array()
            .is_some_and(|e| e.iter().any(|f| f["field"] == "X-Vectra-Terminal"))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_stock_without_service_returns_503() {
    let (products, _, seeds) = test_stock_gateways().await;
    let state = AppState::new().with_products(products);
    let (addr, handle) = serve(state).await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-503", "1").await;
    let path = stock_path(&product_id);
    let body = r#"{"type":"inbound","quantity":"1"}"#;
    let (s, ct, resp) = http_send(addr, "POST", &path, Some(body), &[]).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(s, 503);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "unavailable");
    assert_conforms("POST", &path, 503, "application/problem+json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_stock_movements_is_not_page() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-GET", "1").await;
    let path = stock_path(&product_id);
    let (s, ct, resp) = http_send(addr, "GET", &path, None, &[]).await;
    handle.abort();
    assert_ne!(s, 200);
    let json = json_body(&resp);
    assert_ne!(json.get("items").and_then(|i| i.as_array()), Some(&vec![]));
    if s == 200 {
        panic!("GET stock-movements must not return StockMovementPage");
    }
    let _ = ct;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_stock_movement_is_not_registered() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-PATCH", "1").await;
    let path = stock_path(&product_id);
    let (s, _, _) = http_send(addr, "PATCH", &path, Some(r#"{}"#), &[]).await;
    handle.abort();
    assert_ne!(s, 200);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_stock_movement_is_not_registered() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-DEL", "1").await;
    let mov = Id::<domain::MovimientoStock>::generate();
    let path = format!("/api/v1/products/{product_id}/stock-movements/{mov}");
    let (s, _, _) = http_send(addr, "DELETE", &path, None, &[]).await;
    handle.abort();
    assert_ne!(s, 204);
    assert_ne!(s, 200);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stock_problem_field_is_reason_not_motivo() {
    let (addr, handle, seeds) = setup_stock().await;
    let product_id = create_product_with_stock(addr, &seeds, "SKU-STK-FLD", "5").await;
    let path = stock_path(&product_id);
    let body = r#"{"type":"adjustment","quantity":"-1"}"#;
    let (s, _, resp) = http_send(addr, "POST", &path, Some(body), &[]).await;
    handle.abort();
    assert_eq!(s, 400);
    let json = json_body(&resp);
    let fields: Vec<String> = json["errors"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|e| e["field"].as_str().map(str::to_owned))
        .collect();
    assert!(fields.contains(&"reason".to_string()));
    assert!(!fields.iter().any(|f| f == "motivo"));
}
