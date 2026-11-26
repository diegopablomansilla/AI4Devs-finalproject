//! Crate `search`: Meilisearch adapter and text-search infrastructure (T-03-04).

mod meilisearch;
mod query;

pub use domain::productos_index_settings;
pub use meilisearch::MeiliTextSearch;
pub use query::{SearchGateway, SearchService};
