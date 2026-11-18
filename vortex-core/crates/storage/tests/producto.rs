//! Integration tests for producto schema (T-02-14) and ProductRepo (T-02-18).

use std::sync::Arc;

use domain::{
    Categoria, CategoryRepo, ConflictCode, DomainCode, DomainError, Id, NewCategoria, NewProducto,
    ProductRepo, Producto, UnidadMedida,
};

use storage::testing::test_db;
use storage::{CategoriaRepo, MIGRATION_0005_SQL, ProductoRepo, checksum_hex};

fn product_repo(db: Arc<storage::Db>) -> ProductoRepo {
    ProductoRepo::new(db)
}

async fn seed_categoria_repo(db: Arc<storage::Db>) -> Id<Categoria> {
    let cat = CategoriaRepo::new(Arc::clone(&db))
        .create(NewCategoria {
            nombre: "Fijaciones".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("categoria");
    cat.id
}

fn new_perno(categoria: Id<Categoria>, stock_inicial: &str) -> NewProducto {
    NewProducto {
        sku: "PER-M8X40-ZN".into(),
        nombre: "Perno hexagonal M8 x 40 zincado".into(),
        tipo: "Perno".into(),
        descripcion: None,
        marca: None,
        categoria,
        atributos: vec![],
        codigos: vec![],
        unidad_medida: UnidadMedida::Unidad,
        precio: "120.50".parse().expect("precio"),
        ubicacion: None,
        stock_inicial: stock_inicial.parse().expect("stock"),
    }
}

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
async fn product_repo_creates_perno_with_stock_50() {
    let db = Arc::new(test_db().await);
    let categoria = seed_categoria_repo(Arc::clone(&db)).await;
    let product = product_repo(Arc::clone(&db))
        .create(new_perno(categoria, "50"))
        .await
        .expect("create");

    assert_eq!(product.sku.as_str(), "PER-M8X40-ZN");
    assert_eq!(product.nombre, "Perno hexagonal M8 x 40 zincado");
    assert_eq!(product.version, 1);
    assert!(product.activo);
    assert_eq!(product.stock.to_string(), "50");
    assert!(!product.id.to_string().contains(':'));

    assert_eq!(db.record_count("movimiento_stock").await.expect("movs"), 1);

    db.query(&format!(
        "SELECT tipo, cantidad, stock_resultante FROM movimiento_stock WHERE producto = type::thing('producto', '{}');",
        product.id
    ))
    .await
    .expect("movimiento");

    let out_before = db.record_count("outbox").await.expect("outbox");
    assert!(out_before >= 1);
    db.query(&format!(
        "SELECT entidad, operacion, motivo, record::id(registro) AS registro FROM outbox WHERE record::id(registro) = '{}' AND motivo = 'stock';",
        product.id
    ))
    .await
    .expect("outbox stock");
}

#[tokio::test]
async fn product_repo_create_zero_stock_has_no_movimiento() {
    let db = Arc::new(test_db().await);
    let categoria = seed_categoria_repo(Arc::clone(&db)).await;
    let product = product_repo(Arc::clone(&db))
        .create(new_perno(categoria, "0"))
        .await
        .expect("create zero stock");

    assert_eq!(product.stock.to_string(), "0");
    assert_eq!(product.version, 1);
    assert_eq!(db.record_count("movimiento_stock").await.expect("movs"), 0);

    db.query(&format!(
        "SELECT entidad, operacion, motivo, record::id(registro) AS registro FROM outbox WHERE record::id(registro) = '{}';",
        product.id
    ))
    .await
    .expect("outbox upsert");
}

#[tokio::test]
async fn product_repo_get_returns_persisted() {
    let db = Arc::new(test_db().await);
    let categoria = seed_categoria_repo(Arc::clone(&db)).await;
    let created = product_repo(Arc::clone(&db))
        .create(new_perno(categoria, "50"))
        .await
        .expect("create");
    let loaded = product_repo(Arc::clone(&db))
        .get(created.id.clone())
        .await
        .expect("get");
    assert_eq!(loaded.id, created.id);
    assert_eq!(loaded.sku.as_str(), created.sku.as_str());
    assert_eq!(loaded.nombre, created.nombre);
    assert_eq!(loaded.stock, created.stock);
    assert_eq!(loaded.version, created.version);
}

#[tokio::test]
async fn product_repo_rejects_duplicate_sku() {
    let db = Arc::new(test_db().await);
    let categoria = seed_categoria_repo(Arc::clone(&db)).await;
    product_repo(Arc::clone(&db))
        .create(new_perno(categoria.clone(), "10"))
        .await
        .expect("first");
    let before_products = db.record_count("producto").await.expect("producto");
    let before_movs = db.record_count("movimiento_stock").await.expect("movs");
    let before_outbox = db.record_count("outbox").await.expect("outbox");

    let err = product_repo(Arc::clone(&db))
        .create(new_perno(categoria, "20"))
        .await
        .expect_err("duplicate sku");
    assert_duplicate_sku(err);

    assert_eq!(
        db.record_count("producto").await.expect("producto"),
        before_products
    );
    assert_eq!(
        db.record_count("movimiento_stock").await.expect("movs"),
        before_movs
    );
    assert_eq!(
        db.record_count("outbox").await.expect("outbox"),
        before_outbox
    );
}

#[tokio::test]
async fn product_repo_rejects_negative_stock_inicial() {
    let db = Arc::new(test_db().await);
    let categoria = seed_categoria_repo(Arc::clone(&db)).await;
    let before_products = db.record_count("producto").await.expect("producto");
    let before_movs = db.record_count("movimiento_stock").await.expect("movs");
    let before_outbox = db.record_count("outbox").await.expect("outbox");
    let mut input = new_perno(categoria, "0");
    input.stock_inicial = "-1".parse().expect("neg");
    let err = product_repo(Arc::clone(&db))
        .create(input)
        .await
        .expect_err("negative stock");
    assert_eq!(err.code(), DomainCode::InvalidInput);
    assert_eq!(
        db.record_count("producto").await.expect("producto"),
        before_products
    );
    assert_eq!(
        db.record_count("movimiento_stock").await.expect("movs"),
        before_movs
    );
    assert_eq!(
        db.record_count("outbox").await.expect("outbox"),
        before_outbox
    );
}

#[tokio::test]
async fn product_repo_get_unknown_id_is_not_found() {
    let db = Arc::new(test_db().await);
    let missing = Id::<Producto>::generate();
    let err = product_repo(Arc::clone(&db))
        .get(missing)
        .await
        .expect_err("not found");
    assert_eq!(err.code(), DomainCode::NotFound);
}

fn assert_duplicate_sku(err: DomainError) {
    match err {
        DomainError::Conflict {
            code: ConflictCode::DuplicateSku,
            ..
        } => {}
        other => panic!("expected duplicate_sku, got {other:?}"),
    }
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
