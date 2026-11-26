//! Integration tests for `RelationRepo` (T-04-03 create; T-04-04 update/delete/list).

use std::sync::Arc;

use domain::{
    CategoryRepo, ConflictCode, DomainCode, DomainError, Id, NewProducto, NewRelacion, ProductRepo,
    Producto, RelationRepo, TipoRelacion, UpdateRelacion,
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

async fn prioridad_for_edge(
    db: &storage::Db,
    table: &str,
    origen: &Id<Producto>,
    destino: &Id<Producto>,
) -> i32 {
    let expr = format!(
        "(SELECT VALUE prioridad FROM {table} \
         WHERE in = type::thing('producto', '{}') \
         AND out = type::thing('producto', '{}') LIMIT 1)[0]",
        origen, destino
    );
    db.query_return_i64(&expr).await.expect("prioridad") as i32
}

async fn edge_id_for(
    db: &storage::Db,
    table: &str,
    origen: &Id<Producto>,
    destino: &Id<Producto>,
) -> Id<domain::Relacion> {
    let expr = format!(
        "(SELECT VALUE record::id(id) FROM {table} \
         WHERE in = type::thing('producto', '{}') \
         AND out = type::thing('producto', '{}') LIMIT 1)[0]",
        origen, destino
    );
    let raw = db.query_return_string(&expr).await.expect("edge id");
    let ulid = raw.rsplit(':').next().unwrap_or(raw.as_str());
    Id::from_str(ulid).expect("ulid")
}

use std::str::FromStr;

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
async fn relation_repo_update_priority_from_b_updates_both_edges() {
    let db = Arc::new(test_db().await);
    let (a, b) = seed_perno_y_arandela(Arc::clone(&db)).await;
    let repo = relation_repo(Arc::clone(&db));
    repo.create(NewRelacion {
        origen: a.clone(),
        destino: b.clone(),
        tipo: TipoRelacion::SimilarA,
        prioridad: 1,
        nota: None,
        bidireccional: None,
    })
    .await
    .expect("create");

    let edge_ba = edge_id_for(&db, "similar_a", &b, &a).await;
    let updated = repo
        .update(UpdateRelacion {
            id: edge_ba,
            tipo: TipoRelacion::SimilarA,
            prioridad: Some(3),
            nota: None,
        })
        .await
        .expect("update from B");

    assert_eq!(updated.prioridad, 3);
    assert_eq!(updated.origen, b);
    assert_eq!(updated.destino, a);
    assert_eq!(prioridad_for_edge(&db, "similar_a", &a, &b).await, 3);
    assert_eq!(prioridad_for_edge(&db, "similar_a", &b, &a).await, 3);
}

#[tokio::test]
async fn relation_repo_update_unidirectional_nota_leaves_no_inverse() {
    let db = Arc::new(test_db().await);
    let (perno, arandela) = seed_perno_y_arandela(Arc::clone(&db)).await;
    let repo = relation_repo(Arc::clone(&db));
    let rel = repo
        .create(NewRelacion {
            origen: perno.clone(),
            destino: arandela.clone(),
            tipo: TipoRelacion::Complementa,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        })
        .await
        .expect("create");

    let updated = repo
        .update(UpdateRelacion {
            id: rel.id,
            tipo: TipoRelacion::Complementa,
            prioridad: None,
            nota: Some(Some("Arandela plana M8".into())),
        })
        .await
        .expect("update nota");

    assert_eq!(updated.nota.as_deref(), Some("Arandela plana M8"));
    assert_eq!(updated.prioridad, 1);
    assert_eq!(db.record_count("complementa").await.expect("count"), 1);
    assert_eq!(edge_count(&db, "complementa", &arandela, &perno).await, 0);
}

#[tokio::test]
async fn relation_repo_delete_from_b_removes_both_edges() {
    let db = Arc::new(test_db().await);
    let (a, b) = seed_perno_y_arandela(Arc::clone(&db)).await;
    let repo = relation_repo(Arc::clone(&db));
    repo.create(NewRelacion {
        origen: a.clone(),
        destino: b.clone(),
        tipo: TipoRelacion::SimilarA,
        prioridad: 1,
        nota: None,
        bidireccional: None,
    })
    .await
    .expect("create");

    let edge_ba = edge_id_for(&db, "similar_a", &b, &a).await;
    repo.delete(TipoRelacion::SimilarA, edge_ba)
        .await
        .expect("delete from B");

    assert_eq!(db.record_count("similar_a").await.expect("count"), 0);
    assert_eq!(edge_count(&db, "similar_a", &a, &b).await, 0);
    assert_eq!(edge_count(&db, "similar_a", &b, &a).await, 0);
}

#[tokio::test]
async fn relation_repo_delete_unidirectional_leaves_other_types() {
    let db = Arc::new(test_db().await);
    let (perno, arandela) = seed_perno_y_arandela(Arc::clone(&db)).await;
    let repo = relation_repo(Arc::clone(&db));
    let complement = repo
        .create(NewRelacion {
            origen: perno.clone(),
            destino: arandela.clone(),
            tipo: TipoRelacion::Complementa,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        })
        .await
        .expect("complement");
    repo.create(NewRelacion {
        origen: perno.clone(),
        destino: arandela.clone(),
        tipo: TipoRelacion::SimilarA,
        prioridad: 1,
        nota: None,
        bidireccional: None,
    })
    .await
    .expect("similar");

    repo.delete(TipoRelacion::Complementa, complement.id)
        .await
        .expect("delete complement");

    assert_eq!(db.record_count("complementa").await.expect("count"), 0);
    assert_eq!(db.record_count("similar_a").await.expect("count"), 2);
}

#[tokio::test]
async fn relation_repo_list_for_product_groups_by_type_with_destino_resumen() {
    let db = Arc::new(test_db().await);
    let (perno, arandela) = seed_perno_y_arandela(Arc::clone(&db)).await;
    let repo = relation_repo(Arc::clone(&db));

    ProductoRepo::new(Arc::clone(&db))
        .set_activo(arandela.clone(), false)
        .await
        .expect("deactivate arandela");

    repo.create(NewRelacion {
        origen: perno.clone(),
        destino: arandela.clone(),
        tipo: TipoRelacion::SimilarA,
        prioridad: 1,
        nota: None,
        bidireccional: Some(false),
    })
    .await
    .expect("similar");
    repo.create(NewRelacion {
        origen: perno.clone(),
        destino: arandela.clone(),
        tipo: TipoRelacion::SustituiblePor,
        prioridad: 2,
        nota: None,
        bidireccional: None,
    })
    .await
    .expect("substitute");
    repo.create(NewRelacion {
        origen: perno.clone(),
        destino: arandela.clone(),
        tipo: TipoRelacion::Complementa,
        prioridad: 3,
        nota: Some("nota".into()),
        bidireccional: None,
    })
    .await
    .expect("complement");

    let list = repo.list_for_product(perno).await.expect("list");

    assert_eq!(list.similar_a.len(), 1);
    assert_eq!(list.sustituible_por.len(), 1);
    assert_eq!(list.complementa.len(), 1);
    assert_eq!(list.similar_a[0].destino.sku, "ARA-M8-ZN");
    assert_eq!(list.similar_a[0].destino.nombre, "Arandela M8 zincada");
    assert!(!list.similar_a[0].destino.activo);
    assert_eq!(list.complementa[0].relacion.nota.as_deref(), Some("nota"));
}

#[tokio::test]
async fn relation_repo_list_for_product_empty_when_no_edges() {
    let db = Arc::new(test_db().await);
    let categoria = seed_categoria(Arc::clone(&db)).await;
    let perno = ProductoRepo::new(Arc::clone(&db))
        .create(new_perno(categoria))
        .await
        .expect("perno")
        .id;

    let list = relation_repo(Arc::clone(&db))
        .list_for_product(perno)
        .await
        .expect("list");

    assert!(list.similar_a.is_empty());
    assert!(list.sustituible_por.is_empty());
    assert!(list.complementa.is_empty());
}

#[tokio::test]
async fn relation_repo_update_unknown_id_is_not_found() {
    let db = Arc::new(test_db().await);
    let missing = Id::<domain::Relacion>::generate();
    let err = relation_repo(Arc::clone(&db))
        .update(UpdateRelacion {
            id: missing,
            tipo: TipoRelacion::SimilarA,
            prioridad: Some(2),
            nota: None,
        })
        .await
        .expect_err("update missing");

    assert_eq!(err.code(), DomainCode::NotFound);
}

#[tokio::test]
async fn relation_repo_delete_unknown_id_is_not_found() {
    let db = Arc::new(test_db().await);
    let missing = Id::<domain::Relacion>::generate();
    let err = relation_repo(Arc::clone(&db))
        .delete(TipoRelacion::SimilarA, missing)
        .await
        .expect_err("delete missing");

    assert_eq!(err.code(), DomainCode::NotFound);
}

#[tokio::test]
async fn relation_repo_update_rejects_prioridad_below_one() {
    let db = Arc::new(test_db().await);
    let (a, b) = seed_perno_y_arandela(Arc::clone(&db)).await;
    let repo = relation_repo(Arc::clone(&db));
    let rel = repo
        .create(NewRelacion {
            origen: a,
            destino: b,
            tipo: TipoRelacion::Complementa,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        })
        .await
        .expect("create");

    let err = repo
        .update(UpdateRelacion {
            id: rel.id,
            tipo: TipoRelacion::Complementa,
            prioridad: Some(0),
            nota: None,
        })
        .await
        .expect_err("prioridad 0");

    assert_eq!(err.code(), DomainCode::InvalidInput);
    assert_eq!(db.record_count("complementa").await.expect("count"), 1);
}

#[tokio::test]
async fn relation_repo_update_rejects_nota_over_500() {
    let db = Arc::new(test_db().await);
    let (a, b) = seed_perno_y_arandela(Arc::clone(&db)).await;
    let repo = relation_repo(Arc::clone(&db));
    let rel = repo
        .create(NewRelacion {
            origen: a,
            destino: b,
            tipo: TipoRelacion::Complementa,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        })
        .await
        .expect("create");

    let nota = "a".repeat(501);
    let err = repo
        .update(UpdateRelacion {
            id: rel.id,
            tipo: TipoRelacion::Complementa,
            prioridad: None,
            nota: Some(Some(nota)),
        })
        .await
        .expect_err("nota long");

    assert_eq!(err.code(), DomainCode::InvalidInput);
}

#[tokio::test]
async fn relation_repo_list_incoming_summarizes_origin() {
    let db = Arc::new(test_db().await);
    let (perno, arandela) = seed_perno_y_arandela(Arc::clone(&db)).await;
    relation_repo(Arc::clone(&db))
        .create(NewRelacion {
            origen: perno.clone(),
            destino: arandela.clone(),
            tipo: TipoRelacion::Complementa,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        })
        .await
        .expect("create");

    let repo = relation_repo(Arc::clone(&db));
    let incoming = repo
        .list_incoming_for_product(arandela.clone())
        .await
        .expect("incoming");
    assert_eq!(incoming.complementa.len(), 1);
    assert_eq!(incoming.similar_a.len(), 0);
    assert_eq!(incoming.complementa[0].relacion.origen, perno);
    assert_eq!(incoming.complementa[0].destino.sku, "PER-M8X40-ZN");
    assert_eq!(
        incoming.complementa[0].destino.nombre,
        "Perno hexagonal M8 x 40 zincado"
    );

    let outgoing = repo.list_for_product(arandela).await.expect("outgoing");
    assert!(outgoing.complementa.is_empty());
}

#[tokio::test]
async fn relation_repo_list_incoming_empty() {
    let db = Arc::new(test_db().await);
    let (perno, _) = seed_perno_y_arandela(Arc::clone(&db)).await;
    let incoming = relation_repo(db)
        .list_incoming_for_product(perno)
        .await
        .expect("incoming");
    assert!(incoming.similar_a.is_empty());
    assert!(incoming.sustituible_por.is_empty());
    assert!(incoming.complementa.is_empty());
}
