//! Integration tests for `RelationRepo::create` (T-04-03).

use std::sync::Arc;

use domain::{
    CategoryRepo, ConflictCode, DomainCode, DomainError, Id, NewProducto, NewRelacion, ProductRepo,
    Producto, RelationRepo, TipoRelacion, UnavailableCode, UpdateRelacion,
};
use storage::testing::test_db;
use storage::{CategoriaRepo, ProductoRepo, RelacionRepo};

fn relation_repo(db: Arc<storage::Db>) -> RelacionRepo {
    RelacionRepo::new(db)
}

async fn seed_categoria(db: Arc<storage::Db>) -> domain::Id<domain::Categoria> {
    CategoriaRepo::new(Arc::clone(&db))
        .create(domain::NewCategoria {
            nombre: "Fijaciones".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("categoria")
        .id
}

fn new_perno(categoria: Id<domain::Categoria>) -> NewProducto {
    NewProducto {
        sku: "PER-M8X40-ZN".into(),
        nombre: "Perno hexagonal M8 x 40 zincado".into(),
        tipo: "Perno".into(),
        descripcion: None,
        marca: None,
        categoria,
        atributos: vec![],
        codigos: vec![],
        unidad_medida: domain::UnidadMedida::Unidad,
        precio: "120.50".parse().expect("precio"),
        ubicacion: None,
        stock_inicial: "0".parse().expect("stock"),
        terminal: None,
    }
}

fn new_arandela(categoria: Id<domain::Categoria>) -> NewProducto {
    NewProducto {
        sku: "ARA-M8-ZN".into(),
        nombre: "Arandela M8 zincada".into(),
        tipo: "Arandela".into(),
        descripcion: None,
        marca: None,
        categoria,
        atributos: vec![],
        codigos: vec![],
        unidad_medida: domain::UnidadMedida::Unidad,
        precio: "5.00".parse().expect("precio"),
        ubicacion: None,
        stock_inicial: "0".parse().expect("stock"),
        terminal: None,
    }
}

async fn seed_perno_y_arandela(db: Arc<storage::Db>) -> (Id<Producto>, Id<Producto>) {
    let categoria = seed_categoria(Arc::clone(&db)).await;
    let perno = ProductoRepo::new(Arc::clone(&db))
        .create(new_perno(categoria.clone()))
        .await
        .expect("perno");
    let arandela = ProductoRepo::new(Arc::clone(&db))
        .create(new_arandela(categoria))
        .await
        .expect("arandela");
    (perno.id, arandela.id)
}

async fn edge_count(
    db: &storage::Db,
    table: &str,
    origen: &Id<Producto>,
    destino: &Id<Producto>,
) -> i64 {
    let expr = format!(
        "array::len((SELECT id FROM {table} \
         WHERE in = type::thing('producto', '{}') \
         AND out = type::thing('producto', '{}')))",
        origen, destino
    );
    db.query_return_i64(&expr).await.expect("edge count")
}

async fn grupo_bidireccional_for_edge(
    db: &storage::Db,
    table: &str,
    origen: &Id<Producto>,
    destino: &Id<Producto>,
) -> Option<String> {
    let expr = format!(
        "(SELECT VALUE grupo_bidireccional FROM {table} \
         WHERE in = type::thing('producto', '{}') \
         AND out = type::thing('producto', '{}') LIMIT 1)[0]",
        origen, destino
    );
    let s = db.query_return_string(&expr).await.expect("grupo string");
    if s.is_empty() { None } else { Some(s) }
}

#[tokio::test]
async fn relation_repo_creates_similar_a_to_b_and_b_to_a() {
    let db = Arc::new(test_db().await);
    let (a, b) = seed_perno_y_arandela(Arc::clone(&db)).await;
    let outbox_before = db.record_count("outbox").await.expect("outbox");

    let rel = relation_repo(Arc::clone(&db))
        .create(NewRelacion {
            origen: a.clone(),
            destino: b.clone(),
            tipo: TipoRelacion::SimilarA,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        })
        .await
        .expect("create similar");

    assert!(!rel.id.to_string().contains(':'));
    assert_eq!(rel.origen, a);
    assert_eq!(rel.destino, b);
    assert_eq!(rel.tipo, TipoRelacion::SimilarA);
    assert!(rel.grupo_bidireccional.is_some());

    assert_eq!(db.record_count("similar_a").await.expect("count"), 2);
    assert_eq!(edge_count(&db, "similar_a", &a, &b).await, 1);
    assert_eq!(edge_count(&db, "similar_a", &b, &a).await, 1);

    let g_ab = grupo_bidireccional_for_edge(&db, "similar_a", &a, &b).await;
    let g_ba = grupo_bidireccional_for_edge(&db, "similar_a", &b, &a).await;
    assert_eq!(g_ab, g_ba);
    assert_eq!(
        db.record_count("outbox").await.expect("outbox"),
        outbox_before
    );
}

#[tokio::test]
async fn relation_repo_creates_complement_perno_to_arandela() {
    let db = Arc::new(test_db().await);
    let (perno, arandela) = seed_perno_y_arandela(Arc::clone(&db)).await;

    let rel = relation_repo(Arc::clone(&db))
        .create(NewRelacion {
            origen: perno.clone(),
            destino: arandela.clone(),
            tipo: TipoRelacion::Complementa,
            prioridad: 1,
            nota: Some("Arandela plana M8".into()),
            bidireccional: None,
        })
        .await
        .expect("complement");

    assert_eq!(rel.nota.as_deref(), Some("Arandela plana M8"));
    assert!(rel.grupo_bidireccional.is_none());
    assert_eq!(db.record_count("complementa").await.expect("count"), 1);
    assert_eq!(edge_count(&db, "complementa", &perno, &arandela).await, 1);
    assert_eq!(edge_count(&db, "complementa", &arandela, &perno).await, 0);
}

#[tokio::test]
async fn relation_repo_similar_explicit_false_is_unidirectional() {
    let db = Arc::new(test_db().await);
    let (a, b) = seed_perno_y_arandela(Arc::clone(&db)).await;

    let rel = relation_repo(Arc::clone(&db))
        .create(NewRelacion {
            origen: a.clone(),
            destino: b.clone(),
            tipo: TipoRelacion::SimilarA,
            prioridad: 1,
            nota: None,
            bidireccional: Some(false),
        })
        .await
        .expect("unidirectional similar");

    assert!(rel.grupo_bidireccional.is_none());
    assert_eq!(db.record_count("similar_a").await.expect("count"), 1);
    assert_eq!(edge_count(&db, "similar_a", &a, &b).await, 1);
    assert_eq!(edge_count(&db, "similar_a", &b, &a).await, 0);
}

#[tokio::test]
async fn relation_repo_complement_explicit_true_is_bidirectional() {
    let db = Arc::new(test_db().await);
    let (a, b) = seed_perno_y_arandela(Arc::clone(&db)).await;

    relation_repo(Arc::clone(&db))
        .create(NewRelacion {
            origen: a.clone(),
            destino: b.clone(),
            tipo: TipoRelacion::Complementa,
            prioridad: 1,
            nota: None,
            bidireccional: Some(true),
        })
        .await
        .expect("bidirectional complement");

    assert_eq!(db.record_count("complementa").await.expect("count"), 2);
    let g_ab = grupo_bidireccional_for_edge(&db, "complementa", &a, &b).await;
    let g_ba = grupo_bidireccional_for_edge(&db, "complementa", &b, &a).await;
    assert_eq!(g_ab, g_ba);
}

#[tokio::test]
async fn relation_repo_repeat_similar_is_duplicate_relation() {
    let db = Arc::new(test_db().await);
    let (a, b) = seed_perno_y_arandela(Arc::clone(&db)).await;
    relation_repo(Arc::clone(&db))
        .create(NewRelacion {
            origen: a.clone(),
            destino: b.clone(),
            tipo: TipoRelacion::SimilarA,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        })
        .await
        .expect("first");

    let err = relation_repo(Arc::clone(&db))
        .create(NewRelacion {
            origen: a,
            destino: b,
            tipo: TipoRelacion::SimilarA,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        })
        .await
        .expect_err("duplicate");

    assert!(matches!(
        err,
        DomainError::Conflict {
            code: ConflictCode::DuplicateRelation,
        }
    ));
    assert_eq!(db.record_count("similar_a").await.expect("count"), 2);
}

#[tokio::test]
async fn relation_repo_repeat_complement_is_duplicate_relation() {
    let db = Arc::new(test_db().await);
    let (a, b) = seed_perno_y_arandela(Arc::clone(&db)).await;
    relation_repo(Arc::clone(&db))
        .create(NewRelacion {
            origen: a.clone(),
            destino: b.clone(),
            tipo: TipoRelacion::Complementa,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        })
        .await
        .expect("first");

    let err = relation_repo(Arc::clone(&db))
        .create(NewRelacion {
            origen: a,
            destino: b,
            tipo: TipoRelacion::Complementa,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        })
        .await
        .expect_err("duplicate");

    assert!(matches!(
        err,
        DomainError::Conflict {
            code: ConflictCode::DuplicateRelation,
        }
    ));
    assert_eq!(db.record_count("complementa").await.expect("count"), 1);
}

#[tokio::test]
async fn relation_repo_rejects_self_relation() {
    let db = Arc::new(test_db().await);
    let categoria = seed_categoria(Arc::clone(&db)).await;
    let p = ProductoRepo::new(Arc::clone(&db))
        .create(new_perno(categoria))
        .await
        .expect("product")
        .id;

    let err = relation_repo(Arc::clone(&db))
        .create(NewRelacion {
            origen: p.clone(),
            destino: p,
            tipo: TipoRelacion::Complementa,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        })
        .await
        .expect_err("self");

    assert_eq!(err.code(), DomainCode::SelfRelation);
    assert_eq!(db.record_count("complementa").await.expect("count"), 0);
}

#[tokio::test]
async fn relation_repo_missing_destino_is_not_found() {
    let db = Arc::new(test_db().await);
    let categoria = seed_categoria(Arc::clone(&db)).await;
    let a = ProductoRepo::new(Arc::clone(&db))
        .create(new_perno(categoria))
        .await
        .expect("product")
        .id;
    let missing = Id::<Producto>::generate();

    let err = relation_repo(Arc::clone(&db))
        .create(NewRelacion {
            origen: a,
            destino: missing,
            tipo: TipoRelacion::SustituiblePor,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        })
        .await
        .expect_err("missing destino");

    assert_eq!(err.code(), DomainCode::NotFound);
    assert_eq!(db.record_count("sustituible_por").await.expect("count"), 0);
}

#[tokio::test]
async fn relation_repo_update_delete_list_are_unavailable() {
    let db = Arc::new(test_db().await);
    let repo = relation_repo(Arc::clone(&db));
    let rid = Id::<domain::Relacion>::generate();
    let pid = Id::<Producto>::generate();

    let u = repo
        .update(UpdateRelacion {
            id: rid.clone(),
            tipo: TipoRelacion::SimilarA,
            prioridad: Some(2),
            nota: None,
        })
        .await
        .expect_err("update");
    assert!(matches!(
        u,
        DomainError::Unavailable {
            code: UnavailableCode::Unavailable
        }
    ));

    let d = repo
        .delete(TipoRelacion::SimilarA, rid)
        .await
        .expect_err("delete");
    assert!(matches!(
        d,
        DomainError::Unavailable {
            code: UnavailableCode::Unavailable
        }
    ));

    let l = repo.list_for_product(pid).await.expect_err("list");
    assert!(matches!(
        l,
        DomainError::Unavailable {
            code: UnavailableCode::Unavailable
        }
    ));
}
