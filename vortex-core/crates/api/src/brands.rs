//! `/api/v1/brands` — HTTP adapter over `catalog::BrandGateway` (T-02-09).

#![allow(clippy::result_large_err)]

use std::str::FromStr as _;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use catalog::BrandGateway;
use domain::{Id, Marca, NewMarca, Slug, UpdateMarca};
use serde::{Deserialize, Serialize};

use crate::AppJson;
use crate::AppState;
use crate::error::ApiError;

const Q_MAX_LEN: usize = 100;

#[derive(Debug, Deserialize)]
pub(crate) struct ListBrandsQuery {
    #[serde(default)]
    q: Option<String>,
    #[serde(rename = "includeInactive", default)]
    include_inactive: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BrandResponse {
    id: Id<Marca>,
    name: String,
    slug: String,
    active: bool,
}

impl From<Marca> for BrandResponse {
    fn from(m: Marca) -> Self {
        Self {
            id: m.id,
            name: m.nombre,
            slug: m.slug.to_string(),
            active: m.activo,
        }
    }
}

#[derive(Debug, Serialize)]
struct ListBrandsResponse {
    items: Vec<BrandResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BrandCreateBody {
    name: String,
    slug: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BrandUpdateBody {
    name: Option<String>,
    slug: Option<String>,
    active: Option<bool>,
}

fn parse_brand_id(raw: &str, instance: &str) -> Result<Id<Marca>, ApiError> {
    Id::from_str(raw).map_err(|_| ApiError::invalid_brand_id(instance.to_owned()))
}

fn require_gateway(state: &AppState, instance: &str) -> Result<Arc<dyn BrandGateway>, ApiError> {
    state
        .brands()
        .ok_or_else(|| ApiError::catalog_unavailable(instance.to_owned()))
}

fn validate_q(query: &ListBrandsQuery, instance: &str) -> Result<(), ApiError> {
    if let Some(q) = &query.q
        && q.len() > Q_MAX_LEN
    {
        return Err(ApiError::invalid_query_q(instance.to_owned()));
    }
    Ok(())
}

fn matches_q(nombre: &str, q: &str) -> bool {
    let lower_n = nombre.to_ascii_lowercase();
    let lower_q = q.to_ascii_lowercase();
    lower_n.starts_with(&lower_q)
}

pub async fn list_brands(
    State(state): State<AppState>,
    Query(query): Query<ListBrandsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    const INSTANCE: &str = "/api/v1/brands";
    validate_q(&query, INSTANCE)?;
    let gateway = require_gateway(&state, INSTANCE)?;
    let mut items = gateway
        .list_all()
        .await
        .map_err(|e| ApiError::from_domain(e, INSTANCE))?;
    if !query.include_inactive {
        items.retain(|m| m.activo);
    }
    if let Some(ref q) = query.q
        && !q.is_empty()
    {
        items.retain(|m| matches_q(&m.nombre, q));
    }
    items.sort_by(|a, b| a.nombre.cmp(&b.nombre));
    let items = items.into_iter().map(BrandResponse::from).collect();
    Ok((StatusCode::OK, Json(ListBrandsResponse { items })))
}

pub async fn post_brand(
    State(state): State<AppState>,
    AppJson(body): AppJson<BrandCreateBody>,
) -> Result<impl IntoResponse, ApiError> {
    const INSTANCE: &str = "/api/v1/brands";
    let gateway = require_gateway(&state, INSTANCE)?;
    let slug = body
        .slug
        .as_deref()
        .map(Slug::parse)
        .transpose()
        .map_err(|e| ApiError::from_domain(e, INSTANCE))?;
    let input = NewMarca {
        nombre: body.name,
        slug,
    };
    let created = gateway
        .create(input)
        .await
        .map_err(|e| ApiError::from_domain(e, INSTANCE))?;
    Ok((StatusCode::CREATED, Json(BrandResponse::from(created))))
}

pub async fn patch_brand(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
    AppJson(body): AppJson<BrandUpdateBody>,
) -> Result<impl IntoResponse, ApiError> {
    let instance = format!("/api/v1/brands/{brand_id}");
    validate_patch_body(&body, &instance)?;
    let id = parse_brand_id(&brand_id, &instance)?;
    let gateway = require_gateway(&state, &instance)?;

    let mut updated = if body.name.is_some() || body.slug.is_some() {
        let current = gateway
            .get(id.clone())
            .await
            .map_err(|e| ApiError::from_domain(e, instance.clone()))?;
        let nombre = body.name.clone().unwrap_or(current.nombre);
        let slug = body
            .slug
            .as_deref()
            .map(Slug::parse)
            .transpose()
            .map_err(|e| ApiError::from_domain(e, instance.clone()))?;
        gateway
            .update(UpdateMarca {
                id: id.clone(),
                nombre,
                slug,
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

    Ok((StatusCode::OK, Json(BrandResponse::from(updated))))
}

pub async fn delete_brand(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let instance = format!("/api/v1/brands/{brand_id}");
    let id = parse_brand_id(&brand_id, &instance)?;
    let gateway = require_gateway(&state, &instance)?;
    gateway
        .deactivate(id)
        .await
        .map_err(|e| ApiError::from_domain(e, instance))?;
    Ok(StatusCode::NO_CONTENT)
}

fn validate_patch_body(body: &BrandUpdateBody, instance: &str) -> Result<(), ApiError> {
    if body.name.is_none() && body.slug.is_none() && body.active.is_none() {
        return Err(ApiError::patch_empty(instance.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod patch_validation_tests {
    use super::*;

    #[test]
    fn slug_only_is_allowed() {
        let body = BrandUpdateBody {
            slug: Some("nuevo-slug".into()),
            ..BrandUpdateBody::default()
        };
        assert!(validate_patch_body(&body, "/api/v1/brands/x").is_ok());
    }
}
