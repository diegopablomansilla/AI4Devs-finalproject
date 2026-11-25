//! Integration tests: Listo cuando (T-03-07).

use std::sync::Arc;
use std::time::{Duration, Instant};

use domain::producto::Producto;
use domain::{Id, IndexUid, OutboxRecord, SearchFilters, SearchOrden, SearchQuery, TextSearch};
use search::MeiliTextSearch;
use storage::testing::test_db;
use storage::{AtributoDefRepo, OutboxStore, ProductoRepo};
use sync::{DEFAULT_POLL_INTERVAL, MeiliProductWorker};

const REGISTRO: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";

fn meilisearch_url() -> String {
    std::env::var("MEILISEARCH_URL").unwrap_or_else(|_| "http://127.0.0.1:7700".to_string())
}

fn ephemeral_index_uid() -> IndexUid {
    let name = format!("t0307-{}", Id::<Producto>::generate());
    IndexUid::try_new(&name).expect("index uid")
}

async fn seed_catalog(db: &storage::Db) {
    db.query(&format!(
        "CREATE categoria:cat CONTENT {{ nombre: 'C', slug: 'c', ruta: ['C'] }}; \
         CREATE type::thing('producto', '{REGISTRO}') CONTENT {{ \
         sku: 'PER-M8X40-ZN', nombre: 'Perno', tipo: 'Perno', categoria: categoria:cat, \
         precio: 1dec, activo: true, stock: 1 }};"
    ))
    .await
    .expect("seed product");
    let outbox_id = Id::<OutboxRecord>::generate();
    db.query(&format!(
        "CREATE type::thing('outbox', '{}') CONTENT {{ \
         entidad: 'producto', \
         registro: type::thing('producto', '{REGISTRO}'), \
         operacion: 'upsert' }};",
        outbox_id
    ))
    .await
    .expect("seed outbox");
}

async fn cleanup_index(adapter: &MeiliTextSearch, index: &IndexUid) {
    let _ = adapter.delete_index(index).await;
}

fn worker(
    db: Arc<storage::Db>,
    search: MeiliTextSearch,
    index: IndexUid,
) -> MeiliProductWorker<OutboxStore, ProductoRepo, AtributoDefRepo, MeiliTextSearch> {
    MeiliProductWorker::new(
        OutboxStore::new(Arc::clone(&db)),
        ProductoRepo::new(Arc::clone(&db)),
        AtributoDefRepo::new(Arc::clone(&db)),
        search,
        db.outbox_notifier(),
        index,
        DEFAULT_POLL_INTERVAL,
    )
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn worker_indexes_new_product_within_one_second() {
    let db = Arc::new(test_db().await);
    seed_catalog(&db).await;

    let search = MeiliTextSearch::new(meilisearch_url(), None::<String>).expect("meilisearch");
    let index = ephemeral_index_uid();
    search.create_index(&index).await.expect("create_index");

    let product_id: Id<Producto> = REGISTRO.parse().expect("product id");
    let w = worker(Arc::clone(&db), search.clone(), index.clone());

    let started = Instant::now();
    w.drain_once().await.expect("drain");

    let query = SearchQuery::try_new(
        String::from("PER-M8X40-ZN"),
        SearchFilters::default(),
        SearchOrden::Relevancia,
        0,
        20,
    )
    .expect("query");

    while !search
        .search(&index, query.clone())
        .await
        .expect("search")
        .hits
        .iter()
        .any(|h| h.id == product_id)
    {
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "product not searchable within 1s"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    cleanup_index(&search, &index).await;
}

#[tokio::test]
#[ignore = "meilisearch"]
async fn worker_reprocessing_same_event_is_idempotent() {
    let db = Arc::new(test_db().await);
    seed_catalog(&db).await;

    let search = MeiliTextSearch::new(meilisearch_url(), None::<String>).expect("meilisearch");
    let index = ephemeral_index_uid();
    search.create_index(&index).await.expect("create_index");

    let product_id: Id<Producto> = REGISTRO.parse().expect("product id");
    let w = worker(Arc::clone(&db), search.clone(), index.clone());

    w.drain_once().await.expect("first drain");
    db.query("UPDATE sync_cursor:meilisearch SET ultimo_evento = NONE;")
        .await
        .expect("reset cursor");
    w.drain_once().await.expect("second drain");

    let query = SearchQuery::try_new(
        product_id.to_string(),
        SearchFilters::default(),
        SearchOrden::Relevancia,
        0,
        50,
    )
    .expect("query");
    let page = search.search(&index, query).await.expect("search");
    assert_eq!(
        page.hits.iter().filter(|h| h.id == product_id).count(),
        1,
        "reprocessing must not duplicate the document"
    );

    cleanup_index(&search, &index).await;
}
