//! Product recommendation query (4.6.2, T-04-06).

use std::future::Future;
use std::pin::Pin;

use domain::{DomainError, Id, Producto, Recomendaciones, RelationRepo};

/// Object-safe facade for HTTP injection (`Arc<dyn RecommendationGateway>`).
pub trait RecommendationGateway: Send + Sync {
    fn get(
        &self,
        product: Id<Producto>,
    ) -> Pin<Box<dyn Future<Output = Result<Recomendaciones, DomainError>> + Send + '_>>;
}

/// Recommendation panel query over `RelationRepo` (DC-05).
pub struct RecommendationService<R> {
    relations: R,
}

impl<R> RecommendationService<R> {
    pub fn new(relations: R) -> Self {
        Self { relations }
    }
}

impl<R> RecommendationService<R>
where
    R: RelationRepo + Send + Sync,
{
    pub async fn get(&self, product: Id<Producto>) -> Result<Recomendaciones, DomainError> {
        self.relations.recommendations_for_product(product).await
    }
}

impl<R> RecommendationGateway for RecommendationService<R>
where
    R: RelationRepo + Send + Sync,
{
    fn get(
        &self,
        product: Id<Producto>,
    ) -> Pin<Box<dyn Future<Output = Result<Recomendaciones, DomainError>> + Send + '_>> {
        Box::pin(self.get(product))
    }
}
