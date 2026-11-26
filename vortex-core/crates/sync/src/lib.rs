//! Index Sync Worker (outbox → Meilisearch, T-03-07).

mod meili_product_worker;
mod meili_reindex;

pub use meili_product_worker::{
    DEFAULT_POLL_INTERVAL, MAX_TRANSIENT_BACKOFF, MeiliProductWorker, transient_backoff,
};
pub use meili_reindex::{default_staging_uid, reindex_meilisearch};
