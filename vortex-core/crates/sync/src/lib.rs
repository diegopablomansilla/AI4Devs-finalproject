//! Index Sync Worker (outbox → Meilisearch, T-03-07).

mod meili_product_worker;

pub use meili_product_worker::{
    DEFAULT_POLL_INTERVAL, MAX_TRANSIENT_BACKOFF, MeiliProductWorker, transient_backoff,
};
