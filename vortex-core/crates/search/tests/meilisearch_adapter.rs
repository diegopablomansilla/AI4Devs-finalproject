//! Integration tests for `MeiliTextSearch` (T-03-04).

use domain::error::DomainCode;
use domain::health::ComponentStatus;
use domain::producto::Producto;
use domain::{Id, IndexUid, SearchDocument, SearchFilters, SearchOrden, SearchQuery, TextSearch};
use search::MeiliTextSearch;

fn meilisearch_url() -> String {
    std::env::var("MEILISEARCH_URL").unwrap_or_else(|_| "http://127.0.0.1:7700".to_string())
}

fn ephemeral_index_uid() -> IndexUid {
    let name = format!("t0304-{}", Id::<Producto>::generate());
    IndexUid::try_new(&name).expect("index uid")
}

fn adapter() -> MeiliTextSearch {
    MeiliTextSearch::new(meilisearch_url(), None::<String>).expect("meilisearch client")
}

async fn cleanup_index(adapter: &MeiliTextSearch, index: &IndexUid) {
    let _ = adapter.delete_index(index).await;
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn meilisearch_adapter_health_is_up() {
    let adapter = adapter();
    let health = adapter.health().await.expect("health");
    assert_eq!(health.status, ComponentStatus::Up);
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn meilisearch_adapter_upserts_then_searches_document() {
    let adapter = adapter();
    let index = ephemeral_index_uid();
    adapter.create_index(&index).await.expect("create_index");

    let product_id = Id::<Producto>::generate();
    let document = SearchDocument::try_new(product_id.clone());
    adapter
        .upsert_documents(&index, &[document])
        .await
        .expect("upsert");

    let query = SearchQuery::try_new(
        product_id.to_string(),
        SearchFilters::default(),
        SearchOrden::Relevancia,
        0,
        20,
    )
    .expect("query");
    let page = adapter.search(&index, query).await.expect("search");
    assert!(
        page.hits.iter().any(|hit| hit.id == product_id),
        "expected hit for upserted document"
    );

    cleanup_index(&adapter, &index).await;
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn meilisearch_adapter_deletes_document() {
    let adapter = adapter();
    let index = ephemeral_index_uid();
    adapter.create_index(&index).await.expect("create_index");

    let product_id = Id::<Producto>::generate();
    adapter
        .upsert_documents(&index, &[SearchDocument::try_new(product_id.clone())])
        .await
        .expect("upsert");

    let find_query = SearchQuery::try_new(
        String::new(),
        SearchFilters::default(),
        SearchOrden::Relevancia,
        0,
        50,
    )
    .expect("query");
    let before = adapter
        .search(&index, find_query.clone())
        .await
        .expect("search");
    assert!(before.hits.iter().any(|h| h.id == product_id));

    adapter
        .delete_documents(&index, &[product_id.clone()])
        .await
        .expect("delete");

    let after = adapter
        .search(&index, find_query.clone())
        .await
        .expect("search");
    assert!(!after.hits.iter().any(|h| h.id == product_id));

    cleanup_index(&adapter, &index).await;
}

#[tokio::test]
async fn meilisearch_adapter_unavailable_when_unreachable() {
    let adapter =
        MeiliTextSearch::new("http://127.0.0.1:1", None::<String>).expect("client for dead port");
    let err = adapter.health().await.expect_err("health must fail");
    assert_eq!(err.code(), DomainCode::Unavailable);
}
