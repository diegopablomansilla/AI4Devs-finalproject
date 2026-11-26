//! Integration tests for `OutboxRepo` (T-03-07).

use std::sync::Arc;

use domain::{Id, OutboxRecord, OutboxRepo};

use storage::OutboxStore;
use storage::testing::test_db;

const REGISTRO: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";

fn ordered_outbox_ids() -> (Id<OutboxRecord>, Id<OutboxRecord>) {
    let a = Id::<OutboxRecord>::generate();
    let b = Id::<OutboxRecord>::generate();
    if a.inner_ulid() < b.inner_ulid() {
        (a, b)
    } else {
        (b, a)
    }
}

async fn seed_producto_row(db: &storage::Db) {
    db.query(
        "CREATE categoria:cat CONTENT { nombre: 'C', slug: 'c', ruta: ['C'] }; \
         CREATE type::thing('producto', '01J9ZKA7M3Q8R2T5V9X1Z4B6D8') CONTENT { \
         sku: 'SKU', nombre: 'N', tipo: 'T', categoria: categoria:cat, precio: 1dec, activo: true, stock: 1 };",
    )
    .await
    .expect("seed");
}

async fn create_outbox(db: &storage::Db, id: &str) {
    db.query(&format!(
        "CREATE type::thing('outbox', '{id}') CONTENT {{ \
         entidad: 'producto', \
         registro: type::thing('producto', '{REGISTRO}'), \
         operacion: 'upsert' }};"
    ))
    .await
    .expect("outbox row");
}

#[tokio::test]
async fn storage_outbox_repo_lists_after_cursor() {
    let db = Arc::new(test_db().await);
    seed_producto_row(&db).await;
    let (ev0, ev1) = ordered_outbox_ids();
    create_outbox(&db, &ev0.to_string()).await;
    create_outbox(&db, &ev1.to_string()).await;
    let store = OutboxStore::new(Arc::clone(&db));
    let batch = store.list_after_meilisearch(Some(ev0)).await.expect("list");
    assert_eq!(batch.len(), 1);
    assert_eq!(batch[0].id, ev1);
    assert_eq!(batch[0].event.registro, REGISTRO);
}

#[tokio::test]
async fn storage_outbox_repo_advances_meilisearch_cursor() {
    let db = Arc::new(test_db().await);
    seed_producto_row(&db).await;
    let ev0 = Id::<OutboxRecord>::generate();
    create_outbox(&db, &ev0.to_string()).await;
    let store = OutboxStore::new(Arc::clone(&db));
    let id = ev0;
    store
        .advance_meilisearch_cursor(id.clone())
        .await
        .expect("advance");
    let cursor = store.meilisearch_cursor().await.expect("cursor");
    assert_eq!(cursor, Some(id));
    let batch = store.list_after_meilisearch(cursor).await.expect("list");
    assert!(batch.is_empty());
}

#[tokio::test]
async fn storage_outbox_repo_limits_batch_to_200() {
    let db = Arc::new(test_db().await);
    seed_producto_row(&db).await;
    for _ in 0..201u32 {
        let id = Id::<OutboxRecord>::generate();
        create_outbox(&db, &id.to_string()).await;
    }
    let store = OutboxStore::new(Arc::clone(&db));
    let batch = store.list_after_meilisearch(None).await.expect("list");
    assert_eq!(batch.len(), 200);
}

#[tokio::test]
async fn storage_outbox_repo_advance_unknown_event_is_not_found() {
    let db = Arc::new(test_db().await);
    let store = OutboxStore::new(Arc::clone(&db));
    let missing: Id<OutboxRecord> = Id::generate();
    let err = store
        .advance_meilisearch_cursor(missing)
        .await
        .expect_err("missing");
    assert_eq!(err.code(), domain::DomainCode::NotFound);
}

#[tokio::test]
async fn storage_outbox_repo_records_fallido() {
    let db = Arc::new(test_db().await);
    seed_producto_row(&db).await;
    let ev0 = Id::<OutboxRecord>::generate();
    create_outbox(&db, &ev0.to_string()).await;
    let store = OutboxStore::new(Arc::clone(&db));
    store
        .record_meilisearch_failure(ev0.clone(), "invalid document")
        .await
        .expect("record");
    store
        .record_meilisearch_failure(ev0, "again")
        .await
        .expect("idempotent");
    assert_eq!(store.meilisearch_failed_count().await.expect("count"), 1);
}

#[tokio::test]
async fn storage_outbox_repo_lag_counts_pending() {
    let db = Arc::new(test_db().await);
    seed_producto_row(&db).await;
    let (ev0, ev1) = ordered_outbox_ids();
    create_outbox(&db, &ev0.to_string()).await;
    create_outbox(&db, &ev1.to_string()).await;
    let store = OutboxStore::new(Arc::clone(&db));
    assert_eq!(store.meilisearch_lag().await.expect("lag"), 2);
    store
        .advance_meilisearch_cursor(ev0.clone())
        .await
        .expect("advance");
    assert_eq!(store.meilisearch_lag().await.expect("lag"), 1);
}

#[tokio::test]
async fn storage_outbox_repo_failed_count() {
    let db = Arc::new(test_db().await);
    seed_producto_row(&db).await;
    let ev0 = Id::<OutboxRecord>::generate();
    let ev1 = Id::<OutboxRecord>::generate();
    create_outbox(&db, &ev0.to_string()).await;
    create_outbox(&db, &ev1.to_string()).await;
    let store = OutboxStore::new(Arc::clone(&db));
    store
        .record_meilisearch_failure(ev0, "a")
        .await
        .expect("record");
    store
        .record_meilisearch_failure(ev1, "b")
        .await
        .expect("record");
    assert_eq!(store.meilisearch_failed_count().await.expect("count"), 2);
}

#[tokio::test]
async fn storage_outbox_repo_record_unknown_event_is_not_found() {
    let db = Arc::new(test_db().await);
    let store = OutboxStore::new(Arc::clone(&db));
    let missing = Id::<OutboxRecord>::generate();
    let err = store
        .record_meilisearch_failure(missing, "x")
        .await
        .expect_err("missing");
    assert_eq!(err.code(), domain::DomainCode::NotFound);
}
