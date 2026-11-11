//! API Gateway HTTP shell (T-00-10 … T-00-14, T-00-19).

mod app_json;
mod error;
mod gateway;
mod health;
mod metrics;
mod static_files;

use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    Router,
    http::{StatusCode, Uri},
    middleware,
    response::IntoResponse,
    routing::get,
};
use domain::HealthReport;

pub use app_json::AppJson;
pub use error::ApiError;
pub use gateway::{DEFAULT_BODY_LIMIT, DEFAULT_REQUEST_TIMEOUT, apply_gateway_layers};
pub use metrics::{
    BACKUP_FAILURES_TOTAL, OUTBOX_FAILED_TOTAL, OUTBOX_LAG, operation_duration_seconds,
};

/// Shared application state for HTTP handlers (`Arc<dyn …>` added in later tickets).
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[derive(Clone)]
struct AppStateInner {
    health: Arc<dyn HealthReport>,
    web_root: PathBuf,
    media_root: PathBuf,
    version: String,
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
            inner: Arc::new(AppStateInner {
                health,
                web_root: PathBuf::new(),
                media_root: PathBuf::new(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
            }),
        }
    }

    #[must_use]
    pub fn with_static_roots(mut self, web_root: PathBuf, media_root: PathBuf) -> Self {
        Arc::make_mut(&mut self.inner).web_root = web_root;
        Arc::make_mut(&mut self.inner).media_root = media_root;
        self
    }

    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        Arc::make_mut(&mut self.inner).version = version.into();
        self
    }

    #[must_use]
    pub fn health(&self) -> Arc<dyn HealthReport> {
        Arc::clone(&self.inner.health)
    }

    #[must_use]
    pub fn web_root(&self) -> PathBuf {
        self.inner.web_root.clone()
    }

    #[must_use]
    pub fn media_root(&self) -> PathBuf {
        self.inner.media_root.clone()
    }

    #[must_use]
    pub fn version(&self) -> String {
        self.inner.version.clone()
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

async fn fallback_method_not_allowed() -> impl IntoResponse {
    StatusCode::METHOD_NOT_ALLOWED
}

/// Router shell (fallback 404) before gateway layers; used to attach test-only routes.
pub fn router_shell(state: AppState) -> Router {
    // Eager init so the first scrape and concurrent tests share one recorder.
    let _ = metrics::prometheus_handle();
    Router::new()
        .route(
            "/",
            get(static_files::get_index).post(fallback_method_not_allowed),
        )
        .route(
            "/index.html",
            get(static_files::get_index).post(fallback_method_not_allowed),
        )
        .route("/sw.js", get(static_files::get_sw_js))
        .route("/manifest.webmanifest", get(static_files::get_manifest))
        .route(
            "/static/{version}/{*path}",
            get(static_files::get_versioned_static),
        )
        .route("/media/{*path}", get(static_files::get_media))
        .route("/health", get(health::get_health))
        .route(
            "/metrics",
            get(metrics::get_metrics).post(|| async { StatusCode::METHOD_NOT_ALLOWED }),
        )
        .fallback(fallback_not_found)
        .layer(middleware::from_fn(metrics::record_http_duration))
        .with_state(state)
}

/// Builds the HTTP router without business routes (unknown paths → 404 `problem+json`).
pub fn build_router(state: AppState) -> Router {
    apply_gateway_layers(router_shell(state))
}
