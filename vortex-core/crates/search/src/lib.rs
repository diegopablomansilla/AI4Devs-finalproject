//! Crate `search`: Meilisearch adapter and text-search infrastructure (T-03-04).

mod meilisearch;
mod productos_index_settings;

pub use meilisearch::MeiliTextSearch;
pub use productos_index_settings::productos_index_settings;
