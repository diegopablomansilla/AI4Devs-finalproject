//! `/api/v1/products` — HTTP adapter over `catalog::ProductGateway` (T-02-21).

#![allow(clippy::result_large_err)]

use std::str::FromStr as _;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use catalog::{PatchProducto, ProductGateway};
use domain::{
    AtributoValor, Codigo, Decimal, FichaProducto, Id, Marca, NewProducto, TipoCodigo,
    UnidadMedida, ValorAtributo,
};
use serde::{Deserialize, Serialize};

use crate::AppJson;
use crate::AppState;
use crate::error::ApiError;

const TERMINAL_HEADER: &str = "x-vectra-terminal";
const TERMINAL_MAX_LEN: usize = 64;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttributeInput {
    definition_id: Id<domain::AtributoDef>,
    value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodeInput {
    #[serde(rename = "type")]
    type_: String,
    value: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LocationInput {
    aisle: Option<String>,
    shelf: Option<String>,
    position: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductCreateBody {
    sku: String,
    name: String,
    product_type: String,
    description: Option<String>,
    brand_id: Option<Id<Marca>>,
    category_id: Id<domain::Categoria>,
    #[serde(default)]
    attributes: Vec<AttributeInput>,
    #[serde(default)]
    codes: Vec<CodeInput>,
    #[serde(default = "default_unit_of_measure_str")]
    unit_of_measure: Option<String>,
    price: Decimal,
    location: Option<LocationInput>,
    #[serde(default = "default_initial_stock")]
    initial_stock: Decimal,
}

fn default_unit_of_measure_str() -> Option<String> {
    Some("unit".into())
}

fn default_initial_stock() -> Decimal {
    "0".parse().expect("zero stock")
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductUpdateBody {
    version: i32,
    sku: Option<String>,
    name: Option<String>,
    product_type: Option<String>,
    description: Option<Option<String>>,
    brand_id: Option<Option<Id<Marca>>>,
    category_id: Option<Id<domain::Categoria>>,
    attributes: Option<Vec<AttributeInput>>,
    codes: Option<Vec<CodeInput>>,
    unit_of_measure: Option<String>,
    price: Option<Decimal>,
    location: Option<Option<LocationInput>>,
    active: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BrandRefResponse {
    id: Id<Marca>,
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CategoryRefResponse {
    id: Id<domain::Categoria>,
    name: String,
    path: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AttributeValueResponse {
    definition_id: Id<domain::AtributoDef>,
    value: serde_json::Value,
    key: String,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CodeResponse {
    #[serde(rename = "type")]
    type_: String,
    value: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LocationResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    aisle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shelf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductResponse {
    id: Id<domain::Producto>,
    sku: String,
    name: String,
    product_type: String,
    description: Option<String>,
    brand: Option<BrandRefResponse>,
    category: CategoryRefResponse,
    attributes: Vec<AttributeValueResponse>,
    codes: Vec<CodeResponse>,
    unit_of_measure: String,
    price: Decimal,
    location: Option<LocationResponse>,
    stock: Decimal,
    active: bool,
    version: i32,
    images: Vec<serde_json::Value>,
    created_at: domain::Timestamp,
    updated_at: domain::Timestamp,
}

fn parse_product_id(raw: &str, instance: &str) -> Result<Id<domain::Producto>, ApiError> {
    Id::from_str(raw).map_err(|_| ApiError::invalid_product_id(instance.to_owned()))
}

fn require_gateway(state: &AppState, instance: &str) -> Result<Arc<dyn ProductGateway>, ApiError> {
    state
        .products()
        .ok_or_else(|| ApiError::catalog_unavailable(instance.to_owned()))
}

fn parse_terminal_header(headers: &HeaderMap, instance: &str) -> Result<Option<String>, ApiError> {
    let Some(value) = headers.get(TERMINAL_HEADER) else {
        return Ok(None);
    };
    let raw = value
        .to_str()
        .map_err(|_| ApiError::invalid_terminal_header(instance.to_owned()))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.len() > TERMINAL_MAX_LEN {
        return Err(ApiError::invalid_terminal_header(instance.to_owned()));
    }
    Ok(Some(trimmed.to_owned()))
}

fn unit_to_api(u: UnidadMedida) -> &'static str {
    match u {
        UnidadMedida::Unidad => "unit",
        UnidadMedida::Par => "pair",
        UnidadMedida::Caja => "box",
        UnidadMedida::Metro => "meter",
        UnidadMedida::Kilogramo => "kilogram",
        UnidadMedida::Litro => "liter",
    }
}

fn parse_unit_api(raw: &str) -> Result<UnidadMedida, domain::DomainError> {
    match raw {
        "unit" => Ok(UnidadMedida::Unidad),
        "pair" => Ok(UnidadMedida::Par),
        "box" => Ok(UnidadMedida::Caja),
        "meter" => Ok(UnidadMedida::Metro),
        "kilogram" => Ok(UnidadMedida::Kilogramo),
        "liter" => Ok(UnidadMedida::Litro),
        _ => domain::UnidadMedida::parse_wire("__invalid__"),
    }
}

fn code_type_to_api(t: TipoCodigo) -> &'static str {
    match t {
        TipoCodigo::Fabricante => "manufacturer",
        TipoCodigo::Oem => "oem",
        TipoCodigo::Barras => "barcode",
    }
}

fn parse_code_type(raw: &str) -> Result<TipoCodigo, domain::DomainError> {
    match raw {
        "manufacturer" => Ok(TipoCodigo::Fabricante),
        "oem" => Ok(TipoCodigo::Oem),
        "barcode" => Ok(TipoCodigo::Barras),
        _ => TipoCodigo::parse_wire("__invalid__"),
    }
}

fn json_to_valor(value: &serde_json::Value) -> Result<ValorAtributo, domain::DomainError> {
    match value {
        serde_json::Value::Bool(b) => Ok(ValorAtributo::Booleano(*b)),
        serde_json::Value::Number(n) => {
            let s = n.to_string();
            let d: Decimal = s.parse().map_err(|_| {
                domain::DomainError::invalid_input(domain::InvalidInputCode::InvalidInput, vec![])
            })?;
            Ok(ValorAtributo::Numero(d))
        }
        serde_json::Value::String(s) => Ok(ValorAtributo::Texto(s.clone())),
        _ => Err(domain::DomainError::invalid_input(
            domain::InvalidInputCode::InvalidInput,
            vec![],
        )),
    }
}

fn valor_to_json(v: &ValorAtributo) -> serde_json::Value {
    match v {
        ValorAtributo::Texto(s) => serde_json::Value::String(s.clone()),
        ValorAtributo::Numero(d) => serde_json::Value::String(d.to_string()),
        ValorAtributo::Booleano(b) => serde_json::Value::Bool(*b),
    }
}

fn map_attributes(inputs: &[AttributeInput]) -> Result<Vec<AtributoValor>, domain::DomainError> {
    let mut out = Vec::with_capacity(inputs.len());
    for input in inputs {
        out.push(AtributoValor {
            def: input.definition_id.clone(),
            valor: json_to_valor(&input.value)?,
        });
    }
    Ok(out)
}

fn map_codes(inputs: &[CodeInput]) -> Result<Vec<Codigo>, domain::DomainError> {
    let mut out = Vec::with_capacity(inputs.len());
    for (n, input) in inputs.iter().enumerate() {
        let tipo = parse_code_type(&input.type_).map_err(|mut e| {
            if let domain::DomainError::InvalidInput { ref mut errors, .. } = e {
                for fe in errors {
                    if fe.field == "codigos[].tipo" {
                        fe.field = format!("codes[{n}].type");
                    }
                }
            }
            e
        })?;
        out.push(Codigo::try_new(tipo, input.value.clone())?);
    }
    Ok(out)
}

fn map_location(input: &LocationInput) -> domain::Ubicacion {
    domain::Ubicacion {
        pasillo: input.aisle.clone(),
        estante: input.shelf.clone(),
        posicion: input.position.clone(),
    }
}

fn create_body_to_domain(
    body: ProductCreateBody,
    terminal: Option<String>,
) -> Result<NewProducto, domain::DomainError> {
    let unit_raw = body.unit_of_measure.as_deref().unwrap_or("unit");
    let unidad_medida = parse_unit_api(unit_raw)?;
    let ubicacion = body.location.as_ref().map(map_location);
    Ok(NewProducto {
        sku: body.sku,
        nombre: body.name,
        tipo: body.product_type,
        descripcion: body.description,
        marca: body.brand_id,
        categoria: body.category_id,
        atributos: map_attributes(&body.attributes)?,
        codigos: map_codes(&body.codes)?,
        unidad_medida,
        precio: body.price,
        ubicacion,
        stock_inicial: body.initial_stock,
        terminal,
    })
}

fn product_response(ficha: FichaProducto) -> ProductResponse {
    let p = ficha.producto;
    let brand = ficha.marca.map(|m| BrandRefResponse {
        id: m.id,
        name: m.nombre,
    });
    let category = CategoryRefResponse {
        id: ficha.categoria.id,
        name: ficha.categoria.nombre.clone(),
        path: ficha.categoria.ruta.clone(),
    };
    let attributes = ficha
        .atributos
        .iter()
        .map(|a| AttributeValueResponse {
            definition_id: a.def.clone(),
            value: valor_to_json(&a.valor),
            key: a.clave.clone(),
            label: a.etiqueta.clone(),
            unit: a.unidad.clone(),
        })
        .collect();
    let codes = p
        .codigos
        .iter()
        .map(|c| CodeResponse {
            type_: code_type_to_api(c.tipo).into(),
            value: c.valor.clone(),
        })
        .collect();
    let location = p.ubicacion.as_ref().map(|u| LocationResponse {
        aisle: u.pasillo.clone(),
        shelf: u.estante.clone(),
        position: u.posicion.clone(),
    });
    ProductResponse {
        id: p.id,
        sku: p.sku.as_str().to_owned(),
        name: p.nombre,
        product_type: p.tipo,
        description: p.descripcion,
        brand,
        category,
        attributes,
        codes,
        unit_of_measure: unit_to_api(p.unidad_medida).into(),
        price: p.precio,
        location,
        stock: p.stock,
        active: p.activo,
        version: p.version,
        images: vec![],
        created_at: p.creado_en,
        updated_at: p.actualizado_en,
    }
}

fn created_response(ficha: FichaProducto) -> Response {
    let id = ficha.producto.id.to_string();
    let location = format!("/api/v1/products/{id}");
    let mut headers = HeaderMap::new();
    if let Ok(value) = HeaderValue::from_str(&location) {
        headers.insert(header::LOCATION, value);
    }
    let body = Json(product_response(ficha));
    (StatusCode::CREATED, headers, body).into_response()
}

fn validate_patch_body(body: &ProductUpdateBody, instance: &str) -> Result<(), ApiError> {
    if body.active.is_some() {
        return Err(ApiError::patch_product_active(instance.to_owned()));
    }
    let has_field = body.sku.is_some()
        || body.name.is_some()
        || body.product_type.is_some()
        || body.description.is_some()
        || body.brand_id.is_some()
        || body.category_id.is_some()
        || body.attributes.is_some()
        || body.codes.is_some()
        || body.unit_of_measure.is_some()
        || body.price.is_some()
        || body.location.is_some();
    if !has_field {
        return Err(ApiError::patch_empty(instance.to_owned()));
    }
    Ok(())
}

fn patch_body_to_domain(
    id: Id<domain::Producto>,
    body: ProductUpdateBody,
) -> Result<PatchProducto, domain::DomainError> {
    let unidad_medida = match body.unit_of_measure.as_deref() {
        None => None,
        Some(raw) => Some(parse_unit_api(raw)?),
    };
    let ubicacion = match body.location {
        None => None,
        Some(None) => Some(None),
        Some(Some(loc)) => Some(Some(map_location(&loc))),
    };
    let atributos = match body.attributes {
        None => None,
        Some(attrs) => Some(map_attributes(&attrs)?),
    };
    let codigos = match body.codes {
        None => None,
        Some(codes) => Some(map_codes(&codes)?),
    };
    Ok(PatchProducto {
        id,
        version: body.version,
        sku: body.sku,
        nombre: body.name,
        tipo: body.product_type,
        descripcion: body.description,
        marca: body.brand_id,
        categoria: body.category_id,
        atributos,
        codigos,
        unidad_medida,
        precio: body.price,
        ubicacion,
    })
}

pub async fn post_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    AppJson(body): AppJson<ProductCreateBody>,
) -> Result<impl IntoResponse, ApiError> {
    const INSTANCE: &str = "/api/v1/products";
    let gateway = require_gateway(&state, INSTANCE)?;
    let terminal = parse_terminal_header(&headers, INSTANCE)?;
    let input = create_body_to_domain(body, terminal)
        .map_err(|e| ApiError::from_domain_product(e, INSTANCE))?;
    let created = gateway
        .create(input)
        .await
        .map_err(|e| ApiError::from_domain_product(e, INSTANCE))?;
    Ok(created_response(created))
}

pub async fn get_product(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let instance = format!("/api/v1/products/{product_id}");
    let id = parse_product_id(&product_id, &instance)?;
    let gateway = require_gateway(&state, &instance)?;
    let ficha = gateway
        .get(id)
        .await
        .map_err(|e| ApiError::from_domain_product(e, instance))?;
    Ok((StatusCode::OK, Json(product_response(ficha))))
}

pub async fn patch_product(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
    AppJson(body): AppJson<ProductUpdateBody>,
) -> Result<impl IntoResponse, ApiError> {
    let instance = format!("/api/v1/products/{product_id}");
    validate_patch_body(&body, &instance)?;
    let id = parse_product_id(&product_id, &instance)?;
    let gateway = require_gateway(&state, &instance)?;
    let patch =
        patch_body_to_domain(id, body).map_err(|e| ApiError::from_domain_product(e, &instance))?;
    let updated = gateway
        .update(patch)
        .await
        .map_err(|e| ApiError::from_domain_product(e, instance))?;
    Ok((StatusCode::OK, Json(product_response(updated))))
}
