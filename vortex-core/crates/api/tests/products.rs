mod support;

use api::{AppState, build_router};
use domain::{CategoryRepo, Id, NewCategoria};
use support::fake_catalog::{perno_create_json, product_create_json, test_product_gateway};
use support::openapi::assert_conforms;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn http_send(
    addr: std::net::SocketAddr,
    method: &str,
    path: &str,
    body: Option<&str>,
    extra_headers: &[(&str, &str)],
) -> (u16, String, String, String) {
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
    let mut location = String::new();
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
            let name = name.trim();
            let value = value.trim();
            if name.eq_ignore_ascii_case("content-type") {
                ctype = value.split(';').next().unwrap_or(value).trim().to_string();
            } else if name.eq_ignore_ascii_case("location") {
                location = value.to_string();
            }
        }
    }
    let body = String::from_utf8_lossy(&buf[body_start..n]).into_owned();
    (status, ctype, body, location)
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

async fn setup_gateway() -> (
    std::net::SocketAddr,
    tokio::task::JoinHandle<()>,
    support::fake_catalog::ProductTestSeeds,
    std::sync::Arc<std::sync::Mutex<Option<domain::NewProducto>>>,
) {
    let (gateway, last_create, seeds) = test_product_gateway().await;
    let state = AppState::new().with_products(gateway);
    let (addr, handle) = serve(state).await;
    (addr, handle, seeds, last_create)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_product_perno_m8x40_returns_201_with_location() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let json_body_str = perno_create_json(&seeds);
    let (s, ct, body, location) =
        http_send(addr, "POST", "/api/v1/products", Some(&json_body_str), &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 201);
    assert_eq!(ct, "application/json");
    let id = json["id"].as_str().expect("id");
    assert_eq!(location, format!("/api/v1/products/{id}"));
    assert_eq!(json["sku"], "PER-M8X40-ZN");
    assert_eq!(json["stock"], "50");
    assert_eq!(json["price"], "120.5");
    assert!(json.get("nombre").is_none());
    assert_eq!(json["images"], serde_json::json!([]));
    assert_conforms("POST", "/api/v1/products", 201, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_product_sends_terminal_on_initial_stock() {
    let (addr, handle, seeds, last_create) = setup_gateway().await;
    let json_body_str = perno_create_json(&seeds);
    let (s, _, _, _) = http_send(
        addr,
        "POST",
        "/api/v1/products",
        Some(&json_body_str),
        &[("X-Vectra-Terminal", "caja-1")],
    )
    .await;
    handle.abort();
    assert_eq!(s, 201);
    let terminal = last_create
        .lock()
        .expect("lock")
        .as_ref()
        .and_then(|n| n.terminal.clone());
    assert_eq!(terminal.as_deref(), Some("caja-1"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_product_returns_200() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let json_body_str = perno_create_json(&seeds);
    let (_, _, body, _) =
        http_send(addr, "POST", "/api/v1/products", Some(&json_body_str), &[]).await;
    let created = json_body(&body);
    let id = created["id"].as_str().unwrap();
    let path = format!("/api/v1/products/{id}");
    let (s, ct, body, _) = http_send(addr, "GET", &path, None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 200);
    assert_eq!(ct, "application/json");
    assert_conforms(
        "GET",
        "/api/v1/products/{productId}",
        200,
        "application/json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_product_price_increments_version() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let json_body_str = perno_create_json(&seeds);
    let (_, _, body, _) =
        http_send(addr, "POST", "/api/v1/products", Some(&json_body_str), &[]).await;
    let created = json_body(&body);
    let id = created["id"].as_str().unwrap();
    let patch = r#"{"version":1,"price":"130.00"}"#;
    let path = format!("/api/v1/products/{id}");
    let (s, ct, body, _) = http_send(addr, "PATCH", &path, Some(patch), &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 200);
    assert_eq!(json["price"], "130");
    assert_eq!(json["version"], 2);
    assert_eq!(json["stock"], "50");
    assert_conforms(
        "PATCH",
        "/api/v1/products/{productId}",
        200,
        "application/json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_product_partial_keeps_other_fields() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let json_body_str = perno_create_json(&seeds);
    let (_, _, body, _) =
        http_send(addr, "POST", "/api/v1/products", Some(&json_body_str), &[]).await;
    let created = json_body(&body);
    let id = created["id"].as_str().unwrap();
    let name = created["name"].as_str().unwrap();
    let sku = created["sku"].as_str().unwrap();
    let path = format!("/api/v1/products/{id}");
    let (s, _, body, _) = http_send(
        addr,
        "PATCH",
        &path,
        Some(r#"{"version":1,"price":"99.00"}"#),
        &[],
    )
    .await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 200);
    assert_eq!(json["name"], name);
    assert_eq!(json["sku"], sku);
    assert_eq!(json["version"], 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_product_duplicate_sku_returns_409() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let json_body_str = perno_create_json(&seeds);
    http_send(addr, "POST", "/api/v1/products", Some(&json_body_str), &[]).await;
    let (s, ct, body, _) =
        http_send(addr, "POST", "/api/v1/products", Some(&json_body_str), &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 409);
    assert_eq!(json["code"], "duplicate_sku");
    assert_conforms(
        "POST",
        "/api/v1/products",
        409,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_product_cuarenta_on_number_attribute_returns_400() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let mut json_str = perno_create_json(&seeds);
    json_str = json_str.replace(r#""value": 40"#, r#""value": "cuarenta""#);
    let (s, ct, body, _) = http_send(addr, "POST", "/api/v1/products", Some(&json_str), &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(
        fields
            .iter()
            .any(|f| f.contains("attributes") && f.contains("value"))
    );
    assert_conforms(
        "POST",
        "/api/v1/products",
        400,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_product_stale_version_returns_409() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let json_body_str = perno_create_json(&seeds);
    let (_, _, body, _) =
        http_send(addr, "POST", "/api/v1/products", Some(&json_body_str), &[]).await;
    let created = json_body(&body);
    let id = created["id"].as_str().unwrap();
    let path = format!("/api/v1/products/{id}");
    http_send(
        addr,
        "PATCH",
        &path,
        Some(r#"{"version":1,"price":"130.00"}"#),
        &[],
    )
    .await;
    let (s, ct, body, _) = http_send(
        addr,
        "PATCH",
        &path,
        Some(r#"{"version":1,"price":"140.00"}"#),
        &[],
    )
    .await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 409);
    assert_eq!(json["code"], "version_conflict");
    assert_conforms(
        "PATCH",
        "/api/v1/products/{productId}",
        409,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_product_unknown_id_returns_404() {
    let (addr, handle, _, _) = setup_gateway().await;
    let id = Id::<domain::Producto>::generate();
    let path = format!("/api/v1/products/{id}");
    let (s, ct, body, _) = http_send(addr, "GET", &path, None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 404);
    assert_eq!(json["code"], "not_found");
    assert_conforms(
        "GET",
        "/api/v1/products/{productId}",
        404,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_product_unknown_id_returns_404() {
    let (addr, handle, _, _) = setup_gateway().await;
    let id = Id::<domain::Producto>::generate();
    let path = format!("/api/v1/products/{id}");
    let (s, _, body, _) = http_send(
        addr,
        "PATCH",
        &path,
        Some(r#"{"version":1,"price":"1.00"}"#),
        &[],
    )
    .await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 404);
    assert_eq!(json["code"], "not_found");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn product_id_invalid_ulid_returns_400() {
    let (addr, handle, _, _) = setup_gateway().await;
    let (s, _, body, _) = http_send(addr, "GET", "/api/v1/products/not-a-ulid", None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"productId"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_product_missing_name_returns_400() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let mut json_str = perno_create_json(&seeds);
    json_str = json_str.replace(r#""name": "Perno hexagonal M8 x 40 zincado","#, "");
    let (s, _, body, _) = http_send(addr, "POST", "/api/v1/products", Some(&json_str), &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_product_price_as_json_number_returns_400() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let json_str = perno_create_json(&seeds).replace(r#""price": "120.50""#, r#""price": 120.5"#);
    let (s, _, body, _) = http_send(addr, "POST", "/api/v1/products", Some(&json_str), &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
}

async fn create_perno_id(
    addr: std::net::SocketAddr,
    seeds: &support::fake_catalog::ProductTestSeeds,
) -> String {
    let (_, _, body, _) = http_send(
        addr,
        "POST",
        "/api/v1/products",
        Some(&perno_create_json(seeds)),
        &[],
    )
    .await;
    json_body(&body)["id"].as_str().unwrap().to_string()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_product_returns_204() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let id = create_perno_id(addr, &seeds).await;
    let path = format!("/api/v1/products/{id}");
    let (s, _, body, _) = http_send(addr, "DELETE", &path, None, &[]).await;
    assert_eq!(s, 204);
    assert!(body.is_empty());
    let (s2, _, body2, _) = http_send(addr, "GET", &path, None, &[]).await;
    handle.abort();
    let json = json_body(&body2);
    assert_eq!(s2, 200);
    assert_eq!(json["active"], false);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_product_active_true_reactivates() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let id = create_perno_id(addr, &seeds).await;
    let path = format!("/api/v1/products/{id}");
    let (s, _, _, _) = http_send(addr, "DELETE", &path, None, &[]).await;
    assert_eq!(s, 204);
    let (s, ct, body, _) = http_send(
        addr,
        "PATCH",
        &path,
        Some(r#"{"version":1,"active":true}"#),
        &[],
    )
    .await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 200);
    assert_eq!(json["active"], true);
    assert_eq!(json["version"], 1);
    assert_conforms(
        "PATCH",
        "/api/v1/products/{productId}",
        200,
        "application/json",
        &json,
    );
    let _ = ct;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_product_active_false_deactivates() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let id = create_perno_id(addr, &seeds).await;
    let path = format!("/api/v1/products/{id}");
    let (s, _, body, _) = http_send(
        addr,
        "PATCH",
        &path,
        Some(r#"{"version":1,"active":false}"#),
        &[],
    )
    .await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 200);
    assert_eq!(json["active"], false);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_inactive_product_returns_200() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let id = create_perno_id(addr, &seeds).await;
    let path = format!("/api/v1/products/{id}");
    let (s, _, _, _) = http_send(addr, "DELETE", &path, None, &[]).await;
    assert_eq!(s, 204);
    let (s, ct, body, _) = http_send(addr, "GET", &path, None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 200);
    assert_eq!(json["active"], false);
    assert_conforms(
        "GET",
        "/api/v1/products/{productId}",
        200,
        "application/json",
        &json,
    );
    let _ = ct;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_product_unknown_id_returns_404() {
    let (addr, handle, _, _) = setup_gateway().await;
    let unknown = "01J9ZK4Q6Y3M2V8W0XAB1CD2EF";
    let path = format!("/api/v1/products/{unknown}");
    let (s, ct, body, _) = http_send(addr, "DELETE", &path, None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 404);
    assert_eq!(json["code"], "not_found");
    assert_conforms(
        "DELETE",
        "/api/v1/products/{productId}",
        404,
        "application/problem+json",
        &json,
    );
    let _ = ct;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_product_invalid_ulid_returns_400() {
    let (addr, handle, _, _) = setup_gateway().await;
    let (s, _, body, _) = http_send(addr, "DELETE", "/api/v1/products/not-a-ulid", None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"productId"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_product_without_service_returns_503() {
    let state = AppState::new();
    let (addr, handle) = serve(state).await;
    let unknown = "01J9ZK4Q6Y3M2V8W0XAB1CD2EF";
    let path = format!("/api/v1/products/{unknown}");
    let (s, ct, body, _) = http_send(addr, "DELETE", &path, None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 503);
    assert_eq!(json["code"], "unavailable");
    assert_conforms(
        "DELETE",
        "/api/v1/products/{productId}",
        503,
        "application/problem+json",
        &json,
    );
    let _ = ct;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_product_active_unknown_id_returns_404() {
    let (addr, handle, _, _) = setup_gateway().await;
    let unknown = "01J9ZK4Q6Y3M2V8W0XAB1CD2EF";
    let path = format!("/api/v1/products/{unknown}");
    let (s, _, body, _) = http_send(
        addr,
        "PATCH",
        &path,
        Some(r#"{"version":1,"active":true}"#),
        &[],
    )
    .await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 404);
    assert_eq!(json["code"], "not_found");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_product_terminal_too_long_returns_400() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let term = "x".repeat(65);
    let (s, _, body, _) = http_send(
        addr,
        "POST",
        "/api/v1/products",
        Some(&perno_create_json(&seeds)),
        &[("X-Vectra-Terminal", &term)],
    )
    .await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"X-Vectra-Terminal"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_product_without_service_returns_503() {
    let state = AppState::new();
    let (addr, handle) = serve(state).await;
    let minimal = r#"{"sku":"X","name":"Y","productType":"Perno","categoryId":"01J9ZK3T2N5R7P9Q1S3V5W7Y9A","price":"1"}"#;
    let (s, ct, body, _) = http_send(addr, "POST", "/api/v1/products", Some(minimal), &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 503);
    assert_eq!(json["code"], "unavailable");
    assert_conforms(
        "POST",
        "/api/v1/products",
        503,
        "application/problem+json",
        &json,
    );
}

async fn post_product_raw(addr: std::net::SocketAddr, body: &str) -> (u16, serde_json::Value) {
    let (s, _, resp, _) = http_send(addr, "POST", "/api/v1/products", Some(body), &[]).await;
    (s, json_body(&resp))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_branch_filter_includes_descendants() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let other = seeds
        .categories
        .create(NewCategoria {
            nombre: "Tornillería".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("other cat");
    let (s1, _) = post_product_raw(
        addr,
        &product_create_json(&seeds, "SKU-FIJ", "En Fijaciones", &seeds.fijaciones_id),
    )
    .await;
    assert_eq!(s1, 201);
    let (s2, _) = post_product_raw(
        addr,
        &product_create_json(&seeds, "SKU-PER", "En Pernos", &seeds.category_id),
    )
    .await;
    assert_eq!(s2, 201);
    let (s3, _) = post_product_raw(
        addr,
        &product_create_json(&seeds, "SKU-OTH", "Fuera", &other.id),
    )
    .await;
    assert_eq!(s3, 201);
    let path = format!("/api/v1/products?categoryId={}", seeds.fijaciones_id);
    let (s, ct, body, _) = http_send(addr, "GET", &path, None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 200);
    assert_eq!(ct, "application/json");
    let items = json["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|i| i["categoryPath"][0] == "Fijaciones"));
    let skus: Vec<_> = items.iter().filter_map(|i| i["sku"].as_str()).collect();
    assert!(skus.contains(&"SKU-FIJ"));
    assert!(skus.contains(&"SKU-PER"));
    assert!(!skus.contains(&"SKU-OTH"));
    assert_conforms("GET", "/api/v1/products", 200, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_active_false_returns_inactive_only() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let (s1, j1) = post_product_raw(addr, &perno_create_json(&seeds)).await;
    assert_eq!(s1, 201);
    let active_id = j1["id"].as_str().expect("id");
    let (s2, j2) = post_product_raw(
        addr,
        &product_create_json(&seeds, "SKU-INACT", "Inactivo", &seeds.category_id),
    )
    .await;
    assert_eq!(s2, 201);
    let inactive_id = j2["id"].as_str().expect("id");
    let del_path = format!("/api/v1/products/{inactive_id}");
    let (s_del, _, _, _) = http_send(addr, "DELETE", &del_path, None, &[]).await;
    assert_eq!(s_del, 204);
    let (s, ct, body, _) = http_send(addr, "GET", "/api/v1/products?active=false", None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 200);
    assert_eq!(ct, "application/json");
    let items = json["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], inactive_id);
    assert_eq!(items[0]["active"], false);
    assert_ne!(items[0]["id"].as_str(), Some(active_id));
    assert_conforms("GET", "/api/v1/products", 200, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_default_excludes_inactive() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let (s1, j1) = post_product_raw(addr, &perno_create_json(&seeds)).await;
    assert_eq!(s1, 201);
    let active_id = j1["id"].as_str().expect("id");
    let (s2, j2) = post_product_raw(
        addr,
        &product_create_json(&seeds, "SKU-INACT", "Inactivo", &seeds.category_id),
    )
    .await;
    assert_eq!(s2, 201);
    let inactive_id = j2["id"].as_str().expect("id");
    let del_path = format!("/api/v1/products/{inactive_id}");
    let (s_del, _, _, _) = http_send(addr, "DELETE", &del_path, None, &[]).await;
    assert_eq!(s_del, 204);
    let (s, _, body, _) = http_send(addr, "GET", "/api/v1/products", None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 200);
    let items = json["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], active_id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_filters_by_brand() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let (s1, _) = post_product_raw(addr, &perno_create_json(&seeds)).await;
    assert_eq!(s1, 201);
    let path = format!("/api/v1/products?brandId={}", seeds.brand_id);
    let (s, _, body, _) = http_send(addr, "GET", &path, None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 200);
    let items = json["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["brand"], "Fixser");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_paginates_by_cursor() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let (s1, j1) = post_product_raw(
        addr,
        &product_create_json(&seeds, "SKU-A", "A", &seeds.category_id),
    )
    .await;
    assert_eq!(s1, 201);
    let id_a = j1["id"].as_str().expect("a").to_string();
    let (s2, j2) = post_product_raw(
        addr,
        &product_create_json(&seeds, "SKU-B", "B", &seeds.category_id),
    )
    .await;
    assert_eq!(s2, 201);
    let id_b = j2["id"].as_str().expect("b").to_string();
    let (s3, j3) = post_product_raw(
        addr,
        &product_create_json(&seeds, "SKU-C", "C", &seeds.category_id),
    )
    .await;
    assert_eq!(s3, 201);
    let id_c = j3["id"].as_str().expect("c").to_string();
    let (s, _, body, _) = http_send(addr, "GET", "/api/v1/products?limit=2", None, &[]).await;
    let json = json_body(&body);
    assert_eq!(s, 200);
    let items = json["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    let id0 = items[0]["id"].as_str().expect("id0");
    let id1 = items[1]["id"].as_str().expect("id1");
    assert!(id0 < id1, "items must be ordered by id ascending");
    let cursor = json["nextCursor"].as_str().expect("cursor");
    let path = format!("/api/v1/products?limit=2&cursor={cursor}");
    let (s2, _, body2, _) = http_send(addr, "GET", &path, None, &[]).await;
    handle.abort();
    let json2 = json_body(&body2);
    assert_eq!(s2, 200);
    let page2 = json2["items"].as_array().expect("items");
    assert_eq!(page2.len(), 1);
    assert!(json2["nextCursor"].is_null());
    let mut seen: std::collections::HashSet<_> = [id0, id1, page2[0]["id"].as_str().unwrap()]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert_eq!(seen.len(), 3);
    for id in [id_a, id_b, id_c] {
        assert!(seen.remove(&id));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_empty_page() {
    let (addr, handle, _, _) = setup_gateway().await;
    let (s, ct, body, _) = http_send(addr, "GET", "/api/v1/products", None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 200);
    assert_eq!(ct, "application/json");
    assert_eq!(json["items"], serde_json::json!([]));
    assert!(json["nextCursor"].is_null());
    assert_conforms("GET", "/api/v1/products", 200, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_summary_has_english_dto() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let (s, _) = post_product_raw(addr, &perno_create_json(&seeds)).await;
    assert_eq!(s, 201);
    let (s, _, body, _) = http_send(addr, "GET", "/api/v1/products", None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 200);
    let item = &json["items"][0];
    assert!(item.get("name").is_some());
    assert!(item.get("productType").is_some());
    assert!(item["price"].is_string());
    assert!(item["stock"].is_string());
    assert!(item.get("active").is_some());
    assert!(item["imageUrl"].is_null());
    assert!(item.get("nombre").is_none());
    assert!(item.get("tipo").is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_invalid_cursor_returns_400() {
    let (addr, handle, _, _) = setup_gateway().await;
    let (s, ct, body, _) = http_send(addr, "GET", "/api/v1/products?cursor=%%%", None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 400);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "invalid_input");
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"cursor"));
    assert_conforms(
        "GET",
        "/api/v1/products",
        400,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_limit_zero_returns_400() {
    let (addr, handle, _, _) = setup_gateway().await;
    let (s, _, body, _) = http_send(addr, "GET", "/api/v1/products?limit=0", None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 400);
    assert_eq!(json["code"], "invalid_input");
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"limit"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_limit_101_returns_400() {
    let (addr, handle, _, _) = setup_gateway().await;
    let (s, _, body, _) = http_send(addr, "GET", "/api/v1/products?limit=101", None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 400);
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"limit"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_invalid_category_id_returns_400() {
    let (addr, handle, _, _) = setup_gateway().await;
    let (s, _, body, _) = http_send(
        addr,
        "GET",
        "/api/v1/products?categoryId=not-a-ulid",
        None,
        &[],
    )
    .await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 400);
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"categoryId"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_invalid_brand_id_returns_400() {
    let (addr, handle, _, _) = setup_gateway().await;
    let (s, _, body, _) = http_send(
        addr,
        "GET",
        "/api/v1/products?brandId=not-a-ulid",
        None,
        &[],
    )
    .await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 400);
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"brandId"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_q_does_not_text_search() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    let (s1, _) = post_product_raw(
        addr,
        &product_create_json(&seeds, "SKU-X", "Tuerca", &seeds.category_id),
    )
    .await;
    assert_eq!(s1, 201);
    let (s2, _, body, _) = http_send(addr, "GET", "/api/v1/products", None, &[]).await;
    let base = json_body(&body);
    let (s3, _, body_q, _) = http_send(addr, "GET", "/api/v1/products?q=perno", None, &[]).await;
    handle.abort();
    assert_eq!(s2, 200);
    assert_eq!(s3, 200);
    assert_eq!(base["items"], json_body(&body_q)["items"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_products_without_service_returns_503() {
    let state = AppState::new();
    let (addr, handle) = serve(state).await;
    let (s, ct, body, _) = http_send(addr, "GET", "/api/v1/products", None, &[]).await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 503);
    assert_eq!(ct, "application/problem+json");
    assert_eq!(json["code"], "unavailable");
    assert_conforms(
        "GET",
        "/api/v1/products",
        503,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn product_problem_field_is_category_id_not_categoria() {
    let (addr, handle, seeds, _) = setup_gateway().await;
    seeds
        .categories
        .set_activo(seeds.category_id.clone(), false)
        .await
        .expect("deactivate");
    let (s, _, body, _) = http_send(
        addr,
        "POST",
        "/api/v1/products",
        Some(&perno_create_json(&seeds)),
        &[],
    )
    .await;
    handle.abort();
    let json = json_body(&body);
    assert_eq!(s, 400);
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"categoryId"));
    assert!(!fields.contains(&"categoria"));
}
