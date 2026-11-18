//! Integration tests for producto schema (T-02-14) and ProductRepo (T-02-18).

use std::sync::Arc;

use domain::{
    AtributoDef, AtributoValor, AttributeDefRepo, BrandRepo, Categoria, CategoryRepo, Codigo,
    ConflictCode, DomainCode, DomainError, Id, Marca, NewAtributoDef, NewCategoria, NewMarca,
    NewProducto, ProductRepo, Producto, TipoAtributo, TipoCodigo, UnidadMedida, UpdateProducto,
    ValorAtributo,
};

use storage::testing::test_db;
use storage::{
    AtributoDefRepo, CategoriaRepo, MIGRATION_0005_SQL, MarcaRepo, ProductoRepo, checksum_hex,
};

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
        terminal: None,
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
async fn storage_create_persists_terminal_on_initial_movement() {
    let db = Arc::new(test_db().await);
    let categoria = seed_categoria_repo(Arc::clone(&db)).await;
    let mut input = new_perno(categoria, "50");
    input.terminal = Some("caja-1".into());
    let product = product_repo(Arc::clone(&db))
        .create(input)
        .await
        .expect("create");

    db.query(&format!(
        "IF (SELECT VALUE terminal FROM movimiento_stock WHERE producto = type::thing('producto', '{}'))[0] != 'caja-1' {{ THROW 'terminal mismatch'; }};",
        product.id
    ))
    .await
    .expect("terminal on movement");
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
    assert_eq!(loaded.producto.id, created.id);
    assert_eq!(loaded.producto.sku.as_str(), created.sku.as_str());
    assert_eq!(loaded.producto.nombre, created.nombre);
    assert_eq!(loaded.producto.stock, created.stock);
    assert_eq!(loaded.producto.version, created.version);
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

fn assert_version_conflict(err: DomainError) {
    match err {
        DomainError::Conflict {
            code: ConflictCode::VersionConflict,
            ..
        } => {}
        other => panic!("expected version_conflict, got {other:?}"),
    }
}

async fn seed_ficha_catalog(db: Arc<storage::Db>) -> (Id<Categoria>, Id<Marca>, Id<AtributoDef>) {
    let parent = CategoriaRepo::new(Arc::clone(&db))
        .create(NewCategoria {
            nombre: "Fijaciones".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("parent categoria");
    let child = CategoriaRepo::new(Arc::clone(&db))
        .create(NewCategoria {
            nombre: "Pernos".into(),
            slug: None,
            padre: Some(parent.id),
        })
        .await
        .expect("child categoria");
    let marca = MarcaRepo::new(Arc::clone(&db))
        .create(NewMarca {
            nombre: "Fixser".into(),
            slug: None,
        })
        .await
        .expect("marca");
    let medida = AtributoDefRepo::new(Arc::clone(&db))
        .create(NewAtributoDef {
            clave: "medida".into(),
            etiqueta: "Medida".into(),
            tipo: TipoAtributo::Texto,
            unidad: Some("mm".into()),
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
        })
        .await
        .expect("medida def");
    (child.id, marca.id, medida.id)
}

fn new_perno_rich(
    categoria: Id<Categoria>,
    marca: Id<Marca>,
    medida_def: Id<AtributoDef>,
    sku: &str,
    stock_inicial: &str,
) -> NewProducto {
    NewProducto {
        sku: sku.into(),
        nombre: "Perno hexagonal M8 x 40 zincado".into(),
        tipo: "Perno".into(),
        descripcion: None,
        marca: Some(marca),
        categoria,
        atributos: vec![AtributoValor {
            def: medida_def,
            valor: ValorAtributo::Texto("M8".into()),
        }],
        codigos: vec![Codigo::try_new(TipoCodigo::Fabricante, "HX-0840Z").expect("codigo")],
        unidad_medida: UnidadMedida::Unidad,
        precio: "120.50".parse().expect("precio"),
        ubicacion: None,
        stock_inicial: stock_inicial.parse().expect("stock"),
        terminal: None,
    }
}

fn update_from_producto(producto: &Producto, version: i32) -> UpdateProducto {
    UpdateProducto {
        id: producto.id.clone(),
        version,
        sku: producto.sku.as_str().to_string(),
        nombre: producto.nombre.clone(),
        tipo: producto.tipo.clone(),
        descripcion: producto.descripcion.clone(),
        marca: producto.marca.clone(),
        categoria: producto.categoria.clone(),
        atributos: producto.atributos.clone(),
        codigos: producto.codigos.clone(),
        unidad_medida: producto.unidad_medida,
        precio: producto.precio,
        ubicacion: producto.ubicacion.clone(),
    }
}

#[tokio::test]
async fn product_repo_get_hydrates_marca_ruta_and_attributes() {
    let db = Arc::new(test_db().await);
    let (categoria, marca, medida) = seed_ficha_catalog(Arc::clone(&db)).await;
    let created = product_repo(Arc::clone(&db))
        .create(new_perno_rich(
            categoria,
            marca,
            medida,
            "PER-M8X40-ZN",
            "0",
        ))
        .await
        .expect("create");
    let ficha = product_repo(Arc::clone(&db))
        .get(created.id.clone())
        .await
        .expect("get");
    assert_eq!(
        ficha.marca.as_ref().map(|m| m.nombre.as_str()),
        Some("Fixser")
    );
    assert_eq!(ficha.categoria.ruta, ["Fijaciones", "Pernos"]);
    assert_eq!(ficha.atributos.len(), 1);
    assert_eq!(ficha.atributos[0].clave, "medida");
    assert_eq!(ficha.atributos[0].etiqueta, "Medida");
    assert_eq!(ficha.atributos[0].unidad.as_deref(), Some("mm"));
    assert!(matches!(
        ficha.atributos[0].valor,
        ValorAtributo::Texto(ref s) if s == "M8"
    ));
}

#[tokio::test]
async fn product_repo_update_increments_version() {
    let db = Arc::new(test_db().await);
    let (categoria, marca, medida) = seed_ficha_catalog(Arc::clone(&db)).await;
    let created = product_repo(Arc::clone(&db))
        .create(new_perno_rich(
            categoria,
            marca,
            medida,
            "PER-M8X40-ZN",
            "0",
        ))
        .await
        .expect("create");
    let mut input = update_from_producto(&created, 1);
    input.precio = "130.00".parse().expect("precio");
    let ficha = product_repo(Arc::clone(&db))
        .update(input)
        .await
        .expect("update");
    assert_eq!(ficha.producto.precio, "130.00".parse().expect("precio"));
    assert_eq!(ficha.producto.version, 2);
}

#[tokio::test]
async fn product_repo_update_does_not_write_stock() {
    let db = Arc::new(test_db().await);
    let (categoria, marca, medida) = seed_ficha_catalog(Arc::clone(&db)).await;
    let created = product_repo(Arc::clone(&db))
        .create(new_perno_rich(
            categoria,
            marca,
            medida,
            "PER-M8X40-ZN",
            "50",
        ))
        .await
        .expect("create");
    assert_eq!(created.stock.to_string(), "50");
    let before_movs = db.record_count("movimiento_stock").await.expect("movs");
    let mut input = update_from_producto(&created, 1);
    input.precio = "125.00".parse().expect("precio");
    let ficha = product_repo(Arc::clone(&db))
        .update(input)
        .await
        .expect("update");
    assert_eq!(ficha.producto.stock.to_string(), "50");
    assert_eq!(
        db.record_count("movimiento_stock").await.expect("movs"),
        before_movs
    );
}

#[tokio::test]
async fn product_repo_update_replaces_atributos_and_codigos() {
    let db = Arc::new(test_db().await);
    let (categoria, marca, medida) = seed_ficha_catalog(Arc::clone(&db)).await;
    let created = product_repo(Arc::clone(&db))
        .create(new_perno_rich(
            categoria,
            marca,
            medida,
            "PER-M8X40-ZN",
            "0",
        ))
        .await
        .expect("create");
    assert_eq!(created.codigos.len(), 1);
    assert_eq!(created.atributos.len(), 1);
    let mut input = update_from_producto(&created, 1);
    input.atributos = vec![];
    input.codigos = vec![Codigo::try_new(TipoCodigo::Oem, "OEM-1").expect("oem")];
    let ficha = product_repo(Arc::clone(&db))
        .update(input)
        .await
        .expect("update");
    assert!(ficha.producto.atributos.is_empty());
    assert_eq!(ficha.producto.codigos.len(), 1);
    assert_eq!(ficha.producto.codigos[0].tipo, TipoCodigo::Oem);
}

#[tokio::test]
async fn product_repo_concurrent_updates_same_version_conflict() {
    let db = Arc::new(test_db().await);
    let (categoria, marca, medida) = seed_ficha_catalog(Arc::clone(&db)).await;
    let created = product_repo(Arc::clone(&db))
        .create(new_perno_rich(
            categoria,
            marca,
            medida,
            "PER-M8X40-ZN",
            "0",
        ))
        .await
        .expect("create");
    let before_outbox = db.record_count("outbox").await.expect("outbox");

    let mut first = update_from_producto(&created, 1);
    first.precio = "130.00".parse().expect("precio");
    product_repo(Arc::clone(&db))
        .update(first)
        .await
        .expect("first update");

    let mut second = update_from_producto(&created, 1);
    second.precio = "140.00".parse().expect("precio");
    let err = product_repo(Arc::clone(&db))
        .update(second)
        .await
        .expect_err("version conflict");
    assert_version_conflict(err);

    let ficha = product_repo(Arc::clone(&db))
        .get(created.id.clone())
        .await
        .expect("reload");
    assert_eq!(ficha.producto.precio, "130.00".parse().expect("precio"));
    assert_eq!(ficha.producto.version, 2);
    assert_eq!(
        db.record_count("outbox").await.expect("outbox"),
        before_outbox + 1
    );
}

#[tokio::test]
async fn product_repo_update_unknown_id_is_not_found() {
    let db = Arc::new(test_db().await);
    let missing = Id::<Producto>::generate();
    let err = product_repo(Arc::clone(&db))
        .update(UpdateProducto {
            id: missing,
            version: 1,
            sku: "X".into(),
            nombre: "X".into(),
            tipo: "Perno".into(),
            descripcion: None,
            marca: None,
            categoria: Id::generate(),
            atributos: vec![],
            codigos: vec![],
            unidad_medida: UnidadMedida::Unidad,
            precio: "1".parse().expect("precio"),
            ubicacion: None,
        })
        .await
        .expect_err("not found");
    assert_eq!(err.code(), DomainCode::NotFound);
}

#[tokio::test]
async fn product_repo_update_rejects_duplicate_sku() {
    let db = Arc::new(test_db().await);
    let (categoria, marca, medida) = seed_ficha_catalog(Arc::clone(&db)).await;
    let a = product_repo(Arc::clone(&db))
        .create(new_perno_rich(
            categoria.clone(),
            marca.clone(),
            medida.clone(),
            "SKU-A",
            "0",
        ))
        .await
        .expect("a");
    let b = product_repo(Arc::clone(&db))
        .create(new_perno_rich(categoria, marca, medida, "SKU-B", "0"))
        .await
        .expect("b");
    let before_outbox = db.record_count("outbox").await.expect("outbox");
    let mut input = update_from_producto(&b, 1);
    input.sku = a.sku.as_str().to_string();
    let err = product_repo(Arc::clone(&db))
        .update(input)
        .await
        .expect_err("duplicate sku");
    assert_duplicate_sku(err);
    let reloaded = product_repo(Arc::clone(&db))
        .get(b.id.clone())
        .await
        .expect("b still");
    assert_eq!(reloaded.producto.sku.as_str(), "SKU-B");
    assert_eq!(
        db.record_count("outbox").await.expect("outbox"),
        before_outbox
    );
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
