//! Meilisearch search document 4.11.1 and builder (T-03-06).

use std::collections::BTreeMap;

use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;

use crate::atributo_def::AtributoDef;
use crate::categoria::{Categoria, MAX_DEPTH};
use crate::decimal::Decimal;
use crate::error::{DomainError, FieldError, InvalidInputCode};
use crate::id::{Id, InvalidId};
use crate::marca::Marca;
use crate::producto::{Producto, Ubicacion, UnidadMedida, ValorAtributo};

/// Category levels for Meilisearch (`categoria.lvl0`…`lvl3`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CategoriaMeiliLevels {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lvl0: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lvl1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lvl2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lvl3: Option<String>,
}

/// Denormalized product document for Meilisearch ([4.11.1](../../../../../docs/4-modelo.md#4111-documento)).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchDocument {
    pub id: Id<Producto>,
    pub sku: String,
    pub nombre: String,
    pub tipo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descripcion: Option<String>,
    pub marca: Option<String>,
    pub marca_id: Option<String>,
    pub categoria: CategoriaMeiliLevels,
    pub categoria_ids: Vec<String>,
    pub codigos: Vec<String>,
    pub atributos_texto: Vec<String>,
    #[serde(flatten)]
    pub attr_facetable: BTreeMap<String, String>,
    pub precio: f64,
    pub unidad_medida: String,
    pub stock_disponible: f64,
    pub con_stock: bool,
    pub ubicacion: Option<String>,
    pub imagen: Option<String>,
}

impl SearchDocument {
    /// Minimal document (adapter probes); other fields are empty or null.
    pub fn try_new(id: Id<Producto>) -> Self {
        Self {
            id,
            sku: String::new(),
            nombre: String::new(),
            tipo: String::new(),
            descripcion: None,
            marca: None,
            marca_id: None,
            categoria: CategoriaMeiliLevels::default(),
            categoria_ids: Vec::new(),
            codigos: Vec::new(),
            atributos_texto: Vec::new(),
            attr_facetable: BTreeMap::new(),
            precio: 0.0,
            unidad_medida: UnidadMedida::Unidad.as_str().to_string(),
            stock_disponible: 0.0,
            con_stock: false,
            ubicacion: None,
            imagen: None,
        }
    }

    pub fn try_from_ulid_str(s: &str) -> Result<Self, DomainError> {
        if s.contains(':') {
            return Err(field_invalid_input(
                "document.id",
                "id must not include table prefix",
            ));
        }
        let id = s
            .parse::<Id<Producto>>()
            .map_err(|InvalidId| field_invalid_input("document.id", "id must be a valid ULID"))?;
        Ok(Self::try_new(id))
    }
}

/// Builds a Meilisearch document from catalog entities (pure; no I/O).
pub fn build_search_document(
    producto: &Producto,
    marca: Option<&Marca>,
    categoria: &Categoria,
    defs: &[AtributoDef],
) -> Result<SearchDocument, DomainError> {
    if producto.categoria != categoria.id {
        return Err(field_invalid_input(
            "categoria",
            "categoria does not match producto.categoria",
        ));
    }

    let (marca_nombre, marca_id) = match (&producto.marca, marca) {
        (None, None) => (None, None),
        (Some(pm), Some(m)) if pm == &m.id => (Some(m.nombre.clone()), Some(m.id.to_string())),
        _ => {
            return Err(field_invalid_input(
                "marca",
                "marca does not match producto.marca",
            ));
        }
    };

    let mut atributos_texto = Vec::new();
    let mut attr_facetable = BTreeMap::new();

    for av in &producto.atributos {
        let def = defs.iter().find(|d| d.id == av.def).ok_or_else(|| {
            field_invalid_input("atributos", "missing atributo_def for product attribute")
        })?;
        if def.buscable {
            atributos_texto.push(format_atributo_line(def, &av.valor));
        }
        if def.facetable {
            let key = format!("attr_{}", def.clave);
            attr_facetable.insert(key, format_atributo_valor(def, &av.valor));
        }
    }

    let stock_wire = decimal_to_f64(producto.stock)?;
    let con_stock = producto.stock > zero_decimal();
    let precio = decimal_to_f64(producto.precio)?;

    Ok(SearchDocument {
        id: producto.id.clone(),
        sku: producto.sku.as_str().to_string(),
        nombre: producto.nombre.clone(),
        tipo: producto.tipo.clone(),
        descripcion: producto.descripcion.clone(),
        marca: marca_nombre,
        marca_id,
        categoria: categoria_levels(&categoria.ruta),
        categoria_ids: categoria_ids_wire(categoria),
        codigos: producto.codigos.iter().map(|c| c.valor.clone()).collect(),
        atributos_texto,
        attr_facetable,
        precio,
        unidad_medida: producto.unidad_medida.as_str().to_string(),
        stock_disponible: stock_wire,
        con_stock,
        ubicacion: producto.ubicacion.as_ref().and_then(format_ubicacion),
        imagen: None,
    })
}

fn categoria_levels(ruta: &[String]) -> CategoriaMeiliLevels {
    let mut levels = CategoriaMeiliLevels::default();
    for (i, _) in ruta.iter().enumerate().take(MAX_DEPTH) {
        let val = ruta[..=i].join(" > ");
        match i {
            0 => levels.lvl0 = Some(val),
            1 => levels.lvl1 = Some(val),
            2 => levels.lvl2 = Some(val),
            3 => levels.lvl3 = Some(val),
            _ => {}
        }
    }
    levels
}

fn categoria_ids_wire(categoria: &Categoria) -> Vec<String> {
    let mut ids: Vec<String> = categoria
        .ancestros
        .iter()
        .map(|id| id.to_string())
        .collect();
    ids.push(categoria.id.to_string());
    ids
}

fn format_ubicacion(u: &Ubicacion) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(p) = &u.pasillo {
        let trimmed = p.trim();
        if !trimmed.is_empty() {
            parts.push(format!("P{trimmed}"));
        }
    }
    if let Some(e) = &u.estante {
        let trimmed = e.trim();
        if !trimmed.is_empty() {
            parts.push(format!("E-{trimmed}"));
        }
    }
    if let Some(pos) = &u.posicion {
        let trimmed = pos.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed.to_string());
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

fn format_atributo_line(def: &AtributoDef, valor: &ValorAtributo) -> String {
    format!("{}: {}", def.etiqueta, format_atributo_valor(def, valor))
}

fn format_atributo_valor(def: &AtributoDef, valor: &ValorAtributo) -> String {
    match valor {
        ValorAtributo::Texto(s) => s.clone(),
        ValorAtributo::Numero(d) => {
            let base = d.to_string();
            match &def.unidad {
                Some(u) if !u.is_empty() => format!("{base} {u}"),
                _ => base,
            }
        }
        ValorAtributo::Booleano(b) => b.to_string(),
    }
}

fn zero_decimal() -> Decimal {
    Decimal::from_str("0").expect("zero")
}

fn decimal_to_f64(d: Decimal) -> Result<f64, DomainError> {
    d.inner()
        .to_f64()
        .ok_or_else(|| field_invalid_input("decimal", "decimal out of f64 range"))
}

fn field_invalid_input(field: &str, message: &str) -> DomainError {
    DomainError::invalid_input(
        InvalidInputCode::InvalidInput,
        vec![FieldError::try_new(field, message).expect("field")],
    )
}

use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atributo_def::{AtributoDefDraft, TipoAtributo};
    use crate::categoria::{CategoriaDraft, Slug};
    use crate::error::DomainCode;
    use crate::marca::MarcaDraft;
    use crate::producto::{
        AtributoValor, Codigo, ProductoDraft, TipoCodigo, Ubicacion, UnidadMedida,
    };
    use crate::timestamp::Timestamp;
    use serde_json::{Value, json};

    const PROD_ID: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";
    const MARCA_ID: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D7";
    const CAT_ROOT_ID: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6C9";
    const CAT_LEAF_ID: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";
    const DEF_MEDIDA: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6A1";
    const DEF_LARGO: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6A2";
    const DEF_GRADO: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6A3";

    fn ts() -> Timestamp {
        Timestamp::from_unix(1_700_000_000, 0)
    }

    fn parse_id<T>(s: &str) -> Id<T> {
        s.parse().expect("ulid")
    }

    fn example_defs() -> Vec<AtributoDef> {
        vec![
            AtributoDef::try_new(AtributoDefDraft {
                id: parse_id(DEF_MEDIDA),
                clave: "medida".into(),
                etiqueta: "Medida".into(),
                tipo: TipoAtributo::Texto,
                unidad: None,
                opciones: None,
                facetable: true,
                buscable: true,
                orden: 1,
                activo: true,
                creado_en: ts(),
                actualizado_en: ts(),
            })
            .expect("medida"),
            AtributoDef::try_new(AtributoDefDraft {
                id: parse_id(DEF_LARGO),
                clave: "largo".into(),
                etiqueta: "Largo".into(),
                tipo: TipoAtributo::Numero,
                unidad: Some("mm".into()),
                opciones: None,
                facetable: false,
                buscable: true,
                orden: 2,
                activo: true,
                creado_en: ts(),
                actualizado_en: ts(),
            })
            .expect("largo"),
            AtributoDef::try_new(AtributoDefDraft {
                id: parse_id(DEF_GRADO),
                clave: "grado".into(),
                etiqueta: "Grado".into(),
                tipo: TipoAtributo::Numero,
                unidad: None,
                opciones: None,
                facetable: true,
                buscable: true,
                orden: 3,
                activo: true,
                creado_en: ts(),
                actualizado_en: ts(),
            })
            .expect("grado"),
        ]
    }

    fn example_marca() -> Marca {
        Marca::try_new(MarcaDraft {
            id: parse_id(MARCA_ID),
            nombre: "Fixser".into(),
            slug: Slug::parse("fixser").expect("slug"),
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("marca")
    }

    fn example_categoria() -> Categoria {
        Categoria::try_new(CategoriaDraft {
            id: parse_id(CAT_LEAF_ID),
            nombre: "Pernos".into(),
            slug: Slug::parse("pernos").expect("slug"),
            padre: Some(parse_id(CAT_ROOT_ID)),
            ancestros: vec![parse_id(CAT_ROOT_ID)],
            ruta: vec!["Fijaciones".into(), "Pernos".into()],
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("categoria")
    }

    fn example_producto() -> Producto {
        let defs = example_defs();
        Producto::try_new(
            ProductoDraft {
                id: parse_id(PROD_ID),
                sku: "PER-M8X40-ZN".into(),
                nombre: "Perno hexagonal M8 x 40 zincado".into(),
                tipo: "Perno".into(),
                descripcion: Some("Perno de acero con cabeza hexagonal...".into()),
                marca: Some(parse_id(MARCA_ID)),
                categoria: parse_id(CAT_LEAF_ID),
                atributos: vec![
                    AtributoValor {
                        def: parse_id(DEF_MEDIDA),
                        valor: ValorAtributo::Texto("M8".into()),
                    },
                    AtributoValor {
                        def: parse_id(DEF_LARGO),
                        valor: ValorAtributo::Numero("40".parse().expect("num")),
                    },
                    AtributoValor {
                        def: parse_id(DEF_GRADO),
                        valor: ValorAtributo::Numero("5".parse().expect("num")),
                    },
                ],
                codigos: vec![Codigo::try_new(TipoCodigo::Fabricante, "HX-0840Z").expect("codigo")],
                unidad_medida: UnidadMedida::Unidad,
                precio: "120.50".parse().expect("precio"),
                ubicacion: Some(Ubicacion {
                    pasillo: Some("3".into()),
                    estante: Some("B".into()),
                    posicion: Some("12".into()),
                }),
                stock: "34".parse().expect("stock"),
                activo: true,
                version: 1,
                creado_en: ts(),
                actualizado_en: ts(),
            },
            &defs,
        )
        .expect("producto")
    }

    fn doc_to_value(doc: &SearchDocument) -> Value {
        serde_json::to_value(doc).expect("json")
    }

    #[test]
    fn build_search_document_snapshot_4_4_4() {
        let producto = example_producto();
        let marca = example_marca();
        let categoria = example_categoria();
        let defs = example_defs();
        let doc = build_search_document(&producto, Some(&marca), &categoria, &defs).expect("build");
        let value = doc_to_value(&doc);
        let expected = json!({
            "id": PROD_ID,
            "sku": "PER-M8X40-ZN",
            "nombre": "Perno hexagonal M8 x 40 zincado",
            "tipo": "Perno",
            "marca": "Fixser",
            "marca_id": MARCA_ID,
            "categoria": {
                "lvl0": "Fijaciones",
                "lvl1": "Fijaciones > Pernos"
            },
            "categoria_ids": [CAT_ROOT_ID, CAT_LEAF_ID],
            "codigos": ["HX-0840Z"],
            "atributos_texto": ["Medida: M8", "Largo: 40 mm", "Grado: 5"],
            "attr_medida": "M8",
            "attr_grado": "5",
            "descripcion": "Perno de acero con cabeza hexagonal...",
            "precio": 120.5,
            "unidad_medida": "unidad",
            "stock_disponible": 34.0,
            "con_stock": true,
            "ubicacion": "P3 · E-B · 12",
            "imagen": null
        });
        assert_eq!(value, expected);
    }

    #[test]
    fn build_search_document_formats_ubicacion() {
        let mut producto = example_producto();
        producto.ubicacion = Some(Ubicacion {
            pasillo: Some("3".into()),
            estante: Some("B".into()),
            posicion: Some("12".into()),
        });
        let doc = build_search_document(
            &producto,
            Some(&example_marca()),
            &example_categoria(),
            &example_defs(),
        )
        .expect("build");
        assert_eq!(doc.ubicacion.as_deref(), Some("P3 · E-B · 12"));
    }

    #[test]
    fn build_search_document_atributos_texto_only_buscable() {
        let doc = build_search_document(
            &example_producto(),
            Some(&example_marca()),
            &example_categoria(),
            &example_defs(),
        )
        .expect("build");
        assert!(doc.atributos_texto.iter().any(|s| s.starts_with("Largo:")));
        assert!(!doc.attr_facetable.contains_key("attr_largo"));
    }

    #[test]
    fn build_search_document_attr_keys_only_facetable() {
        let doc = build_search_document(
            &example_producto(),
            Some(&example_marca()),
            &example_categoria(),
            &example_defs(),
        )
        .expect("build");
        assert_eq!(doc.attr_facetable.get("attr_medida"), Some(&"M8".into()));
        assert_eq!(doc.attr_facetable.get("attr_grado"), Some(&"5".into()));
        assert!(!doc.attr_facetable.contains_key("attr_largo"));
    }

    #[test]
    fn build_search_document_imagen_is_null() {
        let doc = build_search_document(
            &example_producto(),
            Some(&example_marca()),
            &example_categoria(),
            &example_defs(),
        )
        .expect("build");
        assert!(doc.imagen.is_none());
        assert_eq!(doc_to_value(&doc)["imagen"], Value::Null);
    }

    #[test]
    fn build_search_document_categoria_ids_ancestors_and_self() {
        let doc = build_search_document(
            &example_producto(),
            Some(&example_marca()),
            &example_categoria(),
            &example_defs(),
        )
        .expect("build");
        assert_eq!(doc.categoria_ids, vec![CAT_ROOT_ID, CAT_LEAF_ID]);
    }

    #[test]
    fn build_search_document_con_stock_false_when_zero() {
        let mut producto = example_producto();
        producto.stock = "0".parse().expect("stock");
        let doc = build_search_document(
            &producto,
            Some(&example_marca()),
            &example_categoria(),
            &example_defs(),
        )
        .expect("build");
        assert!(!doc.con_stock);
        assert_eq!(doc.stock_disponible, 0.0);
    }

    #[test]
    fn build_search_document_without_marca() {
        let defs = example_defs();
        let cat = example_categoria();
        let producto = Producto::try_new(
            ProductoDraft {
                id: parse_id(PROD_ID),
                sku: "SKU".into(),
                nombre: "Sin marca".into(),
                tipo: "Perno".into(),
                descripcion: None,
                marca: None,
                categoria: parse_id(CAT_LEAF_ID),
                atributos: vec![],
                codigos: vec![],
                unidad_medida: UnidadMedida::Unidad,
                precio: "1".parse().expect("precio"),
                ubicacion: None,
                stock: "1".parse().expect("stock"),
                activo: true,
                version: 1,
                creado_en: ts(),
                actualizado_en: ts(),
            },
            &defs,
        )
        .expect("producto");
        let doc = build_search_document(&producto, None, &cat, &defs).expect("build");
        assert!(doc.marca.is_none());
        assert!(doc.marca_id.is_none());
    }

    #[test]
    fn build_search_document_rejects_marca_mismatch() {
        let producto = example_producto();
        let wrong = Marca::try_new(MarcaDraft {
            id: parse_id("01J9ZKA7M3Q8R2T5V9X1Z4B6D9"),
            nombre: "Otra".into(),
            slug: Slug::parse("otra").expect("slug"),
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("marca");
        let err = build_search_document(
            &producto,
            Some(&wrong),
            &example_categoria(),
            &example_defs(),
        )
        .expect_err("mismatch");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn build_search_document_rejects_categoria_mismatch() {
        let producto = example_producto();
        let mut cat = example_categoria();
        cat.id = parse_id("01J9ZKA7M3Q8R2T5V9X1Z4B6D9");
        let err = build_search_document(&producto, Some(&example_marca()), &cat, &example_defs())
            .expect_err("mismatch");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn build_search_document_rejects_missing_atributo_def() {
        let producto = example_producto();
        let err =
            build_search_document(&producto, Some(&example_marca()), &example_categoria(), &[])
                .expect_err("missing def");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }
}
