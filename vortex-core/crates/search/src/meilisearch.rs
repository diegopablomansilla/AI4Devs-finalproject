//! Meilisearch adapter for `domain::TextSearch` (T-03-04).

use std::future::Future;
use std::time::Duration;

use domain::error::{DomainError, FieldError, InvalidInputCode, UnavailableCode};
use domain::health::{ComponentHealth, ComponentStatus};
use domain::producto::Producto;
use domain::{
    Id, IndexSettings, IndexUid, SearchDocument, SearchHighlight, SearchHit, SearchPage,
    SearchQuery, TextSearch, validate_index_swap,
};
use serde_json::Value;

use crate::query::{
    MEILI_HIGHLIGHT_ATTRS, MEILI_HIGHLIGHT_POST, MEILI_HIGHLIGHT_PRE, meili_filter, meili_sort,
};
use meilisearch_sdk::client::{Client, SwapIndexes};
use meilisearch_sdk::errors::{Error, ErrorCode, ErrorType};
use meilisearch_sdk::search::Selectors;
use meilisearch_sdk::settings::{FilterableAttribute, Settings, TypoToleranceSettings};
use meilisearch_sdk::task_info::TaskInfo;
use serde::Deserialize;

const DEFAULT_TASK_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Deserialize, Default)]
struct MeiliHitDocument {
    id: String,
    #[serde(default)]
    sku: String,
    #[serde(default)]
    nombre: String,
    #[serde(default)]
    tipo: String,
    #[serde(default)]
    marca: Option<String>,
    #[serde(default)]
    codigos: Vec<String>,
    #[serde(default)]
    atributos_texto: Vec<String>,
    #[serde(default)]
    precio: f64,
    #[serde(default)]
    unidad_medida: String,
    #[serde(default)]
    stock_disponible: f64,
    #[serde(default)]
    con_stock: bool,
    #[serde(default)]
    ubicacion: Option<String>,
    #[serde(default)]
    imagen: Option<String>,
}

/// Meilisearch-backed implementation of `TextSearch`.
#[derive(Debug, Clone)]
pub struct MeiliTextSearch {
    client: Client,
    task_timeout: Duration,
}

impl MeiliTextSearch {
    pub fn new(
        base_url: impl Into<String>,
        api_key: Option<impl Into<String>>,
    ) -> Result<Self, DomainError> {
        let client =
            Client::new(base_url, api_key.map(Into::into)).map_err(|e| map_meili_error(&e))?;
        Ok(Self {
            client,
            task_timeout: DEFAULT_TASK_TIMEOUT,
        })
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn with_task_timeout(mut self, timeout: Duration) -> Self {
        self.task_timeout = timeout;
        self
    }

    async fn complete_task(&self, task: TaskInfo) -> Result<(), DomainError> {
        let result = task
            .wait_for_completion(&self.client, None, Some(self.task_timeout))
            .await
            .map_err(|e| map_meili_error(&e))?;
        if matches!(result, meilisearch_sdk::tasks::Task::Failed { .. }) {
            return Err(meili_invalid_input("meilisearch task failed"));
        }
        Ok(())
    }

    async fn upsert_documents_inner(
        &self,
        index: &IndexUid,
        documents: &[SearchDocument],
    ) -> Result<(), DomainError> {
        let idx = self.client.index(index.as_str());
        let task = idx
            .add_or_replace(documents, Some("id"))
            .await
            .map_err(|e| map_meili_error(&e))?;
        self.complete_task(task).await
    }

    async fn delete_documents_inner(
        &self,
        index: &IndexUid,
        ids: &[Id<Producto>],
    ) -> Result<(), DomainError> {
        let id_strings: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let idx = self.client.index(index.as_str());
        let task = idx
            .delete_documents(&id_strings)
            .await
            .map_err(|e| map_meili_error(&e))?;
        self.complete_task(task).await
    }

    async fn search_inner(
        &self,
        index: &IndexUid,
        query: SearchQuery,
    ) -> Result<SearchPage, DomainError> {
        let filter_owned = meili_filter(&query.filtros);
        let sort_owned = meili_sort(query.orden);
        let mut sort_refs: Vec<&str> = Vec::new();
        if let Some(ref sort) = sort_owned {
            sort_refs = sort.iter().map(String::as_str).collect();
        }

        let idx = self.client.index(index.as_str());
        let mut builder = idx.search();
        builder
            .with_query(&query.q)
            .with_offset(query.offset as usize)
            .with_limit(query.limit as usize)
            .with_attributes_to_highlight(Selectors::Some(MEILI_HIGHLIGHT_ATTRS))
            .with_highlight_pre_tag(MEILI_HIGHLIGHT_PRE)
            .with_highlight_post_tag(MEILI_HIGHLIGHT_POST);
        if let Some(ref filter) = filter_owned {
            builder.with_filter(filter);
        }
        if !sort_refs.is_empty() {
            builder.with_sort(&sort_refs);
        }
        let results = builder
            .execute::<MeiliHitDocument>()
            .await
            .map_err(|e| map_meili_error(&e))?;

        let hits = map_hits(results.hits)?;
        let estimado_total = results
            .estimated_total_hits
            .unwrap_or(hits.len())
            .min(u32::MAX as usize) as u32;
        let tiempo_ms = results.processing_time_ms.min(u32::MAX as usize) as u32;
        SearchPage::try_new(
            hits,
            estimado_total,
            query.offset,
            query.limit,
            tiempo_ms,
            None,
        )
    }

    fn to_meili_settings(settings: &IndexSettings) -> Settings {
        Settings {
            searchable_attributes: Some(settings.searchable_attributes.clone()),
            ranking_rules: Some(settings.ranking_rules.clone()),
            filterable_attributes: Some(
                settings
                    .filterable_attributes
                    .iter()
                    .map(|name| FilterableAttribute::from(name.as_str()))
                    .collect(),
            ),
            sortable_attributes: Some(settings.sortable_attributes.clone()),
            typo_tolerance: Some(TypoToleranceSettings {
                disable_on_attributes: Some(settings.typo_disable_on_attributes.clone()),
                ..TypoToleranceSettings::default()
            }),
            ..Settings::default()
        }
    }

    fn from_meili_settings(got: &Settings) -> IndexSettings {
        let filterable_attributes = got
            .filterable_attributes
            .as_ref()
            .map(|attrs| {
                attrs
                    .iter()
                    .filter_map(|a| match a {
                        FilterableAttribute::Attribute(name) => Some(name.clone()),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let typo_disable_on_attributes = got
            .typo_tolerance
            .as_ref()
            .and_then(|t| t.disable_on_attributes.clone())
            .unwrap_or_default();
        IndexSettings {
            searchable_attributes: got.searchable_attributes.clone().unwrap_or_default(),
            ranking_rules: got.ranking_rules.clone().unwrap_or_default(),
            filterable_attributes,
            sortable_attributes: got.sortable_attributes.clone().unwrap_or_default(),
            typo_disable_on_attributes,
        }
    }

    /// Hidden helper for integration tests (T-03-08).
    #[doc(hidden)]
    pub async fn test_read_categoria_lvl0(
        &self,
        index: &IndexUid,
        id: &Id<Producto>,
    ) -> Result<Option<String>, DomainError> {
        #[derive(Deserialize, Default)]
        struct CategoriaPartial {
            #[serde(default)]
            lvl0: Option<String>,
        }
        #[derive(Deserialize, Default)]
        struct DocPartial {
            #[serde(default)]
            categoria: CategoriaPartial,
        }
        let doc: DocPartial = self
            .client
            .index(index.as_str())
            .get_document(&id.to_string())
            .await
            .map_err(|e| map_meili_error(&e))?;
        Ok(doc.categoria.lvl0)
    }

    /// Hidden helper for integration tests (Listo cuando T-03-05).
    #[doc(hidden)]
    pub async fn test_read_index_settings(
        &self,
        index: &IndexUid,
    ) -> Result<(IndexSettings, bool), DomainError> {
        let got = self
            .client
            .index(index.as_str())
            .get_settings()
            .await
            .map_err(|e| map_meili_error(&e))?;
        let synonyms_present = got.synonyms.is_some();
        Ok((Self::from_meili_settings(&got), synonyms_present))
    }

    async fn apply_settings_inner(
        &self,
        index: &IndexUid,
        settings: IndexSettings,
    ) -> Result<(), DomainError> {
        if settings.is_empty() {
            return Ok(());
        }
        let idx = self.client.index(index.as_str());
        let meili = Self::to_meili_settings(&settings);
        let task = idx
            .set_settings(&meili)
            .await
            .map_err(|e| map_meili_error(&e))?;
        self.complete_task(task).await
    }

    async fn health_inner(&self) -> Result<ComponentHealth, DomainError> {
        let health = self
            .client
            .health()
            .await
            .map_err(|e| map_meili_error(&e))?;
        if health.status == "available" {
            Ok(ComponentHealth::new(ComponentStatus::Up))
        } else {
            Err(DomainError::unavailable(UnavailableCode::Unavailable))
        }
    }
}

fn formatted_field(formatted: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    formatted
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn map_hits(
    hits: Vec<meilisearch_sdk::search::SearchResult<MeiliHitDocument>>,
) -> Result<Vec<SearchHit>, DomainError> {
    hits.into_iter()
        .map(|hit| {
            let doc = hit.result;
            let id = doc
                .id
                .parse::<Id<Producto>>()
                .map_err(|_| meili_invalid_input("invalid hit id from meilisearch"))?;
            let resaltado = hit.formatted_result.as_ref().map(|m| SearchHighlight {
                nombre: formatted_field(m, "nombre"),
                descripcion: formatted_field(m, "descripcion"),
            });
            let resaltado = resaltado.filter(|h| h.nombre.is_some() || h.descripcion.is_some());
            let unidad_medida = if doc.unidad_medida.is_empty() {
                domain::producto::UnidadMedida::Unidad.as_str().to_string()
            } else {
                doc.unidad_medida
            };
            Ok(SearchHit {
                id,
                sku: doc.sku,
                nombre: doc.nombre,
                tipo: doc.tipo,
                marca: doc.marca,
                codigos: doc.codigos,
                atributos_texto: doc.atributos_texto,
                precio: doc.precio,
                unidad_medida,
                stock_disponible: doc.stock_disponible,
                con_stock: doc.con_stock,
                ubicacion: doc.ubicacion,
                imagen: doc.imagen,
                resaltado,
            })
        })
        .collect()
}

fn meili_invalid_input(message: &str) -> DomainError {
    DomainError::invalid_input(
        InvalidInputCode::InvalidInput,
        vec![FieldError::try_new("meilisearch", message).expect("field")],
    )
}

pub(crate) fn map_meili_error(err: &Error) -> DomainError {
    match err {
        Error::Timeout => DomainError::timeout(),
        Error::Meilisearch(me) => {
            if me.error_type == ErrorType::InvalidRequest
                || matches!(
                    me.error_code,
                    ErrorCode::InvalidDocumentId
                        | ErrorCode::MissingDocumentId
                        | ErrorCode::MalformedPayload
                        | ErrorCode::BadRequest
                        | ErrorCode::BadParameter
                )
            {
                meili_invalid_input(&me.error_message)
            } else {
                DomainError::unavailable(UnavailableCode::Unavailable)
            }
        }
        Error::MeilisearchCommunication(comm) if comm.status_code == 400 => {
            meili_invalid_input(comm.message.as_deref().unwrap_or("meilisearch bad request"))
        }
        Error::MeilisearchCommunication(_) => {
            DomainError::unavailable(UnavailableCode::Unavailable)
        }
        #[allow(unreachable_patterns)]
        Error::HttpError(req_err) => {
            if req_err.is_timeout() {
                DomainError::timeout()
            } else {
                DomainError::unavailable(UnavailableCode::Unavailable)
            }
        }
        _ => DomainError::unavailable(UnavailableCode::Unavailable),
    }
}

impl TextSearch for MeiliTextSearch {
    fn upsert_documents(
        &self,
        index: &IndexUid,
        documents: &[SearchDocument],
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        let this = self.clone();
        let index = index.clone();
        let documents = documents.to_vec();
        async move { this.upsert_documents_inner(&index, &documents).await }
    }

    fn delete_documents(
        &self,
        index: &IndexUid,
        ids: &[Id<Producto>],
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        let this = self.clone();
        let index = index.clone();
        let ids = ids.to_vec();
        async move { this.delete_documents_inner(&index, &ids).await }
    }

    fn search(
        &self,
        index: &IndexUid,
        query: SearchQuery,
    ) -> impl Future<Output = Result<SearchPage, DomainError>> + Send {
        let this = self.clone();
        let index = index.clone();
        async move { this.search_inner(&index, query).await }
    }

    fn facet_search(
        &self,
        index: &IndexUid,
        query: SearchQuery,
    ) -> impl Future<Output = Result<SearchPage, DomainError>> + Send {
        self.search(index, query)
    }

    fn apply_settings(
        &self,
        index: &IndexUid,
        settings: IndexSettings,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        let this = self.clone();
        let index = index.clone();
        async move { this.apply_settings_inner(&index, settings).await }
    }

    fn create_index(
        &self,
        index: &IndexUid,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        let this = self.clone();
        let uid = index.as_str().to_string();
        async move {
            let task = this
                .client
                .create_index(&uid, Some("id"))
                .await
                .map_err(|e| map_meili_error(&e))?;
            this.complete_task(task).await
        }
    }

    fn swap_indexes(
        &self,
        origen: &IndexUid,
        destino: &IndexUid,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        let this = self.clone();
        let origen = origen.clone();
        let destino = destino.clone();
        async move {
            validate_index_swap(&origen, &destino)?;
            let swap = SwapIndexes {
                indexes: (origen.as_str().to_string(), destino.as_str().to_string()),
                rename: None,
            };
            let task = this
                .client
                .swap_indexes([&swap])
                .await
                .map_err(|e| map_meili_error(&e))?;
            this.complete_task(task).await
        }
    }

    fn delete_index(
        &self,
        index: &IndexUid,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        let this = self.clone();
        let uid = index.as_str().to_string();
        async move {
            let task = this
                .client
                .delete_index(&uid)
                .await
                .map_err(|e| map_meili_error(&e))?;
            this.complete_task(task).await
        }
    }

    fn health(&self) -> impl Future<Output = Result<ComponentHealth, DomainError>> + Send {
        let this = self.clone();
        async move { this.health_inner().await }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::error::DomainCode;
    use meilisearch_sdk::errors::MeilisearchError;

    #[test]
    fn meilisearch_error_maps_timeout() {
        let err = map_meili_error(&Error::Timeout);
        assert_eq!(err.code(), DomainCode::Timeout);
    }

    #[test]
    fn meilisearch_error_maps_invalid_document() {
        let me = MeilisearchError {
            error_message: "invalid document id".into(),
            error_code: ErrorCode::InvalidDocumentId,
            error_type: ErrorType::InvalidRequest,
            error_link: "https://docs.meilisearch.com".into(),
        };
        let err = map_meili_error(&Error::Meilisearch(me));
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }
}
