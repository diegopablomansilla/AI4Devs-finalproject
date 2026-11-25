//! Text search use case and Meilisearch query translation (T-03-12).

use std::future::Future;
use std::pin::Pin;

use domain::error::DomainError;
use domain::{IndexUid, SearchFilters, SearchOrden, SearchPage, SearchQuery, TextSearch};

pub(crate) const MEILI_HIGHLIGHT_ATTRS: &[&str] = &["nombre", "descripcion"];
pub(crate) const MEILI_HIGHLIGHT_PRE: &str = "<mark>";
pub(crate) const MEILI_HIGHLIGHT_POST: &str = "</mark>";

fn escape_filter_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Builds a Meilisearch `filter` expression from domain filters (pure; unit-tested).
pub(crate) fn meili_filter(filtros: &SearchFilters) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();

    if !filtros.categoria_ids.is_empty() {
        let ids: Vec<String> = filtros
            .categoria_ids
            .iter()
            .map(|id| format!("\"{}\"", id))
            .collect();
        parts.push(format!("categoria_ids IN [{}]", ids.join(", ")));
    }

    if let Some(marca) = &filtros.marca_id {
        parts.push(format!("marca_id = \"{}\"", marca));
    }

    if let Some(con_stock) = filtros.con_stock {
        parts.push(format!("con_stock = {con_stock}"));
    }

    for (clave, valor) in &filtros.atributos {
        let esc = escape_filter_string(valor);
        parts.push(format!("attr_{clave} = \"{esc}\""));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" AND "))
    }
}

/// Maps domain sort to Meilisearch `sort` (pure; unit-tested).
pub(crate) fn meili_sort(orden: SearchOrden) -> Option<Vec<String>> {
    match orden {
        SearchOrden::Relevancia => None,
        SearchOrden::PrecioAsc => Some(vec!["precio:asc".into()]),
        SearchOrden::PrecioDesc => Some(vec!["precio:desc".into()]),
        SearchOrden::StockDesc => Some(vec!["stock_disponible:desc".into()]),
    }
}

/// Object-safe facade for HTTP injection (`Arc<dyn SearchGateway>`).
pub trait SearchGateway: Send + Sync {
    fn search(
        &self,
        q: String,
        filtros: SearchFilters,
        orden: SearchOrden,
        offset: u32,
        limit: u32,
    ) -> Pin<Box<dyn Future<Output = Result<SearchPage, DomainError>> + Send + '_>>;
}

/// Text search use case over `TextSearch` (T-03-12).
pub struct SearchService<S> {
    search: S,
    index: IndexUid,
}

impl<S> SearchService<S> {
    pub fn new(search: S, index: IndexUid) -> Self {
        Self { search, index }
    }
}

impl<S: TextSearch> SearchService<S> {
    pub async fn search(
        &self,
        q: impl Into<String>,
        filtros: SearchFilters,
        orden: SearchOrden,
        offset: u32,
        limit: u32,
    ) -> Result<SearchPage, DomainError> {
        let query = SearchQuery::try_new(q, filtros, orden, offset, limit)?;
        self.search.search(&self.index, query).await
    }
}

impl<S> SearchGateway for SearchService<S>
where
    S: TextSearch + Send + Sync,
{
    fn search(
        &self,
        q: String,
        filtros: SearchFilters,
        orden: SearchOrden,
        offset: u32,
        limit: u32,
    ) -> Pin<Box<dyn Future<Output = Result<SearchPage, DomainError>> + Send + '_>> {
        Box::pin(SearchService::search(
            self, q, filtros, orden, offset, limit,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    use domain::categoria::Categoria;
    use domain::error::DomainCode;
    use domain::health::ComponentHealth;
    use domain::marca::Marca;
    use domain::producto::Producto;
    use domain::{
        DomainError, Id, IndexSettings, SearchDocument, SearchHighlight, SearchHit,
        UnavailableCode, health::ComponentStatus,
    };

    const CANONICAL_ULID: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";

    #[derive(Clone, Copy)]
    enum RecordingMode {
        Record,
        Unavailable,
        Timeout,
    }

    struct RecordingTextSearch {
        last_query: Arc<Mutex<Option<SearchQuery>>>,
        mode: RecordingMode,
    }

    impl RecordingTextSearch {
        fn record() -> (Self, Arc<Mutex<Option<SearchQuery>>>) {
            let last = Arc::new(Mutex::new(None));
            (
                Self {
                    last_query: Arc::clone(&last),
                    mode: RecordingMode::Record,
                },
                last,
            )
        }

        fn unavailable() -> Self {
            Self {
                last_query: Arc::new(Mutex::new(None)),
                mode: RecordingMode::Unavailable,
            }
        }

        fn timeout() -> Self {
            Self {
                last_query: Arc::new(Mutex::new(None)),
                mode: RecordingMode::Timeout,
            }
        }
    }

    impl TextSearch for RecordingTextSearch {
        fn upsert_documents(
            &self,
            _index: &IndexUid,
            _documents: &[SearchDocument],
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            async { Ok(()) }
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
            query: SearchQuery,
        ) -> impl Future<Output = Result<SearchPage, DomainError>> + Send {
            let last = Arc::clone(&self.last_query);
            let mode = self.mode;
            async move {
                match mode {
                    RecordingMode::Unavailable => {
                        Err(DomainError::unavailable(UnavailableCode::Unavailable))
                    }
                    RecordingMode::Timeout => Err(DomainError::timeout()),
                    RecordingMode::Record => {
                        *last.lock().expect("lock") = Some(query.clone());
                        SearchPage::try_new(vec![], 0, query.offset, query.limit, 0, None)
                    }
                }
            }
        }

        fn facet_search(
            &self,
            _index: &IndexUid,
            query: SearchQuery,
        ) -> impl Future<Output = Result<SearchPage, DomainError>> + Send {
            self.search(_index, query)
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

        fn health(&self) -> impl Future<Output = Result<ComponentHealth, DomainError>> + Send {
            async { Ok(ComponentHealth::new(ComponentStatus::Up)) }
        }
    }

    fn productos_index() -> IndexUid {
        IndexUid::try_new("productos-test").expect("index")
    }

    #[tokio::test]
    async fn search_service_forwards_q_with_empty_filters() {
        let (rec, last) = RecordingTextSearch::record();
        let svc = SearchService::new(rec, productos_index());
        svc.search(
            "perno",
            SearchFilters::default(),
            SearchOrden::Relevancia,
            0,
            20,
        )
        .await
        .expect("ok");
        let q = last.lock().expect("lock").clone().expect("recorded");
        assert_eq!(q.q, "perno");
        assert_eq!(q.offset, 0);
        assert_eq!(q.limit, 20);
        assert!(q.filtros.categoria_ids.is_empty());
        assert!(q.filtros.marca_id.is_none());
        assert!(q.filtros.con_stock.is_none());
        assert!(q.filtros.atributos.is_empty());
    }

    #[tokio::test]
    async fn search_service_forwards_each_filter() {
        let cat = Id::<Categoria>::generate();
        let marca = Id::<Marca>::generate();
        let mut atributos = BTreeMap::new();
        atributos.insert("medida".into(), "M8".into());
        let filtros = SearchFilters {
            categoria_ids: vec![cat.clone()],
            marca_id: Some(marca.clone()),
            con_stock: Some(true),
            atributos,
        };
        let (rec, last) = RecordingTextSearch::record();
        let svc = SearchService::new(rec, productos_index());
        svc.search("perno", filtros, SearchOrden::Relevancia, 5, 10)
            .await
            .expect("ok");
        let q = last.lock().expect("lock").clone().expect("recorded");
        assert_eq!(q.filtros.categoria_ids, [cat]);
        assert_eq!(q.filtros.marca_id, Some(marca));
        assert_eq!(q.filtros.con_stock, Some(true));
        assert_eq!(q.filtros.atributos.get("medida"), Some(&"M8".into()));
        assert!(!q.filtros.atributos.contains_key("attr_medida"));
    }

    #[tokio::test]
    async fn search_service_allows_empty_q_with_con_stock() {
        let filtros = SearchFilters {
            con_stock: Some(true),
            ..SearchFilters::default()
        };
        let (rec, last) = RecordingTextSearch::record();
        let svc = SearchService::new(rec, productos_index());
        svc.search("", filtros, SearchOrden::Relevancia, 0, 20)
            .await
            .expect("ok");
        let q = last.lock().expect("lock").clone().expect("recorded");
        assert!(q.q.is_empty());
        assert_eq!(q.filtros.con_stock, Some(true));
    }

    #[tokio::test]
    async fn search_service_rejects_q_too_long() {
        let svc = SearchService::new(RecordingTextSearch::record().0, productos_index());
        let long = "a".repeat(201);
        let err = svc
            .search(
                long,
                SearchFilters::default(),
                SearchOrden::Relevancia,
                0,
                20,
            )
            .await
            .expect_err("too long");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[tokio::test]
    async fn search_service_rejects_offset_or_limit_out_of_range() {
        let svc = SearchService::new(RecordingTextSearch::record().0, productos_index());
        let err_off = svc
            .search(
                "x",
                SearchFilters::default(),
                SearchOrden::Relevancia,
                1001,
                20,
            )
            .await
            .expect_err("offset");
        assert_eq!(err_off.code(), DomainCode::InvalidInput);
        let err_lim = svc
            .search(
                "x",
                SearchFilters::default(),
                SearchOrden::Relevancia,
                0,
                51,
            )
            .await
            .expect_err("limit");
        assert_eq!(err_lim.code(), DomainCode::InvalidInput);
    }

    #[tokio::test]
    async fn search_service_rejects_attr_prefix_clave() {
        let mut atributos = BTreeMap::new();
        atributos.insert("attr_medida".into(), "M8".into());
        let svc = SearchService::new(RecordingTextSearch::record().0, productos_index());
        let err = svc
            .search(
                "perno",
                SearchFilters {
                    atributos,
                    ..SearchFilters::default()
                },
                SearchOrden::Relevancia,
                0,
                20,
            )
            .await
            .expect_err("attr prefix");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[tokio::test]
    async fn search_service_propagates_unavailable() {
        let svc = SearchService::new(RecordingTextSearch::unavailable(), productos_index());
        let err = svc
            .search(
                "perno",
                SearchFilters::default(),
                SearchOrden::Relevancia,
                0,
                20,
            )
            .await
            .expect_err("unavailable");
        assert_eq!(err.code(), DomainCode::Unavailable);
    }

    #[tokio::test]
    async fn search_service_propagates_timeout() {
        let svc = SearchService::new(RecordingTextSearch::timeout(), productos_index());
        let err = svc
            .search(
                "perno",
                SearchFilters::default(),
                SearchOrden::Relevancia,
                0,
                20,
            )
            .await
            .expect_err("timeout");
        assert_eq!(err.code(), DomainCode::Timeout);
    }

    #[test]
    fn meili_filter_joins_categoria_marca_stock_and_attr() {
        let cat = Id::<Categoria>::generate();
        let marca = Id::<Marca>::generate();
        let mut atributos = BTreeMap::new();
        atributos.insert("medida".into(), "M8".into());
        let filtros = SearchFilters {
            categoria_ids: vec![cat.clone()],
            marca_id: Some(marca.clone()),
            con_stock: Some(true),
            atributos,
        };
        let filter = meili_filter(&filtros).expect("filter");
        assert!(filter.contains(&format!("categoria_ids IN [\"{cat}\"]")));
        assert!(filter.contains(&format!("marca_id = \"{marca}\"")));
        assert!(filter.contains("con_stock = true"));
        assert!(filter.contains("attr_medida = \"M8\""));
        assert_eq!(filter.matches(" AND ").count(), 3);
        assert!(!filter.contains("attr_attr_"));
    }

    #[test]
    fn meili_filter_omitted_when_empty() {
        assert!(meili_filter(&SearchFilters::default()).is_none());
    }

    #[test]
    fn meili_sort_maps_precio_and_stock() {
        assert_eq!(
            meili_sort(SearchOrden::PrecioAsc),
            Some(vec!["precio:asc".to_string()])
        );
        assert_eq!(
            meili_sort(SearchOrden::PrecioDesc),
            Some(vec!["precio:desc".to_string()])
        );
        assert_eq!(
            meili_sort(SearchOrden::StockDesc),
            Some(vec!["stock_disponible:desc".to_string()])
        );
    }

    #[test]
    fn meili_sort_omitted_for_relevancia() {
        assert!(meili_sort(SearchOrden::Relevancia).is_none());
    }

    #[test]
    fn meili_search_requests_nombre_descripcion_highlight() {
        assert_eq!(MEILI_HIGHLIGHT_ATTRS, &["nombre", "descripcion"]);
        assert_eq!(MEILI_HIGHLIGHT_PRE, "<mark>");
        assert_eq!(MEILI_HIGHLIGHT_POST, "</mark>");
    }

    #[tokio::test]
    async fn search_gateway_is_object_safe() {
        let (rec, _) = RecordingTextSearch::record();
        let svc = SearchService::new(rec, productos_index());
        let gateway: Arc<dyn SearchGateway> = Arc::new(svc);
        gateway
            .search(
                "perno".into(),
                SearchFilters::default(),
                SearchOrden::Relevancia,
                0,
                20,
            )
            .await
            .expect("search");
        let _ = SearchHighlight::default();
        let _ = SearchHit::from_id(CANONICAL_ULID.parse().expect("ulid"));
    }
}
