//! Integration tests for product relation schema (T-04-01).

use std::sync::Arc;

use storage::testing::test_db;
use storage::{MIGRATION_0007_SQL, checksum_hex};

async fn seed_two_productos(db: &storage::Db) -> Result<(), storage::OpenError> {
    db.query(
        "CREATE categoria:cat1 CONTENT { nombre: 'Fijaciones', slug: 'fijaciones', ruta: ['Fijaciones'] };",
    )
    .await?;
    db.query(
        "CREATE producto:prdA CONTENT { \
            sku: 'PER-M8X40-ZN', \
            nombre: 'Perno hexagonal M8 x 40 zincado', \
            tipo: 'Perno', \
            categoria: categoria:cat1, \
            precio: 120.50dec \
        };",
    )
    .await?;
    db.query(
        "CREATE producto:prdB CONTENT { \
            sku: 'ARA-M8-ZN', \
            nombre: 'Arandela M8 zincada', \
            tipo: 'Arandela', \
            categoria: categoria:cat1, \
            precio: 5.00dec \
        };",
    )
    .await
}

#[tokio::test]
async fn migrate_applies_0007_relaciones() {
    let db = Arc::new(test_db().await);
    let rows = db.list_migrations().await.expect("list migrations");
    let v7 = rows
        .iter()
        .find(|r| r.version == 7)
        .expect("migration version 7 must be registered");
    assert_eq!(v7.nombre, "relaciones");
    assert_eq!(v7.checksum, checksum_hex(MIGRATION_0007_SQL));

    assert_eq!(db.record_count("similar_a").await.expect("count"), 0);
    assert_eq!(db.record_count("sustituible_por").await.expect("count"), 0);
    assert_eq!(db.record_count("complementa").await.expect("count"), 0);
}

#[tokio::test]
async fn migrate_rerun_keeps_single_0007_record() {
    let db = Arc::new(test_db().await);
    db.migrate()
        .await
        .expect("second migrate must be idempotent");
    let v7_rows: Vec<_> = db
        .list_migrations()
        .await
        .expect("list")
        .into_iter()
        .filter(|r| r.version == 7)
        .collect();
    assert_eq!(v7_rows.len(), 1);
}

#[tokio::test]
async fn relacion_relates_existing_products() {
    let db = Arc::new(test_db().await);
    seed_two_productos(&db).await.expect("seed productos");
    db.query("RELATE producto:prdA -> similar_a -> producto:prdB;")
        .await
        .expect("similar_a");
    db.query("RELATE producto:prdA -> sustituible_por -> producto:prdB;")
        .await
        .expect("sustituible_por");
    db.query("RELATE producto:prdA -> complementa -> producto:prdB;")
        .await
        .expect("complementa");
    assert_eq!(db.record_count("similar_a").await.expect("count"), 1);
    assert_eq!(db.record_count("sustituible_por").await.expect("count"), 1);
    assert_eq!(db.record_count("complementa").await.expect("count"), 1);
}

#[tokio::test]
async fn relacion_allows_same_pair_on_distinct_tables() {
    let db = Arc::new(test_db().await);
    seed_two_productos(&db).await.expect("seed productos");
    db.query("RELATE producto:prdA -> similar_a -> producto:prdB;")
        .await
        .expect("similar_a");
    db.query("RELATE producto:prdA -> sustituible_por -> producto:prdB;")
        .await
        .expect("sustituible_por");
    assert_eq!(db.record_count("similar_a").await.expect("count"), 1);
    assert_eq!(db.record_count("sustituible_por").await.expect("count"), 1);
}

#[tokio::test]
async fn relacion_enforced_rejects_missing_target() {
    let db = Arc::new(test_db().await);
    seed_two_productos(&db).await.expect("seed productos");
    let err = db
        .query("RELATE producto:prdA -> sustituible_por -> producto:missing;")
        .await
        .expect_err("ENFORCED must reject missing out");
    let _ = err.to_string();
    assert_eq!(db.record_count("sustituible_por").await.expect("count"), 0);
}

#[tokio::test]
async fn relacion_unique_rejects_duplicate_edge() {
    let db = Arc::new(test_db().await);
    seed_two_productos(&db).await.expect("seed productos");
    db.query("RELATE producto:prdA -> sustituible_por -> producto:prdB;")
        .await
        .expect("first edge");
    let err = db
        .query("RELATE producto:prdA -> sustituible_por -> producto:prdB;")
        .await
        .expect_err("duplicate edge");
    let _ = err.to_string();
    assert_eq!(db.record_count("sustituible_por").await.expect("count"), 1);
}

#[tokio::test]
async fn relacion_rejects_prioridad_below_one() {
    let db = Arc::new(test_db().await);
    seed_two_productos(&db).await.expect("seed productos");
    let err = db
        .query("RELATE producto:prdA -> similar_a -> producto:prdB SET prioridad = 0;")
        .await
        .expect_err("prioridad < 1");
    let _ = err.to_string();
    assert_eq!(db.record_count("similar_a").await.expect("count"), 0);
}
