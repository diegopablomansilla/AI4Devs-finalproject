//! `POST /api/v1/products/{productId}/stock-movements` (T-02-25).

#![allow(clippy::result_large_err)]

use std::str::FromStr as _;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use catalog::StockGateway;
use domain::{Id, MovimientoStock, NewMovimientoStock, Producto, TipoMovimiento, Venta};
use serde::Serialize;

use crate::AppJson;
use crate::AppState;
use crate::error::ApiError;

const TERMINAL_HEADER: &str = "x-vectra-terminal";
const TERMINAL_MAX_LEN: usize = 64;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StockMovementCreateBody {
    #[serde(rename = "type")]
    type_: StockMovementTypeCreate,
    quantity: domain::Decimal,
    reason: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum StockMovementTypeCreate {
    Inbound,
    Adjustment,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StockMovementResponse {
    id: Id<domain::MovimientoStock>,
    product_id: Id<Producto>,
    #[serde(rename = "type")]
    type_: &'static str,
    quantity: domain::Decimal,
    resulting_stock: domain::Decimal,
    reason: Option<String>,
    sale_id: Option<Id<Venta>>,
    terminal: Option<String>,
    created_at: domain::Timestamp,
}

fn parse_product_id(raw: &str, instance: &str) -> Result<Id<Producto>, ApiError> {
    Id::from_str(raw).map_err(|_| ApiError::invalid_product_id(instance.to_owned()))
}

fn require_stock(state: &AppState, instance: &str) -> Result<Arc<dyn StockGateway>, ApiError> {
    state
        .stock()
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

fn tipo_to_api(tipo: TipoMovimiento) -> &'static str {
    match tipo {
        TipoMovimiento::Inicial => "initial",
        TipoMovimiento::Ingreso => "inbound",
        TipoMovimiento::Ajuste => "adjustment",
        TipoMovimiento::Venta => "sale",
    }
}

fn tipo_from_create(t: StockMovementTypeCreate) -> TipoMovimiento {
    match t {
        StockMovementTypeCreate::Inbound => TipoMovimiento::Ingreso,
        StockMovementTypeCreate::Adjustment => TipoMovimiento::Ajuste,
    }
}

fn movement_response(mov: MovimientoStock) -> StockMovementResponse {
    StockMovementResponse {
        id: mov.id,
        product_id: mov.producto,
        type_: tipo_to_api(mov.tipo),
        quantity: mov.cantidad,
        resulting_stock: mov.stock_resultante,
        reason: mov.motivo,
        sale_id: mov.venta,
        terminal: mov.terminal,
        created_at: mov.creado_en,
    }
}

fn body_to_domain(
    product_id: Id<Producto>,
    body: StockMovementCreateBody,
    terminal: Option<String>,
) -> NewMovimientoStock {
    NewMovimientoStock {
        producto: product_id,
        tipo: tipo_from_create(body.type_),
        cantidad: body.quantity,
        motivo: body.reason,
        venta: None,
        terminal,
    }
}

pub async fn post_stock_movement(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
    headers: HeaderMap,
    AppJson(body): AppJson<StockMovementCreateBody>,
) -> Result<impl IntoResponse, ApiError> {
    let instance = format!("/api/v1/products/{product_id}/stock-movements");
    let id = parse_product_id(&product_id, &instance)?;
    let gateway = require_stock(&state, &instance)?;
    let terminal = parse_terminal_header(&headers, &instance)?;
    let input = body_to_domain(id, body, terminal);
    let registered = gateway
        .register(input)
        .await
        .map_err(|e| ApiError::from_domain(e, &instance))?;
    Ok((
        StatusCode::CREATED,
        [(header::CONTENT_TYPE, "application/json")],
        Json(movement_response(registered)),
    ))
}
