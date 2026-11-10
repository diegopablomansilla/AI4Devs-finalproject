//! API Gateway HTTP shell (T-00-10).

use std::sync::Arc;

use axum::Router;

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

/// Builds the HTTP router without business routes (unknown paths → 404).
pub fn build_router(state: AppState) -> Router {
    Router::new().with_state(state)
}
