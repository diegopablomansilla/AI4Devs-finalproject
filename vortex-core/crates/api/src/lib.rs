//! API Gateway HTTP shell (T-00-10, T-00-11, T-00-12, T-00-13).

mod app_json;
mod error;
mod gateway;
mod health;

use std::sync::Arc;

use axum::{Router, http::Uri, response::IntoResponse, routing::get};
use domain::HealthReport;

pub use app_json::AppJson;
pub use error::ApiError;
pub use gateway::{DEFAULT_BODY_LIMIT, DEFAULT_REQUEST_TIMEOUT, apply_gateway_layers};

/// Shared application state for HTTP handlers (`Arc<dyn …>` added in later tickets).
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    health: Arc<dyn HealthReport>,
}

impl AppState {
    #[must_use]
    pub fn new() -> Self {
        Self::with_health(Arc::new(domain::HealthSnapshot::all_disabled(env!(
            "CARGO_PKG_VERSION"
        ))))
    }

    #[must_use]
    pub fn with_health(health: Arc<dyn HealthReport>) -> Self {
        Self {
            inner: Arc::new(AppStateInner { health }),
        }
    }

    #[must_use]
    pub fn health(&self) -> Arc<dyn HealthReport> {
        Arc::clone(&self.inner.health)
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
    Router::new()
        .route("/health", get(health::get_health))
        .fallback(fallback_not_found)
        .with_state(state)
}

/// Builds the HTTP router without business routes (unknown paths → 404 `problem+json`).
pub fn build_router(state: AppState) -> Router {
    apply_gateway_layers(router_shell(state))
}
