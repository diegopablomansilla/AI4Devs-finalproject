//! Text search port (`TextSearch`) and query types (T-03-03).

use std::collections::BTreeMap;
use std::future::Future;

use crate::categoria::Categoria;
use crate::error::{DomainError, FieldError, InvalidInputCode};
use crate::health::ComponentHealth;
use crate::id::Id;
use crate::marca::Marca;
use crate::producto::Producto;
use crate::search_document::SearchDocument;

const MAX_Q_LEN: usize = 200;
const MAX_OFFSET: u32 = 1000;
const MIN_LIMIT: u32 = 1;
const MAX_LIMIT: u32 = 50;

/// Meilisearch index name (e.g. `productos`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexUid(String);

impl IndexUid {
    pub fn try_new(raw: &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(field_invalid_input("index", "index uid must not be empty"));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Meilisearch index settings payload (`apply_settings`; T-03-05).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexSettings {
    pub searchable_attributes: Vec<String>,
    pub ranking_rules: Vec<String>,
    pub filterable_attributes: Vec<String>,
    pub sortable_attributes: Vec<String>,
    pub typo_disable_on_attributes: Vec<String>,
}

impl IndexSettings {
    /// True when no lists are set (no-op for `apply_settings`).
    pub fn is_empty(&self) -> bool {
        self.searchable_attributes.is_empty()
            && self.ranking_rules.is_empty()
            && self.filterable_attributes.is_empty()
            && self.sortable_attributes.is_empty()
            && self.typo_disable_on_attributes.is_empty()
    }
}

/// Sort order for text search (domain Spanish wire values).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchOrden {
    Relevancia,
    PrecioAsc,
    PrecioDesc,
    StockDesc,
}

impl SearchOrden {
    pub fn try_from_str(s: &str) -> Result<Self, DomainError> {
        match s {
            "relevancia" => Ok(Self::Relevancia),
            "precio_asc" => Ok(Self::PrecioAsc),
            "precio_desc" => Ok(Self::PrecioDesc),
            "stock_desc" => Ok(Self::StockDesc),
            _ => Err(field_invalid_input("orden", "unknown search orden")),
        }
    }
}

/// Filters for a text search (T-03-12 maps from OpenAPI).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchFilters {
    pub categoria_ids: Vec<Id<Categoria>>,
    pub marca_id: Option<Id<Marca>>,
    pub con_stock: Option<bool>,
    pub atributos: BTreeMap<String, String>,
}

fn matches_atributo_clave(clave: &str) -> bool {
    if clave.is_empty() {
        return false;
    }
    let mut chars = clave.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn validate_filtro_atributo_clave(clave: &str) -> Result<(), DomainError> {
    let trimmed = clave.trim();
    if trimmed.is_empty() {
        return Err(field_invalid_input("clave", "clave must not be empty"));
    }
    if trimmed.starts_with("attr_") {
        return Err(field_invalid_input(
            "clave",
            "clave must not include attr_ prefix",
        ));
    }
    if !matches_atributo_clave(trimmed) {
        return Err(field_invalid_input("clave", "invalid clave format"));
    }
    Ok(())
}

fn validate_filtros_atributos(atributos: &BTreeMap<String, String>) -> Result<(), DomainError> {
    for clave in atributos.keys() {
        validate_filtro_atributo_clave(clave)?;
    }
    Ok(())
}

/// Query input for `search` / `facet_search`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    pub q: String,
    pub filtros: SearchFilters,
    pub orden: SearchOrden,
    pub offset: u32,
    pub limit: u32,
}

impl SearchQuery {
    pub fn try_new(
        q: impl Into<String>,
        filtros: SearchFilters,
        orden: SearchOrden,
        offset: u32,
        limit: u32,
    ) -> Result<Self, DomainError> {
        let q = q.into();
        if q.len() > MAX_Q_LEN {
            return Err(field_invalid_input("q", "q must be at most 200 characters"));
        }
        if offset > MAX_OFFSET {
            return Err(field_invalid_input("offset", "offset must be at most 1000"));
        }
        if !(MIN_LIMIT..=MAX_LIMIT).contains(&limit) {
            return Err(field_invalid_input(
                "limit",
                "limit must be between 1 and 50",
            ));
        }
        validate_filtros_atributos(&filtros.atributos)?;
        Ok(Self {
            q,
            filtros,
            orden,
            offset,
            limit,
        })
    }
}

/// Highlight fragments from Meilisearch `_formatted` (T-03-12).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchHighlight {
    pub nombre: Option<String>,
    pub descripcion: Option<String>,
}

/// One row in a search result page (4.11.1 fields for T-03-13 mapping).
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    pub id: Id<Producto>,
    pub sku: String,
    pub nombre: String,
    pub tipo: String,
    pub marca: Option<String>,
    pub codigos: Vec<String>,
    pub atributos_texto: Vec<String>,
    pub precio: f64,
    pub unidad_medida: String,
    pub stock_disponible: f64,
    pub con_stock: bool,
    pub ubicacion: Option<String>,
    pub imagen: Option<String>,
    pub resaltado: Option<SearchHighlight>,
}

impl SearchHit {
    /// Hit with document defaults; only `id` is required (T-03-04 probes).
    pub fn from_id(id: Id<Producto>) -> Self {
        Self {
            id,
            sku: String::new(),
            nombre: String::new(),
            tipo: String::new(),
            marca: None,
            codigos: Vec::new(),
            atributos_texto: Vec::new(),
            precio: 0.0,
            unidad_medida: crate::producto::UnidadMedida::Unidad.as_str().to_string(),
            stock_disponible: 0.0,
            con_stock: false,
            ubicacion: None,
            imagen: None,
            resaltado: None,
        }
    }
}

/// Facet distribution placeholder (T-03-18).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SearchFacets;

/// Result page from `search` / `facet_search`.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchPage {
    pub hits: Vec<SearchHit>,
    pub estimado_total: u32,
    pub offset: u32,
    pub limit: u32,
    pub tiempo_ms: u32,
    pub facetas: Option<SearchFacets>,
}

impl SearchPage {
    pub fn try_new(
        hits: Vec<SearchHit>,
        estimado_total: u32,
        offset: u32,
        limit: u32,
        tiempo_ms: u32,
        facetas: Option<SearchFacets>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            hits,
            estimado_total,
            offset,
            limit,
            tiempo_ms,
            facetas,
        })
    }
}

/// Rejects swapping an index with itself (domain guard before calling the engine).
pub fn validate_index_swap(origen: &IndexUid, destino: &IndexUid) -> Result<(), DomainError> {
    if origen.as_str() == destino.as_str() {
        return Err(field_invalid_input(
            "index",
            "swap origin and destination must differ",
        ));
    }
    Ok(())
}

/// Port for Meilisearch-backed text search (adapter: T-03-04).
pub trait TextSearch {
    fn upsert_documents(
        &self,
        index: &IndexUid,
        documents: &[SearchDocument],
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn delete_documents(
        &self,
        index: &IndexUid,
        ids: &[Id<Producto>],
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn search(
        &self,
        index: &IndexUid,
        query: SearchQuery,
    ) -> impl Future<Output = Result<SearchPage, DomainError>> + Send;

    fn facet_search(
        &self,
        index: &IndexUid,
        query: SearchQuery,
    ) -> impl Future<Output = Result<SearchPage, DomainError>> + Send;

    fn apply_settings(
        &self,
        index: &IndexUid,
        settings: IndexSettings,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn create_index(
        &self,
        index: &IndexUid,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn swap_indexes(
        &self,
        origen: &IndexUid,
        destino: &IndexUid,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn delete_index(
        &self,
        index: &IndexUid,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn health(&self) -> impl Future<Output = Result<ComponentHealth, DomainError>> + Send;
}

fn field_invalid_input(field: &str, message: &str) -> DomainError {
    DomainError::invalid_input(
        InvalidInputCode::InvalidInput,
        vec![FieldError::try_new(field, message).expect("field")],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{DomainCode, UnavailableCode};
    use crate::health::ComponentStatus;

    const CANONICAL_ULID: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";

    fn dummy_run_search(mode: DummyMode, query: SearchQuery) -> Result<SearchPage, DomainError> {
        match mode {
            DummyMode::SearchUnavailable => {
                Err(DomainError::unavailable(UnavailableCode::Unavailable))
            }
            DummyMode::SearchTimeout => Err(DomainError::timeout()),
            _ => SearchPage::try_new(vec![], 0, query.offset, query.limit, 0, None),
        }
    }

    fn dummy_run_health() -> Result<ComponentHealth, DomainError> {
        Ok(ComponentHealth::new(ComponentStatus::Up))
    }

    fn dummy_run_upsert(mode: DummyMode) -> Result<(), DomainError> {
        match mode {
            DummyMode::UpsertInvalid => Err(field_invalid_input(
                "document",
                "invalid document for index",
            )),
            _ => Ok(()),
        }
    }

    fn sample_query() -> SearchQuery {
        SearchQuery::try_new(
            "perno",
            SearchFilters::default(),
            SearchOrden::Relevancia,
            0,
            20,
        )
        .expect("query")
    }

    #[derive(Clone, Copy)]
    enum DummyMode {
        Normal,
        SearchUnavailable,
        SearchTimeout,
        UpsertInvalid,
    }

    struct DummyTextSearch {
        mode: DummyMode,
    }

    impl DummyTextSearch {
        const fn new(mode: DummyMode) -> Self {
            Self { mode }
        }
    }

    impl TextSearch for DummyTextSearch {
        fn upsert_documents(
            &self,
            _index: &IndexUid,
            _documents: &[SearchDocument],
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let mode = self.mode;
            async move { dummy_run_upsert(mode) }
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
            let mode = self.mode;
            async move { dummy_run_search(mode, query) }
        }

        fn facet_search(
            &self,
            _index: &IndexUid,
            query: SearchQuery,
        ) -> impl Future<Output = Result<SearchPage, DomainError>> + Send {
            let mode = self.mode;
            async move { dummy_run_search(mode, query) }
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
            origen: &IndexUid,
            destino: &IndexUid,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            async move {
                validate_index_swap(origen, destino)?;
                Ok(())
            }
        }

        fn delete_index(
            &self,
            _index: &IndexUid,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            async { Ok(()) }
        }

        fn health(&self) -> impl Future<Output = Result<ComponentHealth, DomainError>> + Send {
            async { dummy_run_health() }
        }
    }

    #[test]
    fn search_query_constructs_perno() {
        let q = SearchQuery::try_new(
            "perno",
            SearchFilters::default(),
            SearchOrden::Relevancia,
            0,
            20,
        )
        .expect("valid");
        assert_eq!(q.q, "perno");
        assert_eq!(q.offset, 0);
        assert_eq!(q.limit, 20);
        assert_eq!(q.orden, SearchOrden::Relevancia);
    }

    #[test]
    fn search_query_allows_empty_q() {
        let q = SearchQuery::try_new("", SearchFilters::default(), SearchOrden::Relevancia, 0, 20)
            .expect("valid");
        assert!(q.q.is_empty());
    }

    #[test]
    fn search_filters_constructs_categoria_marca_stock_attr() {
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
        assert_eq!(filtros.categoria_ids, [cat]);
        assert_eq!(filtros.marca_id, Some(marca));
        assert_eq!(filtros.con_stock, Some(true));
        assert_eq!(filtros.atributos.get("medida"), Some(&"M8".into()));
        assert!(!filtros.atributos.contains_key("attr_medida"));
    }

    #[test]
    fn search_page_constructs_hit_without_facets() {
        let id: Id<Producto> = CANONICAL_ULID.parse().expect("ulid");
        let page =
            SearchPage::try_new(vec![SearchHit::from_id(id)], 1, 0, 20, 0, None).expect("page");
        assert_eq!(page.hits.len(), 1);
        assert!(page.facetas.is_none());
        assert_eq!(page.estimado_total, 1);
    }

    #[test]
    fn index_uid_constructs_productos() {
        let idx = IndexUid::try_new("productos").expect("valid");
        assert_eq!(idx.as_str(), "productos");
    }

    #[test]
    fn search_document_constructs_ulid() {
        let doc = SearchDocument::try_from_ulid_str(CANONICAL_ULID).expect("valid");
        assert_eq!(doc.id.to_string(), CANONICAL_ULID);
        assert!(!doc.id.to_string().contains(':'));
    }

    #[test]
    fn index_settings_constructs_empty() {
        let settings = IndexSettings::default();
        assert!(settings.is_empty());
    }

    #[test]
    fn index_settings_constructs_with_lists() {
        let settings = IndexSettings {
            searchable_attributes: vec!["nombre".into()],
            filterable_attributes: vec!["marca_id".into()],
            ..IndexSettings::default()
        };
        assert!(!settings.is_empty());
        assert_eq!(settings.searchable_attributes, ["nombre"]);
        assert_eq!(settings.filterable_attributes, ["marca_id"]);
    }

    #[test]
    fn search_query_rejects_q_too_long() {
        let long = "a".repeat(201);
        let err = SearchQuery::try_new(
            long,
            SearchFilters::default(),
            SearchOrden::Relevancia,
            0,
            20,
        )
        .expect_err("too long");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn search_query_rejects_offset_or_limit_out_of_range() {
        let f = SearchFilters::default();
        let err_offset = SearchQuery::try_new("x", f.clone(), SearchOrden::Relevancia, 1001, 20)
            .expect_err("offset");
        assert_eq!(err_offset.code(), DomainCode::InvalidInput);
        let err_limit_0 = SearchQuery::try_new("x", f.clone(), SearchOrden::Relevancia, 0, 0)
            .expect_err("limit0");
        assert_eq!(err_limit_0.code(), DomainCode::InvalidInput);
        let err_limit_51 =
            SearchQuery::try_new("x", f, SearchOrden::Relevancia, 0, 51).expect_err("limit51");
        assert_eq!(err_limit_51.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn search_query_rejects_english_or_unknown_orden() {
        assert!(SearchOrden::try_from_str("relevance").is_err());
        assert!(SearchOrden::try_from_str("price_asc").is_err());
        assert_eq!(
            SearchOrden::try_from_str("relevance").unwrap_err().code(),
            DomainCode::InvalidInput
        );
    }

    #[test]
    fn index_uid_rejects_blank() {
        let err = IndexUid::try_new("   ").expect_err("blank");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn swap_indexes_rejects_same_uid() {
        let idx = IndexUid::try_new("productos").expect("valid");
        let err = validate_index_swap(&idx, &idx).expect_err("same");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn search_document_rejects_table_prefix() {
        let err = SearchDocument::try_from_ulid_str("producto:01J9ZKA7M3Q8R2T5V9X1Z4B6D8")
            .expect_err("prefix");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn text_search_trait_exposes_nine_ops() {
        fn assert_port<R: TextSearch>(r: &R, index: &IndexUid, query: SearchQuery) {
            let _ = r.upsert_documents(index, &[]);
            let _ = r.delete_documents(index, &[]);
            let _ = r.search(index, query.clone());
            let _ = r.facet_search(index, query);
            let _ = r.apply_settings(index, IndexSettings::default());
            let _ = r.create_index(index);
            let _ = r.delete_index(index);
            let _ = r.swap_indexes(index, index);
            let _ = r.health();
        }
        let index = IndexUid::try_new("productos").unwrap();
        assert_port(
            &DummyTextSearch::new(DummyMode::Normal),
            &index,
            sample_query(),
        );
    }

    #[test]
    fn text_search_dummy_search_returns_page() {
        let query = sample_query();
        let page = dummy_run_search(DummyMode::Normal, query.clone()).expect("ok");
        assert_eq!(page.offset, query.offset);
        assert_eq!(page.limit, query.limit);
        assert!(page.facetas.is_none());
    }

    #[test]
    fn text_search_dummy_health_is_up() {
        let health = dummy_run_health().expect("ok");
        assert_eq!(health.status, ComponentStatus::Up);
    }

    #[test]
    fn text_search_dummy_search_unavailable() {
        let err = dummy_run_search(DummyMode::SearchUnavailable, sample_query())
            .expect_err("unavailable");
        assert_eq!(err.code(), DomainCode::Unavailable);
    }

    #[test]
    fn text_search_dummy_search_timeout() {
        let err = dummy_run_search(DummyMode::SearchTimeout, sample_query()).expect_err("timeout");
        assert_eq!(err.code(), DomainCode::Timeout);
    }

    #[test]
    fn text_search_dummy_upsert_invalid_input() {
        let err = dummy_run_upsert(DummyMode::UpsertInvalid).expect_err("invalid");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn search_hit_constructs_with_resaltado() {
        let id: Id<Producto> = CANONICAL_ULID.parse().expect("ulid");
        let hit = SearchHit {
            resaltado: Some(SearchHighlight {
                nombre: Some("<mark>Perno</mark> hexagonal".into()),
                descripcion: None,
            }),
            ..SearchHit::from_id(id)
        };
        assert!(hit.resaltado.is_some());
        let resaltado = hit.resaltado.expect("resaltado");
        assert!(resaltado.nombre.unwrap().contains("Perno"));
    }

    #[test]
    fn search_query_rejects_attr_prefix_clave() {
        let mut atributos = BTreeMap::new();
        atributos.insert("attr_medida".into(), "M8".into());
        let filtros = SearchFilters {
            atributos,
            ..SearchFilters::default()
        };
        let err = SearchQuery::try_new("perno", filtros, SearchOrden::Relevancia, 0, 20)
            .expect_err("attr prefix");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn search_query_rejects_invalid_atributo_clave() {
        for bad in ["Medida", "medida-mm", ""] {
            let mut atributos = BTreeMap::new();
            atributos.insert(bad.to_string(), "M8".into());
            let filtros = SearchFilters {
                atributos,
                ..SearchFilters::default()
            };
            let err = SearchQuery::try_new("perno", filtros, SearchOrden::Relevancia, 0, 20)
                .expect_err(bad);
            assert_eq!(err.code(), DomainCode::InvalidInput);
        }
    }
}
