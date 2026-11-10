//! API Gateway HTTP shell (T-00-10, T-00-11).

mod app_json;
mod error;

use std::sync::Arc;

use axum::{Router, http::Uri, response::IntoResponse};

pub use app_json::AppJson;
pub use error::ApiError;

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

/// Builds the HTTP router without business routes (unknown paths → 404 `problem+json`).
pub fn build_router(state: AppState) -> Router {
    Router::new().fallback(fallback_not_found).with_state(state)
}
