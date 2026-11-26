//! `GET /api/v1/products/{productId}/recommendations` (T-04-07).

#![allow(clippy::result_large_err)]

use std::str::FromStr as _;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ::recommend::RecommendationGateway;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use domain::{Id, Producto, ProductoRecomendado, Recomendaciones};
use metrics::histogram;
use serde::Serialize;

use crate::AppState;
use crate::error::ApiError;
use crate::metrics::operation_duration_seconds;

const INSTANCE: &str = "/api/v1/products/{productId}/recommendations";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RecommendedProductResponse {
    relation_id: Id<domain::Relacion>,
    id: Id<Producto>,
    sku: String,
    name: String,
    stock: domain::Decimal,
    price: domain::Decimal,
    location: Option<String>,
    image_url: Option<String>,
    priority: i32,
    note: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RecommendationsResponse {
    product_id: Id<Producto>,
    highlight_substitutes: bool,
    similar: Vec<RecommendedProductResponse>,
    substitutes: Vec<RecommendedProductResponse>,
    complements: Vec<RecommendedProductResponse>,
}

fn recommendations_unavailable(instance: &str) -> ApiError {
    ApiError::catalog_unavailable(instance.to_owned())
}

fn require_gateway(
    state: &AppState,
    instance: &str,
) -> Result<Arc<dyn RecommendationGateway>, ApiError> {
    state
        .recommendations()
        .ok_or_else(|| recommendations_unavailable(instance))
}

fn parse_product_id(raw: &str, instance: &str) -> Result<Id<Producto>, ApiError> {
    Id::from_str(raw).map_err(|_| ApiError::invalid_product_id(instance.to_owned()))
}

fn map_item(item: ProductoRecomendado) -> RecommendedProductResponse {
    RecommendedProductResponse {
        relation_id: item.relacion_id,
        id: item.id,
        sku: item.sku,
        name: item.nombre,
        stock: item.stock,
        price: item.precio,
        location: item.ubicacion,
        image_url: None,
        priority: item.prioridad,
        note: item.nota,
    }
}

fn response_body(panel: Recomendaciones) -> RecommendationsResponse {
    RecommendationsResponse {
        product_id: panel.producto_id,
        highlight_substitutes: panel.destacar_sustitutos,
        similar: panel.similares.into_iter().map(map_item).collect(),
        substitutes: panel.sustitutos.into_iter().map(map_item).collect(),
        complements: panel.complementarios.into_iter().map(map_item).collect(),
    }
}

fn record_recommendations_duration(elapsed: Duration) {
    histogram!(operation_duration_seconds("recommendations")).record(elapsed.as_secs_f64());
}

pub async fn get_product_recommendations(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let gateway = require_gateway(&state, INSTANCE)?;
    let pid = parse_product_id(&product_id, INSTANCE)?;
    let start = Instant::now();
    let panel = match gateway.get(pid).await {
        Ok(panel) => panel,
        Err(err) => {
            record_recommendations_duration(start.elapsed());
            return Err(ApiError::from_domain(err, INSTANCE));
        }
    };
    record_recommendations_duration(start.elapsed());
    let body = response_body(panel);
    Ok((StatusCode::OK, Json(body)))
}
