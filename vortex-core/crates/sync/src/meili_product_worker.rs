//! Meilisearch product sync worker (T-03-07, fan-out T-03-08).

use std::time::Duration;

use domain::{
    AtributoDef, AttributeDefRepo, Categoria, DomainCode, DomainError, FiltroProducto, Id,
    IndexUid, InvalidId, Marca, OUTBOX_BATCH_LIMIT, OutboxEntity, OutboxEvent, OutboxRepo,
    OutboxRow, ProductRepo, Producto, TextSearch, build_search_document, productos_index_settings,
};
use storage::OutboxNotifier;
use tokio::time::sleep;

/// Default poll interval when `OutboxNotifier` does not fire (ticket: 1 s).
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Propagates outbox events to Meilisearch (`producto` + fan-out de maestros).
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

    async fn process_row(&self, row: &OutboxRow, defs: &[AtributoDef]) -> Result<(), DomainError> {
        match row.event.entidad {
            OutboxEntity::Producto => self.sync_producto(&row.event, defs).await,
            OutboxEntity::Categoria => self.fanout_categoria(&row.event, defs).await,
            OutboxEntity::Marca => self.fanout_marca(&row.event, defs).await,
            OutboxEntity::AtributoDef => self.fanout_atributo_def(&row.event, defs).await,
            OutboxEntity::ImagenProducto | OutboxEntity::Sinonimo | OutboxEntity::NotaTecnica => {
                Ok(())
            }
        }
    }

    fn parse_registro<T>(event: &OutboxEvent) -> Result<Id<T>, DomainError> {
        event.registro.parse().map_err(|InvalidId| {
            DomainError::invalid_input(domain::error::InvalidInputCode::InvalidInput, vec![])
        })
    }

    async fn sync_producto(
        &self,
        event: &OutboxEvent,
        defs: &[AtributoDef],
    ) -> Result<(), DomainError> {
        let product_id = Self::parse_registro::<Producto>(event)?;
        self.reindex_product(product_id, defs).await
    }

    async fn reindex_product(
        &self,
        product_id: Id<Producto>,
        defs: &[AtributoDef],
    ) -> Result<(), DomainError> {
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

    async fn apply_productos_profile(&self, defs: &[AtributoDef]) -> Result<(), DomainError> {
        let claves = defs
            .iter()
            .filter(|d| d.activo && d.facetable)
            .map(|d| d.clave.as_str());
        let settings = productos_index_settings(claves)?;
        self.search.apply_settings(&self.index, settings).await
    }

    async fn fanout_categoria(
        &self,
        event: &OutboxEvent,
        defs: &[AtributoDef],
    ) -> Result<(), DomainError> {
        let cat_id = Self::parse_registro::<Categoria>(event)?;
        self.fanout_list_filter(
            FiltroProducto {
                categoria: Some(cat_id),
                marca: None,
                activo: true,
            },
            defs,
            |_| true,
        )
        .await
    }

    async fn fanout_marca(
        &self,
        event: &OutboxEvent,
        defs: &[AtributoDef],
    ) -> Result<(), DomainError> {
        let marca_id = Self::parse_registro::<Marca>(event)?;
        self.fanout_list_filter(
            FiltroProducto {
                categoria: None,
                marca: Some(marca_id),
                activo: true,
            },
            defs,
            |_| true,
        )
        .await
    }

    async fn fanout_atributo_def(
        &self,
        event: &OutboxEvent,
        defs: &[AtributoDef],
    ) -> Result<(), DomainError> {
        let def_id = Self::parse_registro::<AtributoDef>(event)?;
        self.apply_productos_profile(defs).await?;
        self.fanout_list_filter(
            FiltroProducto {
                categoria: None,
                marca: None,
                activo: true,
            },
            defs,
            move |ficha| ficha.producto.atributos.iter().any(|a| a.def == def_id),
        )
        .await
    }

    async fn fanout_list_filter<F>(
        &self,
        filtro: FiltroProducto,
        defs: &[AtributoDef],
        mut matches: F,
    ) -> Result<(), DomainError>
    where
        F: FnMut(&domain::FichaProducto) -> bool,
    {
        let mut after = None;
        loop {
            let page = self
                .products
                .list(filtro.clone(), after, OUTBOX_BATCH_LIMIT)
                .await?;
            if page.is_empty() {
                break;
            }
            for summary in &page {
                let ficha = self.products.get(summary.id.clone()).await?;
                if matches(&ficha) {
                    self.reindex_product(ficha.producto.id, defs).await?;
                }
            }
            after = Some(page.last().expect("non-empty page").id.clone());
            if page.len() < OUTBOX_BATCH_LIMIT as usize {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::sync::{Arc, Mutex};

    use domain::{
        AtributoDef, AtributoValor, AttributeDefRepo, Categoria, DomainCode, DomainError,
        FichaProducto, FiltroProducto, Id, IndexSettings, IndexUid, Marca, OutboxEntity,
        OutboxEvent, OutboxOp, OutboxRecord, OutboxRepo, OutboxRow, ProductRepo, Producto,
        ResumenProducto, SearchDocument, SearchQuery, TextSearch, ValorAtributo,
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

    struct CatalogFake {
        fichas: Vec<FichaProducto>,
    }

    impl CatalogFake {
        fn new(fichas: Vec<FichaProducto>) -> Self {
            Self { fichas }
        }

        fn matches_categoria(ficha: &FichaProducto, cat: &Id<Categoria>) -> bool {
            ficha.producto.categoria == *cat || ficha.categoria.ancestros.contains(cat)
        }
    }

    impl ProductRepo for CatalogFake {
        async fn create(&self, _input: domain::NewProducto) -> Result<Producto, DomainError> {
            Err(DomainError::unavailable(
                domain::UnavailableCode::Unavailable,
            ))
        }

        async fn get(&self, id: Id<Producto>) -> Result<FichaProducto, DomainError> {
            self.fichas
                .iter()
                .find(|f| f.producto.id == id)
                .cloned()
                .ok_or_else(DomainError::not_found)
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
            filtro: FiltroProducto,
            after: Option<Id<Producto>>,
            take: u32,
        ) -> Result<Vec<ResumenProducto>, DomainError> {
            let mut out: Vec<ResumenProducto> = self
                .fichas
                .iter()
                .filter(|f| f.producto.activo == filtro.activo)
                .filter(|f| {
                    filtro
                        .marca
                        .as_ref()
                        .is_none_or(|m| f.producto.marca.as_ref() == Some(m))
                })
                .filter(|f| {
                    filtro
                        .categoria
                        .as_ref()
                        .is_none_or(|c| Self::matches_categoria(f, c))
                })
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

    struct FakeAttrDefs {
        defs: Vec<AtributoDef>,
    }

    impl FakeAttrDefs {
        fn with_defs(defs: Vec<AtributoDef>) -> Self {
            Self { defs }
        }
    }

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
            Ok(self.defs.clone())
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
        fail_upsert: Arc<Mutex<Option<DomainError>>>,
        fail_apply: Arc<Mutex<Option<DomainError>>>,
        settings_applied: Arc<Mutex<u32>>,
    }

    impl Default for RecordingSearch {
        fn default() -> Self {
            Self {
                upserts: Arc::new(Mutex::new(vec![])),
                deletes: Arc::new(Mutex::new(vec![])),
                fail_upsert: Arc::new(Mutex::new(None)),
                fail_apply: Arc::new(Mutex::new(None)),
                settings_applied: Arc::new(Mutex::new(0)),
            }
        }
    }

    impl RecordingSearch {
        fn fail_upsert_next(err: DomainError) -> Self {
            Self {
                upserts: Arc::new(Mutex::new(vec![])),
                deletes: Arc::new(Mutex::new(vec![])),
                fail_upsert: Arc::new(Mutex::new(Some(err))),
                fail_apply: Arc::new(Mutex::new(None)),
                settings_applied: Arc::new(Mutex::new(0)),
            }
        }

        fn fail_apply_next(err: DomainError) -> Self {
            Self {
                upserts: Arc::new(Mutex::new(vec![])),
                deletes: Arc::new(Mutex::new(vec![])),
                fail_upsert: Arc::new(Mutex::new(None)),
                fail_apply: Arc::new(Mutex::new(Some(err))),
                settings_applied: Arc::new(Mutex::new(0)),
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
            let fail_upsert = self.fail_upsert.lock().expect("lock").take();
            let upserts = Arc::clone(&self.upserts);
            async move {
                if let Some(err) = fail_upsert {
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
            let fail_upsert = self.fail_upsert.lock().expect("lock").take();
            let deletes = Arc::clone(&self.deletes);
            async move {
                if let Some(err) = fail_upsert {
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
            _settings: IndexSettings,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let fail_apply = self.fail_apply.lock().expect("lock").take();
            let settings_applied = Arc::clone(&self.settings_applied);
            async move {
                if let Some(err) = fail_apply {
                    return Err(err);
                }
                *settings_applied.lock().expect("lock") += 1;
                Ok(())
            }
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
            FakeAttrDefs::with_defs(vec![]),
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
            FakeAttrDefs::with_defs(vec![]),
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
            FakeAttrDefs::with_defs(vec![]),
            search.clone(),
        );
        w.drain_once().await.expect("drain");
        assert_eq!(search.deletes.lock().expect("lock").len(), 1);
    }

    fn branch_fichas() -> (Id<Categoria>, Id<Categoria>, FichaProducto, FichaProducto) {
        let root_id = Id::<Categoria>::generate();
        let child_id = Id::<Categoria>::generate();
        let root = Categoria::try_new(domain::categoria::CategoriaDraft {
            id: root_id.clone(),
            nombre: "Fijaciones".into(),
            slug: domain::categoria::Slug::parse("fijaciones").expect("slug"),
            padre: None,
            ancestros: vec![],
            ruta: vec!["Fijaciones".into()],
            activo: true,
            creado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
            actualizado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
        })
        .expect("root");
        let child = Categoria::try_new(domain::categoria::CategoriaDraft {
            id: child_id.clone(),
            nombre: "Pernos".into(),
            slug: domain::categoria::Slug::parse("pernos").expect("slug"),
            padre: Some(root_id.clone()),
            ancestros: vec![root_id.clone()],
            ruta: vec!["Fijaciones".into(), "Pernos".into()],
            activo: true,
            creado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
            actualizado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
        })
        .expect("child");
        let on_root = ficha_for_category(root.clone(), root_id.clone());
        let on_child = ficha_for_category(child.clone(), child_id.clone());
        (root_id, child_id, on_root, on_child)
    }

    fn ficha_for_category(cat: Categoria, cat_id: Id<Categoria>) -> FichaProducto {
        let producto = Producto::try_new(
            domain::producto::ProductoDraft {
                id: Id::generate(),
                sku: "SKU".into(),
                nombre: "P".into(),
                tipo: "T".into(),
                descripcion: None,
                marca: None,
                categoria: cat_id,
                atributos: vec![],
                codigos: vec![],
                unidad_medida: domain::UnidadMedida::Unidad,
                precio: "1".parse().expect("p"),
                ubicacion: None,
                stock: "1".parse().expect("s"),
                activo: true,
                version: 1,
                creado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
                actualizado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
            },
            &[],
        )
        .expect("producto");
        FichaProducto {
            producto,
            marca: None,
            categoria: cat,
            atributos: vec![],
        }
    }

    fn entity_event(entity: OutboxEntity, registro: &str) -> OutboxRow {
        OutboxRow {
            id: Id::generate(),
            event: OutboxEvent::try_new(entity, OutboxOp::Upsert, registro, None).expect("ev"),
        }
    }

    #[tokio::test]
    async fn worker_fanout_categoria_reindexes_full_branch() {
        let (root_id, _child_id, on_root, on_child) = branch_fichas();
        let search = RecordingSearch::default();
        let w = worker(
            FakeOutbox::with_rows(vec![entity_event(
                OutboxEntity::Categoria,
                root_id.to_string().as_str(),
            )]),
            CatalogFake::new(vec![on_root, on_child]),
            FakeAttrDefs::with_defs(vec![]),
            search.clone(),
        );
        w.drain_once().await.expect("drain");
        let upserted = search.upserts.lock().expect("lock");
        let total: usize = upserted.iter().map(|v| v.len()).sum();
        assert_eq!(total, 2);
    }

    #[tokio::test]
    async fn worker_fanout_marca_reindexes_products() {
        let marca_id = Id::<Marca>::generate();
        let marca = Marca::try_new(domain::marca::MarcaDraft {
            id: marca_id.clone(),
            nombre: "Fixser".into(),
            slug: domain::categoria::Slug::parse("fixser").expect("slug"),
            activo: true,
            creado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
            actualizado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
        })
        .expect("marca");
        let with_marca = {
            let mut f = sample_ficha(true);
            f.producto.marca = Some(marca_id.clone());
            f.marca = Some(marca);
            f
        };
        let without = sample_ficha(true);
        let search = RecordingSearch::default();
        let w = worker(
            FakeOutbox::with_rows(vec![entity_event(
                OutboxEntity::Marca,
                marca_id.to_string().as_str(),
            )]),
            CatalogFake::new(vec![with_marca, without]),
            FakeAttrDefs::with_defs(vec![]),
            search.clone(),
        );
        w.drain_once().await.expect("drain");
        let upserted = search.upserts.lock().expect("lock");
        let total: usize = upserted.iter().map(|v| v.len()).sum();
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn worker_fanout_atributo_def_reindexes_users_and_applies_settings() {
        let def_id = Id::<AtributoDef>::generate();
        let def = AtributoDef::try_new(domain::atributo_def::AtributoDefDraft {
            id: def_id.clone(),
            clave: "medida".into(),
            etiqueta: "Medida".into(),
            tipo: domain::TipoAtributo::Texto,
            unidad: None,
            opciones: None,
            facetable: true,
            buscable: true,
            orden: 0,
            activo: true,
            creado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
            actualizado_en: "2026-01-01T00:00:00Z".parse().expect("ts"),
        })
        .expect("def");
        let mut with_attr = sample_ficha(true);
        with_attr.producto.atributos = vec![AtributoValor {
            def: def_id.clone(),
            valor: ValorAtributo::Texto("M8".into()),
        }];
        let search = RecordingSearch::default();
        let w = worker(
            FakeOutbox::with_rows(vec![entity_event(
                OutboxEntity::AtributoDef,
                def_id.to_string().as_str(),
            )]),
            CatalogFake::new(vec![with_attr, sample_ficha(true)]),
            FakeAttrDefs::with_defs(vec![def]),
            search.clone(),
        );
        w.drain_once().await.expect("drain");
        assert_eq!(*search.settings_applied.lock().expect("lock"), 1);
        let upserted = search.upserts.lock().expect("lock");
        assert_eq!(upserted.len(), 1);
        assert_eq!(upserted[0].len(), 1);
    }

    #[tokio::test]
    async fn worker_fanout_empty_affected_advances_cursor() {
        let cat_id = Id::<Categoria>::generate();
        let outbox = FakeOutbox::with_rows(vec![entity_event(
            OutboxEntity::Categoria,
            cat_id.to_string().as_str(),
        )]);
        let advances = Arc::clone(&outbox.advances);
        let search = RecordingSearch::default();
        let w = worker(
            outbox,
            CatalogFake::new(vec![]),
            FakeAttrDefs::with_defs(vec![]),
            search.clone(),
        );
        w.drain_once().await.expect("drain");
        assert!(search.upserts.lock().expect("lock").is_empty());
        assert_eq!(advances.lock().expect("lock").len(), 1);
    }

    #[tokio::test]
    async fn worker_sinonimo_still_skips_without_mutating_index() {
        let row = entity_event(
            OutboxEntity::Sinonimo,
            &Id::<Producto>::generate().to_string(),
        );
        let outbox = FakeOutbox::with_rows(vec![row]);
        let advances = Arc::clone(&outbox.advances);
        let search = RecordingSearch::default();
        let w = worker(
            outbox,
            CatalogFake::new(vec![]),
            FakeAttrDefs::with_defs(vec![]),
            search.clone(),
        );
        w.drain_once().await.expect("drain");
        assert!(search.upserts.lock().expect("lock").is_empty());
        assert_eq!(*search.settings_applied.lock().expect("lock"), 0);
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
            FakeAttrDefs::with_defs(vec![]),
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
        let search = RecordingSearch::fail_upsert_next(DomainError::unavailable(
            domain::UnavailableCode::Unavailable,
        ));
        let w = worker(
            outbox,
            FakeProducts {
                get: Some(Ok(sample_ficha(true))),
            },
            FakeAttrDefs::with_defs(vec![]),
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
        let search = RecordingSearch::fail_upsert_next(DomainError::invalid_input(
            domain::error::InvalidInputCode::InvalidInput,
            vec![],
        ));
        let w = worker(
            outbox,
            FakeProducts {
                get: Some(Ok(sample_ficha(true))),
            },
            FakeAttrDefs::with_defs(vec![]),
            search,
        );
        let err = w.drain_once().await.expect_err("invalid");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert!(advances.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn worker_fanout_does_not_advance_cursor_when_search_unavailable() {
        let (root_id, _, on_root, _) = branch_fichas();
        let outbox = FakeOutbox::with_rows(vec![entity_event(
            OutboxEntity::Categoria,
            root_id.to_string().as_str(),
        )]);
        let advances = Arc::clone(&outbox.advances);
        let search = RecordingSearch::fail_upsert_next(DomainError::unavailable(
            domain::UnavailableCode::Unavailable,
        ));
        let w = worker(
            outbox,
            CatalogFake::new(vec![on_root]),
            FakeAttrDefs::with_defs(vec![]),
            search,
        );
        let err = w.drain_once().await.expect_err("unavailable");
        assert_eq!(err.code(), DomainCode::Unavailable);
        assert!(advances.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn worker_fanout_does_not_advance_cursor_when_apply_settings_unavailable() {
        let def_id = Id::<AtributoDef>::generate();
        let outbox = FakeOutbox::with_rows(vec![entity_event(
            OutboxEntity::AtributoDef,
            def_id.to_string().as_str(),
        )]);
        let advances = Arc::clone(&outbox.advances);
        let search = RecordingSearch::fail_apply_next(DomainError::unavailable(
            domain::UnavailableCode::Unavailable,
        ));
        let w = worker(
            outbox,
            CatalogFake::new(vec![]),
            FakeAttrDefs::with_defs(vec![]),
            search,
        );
        let err = w.drain_once().await.expect_err("unavailable");
        assert_eq!(err.code(), DomainCode::Unavailable);
        assert!(advances.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn worker_fanout_invalid_registro_does_not_advance() {
        let outbox = FakeOutbox::with_rows(vec![OutboxRow {
            id: Id::generate(),
            event: OutboxEvent {
                entidad: OutboxEntity::Categoria,
                registro: "bad".into(),
                operacion: OutboxOp::Upsert,
                motivo: None,
            },
        }]);
        let advances = Arc::clone(&outbox.advances);
        let w = worker(
            outbox,
            CatalogFake::new(vec![]),
            FakeAttrDefs::with_defs(vec![]),
            RecordingSearch::default(),
        );
        let err = w.drain_once().await.expect_err("invalid");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert!(advances.lock().expect("lock").is_empty());
    }
}
