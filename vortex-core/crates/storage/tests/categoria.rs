//! Integration tests for categoria schema migration (T-02-01).

use storage::testing::test_db;
use storage::{MIGRATION_0002_SQL, checksum_hex};

#[tokio::test]
async fn migrate_applies_0002_categoria() {
    let db = test_db().await;
    let rows = db.list_migrations().await.expect("list migrations");
    let v2 = rows
        .iter()
        .find(|r| r.version == 2)
        .expect("migration version 2 must be registered");
    assert_eq!(v2.nombre, "categoria");
    assert_eq!(v2.checksum, checksum_hex(MIGRATION_0002_SQL));

    assert_eq!(
        db.record_count("categoria").await.expect("categoria count"),
        0
    );
}

#[tokio::test]
async fn migrate_rerun_keeps_single_0002_record() {
    let db = test_db().await;
    db.migrate()
        .await
        .expect("second migrate must be idempotent");
    let v2_rows: Vec<_> = db
        .list_migrations()
        .await
        .expect("list")
        .into_iter()
        .filter(|r| r.version == 2)
        .collect();
    assert_eq!(v2_rows.len(), 1);
}

#[tokio::test]
async fn categoria_allows_same_slug_under_different_padres() {
    let db = test_db().await;
    db.query(
        "CREATE categoria:padre_a CONTENT { nombre: 'Padre A', slug: 'padre-a', ruta: ['Padre A'] }; \
         CREATE categoria:padre_b CONTENT { nombre: 'Padre B', slug: 'padre-b', ruta: ['Padre B'] }; \
         CREATE categoria:hija_a CONTENT { nombre: 'Hija', slug: 'hija', padre: categoria:padre_a, ruta: ['Padre A', 'Hija'] }; \
         CREATE categoria:hija_b CONTENT { nombre: 'Hija', slug: 'hija', padre: categoria:padre_b, ruta: ['Padre B', 'Hija'] };",
    )
    .await
    .expect("same slug under different padres");
    assert_eq!(db.record_count("categoria").await.expect("count"), 4);
}

#[tokio::test]
async fn categoria_rejects_duplicate_padre_slug() {
    let db = test_db().await;
    db.query(
        "CREATE categoria:padre CONTENT { nombre: 'Padre', slug: 'padre', ruta: ['Padre'] }; \
         CREATE categoria:primera CONTENT { nombre: 'Una', slug: 'child', padre: categoria:padre, ruta: ['Padre', 'Una'] };",
    )
    .await
    .expect("first child");
    let err = db
        .query(
            "CREATE categoria:segunda CONTENT { nombre: 'Otra', slug: 'child', padre: categoria:padre, ruta: ['Padre', 'Otra'] };",
        )
        .await
        .expect_err("duplicate padre+slug must fail");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("categoria").await.expect("count"), 2);
}

#[tokio::test]
async fn categoria_rejects_empty_nombre() {
    let db = test_db().await;
    let err = db
        .query("CREATE categoria:bad CONTENT { nombre: '', slug: 'bad', ruta: [] };")
        .await
        .expect_err("empty nombre must fail");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("categoria").await.expect("count"), 0);
}
