//! SurrealDB adapter for `AttributeUsage` (T-02-23).

use std::sync::Arc;

use domain::{AtributoDef, AttributeUsage, DomainError, Id, UnavailableCode, ValorAtributo};
use serde::Deserialize;
use surrealdb::RecordId;

use crate::Db;
use crate::producto::{SurrealValor, record_id_to_domain_id, surreal_valor_to_domain};

/// Reads `producto.atributos` values for a definition id (active and inactive).
pub struct ProductAttributeUsage {
    db: Arc<Db>,
}

impl ProductAttributeUsage {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }
}

#[derive(Debug, Deserialize)]
struct ProductAtributosRow {
    atributos: Vec<AtributoValorRow>,
}

#[derive(Debug, Deserialize)]
struct AtributoValorRow {
    def: RecordId,
    valor: SurrealValor,
}

impl AttributeUsage for ProductAttributeUsage {
    async fn valores(&self, id: Id<AtributoDef>) -> Result<Vec<ValorAtributo>, DomainError> {
        let stmt = "SELECT atributos FROM producto;";
        let response = self
            .db
            .inner
            .query(stmt)
            .await
            .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;
        let mut checked = response
            .check()
            .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;
        let rows: Vec<ProductAtributosRow> = checked
            .take(0)
            .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;

        let mut out = Vec::new();
        for row in rows {
            for attr in &row.atributos {
                let def_id = record_id_to_domain_id::<AtributoDef>(&attr.def)?;
                if def_id == id {
                    out.push(surreal_valor_to_domain(&attr.valor)?);
                }
            }
        }
        Ok(out)
    }
}
