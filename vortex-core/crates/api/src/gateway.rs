//! Transversal Gateway layers (T-00-12).

use std::time::Duration;

use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::ApiError;

/// Default JSON body limit (256 KiB), per [5.10](../../../../docs/5-contract-api.md#510-límites-timeouts-y-comportamiento-del-gateway).
pub const DEFAULT_BODY_LIMIT: usize = 256 * 1024;

/// Default request timeout (5 s) before `504 timeout`.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub fn apply_gateway_layers<S>(router: axum::Router<S>) -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let compression = CompressionLayer::new();

    let trace = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri().path(),
            )
        })
        .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
            tracing::info!(
                status = %response.status().as_u16(),
                latency_ms = latency.as_millis(),
                "finished request"
            );
        });

    // Innermost layer is listed first (see Axum `Router::layer` ordering).
    router
        .layer(axum::extract::DefaultBodyLimit::max(DEFAULT_BODY_LIMIT))
        .layer(RequestBodyLimitLayer::new(DEFAULT_BODY_LIMIT))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            DEFAULT_REQUEST_TIMEOUT,
        ))
        .layer(middleware::from_fn(map_gateway_error_responses))
        .layer(middleware::from_fn(set_no_store_for_api))
        .layer(compression)
        .layer(trace)
}

async fn set_no_store_for_api(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    if path == "/api" || path.starts_with("/api/") {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    response
}

async fn map_gateway_error_responses(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let response = next.run(request).await;

    let is_problem_json = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/problem+json"));

    if is_problem_json {
        return response;
    }

    match response.status() {
        StatusCode::PAYLOAD_TOO_LARGE => ApiError::payload_too_large(Some(path)).into_response(),
        StatusCode::GATEWAY_TIMEOUT => ApiError::gateway_timeout(Some(path)).into_response(),
        _ => response,
    }
}
