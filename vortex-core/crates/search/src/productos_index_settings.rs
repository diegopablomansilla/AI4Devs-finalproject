//! Index `productos` Meilisearch settings profile (T-03-05, docs/4.11.2 + H-03/H-04).

use domain::IndexSettings;
use domain::error::{DomainError, FieldError, InvalidInputCode};

const SEARCHABLE: &[&str] = &[
    "nombre",
    "tipo",
    "sku",
    "codigos",
    "marca",
    "atributos_texto",
    "descripcion",
];

const RANKING: &[&str] = &[
    "words",
    "typo",
    "proximity",
    "attribute",
    "sort",
    "exactness",
    "stock_disponible:desc",
];

const FILTERABLE_FIXED: &[&str] = &[
    "categoria.lvl0",
    "categoria.lvl1",
    "categoria.lvl2",
    "categoria.lvl3",
    "marca",
    "con_stock",
    "marca_id",
    "categoria_ids",
    "tipo",
];

const SORTABLE: &[&str] = &["precio", "stock_disponible"];

const TYPO_DISABLE: &[&str] = &["sku", "codigos"];

fn field_invalid_input(field: &str, message: &str) -> DomainError {
    DomainError::invalid_input(
        InvalidInputCode::InvalidInput,
        vec![FieldError::try_new(field, message).expect("field")],
    )
}

fn matches_atributo_clave(clave: &str) -> bool {
    if clave.is_empty() {
        return false;
    }
    let mut chars = clave.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn validate_facetable_clave(clave: &str) -> Result<&str, DomainError> {
    let trimmed = clave.trim();
    if trimmed.is_empty() {
        return Err(field_invalid_input("clave", "clave must not be empty"));
    }
    if trimmed.starts_with("attr_") {
        return Err(field_invalid_input(
            "clave",
            "clave must not include attr_ prefix",
        ));
    }
    if !matches_atributo_clave(trimmed) {
        return Err(field_invalid_input("clave", "invalid clave format"));
    }
    Ok(trimmed)
}

/// Builds 4.11.2 settings for the `productos` index (H-03/H-04; no synonyms).
pub fn productos_index_settings(
    claves_facetable: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<IndexSettings, DomainError> {
    let mut attr_filterables = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for clave in claves_facetable {
        let clave = validate_facetable_clave(clave.as_ref())?;
        if seen.insert(clave.to_string()) {
            attr_filterables.push(format!("attr_{clave}"));
        }
    }

    let mut filterable: Vec<String> = FILTERABLE_FIXED.iter().map(|s| (*s).to_string()).collect();
    filterable.extend(attr_filterables);

    Ok(IndexSettings {
        searchable_attributes: SEARCHABLE.iter().map(|s| (*s).to_string()).collect(),
        ranking_rules: RANKING.iter().map(|s| (*s).to_string()).collect(),
        filterable_attributes: filterable,
        sortable_attributes: SORTABLE.iter().map(|s| (*s).to_string()).collect(),
        typo_disable_on_attributes: TYPO_DISABLE.iter().map(|s| (*s).to_string()).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::error::DomainCode;

    #[test]
    fn productos_index_settings_includes_4_11_2_h03_h04() {
        let settings = productos_index_settings(["medida"]).expect("valid");
        assert!(!settings.is_empty());
        assert_eq!(
            settings.searchable_attributes,
            SEARCHABLE
                .iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            settings.ranking_rules.last().map(String::as_str),
            Some("stock_disponible:desc")
        );
        assert!(
            settings
                .filterable_attributes
                .contains(&"marca_id".to_string())
        );
        assert!(
            settings
                .filterable_attributes
                .contains(&"categoria_ids".to_string())
        );
        assert!(settings.filterable_attributes.contains(&"tipo".to_string()));
        assert!(
            settings
                .filterable_attributes
                .contains(&"attr_medida".to_string())
        );
        assert_eq!(
            settings.sortable_attributes,
            SORTABLE
                .iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            settings.typo_disable_on_attributes,
            TYPO_DISABLE
                .iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn productos_index_settings_omits_synonyms() {
        let with_attr = productos_index_settings(["medida"]).expect("valid");
        let without_attr = productos_index_settings(std::iter::empty::<&str>()).expect("valid");
        for settings in [&with_attr, &without_attr] {
            assert!(!settings.searchable_attributes.is_empty());
            assert!(
                settings
                    .filterable_attributes
                    .iter()
                    .any(|a| a == "marca_id" || a == "tipo")
            );
        }
        assert!(
            !without_attr
                .filterable_attributes
                .iter()
                .any(|a| a.starts_with("attr_"))
        );
    }

    #[test]
    fn productos_index_settings_rejects_invalid_clave() {
        for clave in ["", "Medida", "attr_medida", "medida-mm"] {
            let err = productos_index_settings([clave]).expect_err(clave);
            assert_eq!(err.code(), DomainCode::InvalidInput, "{clave}");
        }
    }
}
