//! Occupancy port for attribute definitions (T-02-23).

use crate::atributo_def::AtributoDef;
use crate::error::DomainError;
use crate::id::Id;
use crate::producto::ValorAtributo;

/// Lists attribute values stored on products for a definition id.
pub trait AttributeUsage {
    fn valores(
        &self,
        id: Id<AtributoDef>,
    ) -> impl std::future::Future<Output = Result<Vec<ValorAtributo>, DomainError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopUsage;

    impl AttributeUsage for NoopUsage {
        async fn valores(&self, _id: Id<AtributoDef>) -> Result<Vec<ValorAtributo>, DomainError> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn attribute_usage_trait_exposes_valores() {
        fn assert_usage<U: AttributeUsage>(u: &U) {
            let _f = u.valores(Id::generate());
        }
        assert_usage(&NoopUsage);
    }
}
