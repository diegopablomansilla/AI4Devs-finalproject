//! `/api/v1/attribute-definitions` — HTTP adapter over `catalog::AttributeDefGateway` (T-02-12).

#![allow(clippy::result_large_err)]

use std::str::FromStr as _;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use catalog::AttributeDefGateway;
use domain::{AtributoDef, Id, NewAtributoDef, TipoAtributo, UpdateAtributoDef};
use serde::{Deserialize, Serialize};

use crate::AppJson;
use crate::AppState;
use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub(crate) struct ListAttributeDefsQuery {
    #[serde(rename = "includeInactive", default)]
    include_inactive: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AttributeDefinitionResponse {
    id: Id<AtributoDef>,
    key: String,
    label: String,
    #[serde(rename = "type")]
    type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Vec<String>>,
    facetable: bool,
    searchable: bool,
    order: i32,
    active: bool,
}

impl From<AtributoDef> for AttributeDefinitionResponse {
    fn from(d: AtributoDef) -> Self {
        Self {
            id: d.id,
            key: d.clave,
            label: d.etiqueta,
            type_: tipo_to_api(d.tipo).into(),
            unit: d.unidad,
            options: d.opciones,
            facetable: d.facetable,
            searchable: d.buscable,
            order: d.orden,
            active: d.activo,
        }
    }
}

#[derive(Debug, Serialize)]
struct ListAttributeDefsResponse {
    items: Vec<AttributeDefinitionResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AttributeDefCreateBody {
    key: String,
    label: String,
    #[serde(rename = "type")]
    type_: String,
    unit: Option<String>,
    options: Option<Vec<String>>,
    #[serde(default)]
    facetable: bool,
    #[serde(default = "default_searchable")]
    searchable: bool,
    #[serde(default)]
    order: i32,
}

fn default_searchable() -> bool {
    true
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AttributeDefUpdateBody {
    key: Option<String>,
    label: Option<String>,
    #[serde(rename = "type")]
    type_: Option<String>,
    unit: Option<String>,
    options: Option<Vec<String>>,
    facetable: Option<bool>,
    searchable: Option<bool>,
    order: Option<i32>,
    active: Option<bool>,
}

fn parse_attribute_id(raw: &str, instance: &str) -> Result<Id<AtributoDef>, ApiError> {
    Id::from_str(raw).map_err(|_| ApiError::invalid_attribute_id(instance.to_owned()))
}

fn require_gateway(
    state: &AppState,
    instance: &str,
) -> Result<Arc<dyn AttributeDefGateway>, ApiError> {
    state
        .attribute_defs()
        .ok_or_else(|| ApiError::catalog_unavailable(instance.to_owned()))
}

fn tipo_to_api(tipo: TipoAtributo) -> &'static str {
    match tipo {
        TipoAtributo::Texto => "text",
        TipoAtributo::Numero => "number",
        TipoAtributo::Booleano => "boolean",
        TipoAtributo::Enum => "enum",
    }
}

fn parse_api_type(raw: &str) -> Result<TipoAtributo, domain::DomainError> {
    match raw {
        "text" => Ok(TipoAtributo::Texto),
        "number" => Ok(TipoAtributo::Numero),
        "boolean" => Ok(TipoAtributo::Booleano),
        "enum" => Ok(TipoAtributo::Enum),
        _ => domain::TipoAtributo::parse_wire("__invalid__"),
    }
}

pub async fn list_attribute_definitions(
    State(state): State<AppState>,
    Query(query): Query<ListAttributeDefsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    const INSTANCE: &str = "/api/v1/attribute-definitions";
    let gateway = require_gateway(&state, INSTANCE)?;
    let mut items = gateway
        .list_all()
        .await
        .map_err(|e| ApiError::from_domain(e, INSTANCE))?;
    if !query.include_inactive {
        items.retain(|d| d.activo);
    }
    items.sort_by(|a, b| a.orden.cmp(&b.orden).then_with(|| a.clave.cmp(&b.clave)));
    let items = items
        .into_iter()
        .map(AttributeDefinitionResponse::from)
        .collect();
    Ok((StatusCode::OK, Json(ListAttributeDefsResponse { items })))
}

pub async fn post_attribute_definition(
    State(state): State<AppState>,
    AppJson(body): AppJson<AttributeDefCreateBody>,
) -> Result<impl IntoResponse, ApiError> {
    const INSTANCE: &str = "/api/v1/attribute-definitions";
    let gateway = require_gateway(&state, INSTANCE)?;
    let tipo = parse_api_type(&body.type_).map_err(|e| ApiError::from_domain(e, INSTANCE))?;
    let input = NewAtributoDef {
        clave: body.key,
        etiqueta: body.label,
        tipo,
        unidad: body.unit,
        opciones: body.options,
        facetable: body.facetable,
        buscable: body.searchable,
        orden: body.order,
    };
    let created = gateway
        .create(input)
        .await
        .map_err(|e| ApiError::from_domain(e, INSTANCE))?;
    Ok((
        StatusCode::CREATED,
        Json(AttributeDefinitionResponse::from(created)),
    ))
}

pub async fn patch_attribute_definition(
    State(state): State<AppState>,
    Path(attribute_id): Path<String>,
    AppJson(body): AppJson<AttributeDefUpdateBody>,
) -> Result<impl IntoResponse, ApiError> {
    let instance = format!("/api/v1/attribute-definitions/{attribute_id}");
    validate_patch_body(&body, &instance)?;
    let id = parse_attribute_id(&attribute_id, &instance)?;
    let gateway = require_gateway(&state, &instance)?;

    let mut updated = if body_has_editable_fields(&body) {
        let current = gateway
            .get(id.clone())
            .await
            .map_err(|e| ApiError::from_domain(e, instance.clone()))?;
        let tipo = if let Some(ref t) = body.type_ {
            parse_api_type(t).map_err(|e| ApiError::from_domain(e, instance.clone()))?
        } else {
            current.tipo
        };
        let opciones = if body.options.is_some() {
            body.options.clone()
        } else {
            current.opciones.clone()
        };
        gateway
            .update(UpdateAtributoDef {
                id: id.clone(),
                clave: body.key.clone().unwrap_or(current.clave),
                etiqueta: body.label.clone().unwrap_or(current.etiqueta),
                tipo,
                unidad: body.unit.clone().or(current.unidad),
                opciones,
                facetable: body.facetable.unwrap_or(current.facetable),
                buscable: body.searchable.unwrap_or(current.buscable),
                orden: body.order.unwrap_or(current.orden),
            })
            .await
            .map_err(|e| ApiError::from_domain(e, instance.clone()))?
    } else {
        gateway
            .get(id.clone())
            .await
            .map_err(|e| ApiError::from_domain(e, instance.clone()))?
    };

    if let Some(active) = body.active {
        updated = if active {
            gateway
                .reactivate(id.clone())
                .await
                .map_err(|e| ApiError::from_domain(e, instance.clone()))?
        } else {
            gateway
                .deactivate(id.clone())
                .await
                .map_err(|e| ApiError::from_domain(e, instance.clone()))?
        };
    }

    Ok((
        StatusCode::OK,
        Json(AttributeDefinitionResponse::from(updated)),
    ))
}

pub async fn delete_attribute_definition(
    State(state): State<AppState>,
    Path(attribute_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let instance = format!("/api/v1/attribute-definitions/{attribute_id}");
    let id = parse_attribute_id(&attribute_id, &instance)?;
    let gateway = require_gateway(&state, &instance)?;
    gateway
        .deactivate(id)
        .await
        .map_err(|e| ApiError::from_domain(e, instance))?;
    Ok(StatusCode::NO_CONTENT)
}

fn body_has_editable_fields(body: &AttributeDefUpdateBody) -> bool {
    body.key.is_some()
        || body.label.is_some()
        || body.type_.is_some()
        || body.unit.is_some()
        || body.options.is_some()
        || body.facetable.is_some()
        || body.searchable.is_some()
        || body.order.is_some()
}

fn validate_patch_body(body: &AttributeDefUpdateBody, instance: &str) -> Result<(), ApiError> {
    if !body_has_editable_fields(body) && body.active.is_none() {
        return Err(ApiError::patch_empty(instance.to_owned()));
    }
    Ok(())
}
