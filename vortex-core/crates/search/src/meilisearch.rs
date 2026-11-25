//! Meilisearch adapter for `domain::TextSearch` (T-03-04).

use std::future::Future;
use std::time::Duration;

use domain::error::{DomainError, FieldError, InvalidInputCode, UnavailableCode};
use domain::health::{ComponentHealth, ComponentStatus};
use domain::producto::Producto;
use domain::{
    Id, IndexSettings, IndexUid, SearchDocument, SearchHit, SearchPage, SearchQuery, TextSearch,
    validate_index_swap,
};
use meilisearch_sdk::client::{Client, SwapIndexes};
use meilisearch_sdk::errors::{Error, ErrorCode, ErrorType};
use meilisearch_sdk::task_info::TaskInfo;
use serde::{Deserialize, Serialize};

const DEFAULT_TASK_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeiliDocument {
    id: String,
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
        let meili_docs: Vec<MeiliDocument> = documents
            .iter()
            .map(|doc| MeiliDocument {
                id: doc.id.to_string(),
            })
            .collect();
        let idx = self.client.index(index.as_str());
        let task = idx
            .add_or_replace(&meili_docs, Some("id"))
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
        let idx = self.client.index(index.as_str());
        let results = idx
            .search()
            .with_query(&query.q)
            .with_offset(query.offset as usize)
            .with_limit(query.limit as usize)
            .execute::<MeiliDocument>()
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

fn map_hits(
    hits: Vec<meilisearch_sdk::search::SearchResult<MeiliDocument>>,
) -> Result<Vec<SearchHit>, DomainError> {
    hits.into_iter()
        .map(|hit| {
            let id = hit
                .result
                .id
                .parse::<Id<Producto>>()
                .map_err(|_| meili_invalid_input("invalid hit id from meilisearch"))?;
            Ok(SearchHit { id })
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
        _index: &IndexUid,
        _settings: IndexSettings,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        std::future::ready(Ok(()))
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
