//! Integration tests for outbox schema migration (T-03-01).

use storage::testing::test_db;
use storage::{MIGRATION_0001_SQL, checksum_hex};

async fn assert_cursor_exists(db: &storage::Db, id: &str) {
    let sql = format!("IF !record::exists(sync_cursor:{id}) {{ THROW 'missing cursor {id}'; }};");
    db.query(&sql).await.expect("cursor must exist");
}

#[tokio::test]
async fn migrate_applies_0001_outbox_and_seeds_cursors() {
    let db = test_db().await;
    let rows = db.list_migrations().await.expect("list migrations");
    let v1 = rows
        .iter()
        .find(|r| r.version == 1)
        .expect("migration version 1 must be registered");
    assert_eq!(v1.nombre, "outbox");
    assert_eq!(v1.checksum, checksum_hex(MIGRATION_0001_SQL));

    assert_cursor_exists(&db, "meilisearch").await;
    assert_cursor_exists(&db, "vectores").await;

    assert_eq!(db.record_count("outbox").await.expect("outbox count"), 0);
    assert_eq!(
        db.record_count("outbox_fallo").await.expect("fallo count"),
        0
    );
    assert_eq!(
        db.record_count("sync_cursor").await.expect("cursor count"),
        2
    );
}

#[tokio::test]
async fn migrate_rerun_does_not_duplicate_cursors() {
    let db = test_db().await;
    db.migrate()
        .await
        .expect("second migrate must be idempotent");
    let v1_rows: Vec<_> = db
        .list_migrations()
        .await
        .expect("list")
        .into_iter()
        .filter(|r| r.version == 1)
        .collect();
    assert_eq!(v1_rows.len(), 1);
    assert_eq!(
        db.record_count("sync_cursor").await.expect("cursor count"),
        2
    );
}

#[tokio::test]
async fn outbox_accepts_valid_entidad_and_operacion() {
    let db = test_db().await;
    db.query(
        "CREATE outbox:01J0000000000000000000000 CONTENT { \
         entidad: 'producto', registro: producto:seed, operacion: 'upsert' };",
    )
    .await
    .expect("valid outbox row");
    assert_eq!(db.record_count("outbox").await.expect("count"), 1);
}

#[tokio::test]
async fn outbox_rejects_invalid_entidad() {
    let db = test_db().await;
    let err = db
        .query(
            "CREATE outbox:01J0000000000000000000001 CONTENT { \
             entidad: 'foo', registro: producto:seed, operacion: 'upsert' };",
        )
        .await
        .expect_err("invalid entidad must fail");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("outbox").await.expect("count"), 0);
}

#[tokio::test]
async fn outbox_rejects_invalid_operacion() {
    let db = test_db().await;
    let err = db
        .query(
            "CREATE outbox:01J0000000000000000000002 CONTENT { \
             entidad: 'producto', registro: producto:seed, operacion: 'update' };",
        )
        .await
        .expect_err("invalid operacion must fail");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("outbox").await.expect("count"), 0);
}

#[tokio::test]
async fn outbox_fallo_rejects_invalid_consumidor() {
    let db = test_db().await;
    db.query(
        "CREATE outbox:01J0000000000000000000003 CONTENT { \
         entidad: 'producto', registro: producto:seed, operacion: 'upsert' };",
    )
    .await
    .expect("seed outbox row");

    let err = db
        .query(
            "CREATE outbox_fallo CONTENT { \
             evento: outbox:01J0000000000000000000003, \
             consumidor: 'kafka', \
             ultimo_error: 'x', \
             estado: 'fallido' };",
        )
        .await
        .expect_err("invalid consumidor must fail");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("outbox_fallo").await.expect("count"), 0);
}
