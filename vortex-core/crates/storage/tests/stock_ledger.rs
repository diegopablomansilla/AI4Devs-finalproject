//! Integration tests for `StockLedger` adapter (T-02-24).

use std::sync::{Arc, LazyLock};

use tokio::sync::Mutex;

/// SurrealDB `kv-mem` puede compartir estado global entre instancias en tests paralelos.
static STOCK_LEDGER_SUITE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

async fn suite_lock() -> tokio::sync::MutexGuard<'static, ()> {
    STOCK_LEDGER_SUITE_LOCK.lock().await
}

use domain::{
    DomainCode, DomainError, Id, NewMovimientoStock, NewProducto, ProductRepo, Producto,
    StockLedger, TipoMovimiento, UnidadMedida,
};

use storage::testing::test_db;
use storage::{MovimientoStockRepo, ProductoRepo};

fn stock_ledger(db: Arc<storage::Db>) -> MovimientoStockRepo {
    MovimientoStockRepo::new(db)
}

async fn seed_categoria(db: Arc<storage::Db>) -> Id<domain::Categoria> {
    use domain::{CategoryRepo, NewCategoria};
    use storage::CategoriaRepo;
    CategoriaRepo::new(Arc::clone(&db))
        .create(NewCategoria {
            nombre: "Fijaciones".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("categoria")
        .id
}

fn new_perno(categoria: Id<domain::Categoria>, stock_inicial: &str) -> NewProducto {
    NewProducto {
        sku: format!("SKU-{}", Id::<Producto>::generate()),
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
        terminal: None,
    }
}

async fn seed_producto_stock(db: Arc<storage::Db>, stock_inicial: &str) -> (Id<Producto>, i32) {
    let categoria = seed_categoria(Arc::clone(&db)).await;
    let product = ProductoRepo::new(Arc::clone(&db))
        .create(new_perno(categoria, stock_inicial))
        .await
        .expect("create");
    (product.id, product.version)
}

fn ingreso(producto: Id<Producto>, cantidad: &str, terminal: Option<&str>) -> NewMovimientoStock {
    NewMovimientoStock {
        producto,
        tipo: TipoMovimiento::Ingreso,
        cantidad: cantidad.parse().expect("cantidad"),
        motivo: None,
        venta: None,
        terminal: terminal.map(str::to_string),
    }
}

fn ajuste(producto: Id<Producto>, cantidad: &str, motivo: Option<&str>) -> NewMovimientoStock {
    NewMovimientoStock {
        producto,
        tipo: TipoMovimiento::Ajuste,
        cantidad: cantidad.parse().expect("cantidad"),
        motivo: motivo.map(str::to_string),
        venta: None,
        terminal: None,
    }
}

fn invalid_fields(err: &DomainError) -> &[domain::FieldError] {
    match err {
        DomainError::InvalidInput { errors, .. } => errors,
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}

#[tokio::test]
async fn stock_ledger_append_ingreso_guarda_terminal_caja_1() {
    let _guard = suite_lock().await;
    let db = Arc::new(test_db().await);
    let (producto_id, version) = seed_producto_stock(Arc::clone(&db), "10").await;
    assert_eq!(version, 1);

    let mov = stock_ledger(Arc::clone(&db))
        .append(ingreso(producto_id.clone(), "20", Some("caja-1")))
        .await
        .expect("append ingreso");

    assert_eq!(mov.tipo, TipoMovimiento::Ingreso);
    assert_eq!(mov.cantidad.to_string(), "20");
    assert_eq!(mov.stock_resultante.to_string(), "30");
    assert_eq!(mov.terminal.as_deref(), Some("caja-1"));
    assert!(!mov.id.to_string().contains(':'));

    let ficha = ProductoRepo::new(Arc::clone(&db))
        .get(producto_id.clone())
        .await
        .expect("get");
    assert_eq!(ficha.producto.stock.to_string(), "30");
    assert_eq!(ficha.producto.version, 1);

    db.query(&format!(
        "IF (SELECT count() FROM outbox WHERE record::id(registro) = '{}' AND motivo = 'stock' GROUP ALL)[0].count != 2 {{ THROW 'outbox'; }};",
        producto_id
    ))
    .await
    .expect("outbox stock events");
}

#[tokio::test]
async fn stock_ledger_append_ajuste_a_cero_con_motivo() {
    let _guard = suite_lock().await;
    let db = Arc::new(test_db().await);
    let (producto_id, _) = seed_producto_stock(Arc::clone(&db), "5").await;

    let mov = stock_ledger(Arc::clone(&db))
        .append(ajuste(producto_id.clone(), "-5", Some("Conteo físico")))
        .await
        .expect("ajuste");

    assert_eq!(mov.tipo, TipoMovimiento::Ajuste);
    assert_eq!(mov.stock_resultante.to_string(), "0");
    assert_eq!(mov.motivo.as_deref(), Some("Conteo físico"));

    let ficha = ProductoRepo::new(db).get(producto_id).await.expect("get");
    assert_eq!(ficha.producto.stock.to_string(), "0");
}

#[tokio::test]
async fn stock_ledger_append_does_not_increment_version() {
    let _guard = suite_lock().await;
    let db = Arc::new(test_db().await);
    let (producto_id, version) = seed_producto_stock(Arc::clone(&db), "50").await;
    assert_eq!(version, 1);

    stock_ledger(Arc::clone(&db))
        .append(ingreso(producto_id.clone(), "1", None))
        .await
        .expect("append");

    let ficha = ProductoRepo::new(db).get(producto_id).await.expect("get");
    assert_eq!(ficha.producto.version, 1);
}

#[tokio::test]
async fn stock_ledger_concurrent_ingresos_both_apply() {
    let _guard = suite_lock().await;
    let db = Arc::new(test_db().await);
    let (producto_id, _) = seed_producto_stock(Arc::clone(&db), "5").await;
    let ledger = stock_ledger(Arc::clone(&db));
    let input = ingreso(producto_id.clone(), "10", None);

    let (a, b) = tokio::join!(ledger.append(input.clone()), ledger.append(input));

    assert!(a.is_ok(), "first append: {a:?}");
    assert!(b.is_ok(), "second append: {b:?}");

    let ficha = ProductoRepo::new(Arc::clone(&db))
        .get(producto_id.clone())
        .await
        .expect("get");
    assert_eq!(ficha.producto.stock.to_string(), "25");
    assert_eq!(db.record_count("movimiento_stock").await.expect("movs"), 3);
}

#[tokio::test]
async fn stock_ledger_list_by_producto_newest_first() {
    let _guard = suite_lock().await;
    let db = Arc::new(test_db().await);
    let (producto_id, _) = seed_producto_stock(Arc::clone(&db), "50").await;
    let ledger = stock_ledger(Arc::clone(&db));

    stock_ledger(Arc::clone(&db))
        .append(ingreso(producto_id.clone(), "10", None))
        .await
        .expect("ingreso");

    let list = ledger.list_by_producto(producto_id).await.expect("list");
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].tipo, TipoMovimiento::Ingreso);
    assert_eq!(list[1].tipo, TipoMovimiento::Inicial);
}

#[tokio::test]
async fn stock_ledger_append_ajuste_minus_8_on_stock_5_is_negative_stock() {
    let _guard = suite_lock().await;
    let db = Arc::new(test_db().await);
    let (producto_id, _) = seed_producto_stock(Arc::clone(&db), "5").await;
    let before_movs = db.record_count("movimiento_stock").await.expect("movs");
    let before_outbox = db.record_count("outbox").await.expect("outbox");

    let err = stock_ledger(Arc::clone(&db))
        .append(ajuste(producto_id.clone(), "-8", Some("Conteo físico")))
        .await
        .expect_err("negative stock");

    assert_eq!(err.code(), DomainCode::NegativeStock);

    let ficha = ProductoRepo::new(Arc::clone(&db))
        .get(producto_id)
        .await
        .expect("get");
    assert_eq!(ficha.producto.stock.to_string(), "5");
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
async fn stock_ledger_append_ajuste_sin_motivo_is_invalid_input() {
    let _guard = suite_lock().await;
    let db = Arc::new(test_db().await);
    let (producto_id, _) = seed_producto_stock(Arc::clone(&db), "5").await;

    let err = stock_ledger(db)
        .append(ajuste(producto_id, "-1", None))
        .await
        .expect_err("sin motivo");

    assert_eq!(err.code(), DomainCode::InvalidInput);
    assert_eq!(invalid_fields(&err)[0].field, "motivo");
}

#[tokio::test]
async fn stock_ledger_append_unknown_product_is_not_found() {
    let _guard = suite_lock().await;
    let db = Arc::new(test_db().await);
    let missing = Id::<Producto>::generate();
    let err = stock_ledger(db)
        .append(ingreso(missing, "1", None))
        .await
        .expect_err("not found");
    assert_eq!(err.code(), DomainCode::NotFound);
}

#[tokio::test]
async fn stock_ledger_append_ingreso_cantidad_negativa_is_invalid_input() {
    let _guard = suite_lock().await;
    let db = Arc::new(test_db().await);
    let (producto_id, _) = seed_producto_stock(Arc::clone(&db), "5").await;

    let err = stock_ledger(db)
        .append(ingreso(producto_id, "-1", None))
        .await
        .expect_err("bad sign");

    assert_eq!(err.code(), DomainCode::InvalidInput);
    assert_eq!(invalid_fields(&err)[0].field, "cantidad");
}

#[tokio::test]
async fn stock_ledger_adapter_has_no_update_or_delete() {
    let src = include_str!("../src/movimiento_stock.rs");
    let prod = src.split("#[cfg(test)]").next().unwrap_or(src);
    assert!(!prod.contains("fn update("));
    assert!(!prod.contains("fn delete("));
    assert!(!prod.contains("CREATE venta"));
    assert!(!prod.contains("contador:venta"));
}
