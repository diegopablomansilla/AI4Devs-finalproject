//! Meilisearch product sync worker (T-03-07).

use std::time::Duration;

use domain::{
    AttributeDefRepo, DomainCode, DomainError, Id, IndexUid, InvalidId, OutboxEntity, OutboxEvent,
    OutboxRepo, OutboxRow, ProductRepo, Producto, TextSearch, build_search_document,
};
use storage::OutboxNotifier;
use tokio::time::sleep;

/// Default poll interval when `OutboxNotifier` does not fire (ticket: 1 s).
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Propagates `producto` outbox events to Meilisearch; other entities only advance the cursor (MVP).
pub struct MeiliProductWorker<O, P, A, S> {
    outbox: O,
    products: P,
    attr_defs: A,
    search: S,
    notifier: OutboxNotifier,
    index: IndexUid,
    poll_interval: Duration,
}

impl<O, P, A, S> MeiliProductWorker<O, P, A, S>
where
    O: OutboxRepo,
    P: ProductRepo,
    A: AttributeDefRepo,
    S: TextSearch,
{
    pub fn new(
        outbox: O,
        products: P,
        attr_defs: A,
        search: S,
        notifier: OutboxNotifier,
        index: IndexUid,
        poll_interval: Duration,
    ) -> Self {
        Self {
            outbox,
            products,
            attr_defs,
            search,
            notifier,
            index,
            poll_interval,
        }
    }

    /// Processes one batch; stops mid-batch if Meilisearch or DB fails (cursor does not advance).
    pub async fn drain_once(&self) -> Result<(), DomainError> {
        let cursor = self.outbox.meilisearch_cursor().await?;
        let rows = self.outbox.list_after_meilisearch(cursor).await?;
        if rows.is_empty() {
            return Ok(());
        }
        let defs = self.attr_defs.list_all().await?;
        for row in rows {
            self.process_row(&row, &defs).await?;
            self.outbox.advance_meilisearch_cursor(row.id).await?;
        }
        Ok(())
    }

    /// Notify + poll loop. Stops when `should_stop` returns true (composition root in T-00-27).
    pub async fn run_while<F>(&self, mut should_stop: F)
    where
        F: FnMut() -> bool,
    {
        while !should_stop() {
            tokio::select! {
                () = self.notifier.notified() => {}
                () = sleep(self.poll_interval) => {}
            }
            let _ = self.drain_once().await;
        }
    }

    async fn process_row(
        &self,
        row: &OutboxRow,
        defs: &[domain::AtributoDef],
    ) -> Result<(), DomainError> {
        match row.event.entidad {
            OutboxEntity::Producto => self.sync_producto(&row.event, defs).await,
            OutboxEntity::ImagenProducto
            | OutboxEntity::Categoria
            | OutboxEntity::Marca
            | OutboxEntity::AtributoDef
            | OutboxEntity::Sinonimo
            | OutboxEntity::NotaTecnica => Ok(()),
        }
    }

    async fn sync_producto(
        &self,
        event: &OutboxEvent,
        defs: &[domain::AtributoDef],
    ) -> Result<(), DomainError> {
        let product_id: Id<Producto> = event.registro.parse().map_err(|InvalidId| {
            DomainError::invalid_input(domain::error::InvalidInputCode::InvalidInput, vec![])
        })?;
        match self.products.get(product_id.clone()).await {
            Ok(ficha) if ficha.producto.activo => {
                let doc = build_search_document(
                    &ficha.producto,
                    ficha.marca.as_ref(),
                    &ficha.categoria,
                    defs,
                )?;
                self.search.upsert_documents(&self.index, &[doc]).await
            }
            Ok(ficha) => {
                self.search
                    .delete_documents(&self.index, &[ficha.producto.id])
                    .await
            }
            Err(e) if e.code() == DomainCode::NotFound => {
                self.search
                    .delete_documents(&self.index, &[product_id])
                    .await
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::sync::{Arc, Mutex};

    use domain::{
        AtributoDef, AttributeDefRepo, Categoria, DomainCode, DomainError, FichaProducto, Id,
        IndexUid, OutboxEntity, OutboxEvent, OutboxOp, OutboxRecord, OutboxRepo, OutboxRow,
        ProductRepo, Producto, SearchDocument, SearchQuery, TextSearch,
    };

    use super::*;

    struct FakeOutbox {
        rows: Vec<OutboxRow>,
        cursor: Arc<Mutex<Option<Id<OutboxRecord>>>>,
        advances: Arc<Mutex<Vec<Id<OutboxRecord>>>>,
    }

    impl FakeOutbox {
        fn with_rows(rows: Vec<OutboxRow>) -> Self {
            Self {
                rows,
                cursor: Arc::new(Mutex::new(None)),
                advances: Arc::new(Mutex::new(Vec::new())),
            }
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
                    .filter(|r| match after {
                        None => true,
                        Some(c) => r.id.inner_ulid() > c.inner_ulid(),
                    })
                    .collect();
                out.sort_by_key(|r| r.id.inner_ulid());
                Ok(out)
            }
        }

        fn meilisearch_cursor(
            &self,
        ) -> impl Future<Output = Result<Option<Id<OutboxRecord>>, DomainError>> + Send {
            let cursor = *self.cursor.lock().expect("lock");
            async move { Ok(cursor) }
        }

        fn advance_meilisearch_cursor(
            &self,
            event_id: Id<OutboxRecord>,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let cursor = Arc::clone(&self.cursor);
            let advances = Arc::clone(&self.advances);
            async move {
                *cursor.lock().expect("lock") = Some(event_id);
                advances.lock().expect("lock").push(event_id);
                Ok(())
            }
        }
    }

    struct FakeProducts {
        get: Option<Result<FichaProducto, DomainError>>,
    }

    impl ProductRepo for FakeProducts {
        async fn create(&self, _input: domain::NewProducto) -> Result<Producto, DomainError> {
            Err(DomainError::unavailable(
                domain::UnavailableCode::Unavailable,
            ))
        }

        async fn get(&self, _id: Id<Producto>) -> Result<FichaProducto, DomainError> {
            match self.get.as_ref() {
                Some(Ok(ficha)) => Ok(ficha.clone()),
                Some(Err(e)) => Err(DomainError::from_domain_code(e.code(), vec![])),
                None => Err(DomainError::not_found()),
            }
        }

        async fn update(
            &self,
            _input: domain::UpdateProducto,
        ) -> Result<FichaProducto, DomainError> {
            Err(DomainError::unavailable(
                domain::UnavailableCode::Unavailable,
            ))
        }

        async fn set_activo(
            &self,
            _id: Id<Producto>,
            _activo: bool,
        ) -> Result<FichaProducto, DomainError> {
            Err(DomainError::unavailable(
                domain::UnavailableCode::Unavailable,
            ))
        }

        async fn list(
            &self,
            _filtro: domain::FiltroProducto,
            _after: Option<Id<Producto>>,
            _take: u32,
        ) -> Result<Vec<domain::ResumenProducto>, DomainError> {
            Err(DomainError::unavailable(
                domain::UnavailableCode::Unavailable,
            ))
        }
    }

    struct FakeAttrDefs;

    impl AttributeDefRepo for FakeAttrDefs {
        async fn create(&self, _input: domain::NewAtributoDef) -> Result<AtributoDef, DomainError> {
            Err(DomainError::unavailable(
                domain::UnavailableCode::Unavailable,
            ))
        }

        async fn get(&self, _id: Id<domain::AtributoDef>) -> Result<AtributoDef, DomainError> {
            Err(DomainError::not_found())
        }

        async fn list_all(&self) -> Result<Vec<AtributoDef>, DomainError> {
            Ok(vec![])
        }

        async fn update(
            &self,
            _input: domain::UpdateAtributoDef,
        ) -> Result<AtributoDef, DomainError> {
            Err(DomainError::unavailable(
                domain::UnavailableCode::Unavailable,
            ))
        }

        async fn set_activo(
            &self,
            _id: Id<domain::AtributoDef>,
            _activo: bool,
        ) -> Result<AtributoDef, DomainError> {
            Err(DomainError::unavailable(
                domain::UnavailableCode::Unavailable,
            ))
        }
    }

    #[derive(Clone)]
    struct RecordingSearch {
        upserts: Arc<Mutex<Vec<Vec<Id<Producto>>>>>,
        deletes: Arc<Mutex<Vec<Vec<Id<Producto>>>>>,
        fail_with: Arc<Mutex<Option<DomainError>>>,
    }

    impl Default for RecordingSearch {
        fn default() -> Self {
            Self {
                upserts: Arc::new(Mutex::new(vec![])),
                deletes: Arc::new(Mutex::new(vec![])),
                fail_with: Arc::new(Mutex::new(None)),
            }
        }
    }

    impl RecordingSearch {
        fn fail_next(err: DomainError) -> Self {
            Self {
                upserts: Arc::new(Mutex::new(vec![])),
                deletes: Arc::new(Mutex::new(vec![])),
                fail_with: Arc::new(Mutex::new(Some(err))),
            }
        }
    }

    impl TextSearch for RecordingSearch {
        fn upsert_documents(
            &self,
            _index: &IndexUid,
            documents: &[SearchDocument],
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let ids: Vec<_> = documents.iter().map(|d| d.id.clone()).collect();
            let fail_with = self.fail_with.lock().expect("lock").take();
            let upserts = Arc::clone(&self.upserts);
            async move {
                if let Some(err) = fail_with {
                    return Err(err);
                }
                upserts.lock().expect("lock").push(ids);
                Ok(())
            }
        }

        fn delete_documents(
            &self,
            _index: &IndexUid,
            ids: &[Id<Producto>],
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let ids = ids.to_vec();
            let fail_with = self.fail_with.lock().expect("lock").take();
            let deletes = Arc::clone(&self.deletes);
            async move {
                if let Some(err) = fail_with {
                    return Err(err);
                }
                deletes.lock().expect("lock").push(ids);
                Ok(())
            }
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
            _settings: domain::IndexSettings,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            async { Ok(()) }
        }

        fn create_index(
            &self,
            _index: &IndexUid,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            async { Ok(()) }
        }

        fn swap_indexes(
            &self,
            _origen: &IndexUid,
            _destino: &IndexUid,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            async { Ok(()) }
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

    fn product_event(registro: &str) -> OutboxRow {
        OutboxRow {
            id: Id::generate(),
            event: OutboxEvent::try_new(OutboxEntity::Producto, OutboxOp::Upsert, registro, None)
                .expect("ev"),
        }
    }

    fn sample_ficha(activo: bool) -> FichaProducto {
        let cat_id = Id::<Categoria>::generate();
        let producto = Producto::try_new(
            domain::producto::ProductoDraft {
                id: Id::generate(),
                sku: "SKU".into(),
                nombre: "N".into(),
                tipo: "T".into(),
                descripcion: None,
                marca: None,
                categoria: cat_id.clone(),
                atributos: vec![],
                codigos: vec![],
                unidad_medida: domain::UnidadMedida::Unidad,
                precio: "1".parse().expect("p"),
                ubicacion: None,
                stock: "1".parse().expect("s"),
                activo,
                version: 1,
                creado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
                actualizado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
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
            creado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
            actualizado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
        })
        .expect("cat");
        FichaProducto {
            producto,
            marca: None,
            categoria,
            atributos: vec![],
        }
    }

    fn worker<O, P, A, S>(
        outbox: O,
        products: P,
        attr_defs: A,
        search: S,
    ) -> MeiliProductWorker<O, P, A, S>
    where
        O: OutboxRepo,
        P: ProductRepo,
        A: AttributeDefRepo,
        S: TextSearch,
    {
        MeiliProductWorker::new(
            outbox,
            products,
            attr_defs,
            search,
            OutboxNotifier::stub(),
            IndexUid::try_new("test-index").expect("uid"),
            Duration::from_millis(10),
        )
    }

    #[tokio::test]
    async fn worker_empty_outbox_does_not_call_search() {
        let search = RecordingSearch::default();
        let w = worker(
            FakeOutbox::with_rows(vec![]),
            FakeProducts { get: None },
            FakeAttrDefs,
            search,
        );
        w.drain_once().await.expect("drain");
    }

    #[tokio::test]
    async fn worker_deletes_inactive_product() {
        let registro = Id::<Producto>::generate().to_string();
        let search = RecordingSearch::default();
        let w = worker(
            FakeOutbox::with_rows(vec![product_event(&registro)]),
            FakeProducts {
                get: Some(Ok(sample_ficha(false))),
            },
            FakeAttrDefs,
            search.clone(),
        );
        w.drain_once().await.expect("drain");
        let deleted = search.deletes.lock().expect("lock");
        assert_eq!(deleted.len(), 1);
    }

    #[tokio::test]
    async fn worker_deletes_missing_product() {
        let registro = Id::<Producto>::generate().to_string();
        let search = RecordingSearch::default();
        let w = worker(
            FakeOutbox::with_rows(vec![product_event(&registro)]),
            FakeProducts {
                get: Some(Err(DomainError::not_found())),
            },
            FakeAttrDefs,
            search.clone(),
        );
        w.drain_once().await.expect("drain");
        assert_eq!(search.deletes.lock().expect("lock").len(), 1);
    }

    #[tokio::test]
    async fn worker_skips_categoria_without_fanout() {
        let row = OutboxRow {
            id: Id::generate(),
            event: OutboxEvent::try_new(
                OutboxEntity::Categoria,
                OutboxOp::Upsert,
                Id::<Producto>::generate().to_string().as_str(),
                None,
            )
            .expect("ev"),
        };
        let outbox = FakeOutbox::with_rows(vec![row]);
        let advances = Arc::clone(&outbox.advances);
        let search = RecordingSearch::default();
        let w = worker(
            outbox,
            FakeProducts { get: None },
            FakeAttrDefs,
            search.clone(),
        );
        w.drain_once().await.expect("drain");
        assert!(search.upserts.lock().expect("lock").is_empty());
        assert!(search.deletes.lock().expect("lock").is_empty());
        assert_eq!(advances.lock().expect("lock").len(), 1);
    }

    #[tokio::test]
    async fn worker_advances_cursor_on_imagen_producto_without_mutating_index() {
        let row = OutboxRow {
            id: Id::generate(),
            event: OutboxEvent::try_new(
                OutboxEntity::ImagenProducto,
                OutboxOp::Upsert,
                Id::<Producto>::generate().to_string().as_str(),
                None,
            )
            .expect("ev"),
        };
        let search = RecordingSearch::default();
        let w = worker(
            FakeOutbox::with_rows(vec![row]),
            FakeProducts { get: None },
            FakeAttrDefs,
            search.clone(),
        );
        w.drain_once().await.expect("drain");
        assert!(search.upserts.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn worker_does_not_advance_cursor_when_search_unavailable() {
        let registro = Id::<Producto>::generate().to_string();
        let row = product_event(&registro);
        let outbox = FakeOutbox::with_rows(vec![row]);
        let advances = Arc::clone(&outbox.advances);
        let search = RecordingSearch::fail_next(DomainError::unavailable(
            domain::UnavailableCode::Unavailable,
        ));
        let w = worker(
            outbox,
            FakeProducts {
                get: Some(Ok(sample_ficha(true))),
            },
            FakeAttrDefs,
            search,
        );
        let err = w.drain_once().await.expect_err("unavailable");
        assert_eq!(err.code(), DomainCode::Unavailable);
        assert!(advances.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn worker_does_not_advance_cursor_on_invalid_input() {
        let registro = Id::<Producto>::generate().to_string();
        let outbox = FakeOutbox::with_rows(vec![product_event(&registro)]);
        let advances = Arc::clone(&outbox.advances);
        let search = RecordingSearch::fail_next(DomainError::invalid_input(
            domain::error::InvalidInputCode::InvalidInput,
            vec![],
        ));
        let w = worker(
            outbox,
            FakeProducts {
                get: Some(Ok(sample_ficha(true))),
            },
            FakeAttrDefs,
            search,
        );
        let err = w.drain_once().await.expect_err("invalid");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert!(advances.lock().expect("lock").is_empty());
    }
}
