//! Full Meilisearch rebuild with atomic swap (T-03-11, HU-03-02).

use domain::{
    AttributeDefRepo, DomainCode, DomainError, FiltroProducto, Id, IndexUid, OUTBOX_BATCH_LIMIT,
    OutboxRecord, OutboxRepo, ProductRepo, TextSearch, build_search_document,
    productos_index_settings,
};

/// Default staging index uid paired with live `productos`.
pub fn default_staging_uid() -> Result<IndexUid, DomainError> {
    IndexUid::try_new("productos_nuevo")
}

fn index_already_exists(err: &DomainError) -> bool {
    let msg = err.to_string().to_ascii_lowercase();
    msg.contains("already exists") || msg.contains("index_already_exists")
}

fn index_missing(err: &DomainError) -> bool {
    err.code() == DomainCode::NotFound
        || err
            .to_string()
            .to_ascii_lowercase()
            .contains("index_not_found")
        || err.to_string().to_ascii_lowercase().contains("not found")
}

async fn delete_staging_if_present<S: TextSearch>(
    search: &S,
    staging: &IndexUid,
) -> Result<(), DomainError> {
    if let Err(e) = search.delete_index(staging).await
        && !index_missing(&e)
    {
        return Err(e);
    }
    Ok(())
}

async fn ensure_live_index<S: TextSearch>(search: &S, live: &IndexUid) -> Result<(), DomainError> {
    if let Err(e) = search.create_index(live).await
        && !index_already_exists(&e)
    {
        return Err(e);
    }
    Ok(())
}

async fn snapshot_last_outbox<O: OutboxRepo>(
    outbox: &O,
) -> Result<Option<Id<OutboxRecord>>, DomainError> {
    let mut after: Option<Id<OutboxRecord>> = None;
    let mut last = None;
    loop {
        let batch = outbox.list_after_meilisearch(after).await?;
        if batch.is_empty() {
            break;
        }
        last = Some(batch.last().expect("non-empty batch").id);
        after = last;
    }
    Ok(last)
}

async fn index_active_products<P, A, S>(
    products: &P,
    attr_defs: &A,
    search: &S,
    staging: &IndexUid,
) -> Result<(), DomainError>
where
    P: ProductRepo,
    A: AttributeDefRepo,
    S: TextSearch,
{
    let defs = attr_defs.list_all().await?;
    let filtro = FiltroProducto {
        categoria: None,
        marca: None,
        activo: true,
    };
    let mut after = None;
    let mut batch_docs = Vec::with_capacity(OUTBOX_BATCH_LIMIT as usize);
    loop {
        let page = products
            .list(filtro.clone(), after, OUTBOX_BATCH_LIMIT)
            .await?;
        if page.is_empty() {
            break;
        }
        for summary in &page {
            let ficha = products.get(summary.id.clone()).await?;
            let doc = build_search_document(
                &ficha.producto,
                ficha.marca.as_ref(),
                &ficha.categoria,
                &defs,
            )?;
            batch_docs.push(doc);
            if batch_docs.len() >= OUTBOX_BATCH_LIMIT as usize {
                search.upsert_documents(staging, &batch_docs).await?;
                batch_docs.clear();
            }
        }
        after = Some(page.last().expect("non-empty page").id.clone());
        if page.len() < OUTBOX_BATCH_LIMIT as usize {
            break;
        }
    }
    if !batch_docs.is_empty() {
        search.upsert_documents(staging, &batch_docs).await?;
    }
    Ok(())
}

/// Rebuilds the live Meilisearch index from SurrealDB masters and swaps atomically.
pub async fn reindex_meilisearch<O, P, A, S>(
    outbox: &O,
    products: &P,
    attr_defs: &A,
    search: &S,
    live: &IndexUid,
    staging: &IndexUid,
) -> Result<(), DomainError>
where
    O: OutboxRepo,
    P: ProductRepo,
    A: AttributeDefRepo,
    S: TextSearch,
{
    let snapshot = snapshot_last_outbox(outbox).await?;

    delete_staging_if_present(search, staging).await?;
    search.create_index(staging).await?;

    let defs = attr_defs.list_all().await?;
    let claves = defs
        .iter()
        .filter(|d| d.activo && d.facetable)
        .map(|d| d.clave.as_str());
    let settings = productos_index_settings(claves)?;
    search.apply_settings(staging, settings).await?;

    ensure_live_index(search, live).await?;
    index_active_products(products, attr_defs, search, staging).await?;

    search.swap_indexes(staging, live).await?;
    let _ = delete_staging_if_present(search, staging).await;

    if let Some(event_id) = snapshot {
        outbox.advance_meilisearch_cursor(event_id).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use domain::{
        AtributoDef, AttributeDefRepo, Categoria, DomainCode, DomainError, FichaProducto,
        FiltroProducto, Id, IndexSettings, IndexUid, Marca, OutboxEntity, OutboxEvent, OutboxOp,
        OutboxRecord, OutboxRepo, OutboxRow, ProductRepo, Producto, ResumenProducto,
        SearchDocument, SearchQuery, TextSearch, Timestamp,
    };

    use super::*;

    struct FakeOutbox {
        rows: Vec<OutboxRow>,
        cursor: Arc<Mutex<Option<Id<OutboxRecord>>>>,
    }

    impl FakeOutbox {
        fn with_rows(rows: Vec<OutboxRow>) -> Self {
            Self {
                rows,
                cursor: Arc::new(Mutex::new(None)),
            }
        }

        fn cursor_snapshot(&self) -> Option<Id<OutboxRecord>> {
            *self.cursor.lock().expect("lock")
        }
    }

    impl OutboxRepo for FakeOutbox {
        fn list_after_meilisearch(
            &self,
            after: Option<Id<OutboxRecord>>,
        ) -> impl Future<Output = Result<Vec<OutboxRow>, DomainError>> + Send {
            let rows = self.rows.clone();
            async move {
                let mut out: Vec<_> = rows
                    .into_iter()
                    .filter(|row| match after {
                        None => true,
                        Some(cur) => row.id.inner_ulid() > cur.inner_ulid(),
                    })
                    .collect();
                out.sort_by_key(|r| r.id.inner_ulid());
                out.truncate(OUTBOX_BATCH_LIMIT as usize);
                Ok(out)
            }
        }

        fn meilisearch_cursor(
            &self,
        ) -> impl Future<Output = Result<Option<Id<OutboxRecord>>, DomainError>> + Send {
            let cursor = Arc::clone(&self.cursor);
            async move { Ok(*cursor.lock().expect("lock")) }
        }

        fn advance_meilisearch_cursor(
            &self,
            event_id: Id<OutboxRecord>,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let cursor = Arc::clone(&self.cursor);
            async move {
                *cursor.lock().expect("lock") = Some(event_id);
                Ok(())
            }
        }

        fn record_meilisearch_failure(
            &self,
            _event_id: Id<OutboxRecord>,
            _ultimo_error: &str,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            async { Ok(()) }
        }

        fn meilisearch_lag(&self) -> impl Future<Output = Result<u64, DomainError>> + Send {
            async { Ok(0) }
        }

        fn meilisearch_failed_count(
            &self,
        ) -> impl Future<Output = Result<u64, DomainError>> + Send {
            async { Ok(0) }
        }
    }

    struct CatalogFake {
        fichas: Vec<FichaProducto>,
    }

    impl CatalogFake {
        fn new(fichas: Vec<FichaProducto>) -> Self {
            Self { fichas }
        }
    }

    impl ProductRepo for CatalogFake {
        fn create(
            &self,
            _input: domain::NewProducto,
        ) -> impl Future<Output = Result<Producto, DomainError>> + Send {
            async {
                Err(DomainError::unavailable(
                    domain::UnavailableCode::Unavailable,
                ))
            }
        }

        fn get(
            &self,
            id: Id<Producto>,
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
            let fichas = self.fichas.clone();
            async move {
                fichas
                    .into_iter()
                    .find(|f| f.producto.id == id)
                    .ok_or_else(DomainError::not_found)
            }
        }

        fn update(
            &self,
            _input: domain::UpdateProducto,
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
            async {
                Err(DomainError::unavailable(
                    domain::UnavailableCode::Unavailable,
                ))
            }
        }

        fn set_activo(
            &self,
            _id: Id<Producto>,
            _activo: bool,
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
            async {
                Err(DomainError::unavailable(
                    domain::UnavailableCode::Unavailable,
                ))
            }
        }

        fn list(
            &self,
            filtro: FiltroProducto,
            after: Option<Id<Producto>>,
            take: u32,
        ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
            let fichas = self.fichas.clone();
            async move {
                let mut out: Vec<ResumenProducto> = fichas
                    .iter()
                    .filter(|f| f.producto.activo == filtro.activo)
                    .map(|f| ResumenProducto {
                        id: f.producto.id.clone(),
                        sku: f.producto.sku.as_str().to_string(),
                        nombre: f.producto.nombre.clone(),
                        tipo: f.producto.tipo.clone(),
                        precio: f.producto.precio,
                        stock: f.producto.stock,
                        activo: f.producto.activo,
                        marca_nombre: f.marca.as_ref().map(|m| m.nombre.clone()),
                        categoria_ruta: f.categoria.ruta.clone(),
                    })
                    .collect();
                out.sort_by(|a, b| a.id.to_string().cmp(&b.id.to_string()));
                if let Some(after) = after {
                    out.retain(|r| r.id.to_string() > after.to_string());
                }
                out.truncate(take as usize);
                Ok(out)
            }
        }

        fn find_by_code(
            &self,
            _code: String,
        ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
            async { Ok(vec![]) }
        }
    }

    struct FakeAttrDefs;

    impl AttributeDefRepo for FakeAttrDefs {
        fn create(
            &self,
            _input: domain::NewAtributoDef,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            async {
                Err(DomainError::unavailable(
                    domain::UnavailableCode::Unavailable,
                ))
            }
        }

        fn get(
            &self,
            _id: Id<domain::AtributoDef>,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            async { Err(DomainError::not_found()) }
        }

        fn list_all(&self) -> impl Future<Output = Result<Vec<AtributoDef>, DomainError>> + Send {
            async { Ok(vec![]) }
        }

        fn update(
            &self,
            _input: domain::UpdateAtributoDef,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            async {
                Err(DomainError::unavailable(
                    domain::UnavailableCode::Unavailable,
                ))
            }
        }

        fn set_activo(
            &self,
            _id: Id<domain::AtributoDef>,
            _activo: bool,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            async {
                Err(DomainError::unavailable(
                    domain::UnavailableCode::Unavailable,
                ))
            }
        }
    }

    #[derive(Clone)]
    struct ReindexSearch {
        fail_create: Arc<Mutex<Option<DomainError>>>,
        fail_upsert: Arc<Mutex<Option<DomainError>>>,
        swapped: Arc<AtomicBool>,
        upsert_batches: Arc<AtomicUsize>,
    }

    impl ReindexSearch {
        fn ok() -> Self {
            Self {
                fail_create: Arc::new(Mutex::new(None)),
                fail_upsert: Arc::new(Mutex::new(None)),
                swapped: Arc::new(AtomicBool::new(false)),
                upsert_batches: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn fail_create_next(err: DomainError) -> Self {
            let s = Self::ok();
            *s.fail_create.lock().expect("lock") = Some(err);
            s
        }
    }

    impl TextSearch for ReindexSearch {
        fn upsert_documents(
            &self,
            _index: &IndexUid,
            _documents: &[SearchDocument],
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let fail = Arc::clone(&self.fail_upsert);
            let batches = Arc::clone(&self.upsert_batches);
            async move {
                if let Some(err) = fail.lock().expect("lock").take() {
                    return Err(err);
                }
                batches.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        fn delete_documents(
            &self,
            _index: &IndexUid,
            _ids: &[Id<Producto>],
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            async { Ok(()) }
        }

        fn search(
            &self,
            _index: &IndexUid,
            _query: SearchQuery,
        ) -> impl Future<Output = Result<domain::SearchPage, DomainError>> + Send {
            async {
                Err(DomainError::unavailable(
                    domain::UnavailableCode::Unavailable,
                ))
            }
        }

        fn facet_search(
            &self,
            _index: &IndexUid,
            _query: SearchQuery,
        ) -> impl Future<Output = Result<domain::SearchPage, DomainError>> + Send {
            async {
                Err(DomainError::unavailable(
                    domain::UnavailableCode::Unavailable,
                ))
            }
        }

        fn apply_settings(
            &self,
            _index: &IndexUid,
            _settings: IndexSettings,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            async { Ok(()) }
        }

        fn create_index(
            &self,
            _index: &IndexUid,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let fail = Arc::clone(&self.fail_create);
            async move {
                if let Some(err) = fail.lock().expect("lock").take() {
                    return Err(err);
                }
                Ok(())
            }
        }

        fn swap_indexes(
            &self,
            _origen: &IndexUid,
            _destino: &IndexUid,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let swapped = Arc::clone(&self.swapped);
            async move {
                swapped.store(true, Ordering::SeqCst);
                Ok(())
            }
        }

        fn delete_index(
            &self,
            _index: &IndexUid,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            async { Ok(()) }
        }

        fn health(
            &self,
        ) -> impl Future<Output = Result<domain::ComponentHealth, DomainError>> + Send {
            async {
                Ok(domain::ComponentHealth::new(
                    domain::health::ComponentStatus::Up,
                ))
            }
        }
    }

    fn ts() -> Timestamp {
        "2026-01-01T00:00:00Z".parse().expect("ts")
    }

    fn valid_ficha() -> FichaProducto {
        let cat_id = Id::<Categoria>::generate();
        let producto = Producto::try_new(
            domain::producto::ProductoDraft {
                id: Id::generate(),
                sku: "SKU-1".into(),
                nombre: "Perno".into(),
                tipo: "Perno".into(),
                descripcion: None,
                marca: None,
                categoria: cat_id.clone(),
                atributos: vec![],
                codigos: vec![],
                unidad_medida: domain::UnidadMedida::Unidad,
                precio: "1".parse().expect("p"),
                ubicacion: None,
                stock: "1".parse().expect("s"),
                activo: true,
                version: 1,
                creado_en: ts(),
                actualizado_en: ts(),
            },
            &[],
        )
        .expect("producto");
        let categoria = Categoria::try_new(domain::categoria::CategoriaDraft {
            id: cat_id,
            nombre: "C".into(),
            slug: domain::categoria::Slug::parse("c").expect("slug"),
            padre: None,
            ancestros: vec![],
            ruta: vec!["C".into()],
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("cat");
        FichaProducto {
            producto,
            marca: None,
            categoria,
            atributos: vec![],
        }
    }

    fn invalid_marca_ficha() -> FichaProducto {
        let mut ficha = valid_ficha();
        let wrong = Marca::try_new(domain::marca::MarcaDraft {
            id: Id::generate(),
            nombre: "Otra".into(),
            slug: domain::categoria::Slug::parse("otra").expect("slug"),
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("marca");
        ficha.marca = Some(wrong);
        ficha
    }

    fn outbox_row() -> OutboxRow {
        OutboxRow {
            id: Id::generate(),
            event: OutboxEvent::try_new(
                OutboxEntity::Producto,
                OutboxOp::Upsert,
                "01J9ZKA7M3Q8R2T5V9X1Z4B6D8",
                None,
            )
            .expect("ev"),
            creado_en: ts(),
        }
    }

    fn uids() -> (IndexUid, IndexUid) {
        (
            IndexUid::try_new("productos-test-live").expect("live"),
            IndexUid::try_new("productos-test-staging").expect("staging"),
        )
    }

    #[tokio::test]
    async fn reindex_unavailable_does_not_advance_cursor() {
        let row = outbox_row();
        let outbox = FakeOutbox::with_rows(vec![row.clone()]);
        let products = CatalogFake::new(vec![valid_ficha()]);
        let search = ReindexSearch::fail_create_next(DomainError::unavailable(
            domain::UnavailableCode::Unavailable,
        ));
        let (live, staging) = uids();
        let err = reindex_meilisearch(&outbox, &products, &FakeAttrDefs, &search, &live, &staging)
            .await
            .expect_err("unavailable");
        assert_eq!(err.code(), DomainCode::Unavailable);
        assert!(!search.swapped.load(Ordering::SeqCst));
        assert!(outbox.cursor_snapshot().is_none());
    }

    #[tokio::test]
    async fn reindex_invalid_document_aborts_before_swap() {
        let outbox = FakeOutbox::with_rows(vec![]);
        let products = CatalogFake::new(vec![invalid_marca_ficha()]);
        let search = ReindexSearch::ok();
        let (live, staging) = uids();
        let err = reindex_meilisearch(&outbox, &products, &FakeAttrDefs, &search, &live, &staging)
            .await
            .expect_err("invalid");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert!(!search.swapped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn reindex_sync_exposes_total_rebuild() {
        let row = outbox_row();
        let outbox = FakeOutbox::with_rows(vec![row.clone()]);
        let products = CatalogFake::new(vec![valid_ficha()]);
        let search = ReindexSearch::ok();
        let (live, staging) = uids();
        reindex_meilisearch(&outbox, &products, &FakeAttrDefs, &search, &live, &staging)
            .await
            .expect("reindex");
        assert!(search.swapped.load(Ordering::SeqCst));
        assert!(search.upsert_batches.load(Ordering::SeqCst) >= 1);
        assert_eq!(outbox.cursor_snapshot(), Some(row.id));
    }
}
