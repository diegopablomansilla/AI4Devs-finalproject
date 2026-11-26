//! Crate `recommend`: product relation and recommendation use cases (T-04-05, T-04-06).

mod recommendation;
mod relation;

pub use recommendation::{RecommendationGateway, RecommendationService};
pub use relation::{ListRelationsParams, RelationGateway, RelationService};
