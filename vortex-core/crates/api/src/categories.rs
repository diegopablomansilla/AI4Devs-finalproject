//! `/api/v1/categories` — HTTP adapter over `catalog::CategoryGateway` (T-02-06).

#![allow(clippy::result_large_err)]

use std::str::FromStr as _;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use catalog::CategoryGateway;
use domain::{Categoria, Id, NewCategoria, Slug};
use serde::{Deserialize, Serialize};

use crate::AppJson;
use crate::AppState;
use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub(crate) struct ListCategoriesQuery {
    #[serde(rename = "includeInactive", default)]
    include_inactive: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CategoryResponse {
    id: Id<Categoria>,
    name: String,
    slug: String,
    parent_id: Option<Id<Categoria>>,
    ancestor_ids: Vec<Id<Categoria>>,
    path: Vec<String>,
    active: bool,
}

impl From<Categoria> for CategoryResponse {
    fn from(c: Categoria) -> Self {
        Self {
            id: c.id,
            name: c.nombre,
            slug: c.slug.to_string(),
            parent_id: c.padre,
            ancestor_ids: c.ancestros,
            path: c.ruta,
            active: c.activo,
        }
    }
}

#[derive(Debug, Serialize)]
struct ListCategoriesResponse {
    items: Vec<CategoryResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CategoryCreateBody {
    name: String,
    slug: Option<String>,
    parent_id: Option<Id<Categoria>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CategoryUpdateBody {
    name: Option<String>,
    slug: Option<String>,
    #[serde(default)]
    parent_id: Option<Option<Id<Categoria>>>,
    active: Option<bool>,
}

fn parse_category_id(raw: &str, instance: &str) -> Result<Id<Categoria>, ApiError> {
    Id::from_str(raw).map_err(|_| ApiError::invalid_category_id(instance.to_owned()))
}

fn require_gateway(state: &AppState, instance: &str) -> Result<Arc<dyn CategoryGateway>, ApiError> {
    state
        .categories()
        .ok_or_else(|| ApiError::catalog_unavailable(instance.to_owned()))
}

pub async fn list_categories(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    const INSTANCE: &str = "/api/v1/categories";
    let gateway = require_gateway(&state, INSTANCE)?;
    let mut items = gateway
        .list_all()
        .await
        .map_err(|e| ApiError::from_domain(e, INSTANCE))?;
    if !query.include_inactive {
        items.retain(|c| c.activo);
    }
    let items = items.into_iter().map(CategoryResponse::from).collect();
    Ok((StatusCode::OK, Json(ListCategoriesResponse { items })))
}

pub async fn post_category(
    State(state): State<AppState>,
    AppJson(body): AppJson<CategoryCreateBody>,
) -> Result<impl IntoResponse, ApiError> {
    const INSTANCE: &str = "/api/v1/categories";
    let gateway = require_gateway(&state, INSTANCE)?;
    let slug = body
        .slug
        .as_deref()
        .map(Slug::parse)
        .transpose()
        .map_err(|e| ApiError::from_domain(e, INSTANCE))?;
    let input = NewCategoria {
        nombre: body.name,
        slug,
        padre: body.parent_id,
    };
    let created = gateway
        .create(input)
        .await
        .map_err(|e| ApiError::from_domain(e, INSTANCE))?;
    Ok((StatusCode::CREATED, Json(CategoryResponse::from(created))))
}

pub async fn patch_category(
    State(state): State<AppState>,
    Path(category_id): Path<String>,
    AppJson(body): AppJson<CategoryUpdateBody>,
) -> Result<impl IntoResponse, ApiError> {
    let instance = format!("/api/v1/categories/{category_id}");
    validate_patch_body(&body, &instance)?;
    let id = parse_category_id(&category_id, &instance)?;
    let gateway = require_gateway(&state, &instance)?;

    let mut updated = if let Some(name) = body.name {
        gateway
            .rename(id.clone(), name)
            .await
            .map_err(|e| ApiError::from_domain(e, instance.clone()))?
    } else {
        gateway
            .get(id.clone())
            .await
            .map_err(|e| ApiError::from_domain(e, instance.clone()))?
    };

    if let Some(parent) = body.parent_id {
        updated = gateway
            .move_category(id.clone(), parent)
            .await
            .map_err(|e| ApiError::from_domain(e, instance.clone()))?;
    }

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

    Ok((StatusCode::OK, Json(CategoryResponse::from(updated))))
}

pub async fn delete_category(
    State(state): State<AppState>,
    Path(category_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let instance = format!("/api/v1/categories/{category_id}");
    let id = parse_category_id(&category_id, &instance)?;
    let gateway = require_gateway(&state, &instance)?;
    gateway
        .deactivate(id)
        .await
        .map_err(|e| ApiError::from_domain(e, instance))?;
    Ok(StatusCode::NO_CONTENT)
}

fn validate_patch_body(body: &CategoryUpdateBody, instance: &str) -> Result<(), ApiError> {
    let has_name = body.name.is_some();
    let has_parent = body.parent_id.is_some();
    let has_active = body.active.is_some();
    let has_slug = body.slug.is_some();

    if !has_name && !has_parent && !has_active && !has_slug {
        return Err(ApiError::patch_empty(instance.to_owned()));
    }
    if has_slug && !has_name && !has_parent && !has_active {
        return Err(ApiError::patch_slug_only(instance.to_owned()));
    }
    if has_slug {
        return Err(ApiError::patch_slug_only(instance.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod patch_validation_tests {
    use super::*;

    #[test]
    fn slug_only_is_rejected() {
        let body = CategoryUpdateBody {
            slug: Some("nuevo-slug".into()),
            ..CategoryUpdateBody::default()
        };
        assert!(validate_patch_body(&body, "/api/v1/categories/x").is_err());
    }
}
