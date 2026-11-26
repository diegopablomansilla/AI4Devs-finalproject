//! Product entity, value objects, attribute validation and `ProductRepo` port (T-02-16).

use std::future::Future;
use std::str::FromStr;

use crate::atributo_def::{AtributoDef, TipoAtributo};
use crate::categoria::Categoria;
use crate::decimal::Decimal;
use crate::error::{DomainError, FieldError, InvalidInputCode};
use crate::id::Id;
use crate::marca::Marca;
use crate::timestamp::Timestamp;

const MAX_SKU_LEN: usize = 64;
const MAX_NOMBRE_LEN: usize = 200;
const MAX_TIPO_LEN: usize = 60;
const MAX_DESCRIPCION_LEN: usize = 4000;
const MAX_CODIGO_VALOR_LEN: usize = 64;

/// Stock-keeping unit (OpenAPI `ProductCreate.sku`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sku(String);

impl Sku {
    pub fn try_new(raw: &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(field_invalid_input("sku", "sku must not be empty"));
        }
        if trimmed.len() > MAX_SKU_LEN {
            return Err(field_invalid_input(
                "sku",
                "sku must be at most 64 characters",
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Product code type (model 4.4.4 — Spanish wire values).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoCodigo {
    Fabricante,
    Oem,
    Barras,
}

impl TipoCodigo {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fabricante => "fabricante",
            Self::Oem => "oem",
            Self::Barras => "barras",
        }
    }

    pub fn parse_wire(s: &str) -> Result<Self, DomainError> {
        match s {
            "fabricante" => Ok(Self::Fabricante),
            "oem" => Ok(Self::Oem),
            "barras" => Ok(Self::Barras),
            _ => Err(field_invalid_input("codigos[].tipo", "invalid codigo tipo")),
        }
    }
}

impl FromStr for TipoCodigo {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_wire(s)
    }
}

/// Product identifier code (`fabricante` / `oem` / `barras`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Codigo {
    pub tipo: TipoCodigo,
    pub valor: String,
}

impl Codigo {
    pub fn try_new(tipo: TipoCodigo, valor: impl Into<String>) -> Result<Self, DomainError> {
        let valor = valor.into().trim().to_string();
        if valor.is_empty() {
            return Err(field_invalid_input(
                "codigos[].valor",
                "codigo valor must not be empty",
            ));
        }
        if valor.len() > MAX_CODIGO_VALOR_LEN {
            return Err(field_invalid_input(
                "codigos[].valor",
                "codigo valor must be at most 64 characters",
            ));
        }
        Ok(Self { tipo, valor })
    }
}

/// Unit of measure (model 4.4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnidadMedida {
    Unidad,
    Par,
    Caja,
    Metro,
    Kilogramo,
    Litro,
}

impl UnidadMedida {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unidad => "unidad",
            Self::Par => "par",
            Self::Caja => "caja",
            Self::Metro => "metro",
            Self::Kilogramo => "kilogramo",
            Self::Litro => "litro",
        }
    }

    pub fn parse_wire(s: &str) -> Result<Self, DomainError> {
        match s {
            "unidad" => Ok(Self::Unidad),
            "par" => Ok(Self::Par),
            "caja" => Ok(Self::Caja),
            "metro" => Ok(Self::Metro),
            "kilogramo" => Ok(Self::Kilogramo),
            "litro" => Ok(Self::Litro),
            _ => Err(field_invalid_input(
                "unidad_medida",
                "invalid unidad_medida",
            )),
        }
    }
}

impl FromStr for UnidadMedida {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_wire(s)
    }
}

/// Warehouse location (model `ubicacion`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ubicacion {
    pub pasillo: Option<String>,
    pub estante: Option<String>,
    pub posicion: Option<String>,
}

/// Formats `ubicacion` for search and recommendation panels (`P3 · E-B · 12`).
pub fn format_ubicacion_linea(u: &Ubicacion) -> Option<String> {
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

/// Attribute value on a product (model `atributos[*].valor`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValorAtributo {
    Texto(String),
    Numero(Decimal),
    Booleano(bool),
}

/// Product attribute binding (`def` + `valor`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtributoValor {
    pub def: Id<AtributoDef>,
    pub valor: ValorAtributo,
}

/// Snapshot used to construct a validated `Producto`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductoDraft {
    pub id: Id<Producto>,
    pub sku: String,
    pub nombre: String,
    pub tipo: String,
    pub descripcion: Option<String>,
    pub marca: Option<Id<Marca>>,
    pub categoria: Id<Categoria>,
    pub atributos: Vec<AtributoValor>,
    pub codigos: Vec<Codigo>,
    pub unidad_medida: UnidadMedida,
    pub precio: Decimal,
    pub ubicacion: Option<Ubicacion>,
    pub stock: Decimal,
    pub activo: bool,
    pub version: i32,
    pub creado_en: Timestamp,
    pub actualizado_en: Timestamp,
}

/// Catalog product (model 4.4.4).
#[derive(Debug, Clone, PartialEq)]
pub struct Producto {
    pub id: Id<Producto>,
    pub sku: Sku,
    pub nombre: String,
    pub tipo: String,
    pub descripcion: Option<String>,
    pub marca: Option<Id<Marca>>,
    pub categoria: Id<Categoria>,
    pub atributos: Vec<AtributoValor>,
    pub codigos: Vec<Codigo>,
    pub unidad_medida: UnidadMedida,
    pub precio: Decimal,
    pub ubicacion: Option<Ubicacion>,
    pub stock: Decimal,
    pub activo: bool,
    pub version: i32,
    pub creado_en: Timestamp,
    pub actualizado_en: Timestamp,
}

impl Producto {
    pub fn try_new(draft: ProductoDraft, defs: &[AtributoDef]) -> Result<Self, DomainError> {
        let ProductoDraft {
            id,
            sku,
            nombre,
            tipo,
            descripcion,
            marca,
            categoria,
            atributos,
            codigos,
            unidad_medida,
            precio,
            ubicacion,
            stock,
            activo,
            version,
            creado_en,
            actualizado_en,
        } = draft;

        let sku = Sku::try_new(&sku)?;
        let nombre = validate_nombre(&nombre)?;
        let tipo = validate_tipo_producto(&tipo)?;
        let descripcion = validate_descripcion(descripcion)?;
        validate_non_negative(precio, "precio")?;
        validate_non_negative(stock, "stock")?;
        if version < 1 {
            return Err(field_invalid_input("version", "version must be at least 1"));
        }
        for codigo in &codigos {
            let _ = Codigo::try_new(codigo.tipo, codigo.valor.clone())?;
        }
        validate_atributos(&atributos, defs)?;

        Ok(Self {
            id,
            sku,
            nombre,
            tipo,
            descripcion,
            marca,
            categoria,
            atributos,
            codigos,
            unidad_medida,
            precio,
            ubicacion,
            stock,
            activo,
            version,
            creado_en,
            actualizado_en,
        })
    }
}

fn validate_nombre(nombre: &str) -> Result<String, DomainError> {
    let trimmed = nombre.trim();
    if trimmed.is_empty() {
        return Err(field_invalid_input("nombre", "nombre must not be empty"));
    }
    if trimmed.len() > MAX_NOMBRE_LEN {
        return Err(field_invalid_input(
            "nombre",
            "nombre must be at most 200 characters",
        ));
    }
    Ok(trimmed.to_string())
}

fn validate_tipo_producto(tipo: &str) -> Result<String, DomainError> {
    let trimmed = tipo.trim();
    if trimmed.is_empty() {
        return Err(field_invalid_input("tipo", "tipo must not be empty"));
    }
    if trimmed.len() > MAX_TIPO_LEN {
        return Err(field_invalid_input(
            "tipo",
            "tipo must be at most 60 characters",
        ));
    }
    Ok(trimmed.to_string())
}

fn validate_descripcion(descripcion: Option<String>) -> Result<Option<String>, DomainError> {
    match descripcion {
        None => Ok(None),
        Some(d) => {
            if d.len() > MAX_DESCRIPCION_LEN {
                return Err(field_invalid_input(
                    "descripcion",
                    "descripcion must be at most 4000 characters",
                ));
            }
            Ok(Some(d))
        }
    }
}

fn validate_non_negative(value: Decimal, field: &str) -> Result<(), DomainError> {
    if value < Decimal::from_str("0").expect("zero") {
        return Err(field_invalid_input(field, "must not be negative"));
    }
    Ok(())
}

fn validate_atributos(
    atributos: &[AtributoValor],
    defs: &[AtributoDef],
) -> Result<(), DomainError> {
    let mut seen: Vec<Id<AtributoDef>> = Vec::new();
    for (n, attr) in atributos.iter().enumerate() {
        if seen.contains(&attr.def) {
            return Err(field_invalid_input(
                &format!("attributes[{n}].definitionId"),
                "duplicate attribute definition",
            ));
        }
        seen.push(attr.def.clone());
        let Some(def) = defs.iter().find(|d| d.id == attr.def) else {
            return Err(field_invalid_input(
                &format!("attributes[{n}].definitionId"),
                "unknown attribute definition",
            ));
        };
        validate_valor_for_def(n, def, &attr.valor)?;
    }
    Ok(())
}

/// Whether `valor` is valid for a definition with `tipo` and `opciones` (T-02-23 occupancy).
pub fn valor_compatible_con(
    tipo: TipoAtributo,
    opciones: &Option<Vec<String>>,
    valor: &ValorAtributo,
) -> bool {
    match tipo {
        TipoAtributo::Texto => matches!(valor, ValorAtributo::Texto(_)),
        TipoAtributo::Numero => matches!(valor, ValorAtributo::Numero(_)),
        TipoAtributo::Booleano => matches!(valor, ValorAtributo::Booleano(_)),
        TipoAtributo::Enum => match valor {
            ValorAtributo::Texto(s) => {
                let trimmed = s.trim();
                opciones
                    .as_ref()
                    .is_some_and(|opts| opts.iter().any(|o| o.trim() == trimmed))
            }
            _ => false,
        },
    }
}

fn validate_valor_for_def(
    n: usize,
    def: &AtributoDef,
    valor: &ValorAtributo,
) -> Result<(), DomainError> {
    let field = format!("attributes[{n}].value");
    if valor_compatible_con(def.tipo, &def.opciones, valor) {
        Ok(())
    } else {
        let msg = match def.tipo {
            TipoAtributo::Texto => "expected text value",
            TipoAtributo::Numero => "expected number value",
            TipoAtributo::Booleano => "expected boolean value",
            TipoAtributo::Enum => match valor {
                ValorAtributo::Texto(_) => "value not in enum options",
                _ => "expected enum text value",
            },
        };
        Err(field_invalid_input(&field, msg))
    }
}

/// Resolved attribute on a product sheet (OpenAPI `AttributeValue` metadata).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtributoFicha {
    pub def: Id<AtributoDef>,
    pub clave: String,
    pub etiqueta: String,
    pub unidad: Option<String>,
    pub valor: ValorAtributo,
}

/// Product sheet with hydrated marca, category path and resolved attributes.
#[derive(Debug, Clone, PartialEq)]
pub struct FichaProducto {
    pub producto: Producto,
    pub marca: Option<Marca>,
    pub categoria: Categoria,
    pub atributos: Vec<AtributoFicha>,
}

/// Input for `ProductRepo::update` (optimistic lock; no `stock` nor `activo`).
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateProducto {
    pub id: Id<Producto>,
    pub version: i32,
    pub sku: String,
    pub nombre: String,
    pub tipo: String,
    pub descripcion: Option<String>,
    pub marca: Option<Id<Marca>>,
    pub categoria: Id<Categoria>,
    pub atributos: Vec<AtributoValor>,
    pub codigos: Vec<Codigo>,
    pub unidad_medida: UnidadMedida,
    pub precio: Decimal,
    pub ubicacion: Option<Ubicacion>,
}

/// Admin list filters (Spanish field names; HTTP maps query params in T-02-27).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiltroProducto {
    pub categoria: Option<Id<Categoria>>,
    pub marca: Option<Id<Marca>>,
    pub activo: bool,
}

/// Lightweight product row for admin lists (no ficha / attributes).
#[derive(Debug, Clone, PartialEq)]
pub struct ResumenProducto {
    pub id: Id<Producto>,
    pub sku: String,
    pub nombre: String,
    pub tipo: String,
    pub precio: Decimal,
    pub stock: Decimal,
    pub activo: bool,
    pub marca_nombre: Option<String>,
    pub categoria_ruta: Vec<String>,
}

/// Input for `ProductRepo::create`.
#[derive(Debug, Clone, PartialEq)]
pub struct NewProducto {
    pub sku: String,
    pub nombre: String,
    pub tipo: String,
    pub descripcion: Option<String>,
    pub marca: Option<Id<Marca>>,
    pub categoria: Id<Categoria>,
    pub atributos: Vec<AtributoValor>,
    pub codigos: Vec<Codigo>,
    pub unidad_medida: UnidadMedida,
    pub precio: Decimal,
    pub ubicacion: Option<Ubicacion>,
    pub stock_inicial: Decimal,
    pub terminal: Option<String>,
}

pub trait ProductRepo {
    fn create(
        &self,
        input: NewProducto,
    ) -> impl Future<Output = Result<Producto, DomainError>> + Send;

    fn get(
        &self,
        id: Id<Producto>,
    ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send;

    fn update(
        &self,
        input: UpdateProducto,
    ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send;

    fn set_activo(
        &self,
        id: Id<Producto>,
        activo: bool,
    ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send;

    fn list(
        &self,
        filtro: FiltroProducto,
        after: Option<Id<Producto>>,
        take: u32,
    ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send;

    fn find_by_code(
        &self,
        code: String,
    ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send;
}

fn field_invalid_input(field: &str, message: &str) -> DomainError {
    DomainError::invalid_input(
        InvalidInputCode::InvalidInput,
        vec![FieldError::try_new(field, message).expect("field")],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atributo_def::{AtributoDef, AtributoDefDraft};
    use crate::error::{DomainCode, UnavailableCode};

    const TS: &str = "2026-09-24T21:33:00Z";

    fn ts() -> Timestamp {
        TS.parse().expect("timestamp")
    }

    fn def_medida_texto() -> AtributoDef {
        AtributoDef::try_new(AtributoDefDraft {
            id: Id::generate(),
            clave: "medida".into(),
            etiqueta: "Medida".into(),
            tipo: TipoAtributo::Texto,
            unidad: Some("mm".into()),
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("medida def")
    }

    fn def_largo_numero() -> AtributoDef {
        let id = Id::<AtributoDef>::generate();
        AtributoDef {
            id,
            clave: "largo".into(),
            etiqueta: "Largo".into(),
            tipo: TipoAtributo::Numero,
            unidad: Some("mm".into()),
            opciones: None,
            facetable: false,
            buscable: false,
            orden: 1,
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        }
    }

    fn def_grado_enum() -> AtributoDef {
        AtributoDef::try_new(AtributoDefDraft {
            id: Id::generate(),
            clave: "grado".into(),
            etiqueta: "Grado".into(),
            tipo: TipoAtributo::Enum,
            unidad: None,
            opciones: Some(vec!["2".into(), "5".into(), "8".into()]),
            facetable: true,
            buscable: true,
            orden: 2,
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("grado def")
    }

    fn invalid_fields(err: &DomainError) -> &[FieldError] {
        match err {
            DomainError::InvalidInput { errors, .. } => errors,
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn codigo_constructs_fabricante() {
        let codigo = Codigo::try_new(TipoCodigo::Fabricante, "HX-0840Z").expect("codigo");
        assert_eq!(codigo.tipo, TipoCodigo::Fabricante);
        assert_eq!(codigo.valor, "HX-0840Z");
    }

    #[test]
    fn unidad_medida_parses_unidad() {
        let u = UnidadMedida::from_str("unidad").expect("unidad");
        assert_eq!(u, UnidadMedida::Unidad);
    }

    #[test]
    fn codigo_rejects_unknown_tipo() {
        let err = TipoCodigo::from_str("ean").expect_err("ean");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn unidad_medida_rejects_english_or_unknown() {
        for bad in ["unit", "docena"] {
            let err = UnidadMedida::from_str(bad).expect_err(bad);
            assert_eq!(err.code(), DomainCode::InvalidInput);
        }
    }

    #[test]
    fn new_producto_accepts_optional_terminal() {
        let with_terminal = NewProducto {
            sku: "PER-M8X40-ZN".into(),
            nombre: "Perno".into(),
            tipo: "Perno".into(),
            descripcion: None,
            marca: None,
            categoria: Id::generate(),
            atributos: vec![],
            codigos: vec![],
            unidad_medida: UnidadMedida::Unidad,
            precio: "0".parse().expect("precio"),
            ubicacion: None,
            stock_inicial: "0".parse().expect("stock"),
            terminal: Some("caja-1".into()),
        };
        assert_eq!(with_terminal.terminal.as_deref(), Some("caja-1"));

        let without_terminal = NewProducto {
            terminal: None,
            ..with_terminal
        };
        assert!(without_terminal.terminal.is_none());
    }

    fn draft_perno_base(atributos: Vec<AtributoValor>, defs: &[AtributoDef]) -> ProductoDraft {
        let medida = defs.iter().find(|d| d.clave == "medida").expect("medida");
        let largo = defs.iter().find(|d| d.clave == "largo").expect("largo");
        let grado = defs.iter().find(|d| d.clave == "grado").expect("grado");
        let attrs = if atributos.is_empty() {
            vec![
                AtributoValor {
                    def: medida.id.clone(),
                    valor: ValorAtributo::Texto("M8".into()),
                },
                AtributoValor {
                    def: largo.id.clone(),
                    valor: ValorAtributo::Numero("40".parse().expect("40")),
                },
                AtributoValor {
                    def: grado.id.clone(),
                    valor: ValorAtributo::Texto("5".into()),
                },
            ]
        } else {
            atributos
        };
        ProductoDraft {
            id: Id::generate(),
            sku: "PER-M8X40-ZN".into(),
            nombre: "Perno hexagonal M8 x 40 zincado".into(),
            tipo: "Perno".into(),
            descripcion: None,
            marca: Some(Id::generate()),
            categoria: Id::generate(),
            atributos: attrs,
            codigos: vec![Codigo::try_new(TipoCodigo::Fabricante, "HX-0840Z").expect("codigo")],
            unidad_medida: UnidadMedida::Unidad,
            precio: "120.50".parse().expect("precio"),
            ubicacion: None,
            stock: "0".parse().expect("stock"),
            activo: true,
            version: 1,
            creado_en: ts(),
            actualizado_en: ts(),
        }
    }

    #[test]
    fn producto_constructs_perno_m8x40_zn() {
        let defs = vec![def_medida_texto(), def_largo_numero(), def_grado_enum()];
        let draft = draft_perno_base(vec![], &defs);
        let id = draft.id.clone();
        let producto = Producto::try_new(draft, &defs).expect("producto");
        assert!(!id.to_string().contains(':'));
        assert_eq!(producto.sku.as_str(), "PER-M8X40-ZN");
        assert_eq!(producto.nombre, "Perno hexagonal M8 x 40 zincado");
        assert!(producto.activo);
        assert_eq!(producto.version, 1);
    }

    #[test]
    fn producto_rejects_blank_sku() {
        let defs: Vec<AtributoDef> = vec![];
        let draft = ProductoDraft {
            sku: "   ".into(),
            nombre: "Test".into(),
            tipo: "Perno".into(),
            descripcion: None,
            marca: None,
            categoria: Id::generate(),
            atributos: vec![],
            codigos: vec![],
            unidad_medida: UnidadMedida::Unidad,
            precio: "1".parse().unwrap(),
            ubicacion: None,
            stock: "0".parse().unwrap(),
            activo: true,
            version: 1,
            id: Id::generate(),
            creado_en: ts(),
            actualizado_en: ts(),
        };
        let err = Producto::try_new(draft, &defs).expect_err("blank sku");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn producto_rejects_negative_precio() {
        let defs = vec![def_medida_texto(), def_largo_numero(), def_grado_enum()];
        let mut draft = draft_perno_base(vec![], &defs);
        draft.precio = "-1".parse().expect("negative");
        let err = Producto::try_new(draft, &defs).expect_err("negative precio");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn producto_accepts_matching_atributos() {
        let defs = vec![def_medida_texto(), def_largo_numero(), def_grado_enum()];
        let producto = Producto::try_new(draft_perno_base(vec![], &defs), &defs).expect("ok");
        assert_eq!(producto.atributos.len(), 3);
    }

    #[test]
    fn producto_rejects_cuarenta_on_number_attribute() {
        let defs = vec![def_largo_numero()];
        let largo = &defs[0];
        let draft = ProductoDraft {
            atributos: vec![AtributoValor {
                def: largo.id.clone(),
                valor: ValorAtributo::Texto("cuarenta".into()),
            }],
            sku: "SKU-1".into(),
            nombre: "Test".into(),
            tipo: "Perno".into(),
            descripcion: None,
            marca: None,
            categoria: Id::generate(),
            codigos: vec![],
            unidad_medida: UnidadMedida::Unidad,
            precio: "1".parse().unwrap(),
            ubicacion: None,
            stock: "0".parse().unwrap(),
            activo: true,
            version: 1,
            id: Id::generate(),
            creado_en: ts(),
            actualizado_en: ts(),
        };
        let err = Producto::try_new(draft, &defs).expect_err("cuarenta");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        let fields = invalid_fields(&err);
        assert_eq!(fields[0].field, "attributes[0].value");
    }

    #[test]
    fn producto_rejects_duplicate_atributo_def() {
        let defs = vec![def_medida_texto()];
        let def_id = defs[0].id.clone();
        let draft = ProductoDraft {
            atributos: vec![
                AtributoValor {
                    def: def_id.clone(),
                    valor: ValorAtributo::Texto("M8".into()),
                },
                AtributoValor {
                    def: def_id,
                    valor: ValorAtributo::Texto("M10".into()),
                },
            ],
            sku: "SKU-1".into(),
            nombre: "Test".into(),
            tipo: "Perno".into(),
            descripcion: None,
            marca: None,
            categoria: Id::generate(),
            codigos: vec![],
            unidad_medida: UnidadMedida::Unidad,
            precio: "1".parse().unwrap(),
            ubicacion: None,
            stock: "0".parse().unwrap(),
            activo: true,
            version: 1,
            id: Id::generate(),
            creado_en: ts(),
            actualizado_en: ts(),
        };
        let err = Producto::try_new(draft, &defs).expect_err("duplicate def");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        let fields = invalid_fields(&err);
        assert_eq!(fields[0].field, "attributes[1].definitionId");
    }

    #[test]
    fn producto_rejects_enum_value_not_in_opciones() {
        let defs = vec![def_grado_enum()];
        let draft = ProductoDraft {
            atributos: vec![AtributoValor {
                def: defs[0].id.clone(),
                valor: ValorAtributo::Texto("9".into()),
            }],
            sku: "SKU-1".into(),
            nombre: "Test".into(),
            tipo: "Perno".into(),
            descripcion: None,
            marca: None,
            categoria: Id::generate(),
            codigos: vec![],
            unidad_medida: UnidadMedida::Unidad,
            precio: "1".parse().unwrap(),
            ubicacion: None,
            stock: "0".parse().unwrap(),
            activo: true,
            version: 1,
            id: Id::generate(),
            creado_en: ts(),
            actualizado_en: ts(),
        };
        let err = Producto::try_new(draft, &defs).expect_err("enum");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        let fields = invalid_fields(&err);
        assert_eq!(fields[0].field, "attributes[0].value");
    }

    #[test]
    fn ficha_producto_exposes_marca_ruta_and_resolved_attributes() {
        use crate::categoria::{Categoria, CategoriaDraft, Slug};
        use crate::marca::{Marca, MarcaDraft};

        let defs = vec![def_medida_texto()];
        let medida = &defs[0];
        let draft = ProductoDraft {
            id: Id::generate(),
            sku: "PER-M8X40-ZN".into(),
            nombre: "Perno hexagonal M8 x 40 zincado".into(),
            tipo: "Perno".into(),
            descripcion: None,
            marca: Some(Id::generate()),
            categoria: Id::generate(),
            atributos: vec![AtributoValor {
                def: medida.id.clone(),
                valor: ValorAtributo::Texto("M8".into()),
            }],
            codigos: vec![],
            unidad_medida: UnidadMedida::Unidad,
            precio: "120.50".parse().expect("precio"),
            ubicacion: None,
            stock: "0".parse().expect("stock"),
            activo: true,
            version: 1,
            creado_en: ts(),
            actualizado_en: ts(),
        };
        let producto = Producto::try_new(draft, &defs).expect("producto");
        let parent_id = Id::generate();
        let child_slug = Slug::from_nombre("Pernos").expect("slug");
        let categoria = Categoria::try_new(CategoriaDraft {
            id: Id::generate(),
            nombre: "Pernos".into(),
            slug: child_slug,
            padre: Some(parent_id.clone()),
            ancestros: vec![parent_id],
            ruta: vec!["Fijaciones".into(), "Pernos".into()],
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("categoria");
        let marca = Marca::try_new(MarcaDraft {
            id: Id::generate(),
            nombre: "Fixser".into(),
            slug: Slug::from_nombre("Fixser").expect("marca slug"),
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("marca");
        let ficha = FichaProducto {
            producto,
            marca: Some(marca.clone()),
            categoria,
            atributos: vec![AtributoFicha {
                def: medida.id.clone(),
                clave: "medida".into(),
                etiqueta: "Medida".into(),
                unidad: Some("mm".into()),
                valor: ValorAtributo::Texto("M8".into()),
            }],
        };
        assert_eq!(
            ficha.marca.as_ref().map(|m| m.nombre.as_str()),
            Some("Fixser")
        );
        assert_eq!(ficha.categoria.ruta, ["Fijaciones", "Pernos"]);
        assert_eq!(ficha.atributos[0].clave, "medida");
        assert_eq!(ficha.atributos[0].etiqueta, "Medida");
        assert_eq!(ficha.atributos[0].unidad.as_deref(), Some("mm"));
    }

    #[test]
    fn update_producto_has_no_stock_or_activo_fields() {
        let update = UpdateProducto {
            id: Id::generate(),
            version: 1,
            sku: "SKU".into(),
            nombre: "Nombre".into(),
            tipo: "Perno".into(),
            descripcion: None,
            marca: None,
            categoria: Id::generate(),
            atributos: vec![],
            codigos: vec![],
            unidad_medida: UnidadMedida::Unidad,
            precio: "1".parse().expect("precio"),
            ubicacion: None,
        };
        assert_eq!(update.version, 1);
        let _ = (
            update.id,
            update.sku,
            update.nombre,
            update.tipo,
            update.descripcion,
            update.marca,
            update.categoria,
            update.atributos,
            update.codigos,
            update.unidad_medida,
            update.precio,
            update.ubicacion,
        );
    }

    struct DummyProductRepo;

    impl ProductRepo for DummyProductRepo {
        fn create(
            &self,
            _input: NewProducto,
        ) -> impl Future<Output = Result<Producto, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn get(
            &self,
            _id: Id<Producto>,
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
            async { Err(DomainError::not_found()) }
        }

        fn update(
            &self,
            _input: UpdateProducto,
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
            async { Err(DomainError::not_found()) }
        }

        fn set_activo(
            &self,
            _id: Id<Producto>,
            _activo: bool,
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
            async { Err(DomainError::not_found()) }
        }

        fn list(
            &self,
            _filtro: FiltroProducto,
            _after: Option<Id<Producto>>,
            _take: u32,
        ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
            async { Ok(vec![]) }
        }

        fn find_by_code(
            &self,
            _code: String,
        ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
            async { Ok(vec![]) }
        }
    }

    #[test]
    fn domain_filtro_producto_has_categoria_marca_activo() {
        let cat = Id::<Categoria>::generate();
        let filtro = FiltroProducto {
            categoria: Some(cat.clone()),
            marca: None,
            activo: true,
        };
        assert_eq!(filtro.categoria, Some(cat));
        assert!(filtro.marca.is_none());
        assert!(filtro.activo);
    }

    #[test]
    fn domain_resumen_producto_exposes_marca_nombre_and_ruta() {
        let resumen = ResumenProducto {
            id: Id::generate(),
            sku: "SKU-1".into(),
            nombre: "Perno".into(),
            tipo: "Perno".into(),
            precio: "10".parse().expect("precio"),
            stock: "0".parse().expect("stock"),
            activo: true,
            marca_nombre: Some("Fixser".into()),
            categoria_ruta: vec!["Fijaciones".into(), "Pernos".into()],
        };
        assert_eq!(resumen.marca_nombre.as_deref(), Some("Fixser"));
        assert_eq!(resumen.categoria_ruta, ["Fijaciones", "Pernos"]);
    }

    #[test]
    fn domain_product_repo_declares_list() {
        fn assert_list<R: ProductRepo>(r: &R) {
            let _f = r.list(
                FiltroProducto {
                    categoria: None,
                    marca: None,
                    activo: true,
                },
                None,
                50,
            );
        }
        assert_list(&DummyProductRepo);
    }

    #[test]
    fn product_repo_trait_exposes_set_activo() {
        fn assert_set_activo<R: ProductRepo>(r: &R) {
            let _f = r.set_activo(Id::generate(), false);
        }
        assert_set_activo(&DummyProductRepo);
    }

    #[test]
    fn product_repo_trait_exposes_find_by_code() {
        fn assert_find_by_code<R: ProductRepo>(r: &R) {
            let _f = r.find_by_code("HX-0840Z".into());
        }
        assert_find_by_code(&DummyProductRepo);
    }

    #[test]
    fn product_repo_trait_exposes_create_get_update() {
        fn assert_repo<R: ProductRepo>(r: &R) {
            let _f1 = r.create(NewProducto {
                sku: "x".into(),
                nombre: "n".into(),
                tipo: "t".into(),
                descripcion: None,
                marca: None,
                categoria: Id::generate(),
                atributos: vec![],
                codigos: vec![],
                unidad_medida: UnidadMedida::Unidad,
                precio: "0".parse().unwrap(),
                ubicacion: None,
                stock_inicial: "0".parse().unwrap(),
                terminal: None,
            });
            let _f2 = r.get(Id::generate());
            let _f3 = r.update(UpdateProducto {
                id: Id::generate(),
                version: 1,
                sku: "x".into(),
                nombre: "n".into(),
                tipo: "t".into(),
                descripcion: None,
                marca: None,
                categoria: Id::generate(),
                atributos: vec![],
                codigos: vec![],
                unidad_medida: UnidadMedida::Unidad,
                precio: "0".parse().unwrap(),
                ubicacion: None,
            });
            let _f4 = r.set_activo(Id::generate(), true);
            let _f5 = r.list(
                FiltroProducto {
                    categoria: None,
                    marca: None,
                    activo: true,
                },
                None,
                10,
            );
        }
        assert_repo(&DummyProductRepo);
    }

    #[test]
    fn product_repo_trait_get_returns_ficha() {
        fn assert_get_future<R: ProductRepo>(
            r: &R,
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
            r.get(Id::generate())
        }
        let _ = assert_get_future(&DummyProductRepo {});
    }
}
