//! `/api/v1/relations` and product relation lists (T-04-05).

#![allow(clippy::result_large_err)]

use std::str::FromStr as _;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use domain::{
    Id, NewRelacion, Producto, Relacion, RelacionConDestino, TipoRelacion, UpdateRelacion,
};
use recommend::{ListRelationsParams, RelationGateway};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::products::ProductSummaryResponse;
use crate::{AppJson, AppState};

fn default_priority() -> i32 {
    1
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RelationCreateBody {
    #[serde(rename = "type")]
    type_: String,
    source_product_id: Id<Producto>,
    target_product_id: Id<Producto>,
    #[serde(default = "default_priority")]
    priority: i32,
    note: Option<String>,
    bidirectional: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RelationUpdateBody {
    priority: Option<i32>,
    note: Option<Option<String>>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ListRelationsQuery {
    #[serde(rename = "type")]
    type_: Option<String>,
    direction: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelationResponse {
    id: Id<Relacion>,
    #[serde(rename = "type")]
    type_: String,
    source_product_id: Id<Producto>,
    target_product_id: Id<Producto>,
    target: ProductSummaryResponse,
    priority: i32,
    note: Option<String>,
    bidirectional_group: Option<Id<domain::GrupoBidireccional>>,
    created_at: domain::Timestamp,
    updated_at: domain::Timestamp,
}

#[derive(Debug, Serialize)]
struct RelationsItemsResponse {
    items: Vec<RelationResponse>,
}

fn relations_unavailable(instance: &str) -> ApiError {
    ApiError::catalog_unavailable(instance.to_owned())
}

fn require_gateway(state: &AppState, instance: &str) -> Result<Arc<dyn RelationGateway>, ApiError> {
    state
        .relations()
        .ok_or_else(|| relations_unavailable(instance))
}

fn parse_product_id(raw: &str, instance: &str) -> Result<Id<Producto>, ApiError> {
    Id::from_str(raw).map_err(|_| ApiError::invalid_product_id(instance.to_owned()))
}

fn parse_relation_id(raw: &str, instance: &str) -> Result<Id<Relacion>, ApiError> {
    Id::from_str(raw).map_err(|_| ApiError::invalid_relation_id(instance.to_owned()))
}

fn parse_api_relation_type(raw: &str, instance: &str) -> Result<TipoRelacion, ApiError> {
    match raw {
        "similar" => Ok(TipoRelacion::SimilarA),
        "substitute" => Ok(TipoRelacion::SustituiblePor),
        "complement" => Ok(TipoRelacion::Complementa),
        _ => Err(ApiError::invalid_relation_type(instance.to_owned())),
    }
}

fn domain_to_api_type(tipo: TipoRelacion) -> &'static str {
    match tipo {
        TipoRelacion::SimilarA => "similar",
        TipoRelacion::SustituiblePor => "substitute",
        TipoRelacion::Complementa => "complement",
    }
}

fn relation_response(item: RelacionConDestino) -> RelationResponse {
    let rel = item.relacion;
    RelationResponse {
        id: rel.id,
        type_: domain_to_api_type(rel.tipo).into(),
        source_product_id: rel.origen,
        target_product_id: rel.destino,
        target: crate::products::product_summary_from_destino(&item.destino),
        priority: rel.prioridad,
        note: rel.nota,
        bidirectional_group: rel.grupo_bidireccional,
        created_at: rel.creado_en,
        updated_at: rel.actualizado_en,
    }
}

pub async fn post_relation(
    State(state): State<AppState>,
    AppJson(body): AppJson<RelationCreateBody>,
) -> Result<impl IntoResponse, ApiError> {
    const INSTANCE: &str = "/api/v1/relations";
    let gateway = require_gateway(&state, INSTANCE)?;
    let tipo = parse_api_relation_type(&body.type_, INSTANCE)?;
    let input = NewRelacion {
        origen: body.source_product_id,
        destino: body.target_product_id,
        tipo,
        prioridad: body.priority,
        nota: body.note,
        bidireccional: body.bidirectional,
    };
    let items = gateway
        .create(input)
        .await
        .map_err(|e| ApiError::from_domain_relation(e, INSTANCE))?;
    let mapped: Vec<RelationResponse> = items.into_iter().map(relation_response).collect();
    Ok((
        StatusCode::CREATED,
        Json(RelationsItemsResponse { items: mapped }),
    ))
}

pub async fn patch_relation(
    State(state): State<AppState>,
    Path((relation_type, relation_id)): Path<(String, String)>,
    AppJson(body): AppJson<RelationUpdateBody>,
) -> Result<impl IntoResponse, ApiError> {
    const INSTANCE: &str = "/api/v1/relations/{relationType}/{relationId}";
    let gateway = require_gateway(&state, INSTANCE)?;
    let tipo = parse_api_relation_type(&relation_type, INSTANCE)?;
    let id = parse_relation_id(&relation_id, INSTANCE)?;
    if body.priority.is_none() && body.note.is_none() {
        return Err(ApiError::patch_empty(INSTANCE.to_owned()));
    }
    let input = UpdateRelacion {
        id,
        tipo,
        prioridad: body.priority,
        nota: body.note,
    };
    let items = gateway
        .update(input)
        .await
        .map_err(|e| ApiError::from_domain_relation(e, INSTANCE))?;
    let mapped: Vec<RelationResponse> = items.into_iter().map(relation_response).collect();
    Ok((
        StatusCode::OK,
        Json(RelationsItemsResponse { items: mapped }),
    ))
}

pub async fn delete_relation(
    State(state): State<AppState>,
    Path((relation_type, relation_id)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    const INSTANCE: &str = "/api/v1/relations/{relationType}/{relationId}";
    let gateway = require_gateway(&state, INSTANCE)?;
    let tipo = parse_api_relation_type(&relation_type, INSTANCE)?;
    let id = parse_relation_id(&relation_id, INSTANCE)?;
    gateway
        .delete(tipo, id)
        .await
        .map_err(|e| ApiError::from_domain_relation(e, INSTANCE))?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub async fn list_product_relations(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
    Query(query): Query<ListRelationsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    const INSTANCE: &str = "/api/v1/products/{productId}/relations";
    let gateway = require_gateway(&state, INSTANCE)?;
    let pid = parse_product_id(&product_id, INSTANCE)?;
    let tipo = query
        .type_
        .as_deref()
        .map(|t| parse_api_relation_type(t, INSTANCE))
        .transpose()?;
    let incoming = matches!(query.direction.as_deref(), Some("incoming"));
    let params = ListRelationsParams { tipo, incoming };
    let items = gateway
        .list(pid, params)
        .await
        .map_err(|e| ApiError::from_domain_relation(e, INSTANCE))?;
    let mapped: Vec<RelationResponse> = items.into_iter().map(relation_response).collect();
    Ok((
        StatusCode::OK,
        Json(RelationsItemsResponse { items: mapped }),
    ))
}
