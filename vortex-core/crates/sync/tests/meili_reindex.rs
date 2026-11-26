//! Integration tests: T-03-11 Listo cuando.

use std::sync::Arc;
use std::time::{Duration, Instant};

use domain::producto::Producto;
use domain::{Id, IndexUid, SearchFilters, SearchOrden, SearchQuery, TextSearch};
use search::MeiliTextSearch;
use storage::testing::test_db;
use storage::{AtributoDefRepo, OutboxStore, ProductoRepo};
use sync::reindex_meilisearch;

const REGISTRO: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";
const REGISTRO_INACTIVE: &str = "01J9ZKA7M3Q8R2T5V9X1Z4C0D1";

fn meilisearch_url() -> String {
    std::env::var("MEILISEARCH_URL").unwrap_or_else(|_| "http://127.0.0.1:7700".to_string())
}

fn uids() -> (IndexUid, IndexUid) {
    let suffix = Id::<Producto>::generate();
    (
        IndexUid::try_new(&format!("t0311-live-{suffix}")).expect("live"),
        IndexUid::try_new(&format!("t0311-staging-{suffix}")).expect("staging"),
    )
}

async fn seed_active_only(db: &storage::Db) {
    db.query(&format!(
        "CREATE categoria:cat CONTENT {{ nombre: 'C', slug: 'c', ruta: ['C'] }}; \
         CREATE type::thing('producto', '{REGISTRO}') CONTENT {{ \
         sku: 'PER-M8X40-ZN', nombre: 'Perno', tipo: 'Perno', categoria: categoria:cat, \
         precio: 1dec, activo: true, stock: 1 }};"
    ))
    .await
    .expect("seed active");
}

async fn seed_active_and_inactive(db: &storage::Db) {
    db.query(&format!(
        "CREATE categoria:cat CONTENT {{ nombre: 'C', slug: 'c', ruta: ['C'] }}; \
         CREATE type::thing('producto', '{REGISTRO}') CONTENT {{ \
         sku: 'PER-M8X40-ZN', nombre: 'Perno', tipo: 'Perno', categoria: categoria:cat, \
         precio: 1dec, activo: true, stock: 1 }}; \
         CREATE type::thing('producto', '{REGISTRO_INACTIVE}') CONTENT {{ \
         sku: 'OLD-SKU', nombre: 'Viejo', tipo: 'Otro', categoria: categoria:cat, \
         precio: 1dec, activo: false, stock: 0 }};"
    ))
    .await
    .expect("seed both");
}

async fn cleanup(search: &MeiliTextSearch, live: &IndexUid, staging: &IndexUid) {
    let _ = search.delete_index(live).await;
    let _ = search.delete_index(staging).await;
}

fn search_query(q: &str) -> SearchQuery {
    SearchQuery::try_new(
        q.to_string(),
        SearchFilters::default(),
        SearchOrden::Relevancia,
        0,
        20,
    )
    .expect("query")
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn reindex_rebuilds_productos_from_masters_and_swaps() {
    let db = Arc::new(test_db().await);
    seed_active_only(&db).await;
    let search = MeiliTextSearch::new(meilisearch_url(), None::<String>).expect("meilisearch");
    let (live, staging) = uids();
    let product_id: Id<Producto> = REGISTRO.parse().expect("id");

    reindex_meilisearch(
        &OutboxStore::new(Arc::clone(&db)),
        &ProductoRepo::new(Arc::clone(&db)),
        &AtributoDefRepo::new(Arc::clone(&db)),
        &search,
        &live,
        &staging,
    )
    .await
    .expect("reindex");

    let page = search
        .search(&live, search_query("PER-M8X40-ZN"))
        .await
        .expect("search");
    assert!(page.hits.iter().any(|h| h.id == product_id));

    cleanup(&search, &live, &staging).await;
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn reindex_omits_inactive_products() {
    let db = Arc::new(test_db().await);
    seed_active_and_inactive(&db).await;
    let search = MeiliTextSearch::new(meilisearch_url(), None::<String>).expect("meilisearch");
    let (live, staging) = uids();
    let active: Id<Producto> = REGISTRO.parse().expect("active");
    let inactive: Id<Producto> = REGISTRO_INACTIVE.parse().expect("inactive");

    reindex_meilisearch(
        &OutboxStore::new(Arc::clone(&db)),
        &ProductoRepo::new(Arc::clone(&db)),
        &AtributoDefRepo::new(Arc::clone(&db)),
        &search,
        &live,
        &staging,
    )
    .await
    .expect("reindex");

    let page = search
        .search(&live, search_query(""))
        .await
        .expect("search");
    assert!(page.hits.iter().any(|h| h.id == active));
    assert!(!page.hits.iter().any(|h| h.id == inactive));

    cleanup(&search, &live, &staging).await;
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn reindex_second_run_idempotent() {
    let db = Arc::new(test_db().await);
    seed_active_only(&db).await;
    let search = MeiliTextSearch::new(meilisearch_url(), None::<String>).expect("meilisearch");
    let (live, staging) = uids();

    for _ in 0..2 {
        reindex_meilisearch(
            &OutboxStore::new(Arc::clone(&db)),
            &ProductoRepo::new(Arc::clone(&db)),
            &AtributoDefRepo::new(Arc::clone(&db)),
            &search,
            &live,
            &staging,
        )
        .await
        .expect("reindex");
    }

    let product_id: Id<Producto> = REGISTRO.parse().expect("id");
    let page = search
        .search(&live, search_query("PER-M8X40-ZN"))
        .await
        .expect("search");
    assert_eq!(page.hits.iter().filter(|h| h.id == product_id).count(), 1);

    cleanup(&search, &live, &staging).await;
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn reindex_empty_catalog_succeeds() {
    let db = Arc::new(test_db().await);
    let search = MeiliTextSearch::new(meilisearch_url(), None::<String>).expect("meilisearch");
    let (live, staging) = uids();

    reindex_meilisearch(
        &OutboxStore::new(Arc::clone(&db)),
        &ProductoRepo::new(Arc::clone(&db)),
        &AtributoDefRepo::new(Arc::clone(&db)),
        &search,
        &live,
        &staging,
    )
    .await
    .expect("reindex");

    let page = search
        .search(&live, search_query(""))
        .await
        .expect("search");
    assert!(page.hits.is_empty());

    cleanup(&search, &live, &staging).await;
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn reindex_search_loop_never_empty_or_error() {
    let db = Arc::new(test_db().await);
    seed_active_only(&db).await;
    let search =
        Arc::new(MeiliTextSearch::new(meilisearch_url(), None::<String>).expect("meilisearch"));
    let (live, staging) = uids();
    let q = "PER-M8X40-ZN";

    search.create_index(&live).await.expect("create live");
    reindex_meilisearch(
        &OutboxStore::new(Arc::clone(&db)),
        &ProductoRepo::new(Arc::clone(&db)),
        &AtributoDefRepo::new(Arc::clone(&db)),
        search.as_ref(),
        &live,
        &staging,
    )
    .await
    .expect("bootstrap");

    let search_loop = Arc::clone(&search);
    let live_loop = live.clone();
    let query = search_query(q);
    let loop_handle = tokio::spawn(async move {
        let started = Instant::now();
        loop {
            match search_loop.search(&live_loop, query.clone()).await {
                Ok(page) => {
                    if page.hits.is_empty() {
                        panic!("search returned empty hits during reindex");
                    }
                }
                Err(e) => panic!("search failed during reindex: {e}"),
            }
            if started.elapsed() > Duration::from_secs(30) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    });

    reindex_meilisearch(
        &OutboxStore::new(Arc::clone(&db)),
        &ProductoRepo::new(Arc::clone(&db)),
        &AtributoDefRepo::new(Arc::clone(&db)),
        search.as_ref(),
        &live,
        &staging,
    )
    .await
    .expect("reindex during loop");

    loop_handle.await.expect("loop task");

    cleanup(search.as_ref(), &live, &staging).await;
}
