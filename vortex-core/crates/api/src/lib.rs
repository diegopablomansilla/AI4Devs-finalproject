//! API Gateway HTTP shell (T-00-10, T-00-11, T-00-12).

mod app_json;
mod error;
mod gateway;

use std::sync::Arc;

use axum::{Router, http::Uri, response::IntoResponse};

pub use app_json::AppJson;
pub use error::ApiError;
pub use gateway::{DEFAULT_BODY_LIMIT, DEFAULT_REQUEST_TIMEOUT, apply_gateway_layers};

/// Shared application state for HTTP handlers (`Arc<dyn …>` added in later tickets).
#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    inner: Arc<AppStateInner>,
}

#[allow(dead_code)]
struct AppStateInner {}

impl AppState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AppStateInner {}),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

async fn fallback_not_found(uri: Uri) -> impl IntoResponse {
    ApiError::route_not_found(uri.path().to_owned())
}

/// Router shell (fallback 404) before gateway layers; used to attach test-only routes.
pub fn router_shell(state: AppState) -> Router {
    Router::new().fallback(fallback_not_found).with_state(state)
}

/// Builds the HTTP router without business routes (unknown paths → 404 `problem+json`).
pub fn build_router(state: AppState) -> Router {
    apply_gateway_layers(router_shell(state))
}
