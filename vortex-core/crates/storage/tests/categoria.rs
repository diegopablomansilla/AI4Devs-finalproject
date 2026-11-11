//! Integration tests for categoria schema (T-02-01) and CategoryRepo (T-02-03).

use domain::Categoria;
use domain::{
    CategoryRepo, ConflictCode, DomainCode, DomainError, Id, InvalidInputCode, MAX_DEPTH,
    NewCategoria, UpdateCategoria,
};

use storage::testing::test_db;
use storage::{CategoriaRepo as StorageCategoryRepo, MIGRATION_0002_SQL, checksum_hex};

fn repo(db: &storage::Db) -> StorageCategoryRepo<'_> {
    StorageCategoryRepo::new(db)
}

async fn create_fijaciones(db: &storage::Db) -> Categoria {
    repo(db)
        .create(NewCategoria {
            nombre: "Fijaciones".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("Fijaciones")
}

#[tokio::test]
async fn migrate_applies_0002_categoria() {
    let db = test_db().await;
    let rows = db.list_migrations().await.expect("list migrations");
    let v2 = rows
        .iter()
        .find(|r| r.version == 2)
        .expect("migration version 2 must be registered");
    assert_eq!(v2.nombre, "categoria");
    assert_eq!(v2.checksum, checksum_hex(MIGRATION_0002_SQL));

    assert_eq!(
        db.record_count("categoria").await.expect("categoria count"),
        0
    );
}

#[tokio::test]
async fn migrate_rerun_keeps_single_0002_record() {
    let db = test_db().await;
    db.migrate()
        .await
        .expect("second migrate must be idempotent");
    let v2_rows: Vec<_> = db
        .list_migrations()
        .await
        .expect("list")
        .into_iter()
        .filter(|r| r.version == 2)
        .collect();
    assert_eq!(v2_rows.len(), 1);
}

#[tokio::test]
async fn categoria_allows_same_slug_under_different_padres() {
    let db = test_db().await;
    db.query(
        "CREATE categoria:padre_a CONTENT { nombre: 'Padre A', slug: 'padre-a', ruta: ['Padre A'] }; \
         CREATE categoria:padre_b CONTENT { nombre: 'Padre B', slug: 'padre-b', ruta: ['Padre B'] }; \
         CREATE categoria:hija_a CONTENT { nombre: 'Hija', slug: 'hija', padre: categoria:padre_a, ruta: ['Padre A', 'Hija'] }; \
         CREATE categoria:hija_b CONTENT { nombre: 'Hija', slug: 'hija', padre: categoria:padre_b, ruta: ['Padre B', 'Hija'] };",
    )
    .await
    .expect("same slug under different padres");
    assert_eq!(db.record_count("categoria").await.expect("count"), 4);
}

#[tokio::test]
async fn categoria_rejects_duplicate_padre_slug() {
    let db = test_db().await;
    db.query(
        "CREATE categoria:padre CONTENT { nombre: 'Padre', slug: 'padre', ruta: ['Padre'] }; \
         CREATE categoria:primera CONTENT { nombre: 'Una', slug: 'child', padre: categoria:padre, ruta: ['Padre', 'Una'] };",
    )
    .await
    .expect("first child");
    let err = db
        .query(
            "CREATE categoria:segunda CONTENT { nombre: 'Otra', slug: 'child', padre: categoria:padre, ruta: ['Padre', 'Otra'] };",
        )
        .await
        .expect_err("duplicate padre+slug must fail");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("categoria").await.expect("count"), 2);
}

#[tokio::test]
async fn categoria_rejects_empty_nombre() {
    let db = test_db().await;
    let err = db
        .query("CREATE categoria:bad CONTENT { nombre: '', slug: 'bad', ruta: [] };")
        .await
        .expect_err("empty nombre must fail");
    let msg = err.to_string();
    assert!(!msg.is_empty());
    assert_eq!(db.record_count("categoria").await.expect("count"), 0);
}

// --- T-02-03 CategoryRepo integration ---

#[tokio::test]
async fn category_repo_creates_root_fijaciones() {
    let db = test_db().await;
    let cat = create_fijaciones(&db).await;
    assert_eq!(cat.nombre, "Fijaciones");
    assert_eq!(cat.ruta, vec!["Fijaciones"]);
    assert!(cat.padre.is_none());
    assert!(cat.ancestros.is_empty());
    assert_eq!(cat.slug.as_str(), "fijaciones");
    assert!(!cat.id.to_string().contains(':'));
    assert!(cat.activo);
}

#[tokio::test]
async fn category_repo_creates_child_pernos_with_ruta() {
    let db = test_db().await;
    let fijaciones = create_fijaciones(&db).await;
    let fijaciones_id = fijaciones.id.clone();
    let pernos = repo(&db)
        .create(NewCategoria {
            nombre: "Pernos".into(),
            slug: None,
            padre: Some(fijaciones_id.clone()),
        })
        .await
        .expect("Pernos");
    assert_eq!(pernos.ruta, vec!["Fijaciones", "Pernos"]);
    assert_eq!(pernos.ancestros, vec![fijaciones_id.clone()]);
    assert_eq!(pernos.slug.as_str(), "pernos");
    assert_eq!(pernos.padre, Some(fijaciones_id));
}

#[tokio::test]
async fn category_repo_get_returns_persisted() {
    let db = test_db().await;
    let created = create_fijaciones(&db).await;
    let created_id = created.id.clone();
    let loaded = repo(&db).get(created_id.clone()).await.expect("get");
    assert_eq!(loaded.id, created_id);
    assert_eq!(loaded.nombre, created.nombre);
    assert_eq!(loaded.ruta, created.ruta);
}

#[tokio::test]
async fn category_repo_list_all_includes_root_and_child() {
    let db = test_db().await;
    let fijaciones = create_fijaciones(&db).await;
    let _pernos = repo(&db)
        .create(NewCategoria {
            nombre: "Pernos".into(),
            slug: None,
            padre: Some(fijaciones.id),
        })
        .await
        .expect("Pernos");
    let all = repo(&db).list_all().await.expect("list_all");
    assert_eq!(all.len(), 2);
    assert!(all.iter().any(|c| c.nombre == "Fijaciones"));
    assert!(all.iter().any(|c| c.nombre == "Pernos"));
}

#[tokio::test]
async fn category_repo_create_emits_outbox_upsert() {
    let db = test_db().await;
    let before = db.record_count("outbox").await.expect("outbox count");
    let cat = create_fijaciones(&db).await;
    assert_eq!(
        db.record_count("outbox").await.expect("outbox count"),
        before + 1
    );
    db.query(&format!(
        "SELECT entidad, operacion, record::id(registro) AS registro FROM outbox WHERE record::id(registro) = '{id}';",
        id = cat.id
    ))
    .await
    .expect("outbox row");
}

#[tokio::test]
async fn category_repo_allows_same_slug_under_different_padres() {
    let db = test_db().await;
    let a = repo(&db)
        .create(NewCategoria {
            nombre: "Padre A".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("padre A");
    let b = repo(&db)
        .create(NewCategoria {
            nombre: "Padre B".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("padre B");
    repo(&db)
        .create(NewCategoria {
            nombre: "Hija".into(),
            slug: Some(domain::Slug::parse("hija").expect("slug")),
            padre: Some(a.id),
        })
        .await
        .expect("hija A");
    repo(&db)
        .create(NewCategoria {
            nombre: "Hija".into(),
            slug: Some(domain::Slug::parse("hija").expect("slug")),
            padre: Some(b.id),
        })
        .await
        .expect("hija B");
    assert_eq!(db.record_count("categoria").await.expect("count"), 4);
}

#[tokio::test]
async fn category_repo_rejects_duplicate_slug_under_same_padre() {
    let db = test_db().await;
    let padre = create_fijaciones(&db).await;
    let padre_id = padre.id.clone();
    repo(&db)
        .create(NewCategoria {
            nombre: "Uno".into(),
            slug: Some(domain::Slug::parse("dup-child").expect("slug")),
            padre: Some(padre_id.clone()),
        })
        .await
        .expect("first");
    let err = repo(&db)
        .create(NewCategoria {
            nombre: "Otro".into(),
            slug: Some(domain::Slug::parse("dup-child").expect("slug")),
            padre: Some(padre_id),
        })
        .await
        .expect_err("duplicate");
    assert!(matches!(
        err,
        DomainError::Conflict {
            code: ConflictCode::DuplicateSlug
        }
    ));
    assert_eq!(db.record_count("categoria").await.expect("count"), 2);
    assert_eq!(db.record_count("outbox").await.expect("outbox"), 2);
}

#[tokio::test]
async fn category_repo_rejects_duplicate_root_slug() {
    let db = test_db().await;
    create_fijaciones(&db).await;
    let err = repo(&db)
        .create(NewCategoria {
            nombre: "Otra Fijaciones".into(),
            slug: Some(domain::Slug::parse("fijaciones").expect("slug")),
            padre: None,
        })
        .await
        .expect_err("duplicate root slug");
    assert!(matches!(
        err,
        DomainError::Conflict {
            code: ConflictCode::DuplicateSlug
        }
    ));
    assert_eq!(db.record_count("categoria").await.expect("count"), 1);
}

#[tokio::test]
async fn category_repo_get_unknown_id_is_not_found() {
    let db = test_db().await;
    let missing = Id::<Categoria>::generate();
    let err = repo(&db).get(missing).await.expect_err("not found");
    assert_eq!(err.code(), DomainCode::NotFound);
}

#[tokio::test]
async fn category_repo_create_missing_padre_is_not_found() {
    let db = test_db().await;
    let ghost = Id::<Categoria>::generate();
    let before = db.record_count("categoria").await.expect("count");
    let out_before = db.record_count("outbox").await.expect("outbox");
    let err = repo(&db)
        .create(NewCategoria {
            nombre: "Huerfana".into(),
            slug: None,
            padre: Some(ghost),
        })
        .await
        .expect_err("missing padre");
    assert_eq!(err.code(), DomainCode::NotFound);
    assert_eq!(db.record_count("categoria").await.expect("count"), before);
    assert_eq!(db.record_count("outbox").await.expect("outbox"), out_before);
}

#[tokio::test]
async fn category_repo_rejects_depth_five() {
    let db = test_db().await;
    let mut padre: Option<Id<Categoria>> = None;
    let names = ["Nivel1", "Nivel2", "Nivel3", "Nivel4"];
    for name in names {
        let cat = repo(&db)
            .create(NewCategoria {
                nombre: name.into(),
                slug: None,
                padre,
            })
            .await
            .expect(name);
        padre = Some(cat.id);
    }
    let depth4 = padre.expect("depth 4 parent");
    let err = repo(&db)
        .create(NewCategoria {
            nombre: "Nivel5".into(),
            slug: None,
            padre: Some(depth4),
        })
        .await
        .expect_err("depth 5");
    assert!(matches!(
        err,
        DomainError::InvalidInput {
            code: InvalidInputCode::InvalidInput,
            ..
        }
    ));
    assert_eq!(
        db.record_count("categoria").await.expect("count"),
        MAX_DEPTH as usize
    );
}

// --- T-02-04 CategoryRepo::update ---

async fn create_fijaciones_pernos_hex(db: &storage::Db) -> (Categoria, Categoria, Categoria) {
    let fijaciones = create_fijaciones(db).await;
    let pernos = repo(db)
        .create(NewCategoria {
            nombre: "Pernos".into(),
            slug: None,
            padre: Some(fijaciones.id.clone()),
        })
        .await
        .expect("Pernos");
    let hex = repo(db)
        .create(NewCategoria {
            nombre: "Hex".into(),
            slug: None,
            padre: Some(pernos.id.clone()),
        })
        .await
        .expect("Hex");
    (fijaciones, pernos, hex)
}

#[tokio::test]
async fn category_repo_renames_three_level_branch_ruta() {
    let db = test_db().await;
    let (fijaciones, _pernos, _hex) = create_fijaciones_pernos_hex(&db).await;
    let updated = repo(&db)
        .update(UpdateCategoria {
            id: fijaciones.id.clone(),
            nombre: "Tornillería".into(),
            padre: None,
        })
        .await
        .expect("rename branch");
    assert_eq!(updated.ruta, vec!["Tornillería"]);
    let pernos = repo(&db)
        .get(
            repo(&db)
                .list_all()
                .await
                .expect("list")
                .into_iter()
                .find(|c| c.nombre == "Pernos")
                .expect("pernos")
                .id,
        )
        .await
        .expect("pernos");
    assert_eq!(pernos.ruta, vec!["Tornillería", "Pernos"]);
    let hex = repo(&db)
        .get(
            repo(&db)
                .list_all()
                .await
                .expect("list")
                .into_iter()
                .find(|c| c.nombre == "Hex")
                .expect("hex")
                .id,
        )
        .await
        .expect("hex");
    assert_eq!(hex.ruta, vec!["Tornillería", "Pernos", "Hex"]);
}

#[tokio::test]
async fn category_repo_update_emits_outbox_per_affected() {
    let db = test_db().await;
    let (fijaciones, _, _) = create_fijaciones_pernos_hex(&db).await;
    let before = db.record_count("outbox").await.expect("outbox");
    repo(&db)
        .update(UpdateCategoria {
            id: fijaciones.id,
            nombre: "Tornillería".into(),
            padre: None,
        })
        .await
        .expect("rename");
    assert_eq!(db.record_count("outbox").await.expect("outbox"), before + 3);
}

#[tokio::test]
async fn category_repo_move_under_descendant_is_category_cycle() {
    let db = test_db().await;
    let (fijaciones, pernos, _) = create_fijaciones_pernos_hex(&db).await;
    let before = db.record_count("outbox").await.expect("outbox");
    let cat_before = db.record_count("categoria").await.expect("count");
    let err = repo(&db)
        .update(UpdateCategoria {
            id: fijaciones.id,
            nombre: "Fijaciones".into(),
            padre: Some(pernos.id),
        })
        .await
        .expect_err("cycle");
    assert_eq!(err.code(), DomainCode::CategoryCycle);
    assert_eq!(
        db.record_count("categoria").await.expect("count"),
        cat_before
    );
    assert_eq!(db.record_count("outbox").await.expect("outbox"), before);
}

#[tokio::test]
async fn category_repo_move_under_self_is_category_cycle() {
    let db = test_db().await;
    let (fijaciones, _, _) = create_fijaciones_pernos_hex(&db).await;
    let err = repo(&db)
        .update(UpdateCategoria {
            id: fijaciones.id.clone(),
            nombre: "Fijaciones".into(),
            padre: Some(fijaciones.id),
        })
        .await
        .expect_err("self cycle");
    assert_eq!(err.code(), DomainCode::CategoryCycle);
}

#[tokio::test]
async fn category_repo_update_unknown_id_is_not_found() {
    let db = test_db().await;
    let missing = Id::<Categoria>::generate();
    let err = repo(&db)
        .update(UpdateCategoria {
            id: missing,
            nombre: "X".into(),
            padre: None,
        })
        .await
        .expect_err("not found");
    assert_eq!(err.code(), DomainCode::NotFound);
}

#[tokio::test]
async fn category_repo_update_missing_padre_is_not_found() {
    let db = test_db().await;
    let (fijaciones, _, _) = create_fijaciones_pernos_hex(&db).await;
    let ghost = Id::<Categoria>::generate();
    let out_before = db.record_count("outbox").await.expect("outbox");
    let err = repo(&db)
        .update(UpdateCategoria {
            id: fijaciones.id,
            nombre: "Fijaciones".into(),
            padre: Some(ghost),
        })
        .await
        .expect_err("missing padre");
    assert_eq!(err.code(), DomainCode::NotFound);
    assert_eq!(db.record_count("outbox").await.expect("outbox"), out_before);
}

#[tokio::test]
async fn category_repo_move_rejects_depth_five() {
    let db = test_db().await;
    let solo = repo(&db)
        .create(NewCategoria {
            nombre: "Suelta".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("suelta");
    let mut padre: Option<Id<Categoria>> = None;
    for name in ["B1", "B2", "B3", "B4"] {
        let cat = repo(&db)
            .create(NewCategoria {
                nombre: name.into(),
                slug: None,
                padre,
            })
            .await
            .expect(name);
        padre = Some(cat.id);
    }
    let depth4 = padre.expect("b4");
    let err = repo(&db)
        .update(UpdateCategoria {
            id: solo.id,
            nombre: "Suelta".into(),
            padre: Some(depth4),
        })
        .await
        .expect_err("depth");
    assert!(matches!(
        err,
        DomainError::InvalidInput {
            code: InvalidInputCode::InvalidInput,
            ..
        }
    ));
}

#[tokio::test]
async fn category_repo_rename_rejects_duplicate_slug() {
    let db = test_db().await;
    repo(&db)
        .create(NewCategoria {
            nombre: "Tornillería".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("tornilleria");
    let fijaciones = create_fijaciones(&db).await;
    let err = repo(&db)
        .update(UpdateCategoria {
            id: fijaciones.id,
            nombre: "Tornillería".into(),
            padre: None,
        })
        .await
        .expect_err("duplicate");
    assert!(matches!(
        err,
        DomainError::Conflict {
            code: ConflictCode::DuplicateSlug
        }
    ));
}
