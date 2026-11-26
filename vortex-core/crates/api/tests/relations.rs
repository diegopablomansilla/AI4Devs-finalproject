mod support;

use std::sync::Arc;

use api::{AppState, build_router};
use domain::Id;
use domain::NewRelacion;
use domain::Producto;
use domain::Relacion;
use domain::TipoRelacion;
use support::fake_catalog::{perno_create_json, test_product_gateway};
use support::fake_relations::{
    FakeRelationGateway, PernoArandelaFixtures, empty_known_gateway, pair_gateway,
    perno_arandela_gateway, resumen,
};
use support::openapi::assert_conforms;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const RELATIONS_PATH: &str = "/api/v1/relations";
const RELATION_BY_ID: &str = "/api/v1/relations/{relationType}/{relationId}";
const PRODUCT_RELATIONS: &str = "/api/v1/products/{productId}/relations";
const UNKNOWN_ULID: &str = "01J9ZK4Q6Y3M2V8W0XAB1CD2EF";

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

async fn serve_relations(
    gateway: Arc<dyn recommend::RelationGateway>,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let state = AppState::new().with_relations(gateway);
    serve(state).await
}

fn product_relations_path(product_id: &Id<Producto>) -> String {
    format!("/api/v1/products/{product_id}/relations")
}

fn relation_path(relation_type: &str, relation_id: &str) -> String {
    format!("/api/v1/relations/{relation_type}/{relation_id}")
}

async fn setup_perno_arandela() -> (
    std::net::SocketAddr,
    tokio::task::JoinHandle<()>,
    PernoArandelaFixtures,
) {
    let (gateway, fixtures) = perno_arandela_gateway();
    let (addr, handle) = serve_relations(gateway).await;
    (addr, handle, fixtures)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_relation_complement_perno_arandela_returns_201() {
    let (addr, handle, fx) = setup_perno_arandela().await;
    let body = format!(
        r#"{{
            "type": "complement",
            "sourceProductId": "{}",
            "targetProductId": "{}",
            "priority": 1
        }}"#,
        fx.perno, fx.arandela
    );
    let (status, ctype, resp) = http_send(addr, "POST", RELATIONS_PATH, Some(&body)).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(status, 201);
    assert_eq!(ctype, "application/json");
    let items = json["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["type"], "complement");
    assert_eq!(items[0]["sourceProductId"], fx.perno.to_string());
    assert_eq!(items[0]["targetProductId"], fx.arandela.to_string());
    assert_eq!(items[0]["priority"], 1);
    assert!(json.get("origen").is_none());
    assert!(json.get("destino").is_none());
    assert_conforms("POST", RELATIONS_PATH, 201, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_relation_similar_omits_bidirectional_creates_pair() {
    let (gateway, a, b) = pair_gateway();
    let (addr, handle) = serve_relations(gateway).await;
    let body = format!(
        r#"{{
            "type": "similar",
            "sourceProductId": "{a}",
            "targetProductId": "{b}"
        }}"#
    );
    let (status, _, resp) = http_send(addr, "POST", RELATIONS_PATH, Some(&body)).await;
    let json = json_body(&resp);
    assert_eq!(status, 201);
    let items = json["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    let g0 = items[0]["bidirectionalGroup"].as_str();
    let g1 = items[1]["bidirectionalGroup"].as_str();
    assert_eq!(g0, g1);
    assert!(g0.is_some());

    let (st_a, _, body_a) = http_send(addr, "GET", &product_relations_path(&a), None).await;
    let (st_b, _, body_b) = http_send(addr, "GET", &product_relations_path(&b), None).await;
    handle.abort();
    assert_eq!(st_a, 200);
    assert_eq!(st_b, 200);
    let ja = json_body(&body_a);
    let jb = json_body(&body_b);
    assert!(
        ja["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["targetProductId"] == b.to_string())
    );
    assert!(
        jb["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["targetProductId"] == a.to_string())
    );
    assert_conforms("POST", RELATIONS_PATH, 201, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_relation_priority_from_b_updates_both() {
    let (gateway, a, b) = pair_gateway();
    let (addr, handle) = serve_relations(gateway).await;
    let create = format!(
        r#"{{
            "type": "similar",
            "sourceProductId": "{a}",
            "targetProductId": "{b}"
        }}"#
    );
    let (_, _, created_body) = http_send(addr, "POST", RELATIONS_PATH, Some(&create)).await;
    let created = json_body(&created_body);
    let b_to_a = created["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["sourceProductId"] == b.to_string())
        .expect("b to a");
    let rel_id = b_to_a["id"].as_str().expect("id");
    let patch_body = r#"{"priority": 3}"#;
    let path = relation_path("similar", rel_id);
    let (status, ctype, resp) = http_send(addr, "PATCH", &path, Some(patch_body)).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(status, 200);
    assert_eq!(ctype, "application/json");
    let items = json["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|i| i["priority"] == 3));
    assert_conforms("PATCH", RELATION_BY_ID, 200, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_relation_from_b_removes_both() {
    let (gateway, a, b) = pair_gateway();
    let (addr, handle) = serve_relations(gateway).await;
    let create = format!(
        r#"{{
            "type": "similar",
            "sourceProductId": "{a}",
            "targetProductId": "{b}"
        }}"#
    );
    let (_, _, created_body) = http_send(addr, "POST", RELATIONS_PATH, Some(&create)).await;
    let created = json_body(&created_body);
    let b_to_a = created["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["sourceProductId"] == b.to_string())
        .expect("b to a");
    let rel_id = b_to_a["id"].as_str().expect("id");
    let path = relation_path("similar", rel_id);
    let (status, _, _) = http_send(addr, "DELETE", &path, None).await;
    assert_eq!(status, 204);
    let (_, _, body_a) = http_send(addr, "GET", &product_relations_path(&a), None).await;
    let (_, _, body_b) = http_send(addr, "GET", &product_relations_path(&b), None).await;
    handle.abort();
    assert_eq!(json_body(&body_a)["items"].as_array().unwrap().len(), 0);
    assert_eq!(json_body(&body_b)["items"].as_array().unwrap().len(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_product_relations_lists_three_types() {
    let src = Id::<Producto>::generate();
    let active = Id::<Producto>::generate();
    let inactive = Id::<Producto>::generate();
    let map = vec![
        (src.clone(), resumen(src.clone(), "SRC", "Source", true)),
        (
            active.clone(),
            resumen(active.clone(), "ACT", "Active", true),
        ),
        (
            inactive.clone(),
            resumen(inactive.clone(), "OFF", "Inactive target", false),
        ),
    ];
    let gw = Arc::new(FakeRelationGateway::with_products(map));
    for (tipo, dest) in [
        (TipoRelacion::SimilarA, inactive.clone()),
        (TipoRelacion::SustituiblePor, active.clone()),
        (TipoRelacion::Complementa, inactive.clone()),
    ] {
        gw.seed_create(NewRelacion {
            origen: src.clone(),
            destino: dest,
            tipo,
            prioridad: 1,
            nota: Some("note".into()),
            bidireccional: Some(false),
        })
        .await
        .expect("seed");
    }
    let (addr, handle) = serve_relations(gw).await;
    let path = product_relations_path(&src);
    let (status, ctype, resp) = http_send(addr, "GET", &path, None).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(status, 200);
    assert_eq!(ctype, "application/json");
    let items = json["items"].as_array().expect("items");
    assert_eq!(items.len(), 3);
    let types: Vec<_> = items.iter().filter_map(|i| i["type"].as_str()).collect();
    assert!(types.contains(&"similar"));
    assert!(types.contains(&"substitute"));
    assert!(types.contains(&"complement"));
    assert!(items.iter().all(|i| i.get("priority").is_some()));
    assert!(items.iter().all(|i| i.get("note").is_some()));
    let inactive_item = items
        .iter()
        .find(|i| i["target"]["sku"] == "OFF")
        .expect("inactive");
    assert_eq!(inactive_item["target"]["active"], false);
    assert_conforms("GET", PRODUCT_RELATIONS, 200, "application/json", &json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_product_relations_incoming() {
    let (gateway, a, b) = pair_gateway();
    let (addr, handle) = serve_relations(gateway).await;
    let body = format!(
        r#"{{
            "type": "complement",
            "sourceProductId": "{a}",
            "targetProductId": "{b}"
        }}"#
    );
    let (_, _, _) = http_send(addr, "POST", RELATIONS_PATH, Some(&body)).await;
    let path = format!("/api/v1/products/{b}/relations?direction=incoming");
    let (status, _, resp) = http_send(addr, "GET", &path, None).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(status, 200);
    let items = json["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["sourceProductId"], a.to_string());
    assert_eq!(items[0]["targetProductId"], b.to_string());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_relation_self_returns_400() {
    let (addr, handle, fx) = setup_perno_arandela().await;
    let body = format!(
        r#"{{
            "type": "complement",
            "sourceProductId": "{}",
            "targetProductId": "{}"
        }}"#,
        fx.perno, fx.perno
    );
    let (status, ctype, resp) = http_send(addr, "POST", RELATIONS_PATH, Some(&body)).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(status, 400);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(json["code"], "self_relation");
    assert_conforms(
        "POST",
        RELATIONS_PATH,
        400,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_relation_duplicate_returns_409() {
    let (addr, handle, fx) = setup_perno_arandela().await;
    let body = format!(
        r#"{{
            "type": "complement",
            "sourceProductId": "{}",
            "targetProductId": "{}"
        }}"#,
        fx.perno, fx.arandela
    );
    let (_, _, _) = http_send(addr, "POST", RELATIONS_PATH, Some(&body)).await;
    let (status, ctype, resp) = http_send(addr, "POST", RELATIONS_PATH, Some(&body)).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(status, 409);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(json["code"], "duplicate_relation");
    assert_conforms(
        "POST",
        RELATIONS_PATH,
        409,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_relation_missing_product_returns_404() {
    let (addr, handle, fx) = setup_perno_arandela().await;
    let missing = Id::<Producto>::generate();
    let body = format!(
        r#"{{
            "type": "complement",
            "sourceProductId": "{}",
            "targetProductId": "{}"
        }}"#,
        fx.perno, missing
    );
    let (status, ctype, resp) = http_send(addr, "POST", RELATIONS_PATH, Some(&body)).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(status, 404);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(json["code"], "not_found");
    assert_conforms(
        "POST",
        RELATIONS_PATH,
        404,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_product_relations_unknown_product_returns_404() {
    let (product_gw, _, seeds) = test_product_gateway().await;
    let relations_gw = empty_known_gateway();
    let state = AppState::new()
        .with_products(product_gw)
        .with_relations(relations_gw);
    let (addr, handle) = serve(state).await;
    let (_, _, _, location) = {
        let create_body = perno_create_json(&seeds);
        let mut stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
        let host = format!("{}:{}", addr.ip(), addr.port());
        let req = format!(
            "POST /api/v1/products HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{create_body}",
            create_body.len()
        );
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        stream.write_all(req.as_bytes()).await.expect("write");
        let mut buf = vec![0u8; 65536];
        let n = stream.read(&mut buf).await.expect("read");
        let response = String::from_utf8_lossy(&buf[..n]);
        let mut loc = String::new();
        for line in response.lines() {
            if let Some((name, value)) = line.split_once(':') {
                if name.trim().eq_ignore_ascii_case("location") {
                    loc = value.trim().to_string();
                }
            }
        }
        (0u16, String::new(), String::new(), loc)
    };
    let product_id = location.rsplit('/').next().unwrap_or("");
    let path = format!("/api/v1/products/{product_id}/relations");
    let (status, ctype, resp) = http_send(addr, "GET", &path, None).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(status, 404);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(json["code"], "not_found");
    assert_conforms(
        "GET",
        PRODUCT_RELATIONS,
        404,
        "application/problem+json",
        &json,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_relation_unknown_id_returns_404() {
    let (addr, handle, _) = setup_perno_arandela().await;
    let unknown = Id::<Relacion>::generate();
    let path = relation_path("similar", &unknown.to_string());
    let (status, ctype, resp) = http_send(addr, "PATCH", &path, Some(r#"{"priority": 2}"#)).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(status, 404);
    assert_eq!(ctype, "application/problem+json");
    assert_eq!(json["code"], "not_found");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn relation_id_invalid_ulid_returns_400() {
    let (addr, handle, _) = setup_perno_arandela().await;
    let (st_get, _, body_get) =
        http_send(addr, "GET", "/api/v1/products/not-a-ulid/relations", None).await;
    let json_get = json_body(&body_get);
    assert_eq!(st_get, 400);
    assert_eq!(json_get["code"], "invalid_input");
    assert!(
        json_get["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["field"] == "productId")
    );

    let (st_del, _, body_del) =
        http_send(addr, "DELETE", "/api/v1/relations/similar/not-a-ulid", None).await;
    handle.abort();
    let json_del = json_body(&body_del);
    assert_eq!(st_del, 400);
    assert_eq!(json_del["code"], "invalid_input");
    assert!(
        json_del["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["field"] == "relationId")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn relation_type_unknown_returns_400() {
    let (addr, handle, _) = setup_perno_arandela().await;
    let id = Id::<Relacion>::generate();
    let path = relation_path("sibling", &id.to_string());
    let (status, _, resp) = http_send(addr, "PATCH", &path, Some(r#"{"priority": 1}"#)).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(status, 400);
    assert_eq!(json["code"], "invalid_input");
    assert!(
        json["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["field"] == "relationType")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_relation_priority_zero_returns_400() {
    let (addr, handle, fx) = setup_perno_arandela().await;
    let body = format!(
        r#"{{
            "type": "complement",
            "sourceProductId": "{}",
            "targetProductId": "{}",
            "priority": 0
        }}"#,
        fx.perno, fx.arandela
    );
    let (status, _, resp) = http_send(addr, "POST", RELATIONS_PATH, Some(&body)).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(status, 400);
    assert_eq!(json["code"], "invalid_input");
    assert!(
        json["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["field"] == "priority")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_relation_without_service_returns_503() {
    let (addr, handle) = serve(AppState::new()).await;
    let path = format!("/api/v1/products/{UNKNOWN_ULID}/relations");
    let (st_get, ct_get, body_get) = http_send(addr, "GET", &path, None).await;
    let json_get = json_body(&body_get);
    assert_eq!(st_get, 503);
    assert_eq!(ct_get, "application/problem+json");
    assert_eq!(json_get["code"], "unavailable");
    assert_conforms(
        "GET",
        PRODUCT_RELATIONS,
        503,
        "application/problem+json",
        &json_get,
    );

    let body = format!(
        r#"{{
            "type": "complement",
            "sourceProductId": "{UNKNOWN_ULID}",
            "targetProductId": "{UNKNOWN_ULID}"
        }}"#
    );
    let (st_post, ct_post, body_post) = http_send(addr, "POST", RELATIONS_PATH, Some(&body)).await;
    handle.abort();
    let json_post = json_body(&body_post);
    assert_eq!(st_post, 503);
    assert_eq!(ct_post, "application/problem+json");
    assert_eq!(json_post["code"], "unavailable");
    assert_conforms(
        "POST",
        RELATIONS_PATH,
        503,
        "application/problem+json",
        &json_post,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_recommendations_is_not_registered() {
    let (addr, handle, fx) = setup_perno_arandela().await;
    let path = format!("/api/v1/products/{}/recommendations", fx.perno);
    let (status, _, resp) = http_send(addr, "GET", &path, None).await;
    handle.abort();
    let json = json_body(&resp);
    assert_eq!(status, 404);
    assert_eq!(json["code"], "not_found");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn relation_problem_field_is_target_product_id() {
    let (addr, handle, fx) = setup_perno_arandela().await;
    let body = format!(
        r#"{{
            "type": "complement",
            "sourceProductId": "{}",
            "targetProductId": "{}"
        }}"#,
        fx.perno, fx.perno
    );
    let (_, _, resp) = http_send(addr, "POST", RELATIONS_PATH, Some(&body)).await;
    handle.abort();
    let json = json_body(&resp);
    let fields: Vec<_> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["field"].as_str())
        .collect();
    assert!(fields.contains(&"targetProductId"));
    assert!(!fields.contains(&"destino"));
}
