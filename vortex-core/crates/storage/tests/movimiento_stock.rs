//! Integration tests for movimiento_stock schema (T-02-15).

use std::sync::Arc;

use storage::testing::test_db;
use storage::{MIGRATION_0006_SQL, checksum_hex};

async fn seed_producto(db: &storage::Db) -> Result<(), storage::OpenError> {
    db.query(
        "CREATE categoria:cat1 CONTENT { nombre: 'Fijaciones', slug: 'fijaciones', ruta: ['Fijaciones'] };",
    )
    .await?;
    db.query(
        "CREATE producto:prd1 CONTENT { \
            sku: 'PER-M8X40-ZN', \
            nombre: 'Perno hexagonal M8 x 40 zincado', \
            tipo: 'Perno', \
            categoria: categoria:cat1, \
            precio: 120.50dec \
        };",
    )
    .await
}

#[tokio::test]
async fn migrate_applies_0006_movimiento_stock() {
    let db = Arc::new(test_db().await);
    let rows = db.list_migrations().await.expect("list migrations");
    let v6 = rows
        .iter()
        .find(|r| r.version == 6)
        .expect("migration version 6 must be registered");
    assert_eq!(v6.nombre, "movimiento_stock");
    assert_eq!(v6.checksum, checksum_hex(MIGRATION_0006_SQL));

    assert_eq!(db.record_count("movimiento_stock").await.expect("count"), 0);
}

#[tokio::test]
async fn migrate_rerun_keeps_single_0006_record() {
    let db = Arc::new(test_db().await);
    db.migrate()
        .await
        .expect("second migrate must be idempotent");
    let v6_rows: Vec<_> = db
        .list_migrations()
        .await
        .expect("list")
        .into_iter()
        .filter(|r| r.version == 6)
        .collect();
    assert_eq!(v6_rows.len(), 1);
}

#[tokio::test]
async fn movimiento_stock_creates_ingreso() {
    let db = Arc::new(test_db().await);
    seed_producto(&db).await.expect("seed producto");
    db.query(
        "CREATE movimiento_stock:mov1 CONTENT { \
            producto: producto:prd1, \
            tipo: 'ingreso', \
            cantidad: 20dec, \
            stock_resultante: 20dec \
        };",
    )
    .await
    .expect("ingreso movement");
    assert_eq!(db.record_count("movimiento_stock").await.expect("count"), 1);
}

#[tokio::test]
async fn movimiento_stock_rejects_update() {
    let db = Arc::new(test_db().await);
    seed_producto(&db).await.expect("seed producto");
    db.query(
        "CREATE movimiento_stock:mov1 CONTENT { \
            producto: producto:prd1, \
            tipo: 'ingreso', \
            cantidad: 20dec, \
            stock_resultante: 20dec \
        };",
    )
    .await
    .expect("seed movement");
    let err = db
        .query("UPDATE movimiento_stock:mov1 SET motivo = 'hack';")
        .await
        .expect_err("update must fail on READONLY ledger");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("movimiento_stock").await.expect("count"), 1);
}

#[tokio::test]
async fn movimiento_stock_rejects_invalid_tipo() {
    let db = Arc::new(test_db().await);
    seed_producto(&db).await.expect("seed producto");
    let err = db
        .query(
            "CREATE movimiento_stock:bad CONTENT { \
                producto: producto:prd1, \
                tipo: 'transferencia', \
                cantidad: 1dec, \
                stock_resultante: 1dec \
            };",
        )
        .await
        .expect_err("invalid tipo");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("movimiento_stock").await.expect("count"), 0);
}

#[tokio::test]
async fn movimiento_stock_rejects_zero_cantidad() {
    let db = Arc::new(test_db().await);
    seed_producto(&db).await.expect("seed producto");
    let err = db
        .query(
            "CREATE movimiento_stock:bad CONTENT { \
                producto: producto:prd1, \
                tipo: 'ingreso', \
                cantidad: 0dec, \
                stock_resultante: 0dec \
            };",
        )
        .await
        .expect_err("zero cantidad");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("movimiento_stock").await.expect("count"), 0);
}

#[tokio::test]
async fn movimiento_stock_rejects_negative_stock_resultante() {
    let db = Arc::new(test_db().await);
    seed_producto(&db).await.expect("seed producto");
    let err = db
        .query(
            "CREATE movimiento_stock:bad CONTENT { \
                producto: producto:prd1, \
                tipo: 'ajuste', \
                cantidad: -5dec, \
                stock_resultante: -1dec \
            };",
        )
        .await
        .expect_err("negative stock_resultante");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("movimiento_stock").await.expect("count"), 0);
}
