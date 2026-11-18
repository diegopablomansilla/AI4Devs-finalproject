//! Integration tests for atributo_def schema (T-02-11) and AttributeDefRepo (T-02-11).

use domain::{
    AtributoDef, AttributeDefRepo, ConflictCode, DomainCode, DomainError, Id, NewAtributoDef,
    TipoAtributo, UpdateAtributoDef,
};

use std::sync::Arc;

use storage::testing::test_db;
use storage::{AtributoDefRepo as StorageAtributoDefRepo, MIGRATION_0004_SQL, checksum_hex};

fn repo(db: Arc<storage::Db>) -> StorageAtributoDefRepo {
    StorageAtributoDefRepo::new(db)
}

fn new_grado() -> NewAtributoDef {
    NewAtributoDef {
        clave: "grado".into(),
        etiqueta: "Grado".into(),
        tipo: TipoAtributo::Enum,
        unidad: None,
        opciones: Some(vec!["2".into(), "5".into(), "8".into()]),
        facetable: true,
        buscable: true,
        orden: 0,
    }
}

async fn create_grado(db: Arc<storage::Db>) -> AtributoDef {
    repo(db).create(new_grado()).await.expect("grado")
}

#[tokio::test]
async fn migrate_applies_0004_atributo_def() {
    let db = Arc::new(test_db().await);
    let rows = db.list_migrations().await.expect("list migrations");
    let v4 = rows
        .iter()
        .find(|r| r.version == 4)
        .expect("migration version 4 must be registered");
    assert_eq!(v4.nombre, "atributo_def");
    assert_eq!(v4.checksum, checksum_hex(MIGRATION_0004_SQL));

    assert_eq!(db.record_count("atributo_def").await.expect("count"), 0);
}

#[tokio::test]
async fn migrate_rerun_keeps_single_0004_record() {
    let db = Arc::new(test_db().await);
    db.migrate()
        .await
        .expect("second migrate must be idempotent");
    let v4_rows: Vec<_> = db
        .list_migrations()
        .await
        .expect("list")
        .into_iter()
        .filter(|r| r.version == 4)
        .collect();
    assert_eq!(v4_rows.len(), 1);
}

#[tokio::test]
async fn atributo_def_rejects_duplicate_clave() {
    let db = Arc::new(test_db().await);
    db.query("CREATE atributo_def:uno CONTENT { clave: 'grado', etiqueta: 'G', tipo: 'enum', opciones: ['2'] };")
        .await
        .expect("first");
    let err = db
        .query("CREATE atributo_def:dos CONTENT { clave: 'grado', etiqueta: 'G2', tipo: 'enum', opciones: ['5'] };")
        .await
        .expect_err("duplicate clave");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("atributo_def").await.expect("count"), 1);
}

#[tokio::test]
async fn atributo_def_rejects_invalid_tipo() {
    let db = Arc::new(test_db().await);
    let err = db
        .query("CREATE atributo_def:bad CONTENT { clave: 'x', etiqueta: 'X', tipo: 'text' };")
        .await
        .expect_err("invalid tipo");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("atributo_def").await.expect("count"), 0);
}

#[tokio::test]
async fn attribute_def_repo_creates_grado() {
    let db = Arc::new(test_db().await);
    let def = create_grado(Arc::clone(&db)).await;
    assert_eq!(def.clave, "grado");
    assert_eq!(def.tipo, TipoAtributo::Enum);
    assert_eq!(def.opciones, Some(vec!["2".into(), "5".into(), "8".into()]));
    assert!(def.facetable);
    assert!(def.activo);
    assert!(!def.id.to_string().contains(':'));
}

#[tokio::test]
async fn attribute_def_repo_get_returns_persisted() {
    let db = Arc::new(test_db().await);
    let created = create_grado(Arc::clone(&db)).await;
    let created_id = created.id.clone();
    let loaded = repo(Arc::clone(&db))
        .get(created_id.clone())
        .await
        .expect("get");
    assert_eq!(loaded.id, created_id);
    assert_eq!(loaded.clave, created.clave);
    assert_eq!(loaded.etiqueta, created.etiqueta);
}

#[tokio::test]
async fn attribute_def_repo_list_all_ordered_by_orden() {
    let db = Arc::new(test_db().await);
    repo(Arc::clone(&db))
        .create(NewAtributoDef {
            clave: "zzz".into(),
            etiqueta: "Z".into(),
            tipo: TipoAtributo::Texto,
            unidad: None,
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 10,
        })
        .await
        .expect("zzz");
    repo(Arc::clone(&db))
        .create(NewAtributoDef {
            clave: "aaa".into(),
            etiqueta: "A".into(),
            tipo: TipoAtributo::Texto,
            unidad: None,
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
        })
        .await
        .expect("aaa");
    let all = repo(Arc::clone(&db)).list_all().await.expect("list");
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].clave, "aaa");
    assert_eq!(all[1].clave, "zzz");
}

#[tokio::test]
async fn attribute_def_repo_create_emits_outbox_upsert() {
    let db = Arc::new(test_db().await);
    let before = db.record_count("outbox").await.expect("outbox count");
    let def = create_grado(Arc::clone(&db)).await;
    assert_eq!(
        db.record_count("outbox").await.expect("outbox count"),
        before + 1
    );
    db.query(&format!(
        "SELECT entidad, operacion, record::id(registro) AS registro FROM outbox WHERE record::id(registro) = '{id}';",
        id = def.id
    ))
    .await
    .expect("outbox row");
}

#[tokio::test]
async fn attribute_def_repo_update_changes_etiqueta() {
    let db = Arc::new(test_db().await);
    let grado = create_grado(Arc::clone(&db)).await;
    let updated = repo(Arc::clone(&db))
        .update(UpdateAtributoDef {
            id: grado.id.clone(),
            clave: "grado".into(),
            etiqueta: "Grado SAE".into(),
            tipo: TipoAtributo::Enum,
            unidad: None,
            opciones: Some(vec!["2".into(), "5".into(), "8".into()]),
            facetable: true,
            buscable: true,
            orden: 0,
        })
        .await
        .expect("update");
    assert_eq!(updated.etiqueta, "Grado SAE");
    assert_eq!(updated.clave, "grado");
}

#[tokio::test]
async fn attribute_def_repo_set_activo_persists_inactive() {
    let db = Arc::new(test_db().await);
    let def = create_grado(Arc::clone(&db)).await;
    let deactivated = repo(Arc::clone(&db))
        .set_activo(def.id.clone(), false)
        .await
        .expect("set inactive");
    assert!(!deactivated.activo);
    let loaded = repo(Arc::clone(&db))
        .get(def.id.clone())
        .await
        .expect("get");
    assert!(!loaded.activo);
    assert_eq!(db.record_count("atributo_def").await.expect("count"), 1);
}

#[tokio::test]
async fn attribute_def_repo_rejects_duplicate_clave() {
    let db = Arc::new(test_db().await);
    repo(Arc::clone(&db))
        .create(new_grado())
        .await
        .expect("first");
    let err = repo(Arc::clone(&db))
        .create(NewAtributoDef {
            clave: "grado".into(),
            etiqueta: "Otro".into(),
            tipo: TipoAtributo::Texto,
            unidad: None,
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
        })
        .await
        .expect_err("duplicate");
    assert_conflict(err);
    assert_eq!(db.record_count("atributo_def").await.expect("count"), 1);
}

#[tokio::test]
async fn attribute_def_repo_update_duplicate_clave_is_conflict() {
    let db = Arc::new(test_db().await);
    let _grado = create_grado(Arc::clone(&db)).await;
    let medida = repo(Arc::clone(&db))
        .create(NewAtributoDef {
            clave: "medida".into(),
            etiqueta: "Medida".into(),
            tipo: TipoAtributo::Texto,
            unidad: None,
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
        })
        .await
        .expect("medida");
    let err = repo(Arc::clone(&db))
        .update(UpdateAtributoDef {
            id: medida.id,
            clave: "grado".into(),
            etiqueta: "Medida".into(),
            tipo: TipoAtributo::Texto,
            unidad: None,
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
        })
        .await
        .expect_err("collision");
    assert_conflict(err);
}

#[tokio::test]
async fn attribute_def_repo_create_enum_without_options_is_invalid_input() {
    let db = Arc::new(test_db().await);
    let err = repo(Arc::clone(&db))
        .create(NewAtributoDef {
            clave: "grado".into(),
            etiqueta: "Grado".into(),
            tipo: TipoAtributo::Enum,
            unidad: None,
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
        })
        .await
        .expect_err("no options");
    assert_eq!(err.code(), DomainCode::InvalidInput);
    assert_eq!(db.record_count("atributo_def").await.expect("count"), 0);
    assert_eq!(db.record_count("outbox").await.expect("outbox"), 0);
}

#[tokio::test]
async fn attribute_def_repo_get_unknown_id_is_not_found() {
    let db = Arc::new(test_db().await);
    let missing = Id::<AtributoDef>::generate();
    let err = repo(Arc::clone(&db))
        .get(missing)
        .await
        .expect_err("not found");
    assert_eq!(err.code(), DomainCode::NotFound);
}

#[tokio::test]
async fn attribute_def_repo_update_unknown_id_is_not_found() {
    let db = Arc::new(test_db().await);
    let missing = Id::<AtributoDef>::generate();
    let err = repo(Arc::clone(&db))
        .update(UpdateAtributoDef {
            id: missing,
            clave: "x".into(),
            etiqueta: "X".into(),
            tipo: TipoAtributo::Texto,
            unidad: None,
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
        })
        .await
        .expect_err("not found");
    assert_eq!(err.code(), DomainCode::NotFound);
}

#[tokio::test]
async fn attribute_def_repo_set_activo_unknown_id_is_not_found() {
    let db = Arc::new(test_db().await);
    let missing = Id::<AtributoDef>::generate();
    let err = repo(Arc::clone(&db))
        .set_activo(missing, false)
        .await
        .expect_err("not found");
    assert_eq!(err.code(), DomainCode::NotFound);
}

#[tokio::test]
async fn attribute_def_repo_create_invalid_clave_is_invalid_input() {
    let db = Arc::new(test_db().await);
    let err = repo(Arc::clone(&db))
        .create(NewAtributoDef {
            clave: "Grado".into(),
            etiqueta: "X".into(),
            tipo: TipoAtributo::Texto,
            unidad: None,
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
        })
        .await
        .expect_err("invalid clave");
    assert_eq!(err.code(), DomainCode::InvalidInput);
    assert_eq!(db.record_count("atributo_def").await.expect("count"), 0);
    assert_eq!(db.record_count("outbox").await.expect("outbox"), 0);
}

fn assert_conflict(err: DomainError) {
    match err {
        DomainError::Conflict {
            code: ConflictCode::Conflict,
            ..
        } => {}
        _ => panic!("expected conflict, got {:?}", err),
    }
    assert_eq!(err.code(), DomainCode::Conflict);
}
