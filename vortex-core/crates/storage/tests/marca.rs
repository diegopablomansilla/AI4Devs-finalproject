//! Integration tests for marca schema (T-02-08) and BrandRepo (T-02-08).

use domain::{
    BrandRepo, ConflictCode, DomainCode, DomainError, Id, Marca, NewMarca, Slug, UpdateMarca,
};

use std::sync::Arc;

use storage::testing::test_db;
use storage::{MIGRATION_0003_SQL, MarcaRepo as StorageMarcaRepo, checksum_hex};

fn repo(db: Arc<storage::Db>) -> StorageMarcaRepo {
    StorageMarcaRepo::new(db)
}

async fn create_fixser(db: Arc<storage::Db>) -> Marca {
    repo(db)
        .create(NewMarca {
            nombre: "Fixser".into(),
            slug: None,
        })
        .await
        .expect("Fixser")
}

#[tokio::test]
async fn migrate_applies_0003_marca() {
    let db = Arc::new(test_db().await);
    let rows = db.list_migrations().await.expect("list migrations");
    let v3 = rows
        .iter()
        .find(|r| r.version == 3)
        .expect("migration version 3 must be registered");
    assert_eq!(v3.nombre, "marca");
    assert_eq!(v3.checksum, checksum_hex(MIGRATION_0003_SQL));

    assert_eq!(db.record_count("marca").await.expect("marca count"), 0);
}

#[tokio::test]
async fn migrate_rerun_keeps_single_0003_record() {
    let db = Arc::new(test_db().await);
    db.migrate()
        .await
        .expect("second migrate must be idempotent");
    let v3_rows: Vec<_> = db
        .list_migrations()
        .await
        .expect("list")
        .into_iter()
        .filter(|r| r.version == 3)
        .collect();
    assert_eq!(v3_rows.len(), 1);
}

#[tokio::test]
async fn marca_rejects_duplicate_slug() {
    let db = Arc::new(test_db().await);
    db.query("CREATE marca:uno CONTENT { nombre: 'Uno', slug: 'fixser' };")
        .await
        .expect("first marca");
    let err = db
        .query("CREATE marca:dos CONTENT { nombre: 'Dos', slug: 'fixser' };")
        .await
        .expect_err("duplicate slug must fail");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("marca").await.expect("count"), 1);
}

#[tokio::test]
async fn marca_rejects_empty_nombre() {
    let db = Arc::new(test_db().await);
    let err = db
        .query("CREATE marca:bad CONTENT { nombre: '', slug: 'bad' };")
        .await
        .expect_err("empty nombre must fail");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("marca").await.expect("count"), 0);
}

// --- BrandRepo integration ---

#[tokio::test]
async fn brand_repo_creates_fixser() {
    let db = Arc::new(test_db().await);
    let marca = create_fixser(Arc::clone(&db)).await;
    assert_eq!(marca.nombre, "Fixser");
    assert_eq!(marca.slug.as_str(), "fixser");
    assert!(marca.activo);
    assert!(!marca.id.to_string().contains(':'));
}

#[tokio::test]
async fn brand_repo_get_returns_persisted() {
    let db = Arc::new(test_db().await);
    let created = create_fixser(Arc::clone(&db)).await;
    let created_id = created.id.clone();
    let loaded = repo(Arc::clone(&db))
        .get(created_id.clone())
        .await
        .expect("get");
    assert_eq!(loaded.id, created_id);
    assert_eq!(loaded.nombre, created.nombre);
    assert_eq!(loaded.slug.as_str(), created.slug.as_str());
}

#[tokio::test]
async fn brand_repo_list_all_includes_inactive() {
    let db = Arc::new(test_db().await);
    let active = create_fixser(Arc::clone(&db)).await;
    let other = repo(Arc::clone(&db))
        .create(NewMarca {
            nombre: "Fischer".into(),
            slug: None,
        })
        .await
        .expect("Fischer");
    repo(Arc::clone(&db))
        .set_activo(other.id.clone(), false)
        .await
        .expect("deactivate");
    let all = repo(Arc::clone(&db)).list_all().await.expect("list_all");
    assert_eq!(all.len(), 2);
    assert!(all.iter().any(|m| m.id == active.id && m.activo));
    assert!(all.iter().any(|m| m.id == other.id && !m.activo));
}

#[tokio::test]
async fn brand_repo_create_emits_outbox_upsert() {
    let db = Arc::new(test_db().await);
    let before = db.record_count("outbox").await.expect("outbox count");
    let marca = create_fixser(Arc::clone(&db)).await;
    assert_eq!(
        db.record_count("outbox").await.expect("outbox count"),
        before + 1
    );
    db.query(&format!(
        "SELECT entidad, operacion, record::id(registro) AS registro FROM outbox WHERE record::id(registro) = '{id}';",
        id = marca.id
    ))
    .await
    .expect("outbox row");
}

#[tokio::test]
async fn brand_repo_update_renames_and_regenerates_slug() {
    let db = Arc::new(test_db().await);
    let fixser = create_fixser(Arc::clone(&db)).await;
    let updated = repo(Arc::clone(&db))
        .update(UpdateMarca {
            id: fixser.id.clone(),
            nombre: "Fischer".into(),
            slug: None,
        })
        .await
        .expect("update");
    assert_eq!(updated.nombre, "Fischer");
    assert_eq!(updated.slug.as_str(), "fischer");
}

#[tokio::test]
async fn brand_repo_set_activo_persists_inactive() {
    let db = Arc::new(test_db().await);
    let marca = create_fixser(Arc::clone(&db)).await;
    let deactivated = repo(Arc::clone(&db))
        .set_activo(marca.id.clone(), false)
        .await
        .expect("set inactive");
    assert!(!deactivated.activo);
    let loaded = repo(Arc::clone(&db))
        .get(marca.id.clone())
        .await
        .expect("get");
    assert!(!loaded.activo);
    assert_eq!(db.record_count("marca").await.expect("count"), 1);
}

#[tokio::test]
async fn brand_repo_rejects_duplicate_slug() {
    let db = Arc::new(test_db().await);
    repo(Arc::clone(&db))
        .create(NewMarca {
            nombre: "Fixser".into(),
            slug: Some(Slug::parse("fixser").expect("slug")),
        })
        .await
        .expect("first");
    let err = repo(Arc::clone(&db))
        .create(NewMarca {
            nombre: "Otra Fixser".into(),
            slug: Some(Slug::parse("fixser").expect("slug")),
        })
        .await
        .expect_err("duplicate");
    assert_duplicate_slug(err);
    assert_eq!(db.record_count("marca").await.expect("count"), 1);
}

#[tokio::test]
async fn brand_repo_update_duplicate_slug_is_duplicate_slug() {
    let db = Arc::new(test_db().await);
    let _fixser = create_fixser(Arc::clone(&db)).await;
    let fischer = repo(Arc::clone(&db))
        .create(NewMarca {
            nombre: "Fischer".into(),
            slug: None,
        })
        .await
        .expect("Fischer");
    let err = repo(Arc::clone(&db))
        .update(UpdateMarca {
            id: fischer.id,
            nombre: "Fischer".into(),
            slug: Some(Slug::parse("fixser").expect("slug")),
        })
        .await
        .expect_err("collision");
    assert_duplicate_slug(err);
}

#[tokio::test]
async fn brand_repo_get_unknown_id_is_not_found() {
    let db = Arc::new(test_db().await);
    let missing = Id::<Marca>::generate();
    let err = repo(Arc::clone(&db))
        .get(missing)
        .await
        .expect_err("not found");
    assert_eq!(err.code(), DomainCode::NotFound);
}

#[tokio::test]
async fn brand_repo_update_unknown_id_is_not_found() {
    let db = Arc::new(test_db().await);
    let missing = Id::<Marca>::generate();
    let err = repo(Arc::clone(&db))
        .update(UpdateMarca {
            id: missing,
            nombre: "X".into(),
            slug: None,
        })
        .await
        .expect_err("not found");
    assert_eq!(err.code(), DomainCode::NotFound);
}

#[tokio::test]
async fn brand_repo_set_activo_unknown_id_is_not_found() {
    let db = Arc::new(test_db().await);
    let missing = Id::<Marca>::generate();
    let err = repo(Arc::clone(&db))
        .set_activo(missing, false)
        .await
        .expect_err("not found");
    assert_eq!(err.code(), DomainCode::NotFound);
}

#[tokio::test]
async fn brand_repo_create_blank_nombre_is_invalid_input() {
    let db = Arc::new(test_db().await);
    let err = repo(Arc::clone(&db))
        .create(NewMarca {
            nombre: "   ".into(),
            slug: None,
        })
        .await
        .expect_err("blank");
    assert_eq!(err.code(), DomainCode::InvalidInput);
    assert_eq!(db.record_count("marca").await.expect("count"), 0);
    assert_eq!(db.record_count("outbox").await.expect("outbox"), 0);
}

fn assert_duplicate_slug(err: DomainError) {
    match err {
        DomainError::Conflict {
            code: ConflictCode::DuplicateSlug,
            ..
        } => {}
        _ => panic!("expected duplicate_slug, got {:?}", err),
    }
    assert_eq!(err.code(), DomainCode::DuplicateSlug);
}
