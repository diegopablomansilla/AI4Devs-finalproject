//! Ranking and typo tolerance against real Meilisearch (T-03-14).

use domain::producto::Producto;
use domain::{Id, IndexUid, SearchDocument, SearchFilters, SearchOrden, SearchQuery, TextSearch};
use search::{MeiliTextSearch, productos_index_settings};

fn meilisearch_url() -> String {
    std::env::var("MEILISEARCH_URL").unwrap_or_else(|_| "http://127.0.0.1:7700".to_string())
}

fn ranking_index_uid() -> IndexUid {
    let name = format!("t0314-{}", Id::<Producto>::generate());
    IndexUid::try_new(&name).expect("index uid")
}

fn adapter() -> MeiliTextSearch {
    MeiliTextSearch::new(meilisearch_url(), None::<String>).expect("meilisearch client")
}

async fn cleanup_index(adapter: &MeiliTextSearch, index: &IndexUid) {
    let _ = adapter.delete_index(index).await;
}

async fn prepare_ranking_index(adapter: &MeiliTextSearch) -> IndexUid {
    let index = ranking_index_uid();
    adapter.create_index(&index).await.expect("create_index");
    let settings = productos_index_settings(std::iter::empty::<&str>()).expect("productos profile");
    adapter
        .apply_settings(&index, settings)
        .await
        .expect("apply_settings");
    index
}

fn catalog_doc(nombre: &str, tipo: &str, sku: &str, stock_disponible: f64) -> SearchDocument {
    let mut doc = SearchDocument::try_new(Id::<Producto>::generate());
    doc.nombre = nombre.to_string();
    doc.tipo = tipo.to_string();
    doc.sku = sku.to_string();
    doc.stock_disponible = stock_disponible;
    doc.con_stock = stock_disponible > 0.0;
    doc
}

fn hit_position_by_nombre(hits: &[domain::SearchHit], nombre: &str) -> Option<usize> {
    hits.iter().position(|hit| hit.nombre == nombre)
}

async fn search_relevance(
    adapter: &MeiliTextSearch,
    index: &IndexUid,
    q: &str,
) -> domain::SearchPage {
    let query = SearchQuery::try_new(
        q.to_string(),
        SearchFilters::default(),
        SearchOrden::Relevancia,
        0,
        20,
    )
    .expect("query");
    adapter.search(index, query).await.expect("search")
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn meilisearch_ranking_tornilo_finds_tornillo_autoperforante() {
    let adapter = adapter();
    let index = prepare_ranking_index(&adapter).await;

    let target = "Tornillo autoperforante 10 x 1";
    let doc = catalog_doc(target, "Tornillo", "TOR-AUTO-10X1", 40.0);
    adapter
        .upsert_documents(&index, &[doc])
        .await
        .expect("upsert");

    let page = search_relevance(&adapter, &index, "tornilo").await;
    assert!(
        page.hits.iter().any(|hit| hit.nombre == target),
        "expected typo query to find {target:?}"
    );

    cleanup_index(&adapter, &index).await;
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn meilisearch_ranking_perno_orders_steel_before_washer() {
    let adapter = adapter();
    let index = prepare_ranking_index(&adapter).await;

    let steel = "Perno de Acero";
    let washer = "Arandela para Perno";
    let docs = vec![
        catalog_doc(washer, "Arandela", "ARA-PERNO-01", 3.0),
        catalog_doc(steel, "Perno", "PER-ACERO-01", 40.0),
    ];
    adapter
        .upsert_documents(&index, &docs)
        .await
        .expect("upsert");

    let page = search_relevance(&adapter, &index, "Perno").await;
    let pos_steel = hit_position_by_nombre(&page.hits, steel).expect("steel hit");
    let pos_washer = hit_position_by_nombre(&page.hits, washer).expect("washer hit");
    assert!(
        pos_steel < pos_washer,
        "expected {steel:?} before {washer:?}; hits: {:?}",
        page.hits.iter().map(|h| &h.nombre).collect::<Vec<_>>()
    );

    cleanup_index(&adapter, &index).await;
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn meilisearch_ranking_equal_relevance_orders_stock_40_before_3() {
    let adapter = adapter();
    let index = prepare_ranking_index(&adapter).await;

    let nombre = "Perno de prueba desempate T0314";
    let docs = vec![
        catalog_doc(nombre, "Perno", "RANK-STOCK-LOW", 3.0),
        catalog_doc(nombre, "Perno", "RANK-STOCK-HIGH", 40.0),
    ];
    adapter
        .upsert_documents(&index, &docs)
        .await
        .expect("upsert");

    let page = search_relevance(&adapter, &index, nombre).await;
    let high = page
        .hits
        .iter()
        .find(|h| h.stock_disponible == 40.0)
        .expect("hit with stock 40");
    let low = page
        .hits
        .iter()
        .find(|h| h.stock_disponible == 3.0)
        .expect("hit with stock 3");
    let pos_high = page
        .hits
        .iter()
        .position(|h| h.id == high.id)
        .expect("position high");
    let pos_low = page
        .hits
        .iter()
        .position(|h| h.id == low.id)
        .expect("position low");
    assert!(
        pos_high < pos_low,
        "expected higher stock first; order: {:?}",
        page.hits
            .iter()
            .map(|h| (h.sku.as_str(), h.stock_disponible))
            .collect::<Vec<_>>()
    );

    cleanup_index(&adapter, &index).await;
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn meilisearch_ranking_sku_typo_does_not_match() {
    let adapter = adapter();
    let index = prepare_ranking_index(&adapter).await;

    const SKU: &str = "SKUABCDEFGH";
    let doc = catalog_doc("Llave ajustable industrial", "Herramienta", SKU, 40.0);
    adapter
        .upsert_documents(&index, &[doc.clone()])
        .await
        .expect("upsert");

    let exact = search_relevance(&adapter, &index, SKU).await;
    assert!(
        exact.hits.iter().any(|hit| hit.id == doc.id),
        "exact sku query must find the document"
    );

    let page = search_relevance(&adapter, &index, "SKUABCDEFGI").await;
    assert!(
        !page.hits.iter().any(|hit| hit.id == doc.id),
        "single-character sku typo must not match when typo tolerance is disabled on sku"
    );

    cleanup_index(&adapter, &index).await;
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn meilisearch_ranking_unknown_query_returns_empty_hits() {
    let adapter = adapter();
    let index = prepare_ranking_index(&adapter).await;

    let doc = catalog_doc(
        "Tornillo autoperforante 10 x 1",
        "Tornillo",
        "TOR-AUTO-10X1",
        40.0,
    );
    adapter
        .upsert_documents(&index, &[doc])
        .await
        .expect("upsert");

    let page = search_relevance(&adapter, &index, "xyzzy-no-match-9f3").await;
    assert!(
        page.hits.is_empty(),
        "expected no hits for non-matching query, got {:?}",
        page.hits
    );

    cleanup_index(&adapter, &index).await;
}
