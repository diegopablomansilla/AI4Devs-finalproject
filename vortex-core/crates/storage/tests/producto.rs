//! Integration tests for producto schema (T-02-14).

use std::sync::Arc;

use storage::testing::test_db;
use storage::{MIGRATION_0005_SQL, checksum_hex};

async fn seed_categoria(db: &storage::Db) {
    db.query(
        "CREATE categoria:cat1 CONTENT { nombre: 'Fijaciones', slug: 'fijaciones', ruta: ['Fijaciones'] };",
    )
    .await
    .expect("seed categoria");
}

#[tokio::test]
async fn migrate_applies_0005_producto() {
    let db = Arc::new(test_db().await);
    let rows = db.list_migrations().await.expect("list migrations");
    let v5 = rows
        .iter()
        .find(|r| r.version == 5)
        .expect("migration version 5 must be registered");
    assert_eq!(v5.nombre, "producto");
    assert_eq!(v5.checksum, checksum_hex(MIGRATION_0005_SQL));

    assert_eq!(db.record_count("producto").await.expect("count"), 0);
}

#[tokio::test]
async fn migrate_rerun_keeps_single_0005_record() {
    let db = Arc::new(test_db().await);
    db.migrate()
        .await
        .expect("second migrate must be idempotent");
    let v5_rows: Vec<_> = db
        .list_migrations()
        .await
        .expect("list")
        .into_iter()
        .filter(|r| r.version == 5)
        .collect();
    assert_eq!(v5_rows.len(), 1);
}

#[tokio::test]
async fn producto_creates_minimal_sku() {
    let db = Arc::new(test_db().await);
    seed_categoria(&db).await;
    db.query(
        "CREATE producto:uno CONTENT { \
            sku: 'PER-M8X40-ZN', \
            nombre: 'Perno hexagonal M8 x 40 zincado', \
            tipo: 'Perno', \
            categoria: categoria:cat1, \
            precio: 120.50dec \
        };",
    )
    .await
    .expect("minimal product");
    assert_eq!(db.record_count("producto").await.expect("count"), 1);
}

#[tokio::test]
async fn producto_accepts_zero_precio() {
    let db = Arc::new(test_db().await);
    seed_categoria(&db).await;
    db.query(
        "CREATE producto:free CONTENT { \
            sku: 'ZERO-SKU', \
            nombre: 'Muestra gratis', \
            tipo: 'Muestra', \
            categoria: categoria:cat1, \
            precio: 0dec \
        };",
    )
    .await
    .expect("precio cero");
    assert_eq!(db.record_count("producto").await.expect("count"), 1);
}

#[tokio::test]
async fn producto_allows_duplicate_oem_code() {
    let db = Arc::new(test_db().await);
    seed_categoria(&db).await;
    db.query(
        "CREATE producto:a CONTENT { \
            sku: 'SKU-A', \
            nombre: 'Producto A', \
            tipo: 'Perno', \
            categoria: categoria:cat1, \
            precio: 10dec, \
            codigos: [{ tipo: 'oem', valor: 'HX-0840Z' }] \
        };",
    )
    .await
    .expect("producto A");
    db.query(
        "CREATE producto:b CONTENT { \
            sku: 'SKU-B', \
            nombre: 'Producto B', \
            tipo: 'Perno', \
            categoria: categoria:cat1, \
            precio: 20dec, \
            codigos: [{ tipo: 'oem', valor: 'HX-0840Z' }] \
        };",
    )
    .await
    .expect("producto B");
    assert_eq!(db.record_count("producto").await.expect("count"), 2);
}

#[tokio::test]
async fn producto_rejects_duplicate_sku() {
    let db = Arc::new(test_db().await);
    seed_categoria(&db).await;
    db.query(
        "CREATE producto:uno CONTENT { \
            sku: 'PER-M8X40-ZN', \
            nombre: 'Uno', \
            tipo: 'Perno', \
            categoria: categoria:cat1, \
            precio: 1dec \
        };",
    )
    .await
    .expect("first sku");
    let err = db
        .query(
            "CREATE producto:dos CONTENT { \
                sku: 'PER-M8X40-ZN', \
                nombre: 'Dos', \
                tipo: 'Perno', \
                categoria: categoria:cat1, \
                precio: 2dec \
            };",
        )
        .await
        .expect_err("duplicate sku");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("producto").await.expect("count"), 1);
}

#[tokio::test]
async fn producto_rejects_negative_precio() {
    let db = Arc::new(test_db().await);
    seed_categoria(&db).await;
    let err = db
        .query(
            "CREATE producto:bad CONTENT { \
                sku: 'NEG', \
                nombre: 'Negativo', \
                tipo: 'Perno', \
                categoria: categoria:cat1, \
                precio: -1dec \
            };",
        )
        .await
        .expect_err("negative precio");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("producto").await.expect("count"), 0);
}

#[tokio::test]
async fn producto_rejects_invalid_codigo_tipo() {
    let db = Arc::new(test_db().await);
    seed_categoria(&db).await;
    let err = db
        .query(
            "CREATE producto:bad CONTENT { \
                sku: 'EAN-BAD', \
                nombre: 'Código inválido', \
                tipo: 'Perno', \
                categoria: categoria:cat1, \
                precio: 1dec, \
                codigos: [{ tipo: 'ean', valor: '123' }] \
            };",
        )
        .await
        .expect_err("invalid codigo tipo");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("producto").await.expect("count"), 0);
}

#[tokio::test]
async fn producto_rejects_empty_sku() {
    let db = Arc::new(test_db().await);
    seed_categoria(&db).await;
    let err = db
        .query(
            "CREATE producto:bad CONTENT { \
                sku: '', \
                nombre: 'Sin SKU', \
                tipo: 'Perno', \
                categoria: categoria:cat1, \
                precio: 1dec \
            };",
        )
        .await
        .expect_err("empty sku");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("producto").await.expect("count"), 0);
}
