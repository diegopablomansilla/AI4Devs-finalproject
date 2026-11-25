//! `GET /api/v1/search` — HTTP adapter over `search::SearchGateway` (T-03-13).

#![allow(clippy::result_large_err)]

use std::collections::BTreeMap;
use std::str::FromStr as _;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ::search::SearchGateway;
use axum::Json;
use axum::extract::{Query, RawQuery, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use domain::producto::{Producto, UnidadMedida};
use domain::{
    Categoria, Decimal, Id, Marca, SearchFilters, SearchHighlight, SearchHit, SearchOrden,
    SearchPage, SearchQuery,
};
use metrics::histogram;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

use crate::AppState;
use crate::error::ApiError;
use crate::metrics::operation_duration_seconds;

const INSTANCE: &str = "/api/v1/search";
const SEARCH_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchListQuery {
    #[serde(default)]
    q: String,
    category_id: Option<String>,
    brand_id: Option<String>,
    in_stock: Option<bool>,
    #[serde(default = "default_sort")]
    sort: String,
    #[serde(default)]
    offset: u32,
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    facets: bool,
}

fn default_sort() -> String {
    "relevance".into()
}

fn default_limit() -> u32 {
    20
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HighlightResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchHitResponse {
    id: Id<Producto>,
    sku: String,
    name: String,
    product_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    brand: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    category_path: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    attributes: Vec<String>,
    price: Decimal,
    unit_of_measure: String,
    stock: Decimal,
    in_stock: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
    #[serde(rename = "imageUrl", skip_serializing_if = "Option::is_none")]
    image_url: Option<String>,
    highlight: Option<HighlightResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResponseBody {
    query: String,
    hits: Vec<SearchHitResponse>,
    estimated_total_hits: u32,
    offset: u32,
    limit: u32,
    processing_time_ms: u32,
    facets: Option<serde_json::Value>,
    degraded: bool,
}

fn require_search(state: &AppState) -> Result<Arc<dyn SearchGateway>, ApiError> {
    state
        .search()
        .ok_or_else(|| ApiError::catalog_unavailable(INSTANCE.to_owned()))
}

fn decode_query_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                if let Ok(byte) =
                    u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
                {
                    out.push(byte as char);
                    i += 3;
                    continue;
                }
                out.push('%');
            }
            b'+' => {
                out.push(' ');
                i += 1;
                continue;
            }
            b => {
                out.push(b as char);
                i += 1;
            }
        }
    }
    out
}

fn parse_attr_filters(raw: Option<&str>) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let Some(q) = raw else {
        return map;
    };
    for segment in q.split('&') {
        let Some((k, v)) = segment.split_once('=') else {
            continue;
        };
        let key = decode_query_component(k);
        let Some(inner) = key.strip_prefix("attr[").and_then(|s| s.strip_suffix(']')) else {
            continue;
        };
        if inner.is_empty() {
            continue;
        }
        map.insert(inner.to_string(), decode_query_component(v));
    }
    map
}

fn parse_sort_api(raw: &str) -> Result<SearchOrden, ApiError> {
    match raw {
        "relevance" => Ok(SearchOrden::Relevancia),
        "price_asc" => Ok(SearchOrden::PrecioAsc),
        "price_desc" => Ok(SearchOrden::PrecioDesc),
        "stock_desc" => Ok(SearchOrden::StockDesc),
        _ => Err(ApiError::invalid_search_sort(INSTANCE.to_owned())),
    }
}

fn parse_optional_category(raw: &Option<String>) -> Result<Option<Id<Categoria>>, ApiError> {
    match raw {
        None => Ok(None),
        Some(s) => Id::from_str(s)
            .map(Some)
            .map_err(|_| ApiError::invalid_category_id(INSTANCE.to_owned())),
    }
}

fn parse_optional_brand(raw: &Option<String>) -> Result<Option<Id<Marca>>, ApiError> {
    match raw {
        None => Ok(None),
        Some(s) => Id::from_str(s)
            .map(Some)
            .map_err(|_| ApiError::invalid_brand_id(INSTANCE.to_owned())),
    }
}

fn unit_of_measure_api(raw: &str) -> String {
    let api = match UnidadMedida::parse_wire(raw) {
        Ok(UnidadMedida::Unidad) => "unit",
        Ok(UnidadMedida::Par) => "pair",
        Ok(UnidadMedida::Caja) => "box",
        Ok(UnidadMedida::Metro) => "meter",
        Ok(UnidadMedida::Kilogramo) => "kilogram",
        Ok(UnidadMedida::Litro) => "liter",
        Err(_) => "unit",
    };
    api.to_owned()
}

fn decimal_from_f64(v: f64) -> Decimal {
    if !v.is_finite() || v < 0.0 {
        return "0".parse().expect("zero decimal");
    }
    let s = format!("{v:.4}");
    let trimmed = s.trim_end_matches('0').trim_end_matches('.').to_string();
    let wire = if trimmed.is_empty() {
        "0"
    } else {
        trimmed.as_str()
    };
    Decimal::from_str(wire).unwrap_or_else(|_| "0".parse().expect("zero decimal"))
}

fn highlight_response(h: &SearchHighlight) -> HighlightResponse {
    HighlightResponse {
        name: h.nombre.clone(),
        description: h.descripcion.clone(),
    }
}

fn hit_response(hit: SearchHit) -> SearchHitResponse {
    let highlight = hit.resaltado.as_ref().map(highlight_response);
    SearchHitResponse {
        id: hit.id,
        sku: hit.sku,
        name: hit.nombre,
        product_type: hit.tipo,
        brand: hit.marca,
        category_path: Vec::new(),
        codes: hit.codigos,
        attributes: hit.atributos_texto,
        price: decimal_from_f64(hit.precio),
        unit_of_measure: unit_of_measure_api(&hit.unidad_medida),
        stock: decimal_from_f64(hit.stock_disponible),
        in_stock: hit.con_stock,
        location: hit.ubicacion,
        image_url: hit.imagen,
        highlight,
    }
}

fn response_body(q: &str, page: SearchPage) -> SearchResponseBody {
    SearchResponseBody {
        query: q.to_owned(),
        hits: page.hits.into_iter().map(hit_response).collect(),
        estimated_total_hits: page.estimado_total,
        offset: page.offset,
        limit: page.limit,
        processing_time_ms: page.tiempo_ms,
        facets: None,
        degraded: false,
    }
}

fn build_filters(
    query: &SearchListQuery,
    attrs: BTreeMap<String, String>,
) -> Result<SearchFilters, ApiError> {
    let category_id = parse_optional_category(&query.category_id)?;
    let marca_id = parse_optional_brand(&query.brand_id)?;
    Ok(SearchFilters {
        categoria_ids: category_id.into_iter().collect::<Vec<_>>(),
        marca_id,
        con_stock: query.in_stock,
        atributos: attrs,
    })
}

fn record_search_duration(elapsed: Duration) {
    histogram!(operation_duration_seconds("search")).record(elapsed.as_secs_f64());
}

pub async fn get_search(
    State(state): State<AppState>,
    Query(query): Query<SearchListQuery>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, ApiError> {
    let _ignore_facets = query.facets;
    let gateway = require_search(&state)?;
    let orden = parse_sort_api(&query.sort)?;
    let attrs = parse_attr_filters(raw_query.as_deref());
    let filtros = build_filters(&query, attrs)?;
    let q = query.q.clone();
    SearchQuery::try_new(q.clone(), filtros.clone(), orden, query.offset, query.limit)
        .map_err(|e| ApiError::from_domain(e, INSTANCE))?;

    let start = Instant::now();
    let search_fut = gateway.search(q.clone(), filtros, orden, query.offset, query.limit);
    let page = match timeout(SEARCH_TIMEOUT, search_fut).await {
        Ok(Ok(page)) => page,
        Ok(Err(err)) => {
            record_search_duration(start.elapsed());
            return Err(ApiError::from_domain(err, INSTANCE));
        }
        Err(_) => {
            record_search_duration(start.elapsed());
            return Err(ApiError::gateway_timeout(Some(INSTANCE.to_owned())));
        }
    };
    record_search_duration(start.elapsed());

    let body = response_body(&q, page);
    Ok((StatusCode::OK, Json(body)))
}
